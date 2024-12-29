use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime, ParseError};
use uuid::Uuid;
use crate::schema::alerts; 
use crate::database::establish_connection;
use crate::schema::alerts::dsl::*;
use std::fmt;

fn db_conn() -> SqliteConnection {
    let conn: SqliteConnection = establish_connection();
    conn
}

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
    DatabaseError(diesel::result::Error),
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

impl From<diesel::result::Error> for AlertError {
    fn from(err: diesel::result::Error) -> Self {
        AlertError::DatabaseError(err)
    }
}

impl From<ParseError> for AlertError {
    fn from(err: ParseError) -> Self {
        AlertError::ParseError(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = alerts)]
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

    let mut conn: SqliteConnection = db_conn();
    diesel::insert_into(alerts::table)
        .values(&new_alert)
        .execute(&mut conn)?;

    Ok(new_alert)
}

pub fn get_alert(alert_id: &String) -> Result<Option<Alert>, AlertError> {
    if alert_id.is_empty() {
        return Err(AlertError::ValidationError("Alert ID cannot be empty".to_string()));
    }
    let mut conn: SqliteConnection = db_conn();
    Ok(alerts::table.find(alert_id).first(&mut conn).optional()?)
}

pub fn list_alerts(acct_id: &String) -> Result<Vec<Alert>, AlertError> {
    if acct_id.is_empty() {
        return Err(AlertError::ValidationError("Account ID cannot be empty".to_string()));
    }
    let mut conn: SqliteConnection = db_conn();
    Ok(alerts::table
        .filter(alerts::account_id.eq(acct_id))
        .order(alerts::created_at.desc())
        .load::<Alert>(&mut conn)?)
}

pub fn delete_alert(alert_id: &String) -> Result<bool, AlertError> {
    if alert_id.is_empty() {
        return Err(AlertError::ValidationError("Alert ID cannot be empty".to_string()));
    }
    let mut conn: SqliteConnection = db_conn();
    let result = diesel::delete(alerts::table.find(alert_id)).execute(&mut conn)?;
    Ok(result > 0)
}

pub fn acknowledge_alert(alert_id: &String) -> Result<bool, AlertError> {
    if alert_id.is_empty() {
        return Err(AlertError::ValidationError("Alert ID cannot be empty".to_string()));
    }
    let mut conn: SqliteConnection = db_conn();
    let updated_rows = diesel::update(alerts.find(alert_id))
        .set(acknowledged.eq(true))
        .execute(&mut conn)?;

    Ok(updated_rows > 0)
}