use rusqlite::{Error as SqliteError, params};
use crate::database::establish_connection;
use chrono::{Utc, DateTime, ParseError};
use serde::{Deserialize, Serialize};
use rusqlite::OptionalExtension;
use uuid::Uuid;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
}

impl fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlertSeverity::Low => write!(f, "Low"),
            AlertSeverity::Medium => write!(f, "Medium"),
            AlertSeverity::High => write!(f, "High"),
        }
    }
}

impl From<String> for AlertSeverity {
    fn from(s: String) -> Self {
        match s.to_lowercase().as_str() {
            "low" => AlertSeverity::Low,
            "medium" => AlertSeverity::Medium,
            "high" => AlertSeverity::High,
            _ => AlertSeverity::Low, // Default to Low if unknown
        }
    }
}

#[derive(Debug)]
pub enum AlertError {
    DatabaseError(SqliteError),
    ValidationError(String),
    ParseError(ParseError),
}

impl fmt::Display for AlertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlertError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AlertError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AlertError::ParseError(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl From<SqliteError> for AlertError {
    fn from(err: SqliteError) -> Self {
        AlertError::DatabaseError(err)
    }
}

impl From<ParseError> for AlertError {
    fn from(err: ParseError) -> Self {
        AlertError::ParseError(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub account_id: String,
    pub severity: String,
    pub message: String,
    pub acknowledged: bool,
    pub created_at: String,
}

impl Alert {
    fn validate(&self, alert: &Alert) -> Result<(), AlertError> {
        if alert.rule_id.is_empty() {
            return Err(AlertError::ValidationError("Rule ID cannot be empty".to_string()));
        }
        if alert.account_id.is_empty() {
            return Err(AlertError::ValidationError("Account ID cannot be empty".to_string()));
        }
        if alert.message.is_empty() {
            return Err(AlertError::ValidationError("Message cannot be empty".to_string()));
        }
        let _ = AlertSeverity::from(alert.severity.clone());
        DateTime::parse_from_rfc3339(&alert.created_at)?;
        Ok(())
    }
}

pub fn create_alert(alert: &Alert) -> Result<Alert, AlertError> {
    let new_alert = Alert {
        id: Uuid::new_v4().to_string(),
        rule_id: alert.rule_id.clone(),
        account_id: alert.account_id.clone(),
        severity: alert.severity.clone(),
        message: alert.message.clone(),
        acknowledged: false,
        created_at: Utc::now().to_rfc3339(),
    };
    new_alert.validate(&new_alert)?;

    let conn = establish_connection()?;
    conn.execute(
        "INSERT INTO alerts (id, rule_id, account_id, severity, message, acknowledged, created_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            new_alert.id,
            new_alert.rule_id,
            new_alert.account_id,
            new_alert.severity,
            new_alert.message,
            new_alert.acknowledged,
            new_alert.created_at,
        ],
    )?;

    Ok(new_alert)
}

pub fn get_alert(alert_id: &String) -> Result<Option<Alert>, AlertError> {
    if alert_id.is_empty() {
        return Err(AlertError::ValidationError("Alert ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, rule_id, account_id, severity, message, acknowledged, created_at 
         FROM alerts WHERE id = ?1"
    )?;

    let alert = stmt.query_row(params![alert_id], |row| {
        Ok(Alert {
            id: row.get(0)?,
            rule_id: row.get(1)?,
            account_id: row.get(2)?,
            severity: row.get(3)?,
            message: row.get(4)?,
            acknowledged: row.get(5)?,
            created_at: row.get(6)?,
        })
    }).optional()?;

    Ok(alert)
}

pub fn list_alerts(acct_id: &String) -> Result<Vec<Alert>, AlertError> {
    if acct_id.is_empty() {
        return Err(AlertError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, rule_id, account_id, severity, message, acknowledged, created_at 
         FROM alerts WHERE account_id = ?1 
         ORDER BY created_at DESC"
    )?;

    let alerts_iter = stmt.query_map(params![acct_id], |row| {
        Ok(Alert {
            id: row.get(0)?,
            rule_id: row.get(1)?,
            account_id: row.get(2)?,
            severity: row.get(3)?,
            message: row.get(4)?,
            acknowledged: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let alerts: Result<Vec<Alert>, SqliteError> = alerts_iter.collect();
    Ok(alerts?)
}

pub fn delete_alert(alert_id: &String) -> Result<bool, AlertError> {
    if alert_id.is_empty() {
        return Err(AlertError::ValidationError("Alert ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "DELETE FROM alerts WHERE id = ?1",
        params![alert_id],
    )?;

    Ok(affected_rows > 0)
}

pub fn acknowledge_alert(alert_id: &String) -> Result<bool, AlertError> {
    if alert_id.is_empty() {
        return Err(AlertError::ValidationError("Alert ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "UPDATE alerts SET acknowledged = TRUE WHERE id = ?1",
        params![alert_id],
    )?;

    Ok(affected_rows > 0)
}