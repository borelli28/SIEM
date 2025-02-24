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
    pub timestamp: Option<String>,
    pub log_data: String,
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
        hasher.update(self.log_data.as_bytes());
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

    if count > 0 {
        return Ok(None); // Duplicate log
    }

    let new_log = Log {
        id: Uuid::new_v4().to_string(),
        hash,
        account_id: log.account_id.clone(),
        host_id: log.host_id.clone(),
        timestamp: log.timestamp.clone(),
        log_data: log.log_data.clone(),
    };

    conn.execute(
        "INSERT INTO logs (id, hash, account_id, host_id, timestamp, log_data) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            &new_log.id,
            &new_log.hash,
            &new_log.account_id,
            &new_log.host_id,
            &new_log.timestamp,
            &new_log.log_data,
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

    // First add base parameters
    params.extend(base_params.into_iter().map(|p| Box::new(p) as Box<dyn ToSql>));

    // Then add timestamp conditions if provided
    if let Some(start) = start_time {
        if let Some(end) = end_time {
            where_conditions.push("timestamp BETWEEN datetime(?) AND datetime(?)");
            params.push(Box::new(start));
            params.push(Box::new(end));
        }
    }

    // Construct final query
    let mut final_query = if !where_conditions.is_empty() {
        if base_query.contains("WHERE") {
            format!("{} AND {}", base_query, where_conditions.join(" AND "))
        } else {
            format!("{} WHERE {}", base_query, where_conditions.join(" AND "))
        }
    } else {
        base_query
    };

    // Add sorting by timestamp
    final_query.push_str(" ORDER BY timestamp DESC");

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(&final_query)?;

    let log_iter = stmt.query_map(params_from_iter(params.iter().map(|p| &**p)), |row| {
        Ok(Log {
            id: row.get(0)?,
            hash: row.get(1)?,
            account_id: row.get(2)?,
            host_id: row.get(3)?,
            timestamp: row.get(4)?,
            log_data: row.get(5)?,
        })
    })?;

    let logs: Result<Vec<Log>, SqliteError> = log_iter.collect();
    Ok(logs?)
}

pub fn get_all_logs(account_id: &String) -> Result<Vec<Log>, LogError> {
    if account_id.is_empty() {
        return Err(LogError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, hash, account_id, host_id, timestamp, log_data 
         FROM logs WHERE account_id = ?1"
    )?;

    let logs_iter = stmt.query_map(params![account_id], |row| {
        Ok(Log {
            id: row.get(0)?,
            hash: row.get(1)?,
            account_id: row.get(2)?,
            host_id: row.get(3)?,
            timestamp: row.get(4)?,
            log_data: row.get(5)?,
        })
    })?;

    let logs: Result<Vec<Log>, SqliteError> = logs_iter.collect();
    Ok(logs?)
}