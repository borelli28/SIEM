use rusqlite::{Error as SqliteError, params};
use crate::database::establish_connection;
use serde::{Serialize, Deserialize};
use crate::eql::QueryExecutor;
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

    let mut log_uuid = Uuid::new_v4().to_string();
    // Check if this UUID already exists
    let mut id_check = conn.prepare("SELECT COUNT(*) FROM logs WHERE id = ?1")?;
    let id_count: i64 = id_check.query_row(params![&log_uuid], |row| row.get(0))?;
    if id_count > 0 {
        log_uuid = Uuid::new_v4().to_string();
    }

    let new_log = Log {
        id: log_uuid,
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
    let account_id = "acc123"; // Replace with actual account ID from context
    let start_time = start_time.unwrap_or("1970-01-01".to_string());
    let end_time = end_time.unwrap_or("9999-12-31".to_string());
    let log_data_vec = QueryExecutor::execute_query(account_id, &start_time, &end_time, eql_query)
        .map_err(|_| LogError::DatabaseError(rusqlite::Error::QueryReturnedNoRows))?; // Adjust error mapping
    let logs: Vec<Log> = log_data_vec.into_iter().map(|log_data| Log {
        id: String::new(), // Placeholder, fetch id if needed
        hash: String::new(), // Recalculate if needed
        account_id: account_id.to_string(),
        host_id: String::new(), // Fetch or assume from context
        timestamp: None, // Could extract from log_data if needed
        log_data,
    }).collect();
    Ok(logs)
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