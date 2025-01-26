use rusqlite::{Error as SqliteError, ToSql, params, params_from_iter};
use crate::eql::{EqlParser, QueryBuilder};
use crate::database::establish_connection;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use uuid::Uuid;
use std::fmt;

#[derive(Debug)]
pub enum LogError {
    DatabaseError(SqliteError),
    ValidationError(String),
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogError::DatabaseError(err) => write!(f, "Database error: {}", err),
            LogError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for LogError {}

impl From<SqliteError> for LogError {
    fn from(err: SqliteError) -> Self {
        LogError::DatabaseError(err)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Log {
    pub id: String,
    pub hash: String,
    pub account_id: String,
    pub host_id: String,
    pub version: Option<String>,
    pub device_vendor: Option<String>,
    pub device_product: Option<String>,
    pub device_version: Option<String>,
    pub signature_id: Option<String>,
    pub name: Option<String>,
    pub severity: Option<String>,
    pub extensions: Option<String>,
    pub timestamp: Option<String>
}

impl Log {
    fn validate(&self) -> Result<(), LogError> {
        if self.account_id.is_empty() {
            return Err(LogError::ValidationError("Account ID cannot be empty".to_string()));
        }
        if self.host_id.is_empty() {
            return Err(LogError::ValidationError("Host ID cannot be empty".to_string()));
        }
        Ok(())
    }

    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        let content = format!("{}{}{}{}{}{}{}{}{}{}", 
            self.account_id,
            self.host_id,
            self.version.as_deref().unwrap_or(""),
            self.device_vendor.as_deref().unwrap_or(""),
            self.device_product.as_deref().unwrap_or(""),
            self.device_version.as_deref().unwrap_or(""),
            self.signature_id.as_deref().unwrap_or(""),
            self.name.as_deref().unwrap_or(""),
            self.severity.as_deref().unwrap_or(""),
            self.extensions.as_deref().unwrap_or("")
        );
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

pub fn create_log(log: &Log) -> Result<Option<Log>, LogError> {
    log.validate()?;
    let hash = log.calculate_hash();
    let conn = establish_connection()?;

    // Check for existing log with the same hash
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM logs WHERE hash = ?1")?;
    let count: i64 = stmt.query_row(params![&hash], |row| row.get(0))?;

    // If a duplicate, skip the log
    if count > 0 {
        return Ok(None);
    }

    let new_log = Log {
        id: Uuid::new_v4().to_string(),
        hash: hash.clone(),
        account_id: log.account_id.clone(),
        host_id: log.host_id.clone(),
        version: log.version.clone(),
        device_vendor: log.device_vendor.clone(),
        device_product: log.device_product.clone(),
        device_version: log.device_version.clone(),
        signature_id: log.signature_id.clone(),
        name: log.name.clone(),
        severity: log.severity.clone(),
        extensions: log.extensions.clone(),
    };

    conn.execute(
        "INSERT INTO logs (id, hash, account_id, host_id, version, device_vendor, device_product, 
         device_version, signature_id, name, severity, extensions) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            new_log.id,
            new_log.hash,
            new_log.account_id,
            new_log.host_id,
            new_log.version,
            new_log.device_vendor,
            new_log.device_product,
            new_log.device_version,
            new_log.signature_id,
            new_log.name,
            new_log.severity,
            new_log.extensions,
        ],
    )?;

    Ok(Some(new_log))
}

pub fn get_query_logs(eql_query: &str, start_time: Option<String>, end_time: Option<String>) -> Result<Vec<Log>, LogError> {
    let mut where_conditions = Vec::new();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    // Parse EQL query first
    let tokens = EqlParser::parse(eql_query)
        .map_err(|e| LogError::ValidationError(e.to_string()))?;

    let (base_query, base_params) = QueryBuilder::build_query(tokens)
        .map_err(|e| LogError::ValidationError(e.to_string()))?;

    // Extract timestamp from extensions JSON field
    if let Some(start) = start_time {
        where_conditions.push("json_extract(extensions, '$.rt') >= ?");
        params.push(Box::new(start));
    }
    if let Some(end) = end_time {
        where_conditions.push("json_extract(extensions, '$.rt') <= ?");
        params.push(Box::new(end));
    }

    // Construct final query
    let mut final_query = if base_query.contains("WHERE") {
        format!("{} AND {}", base_query, where_conditions.join(" AND "))
    } else {
        format!("{} WHERE {}", base_query, where_conditions.join(" AND "))
    };

    // Add sorting by timestamp
    final_query.push_str(" ORDER BY json_extract(extensions, '$.rt') DESC");

    // Combine all parameters
    params.extend(base_params.into_iter().map(|p| Box::new(p) as Box<dyn ToSql>));

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(&final_query)?;

    let log_iter = stmt.query_map(params_from_iter(params.iter().map(|p| &**p)), |row| {
        let extensions: Option<String> = row.get(11)?;
        let timestamp = if let Some(ext) = &extensions {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(ext) {
                json["rt"].as_str().map(|t| t.to_string())
            } else {
                None
            }
        } else {
            None
        };

        Ok(Log {
            id: row.get(0)?,
            hash: row.get(1)?,
            account_id: row.get(2)?,
            host_id: row.get(3)?,
            version: row.get(4)?,
            device_vendor: row.get(5)?,
            device_product: row.get(6)?,
            device_version: row.get(7)?,
            signature_id: row.get(8)?,
            name: row.get(9)?,
            severity: row.get(10)?,
            extensions,
            timestamp,
        })
    })?;

    let mut logs = Vec::new();
    for log in log_iter {
        logs.push(log?);
    }

    Ok(logs)
}

pub fn get_all_logs(account_id: &String) -> Result<Vec<Log>, LogError> {
    if account_id.is_empty() {
        return Err(LogError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, hash, account_id, host_id, version, device_vendor, device_product, 
         device_version, signature_id, name, severity, extensions 
         FROM logs WHERE account_id = ?1"
    )?;

    let logs_iter = stmt.query_map(params![account_id], |row| {
        Ok(Log {
            id: row.get(0)?,
            hash: row.get(1)?,
            account_id: row.get(2)?,
            host_id: row.get(3)?,
            version: row.get(4)?,
            device_vendor: row.get(5)?,
            device_product: row.get(6)?,
            device_version: row.get(7)?,
            signature_id: row.get(8)?,
            name: row.get(9)?,
            severity: row.get(10)?,
            extensions: row.get(11)?,
        })
    })?;

    let logs: Result<Vec<Log>, SqliteError> = logs_iter.collect();
    Ok(logs?)
}

// pub fn delete_log(log_id: &String) -> Result<bool, LogError> {
//     if log_id.is_empty() {
//         return Err(LogError::ValidationError("Log ID cannot be empty".to_string()));
//     }

//     let conn = establish_connection()?;
//     let affected_rows = conn.execute(
//         "DELETE FROM logs WHERE id = ?1",
//         params![log_id],
//     )?;

//     Ok(affected_rows > 0)
// }