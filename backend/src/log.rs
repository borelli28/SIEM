use crate::database::establish_connection;
use serde::{Serialize, Deserialize};
use rusqlite::{Error as SqliteError, params};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Log {
    pub id: String,
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
}

pub fn create_log(log: &Log) -> Result<Log, LogError> {
    log.validate()?;

    let conn = establish_connection()?;
    let new_log = Log {
        id: Uuid::new_v4().to_string(),
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
        "INSERT INTO logs (id, account_id, host_id, version, device_vendor, device_product, 
         device_version, signature_id, name, severity, extensions) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            new_log.id,
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

    Ok(new_log)
}

pub fn get_log(log_id: &String) -> Result<Option<Log>, LogError> {
    if log_id.is_empty() {
        return Err(LogError::ValidationError("Log ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, host_id, version, device_vendor, device_product, 
         device_version, signature_id, name, severity, extensions 
         FROM logs WHERE id = ?1"
    )?;

    let log = stmt.query_row(params![log_id], |row| {
        Ok(Log {
            id: row.get(0)?,
            account_id: row.get(1)?,
            host_id: row.get(2)?,
            version: row.get(3)?,
            device_vendor: row.get(4)?,
            device_product: row.get(5)?,
            device_version: row.get(6)?,
            signature_id: row.get(7)?,
            name: row.get(8)?,
            severity: row.get(9)?,
            extensions: row.get(10)?,
        })
    }).optional()?;

    Ok(log)
}

pub fn get_all_logs(account_id: &String) -> Result<Vec<Log>, LogError> {
    if account_id.is_empty() {
        return Err(LogError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, host_id, version, device_vendor, device_product, 
         device_version, signature_id, name, severity, extensions 
         FROM logs WHERE account_id = ?1"
    )?;

    let logs_iter = stmt.query_map(params![account_id], |row| {
        Ok(Log {
            id: row.get(0)?,
            account_id: row.get(1)?,
            host_id: row.get(2)?,
            version: row.get(3)?,
            device_vendor: row.get(4)?,
            device_product: row.get(5)?,
            device_version: row.get(6)?,
            signature_id: row.get(7)?,
            name: row.get(8)?,
            severity: row.get(9)?,
            extensions: row.get(10)?,
        })
    })?;

    let logs: Result<Vec<Log>, SqliteError> = logs_iter.collect();
    Ok(logs?)
}

pub fn update_log(log_id: &String, updated_log: &Log) -> Result<Log, LogError> {
    if log_id.is_empty() {
        return Err(LogError::ValidationError("Log ID cannot be empty".to_string()));
    }
    updated_log.validate()?;

    let conn = establish_connection()?;
    conn.execute(
        "UPDATE logs SET account_id = ?1, host_id = ?2, version = ?3, device_vendor = ?4, 
         device_product = ?5, device_version = ?6, signature_id = ?7, name = ?8, 
         severity = ?9, extensions = ?10 
         WHERE id = ?11",
        params![
            updated_log.account_id,
            updated_log.host_id,
            updated_log.version,
            updated_log.device_vendor,
            updated_log.device_product,
            updated_log.device_version,
            updated_log.signature_id,
            updated_log.name,
            updated_log.severity,
            updated_log.extensions,
            log_id,
        ],
    )?;

    get_log(log_id)?.ok_or_else(|| LogError::ValidationError("Log not found after update".to_string()))
}

pub fn delete_log(log_id: &String) -> Result<bool, LogError> {
    if log_id.is_empty() {
        return Err(LogError::ValidationError("Log ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "DELETE FROM logs WHERE id = ?1",
        params![log_id],
    )?;

    Ok(affected_rows > 0)
}