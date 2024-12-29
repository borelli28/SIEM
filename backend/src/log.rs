use crate::database::establish_connection;
use serde::{Serialize, Deserialize};
use crate::schema::logs;
use diesel::prelude::*;
use uuid::Uuid;
use std::fmt;

#[derive(Debug)]
pub enum LogError {
    DatabaseError(diesel::result::Error),
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

impl From<diesel::result::Error> for LogError {
    fn from(err: diesel::result::Error) -> Self {
        LogError::DatabaseError(err)
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = logs)]
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

    let mut conn = establish_connection();
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

    diesel::insert_into(logs::table)
        .values(&new_log)
        .execute(&mut conn)?;

    Ok(new_log)
}

pub fn get_log(log_id: &String) -> Result<Option<Log>, LogError> {
    if log_id.is_empty() {
        return Err(LogError::ValidationError("Log ID cannot be empty".to_string()));
    }

    let mut conn = establish_connection();
    Ok(logs::table.find(log_id).first(&mut conn).optional()?)
}

pub fn get_all_logs(account_id: &String) -> Result<Vec<Log>, LogError> {
    if account_id.is_empty() {
        return Err(LogError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let mut conn = establish_connection();
    Ok(logs::table
        .filter(logs::account_id.eq(account_id))
        .load::<Log>(&mut conn)?)
}

pub fn update_log(log_id: &String, updated_log: &Log) -> Result<Log, LogError> {
    if log_id.is_empty() {
        return Err(LogError::ValidationError("Log ID cannot be empty".to_string()));
    }
    updated_log.validate()?;

    let mut conn = establish_connection();
    diesel::update(logs::table.find(log_id))
        .set(updated_log)
        .execute(&mut conn)?;

    Ok(logs::table.find(log_id).first(&mut conn)?)
}

pub fn delete_log(log_id: &String) -> Result<bool, LogError> {
    if log_id.is_empty() {
        return Err(LogError::ValidationError("Log ID cannot be empty".to_string()));
    }

    let mut conn = establish_connection();
    let affected_rows = diesel::delete(logs::table.find(log_id))
        .execute(&mut conn)?;
    Ok(affected_rows > 0)
}