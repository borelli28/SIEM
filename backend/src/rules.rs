use rusqlite::{Error as SqliteError, params};
use crate::database::establish_connection;
use crate::alert::{create_alert, Alert};
use crate::log_parser::NormalizedLog;
use serde::{Serialize, Deserialize};
use chrono::{Utc, NaiveDateTime};
use rusqlite::OptionalExtension;
use uuid::Uuid;
use serde_json;
use log::info;
use std::fmt;

#[derive(Debug)]
pub enum RuleError {
    DatabaseError(SqliteError),
    ValidationError(String),
    AlertCreationError(String),
    SerializationError(String),
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleError::DatabaseError(err) => write!(f, "Database error: {}", err),
            RuleError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RuleError::AlertCreationError(msg) => write!(f, "Alert creation error: {}", msg),
            RuleError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for RuleError {}

impl From<SqliteError> for RuleError {
    fn from(err: SqliteError) -> Self {
        RuleError::DatabaseError(err)
    }
}

impl From<serde_json::Error> for RuleError {
    fn from(err: serde_json::Error) -> Self {
        RuleError::SerializationError(err.to_string())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LogSource {
    pub category: String,
    pub product: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub selection: std::collections::HashMap<String, serde_json::Value>,
    pub condition: String,
}

impl fmt::Display for Detection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Selection: {{\n")?;
        for (key, value) in &self.selection {
            write!(f, " {}: {},\n", key, value)?;
        }
        write!(f, "}}\nCondition: {}", self.condition)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Levels {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for Levels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Levels::Informational => write!(f, "Informational"),
            Levels::Low => write!(f, "Low"),
            Levels::Medium => write!(f, "Medium"),
            Levels::High => write!(f, "High"),
            Levels::Critical => write!(f, "Critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub status: String,
    pub description: String,
    #[serde(rename = "references")]
    pub ref_list: Vec<String>,
    pub tags: Vec<String>,
    pub author: String,
    pub date: String,
    pub logsource: LogSource,
    pub detection: Detection,
    pub fields: Vec<String>,
    pub falsepositives: Vec<String>,
    pub level: Levels,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Rule {
    fn validate(&self) -> Result<(), RuleError> {
        if self.account_id.is_empty() {
            return Err(RuleError::ValidationError("Account ID cannot be empty".to_string()));
        }
        if self.title.is_empty() {
            return Err(RuleError::ValidationError("Rule title cannot be empty".to_string()));
        }
        if self.detection.condition.is_empty() {
            return Err(RuleError::ValidationError("Rule condition cannot be empty".to_string()));
        }
        Ok(())
    }

    fn format_sigma_date(&self) -> Result<String, RuleError> {
        let parsed_date = NaiveDateTime::parse_from_str(&self.date, "%Y-%m-%d %H:%M:%S")
            .map_err(|e| RuleError::ValidationError(format!("Invalid date format: {}", e)))?;
        Ok(parsed_date.format("%Y/%m/%d").to_string())
    }
}

pub fn create_rule(rule: &Rule) -> Result<(), RuleError> {
    rule.validate()?;
    let conn = establish_connection()?;
    let now = Utc::now();
    let formatted_date = rule.format_sigma_date()?;

    let new_rule = Rule {
        id: Uuid::new_v4().to_string(),
        account_id: rule.account_id.clone(),
        title: rule.title.clone(),
        status: rule.status.clone(),
        description: rule.description.clone(),
        ref_list: rule.ref_list.clone(),
        tags: rule.tags.clone(),
        author: rule.author.clone(),
        date: formatted_date,
        logsource: rule.logsource.clone(),
        detection: rule.detection.clone(),
        fields: rule.fields.clone(),
        falsepositives: rule.falsepositives.clone(),
        level: rule.level.clone(),
        enabled: true,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    };

    conn.execute(
        "INSERT INTO rules (id, account_id, title, status, description, ref_list, tags, author, date, logsource, detection, fields, falsepositives, level, enabled, created_at, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
        params![
            new_rule.id,
            new_rule.account_id,
            new_rule.title,
            new_rule.status,
            new_rule.description,
            serde_json::to_string(&new_rule.ref_list)?,
            serde_json::to_string(&new_rule.tags)?,
            new_rule.author,
            new_rule.date,
            serde_json::to_string(&new_rule.logsource)?,
            serde_json::to_string(&new_rule.detection)?,
            serde_json::to_string(&new_rule.fields)?,
            serde_json::to_string(&new_rule.falsepositives)?,
            serde_json::to_string(&new_rule.level)?,
            new_rule.enabled,
            new_rule.created_at,
            new_rule.updated_at,
        ],
    )?;

    Ok(())
}

pub fn get_rule(id: &String) -> Result<Option<Rule>, RuleError> {
    if id.is_empty() {
        return Err(RuleError::ValidationError("Rule ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare("SELECT * FROM rules WHERE id = ?1")?;

    let rule = stmt.query_row(params![id], |row| {
        Ok(Rule {
            id: row.get(0)?,
            account_id: row.get(1)?,
            title: row.get(2)?,
            status: row.get(3)?,
            description: row.get(4)?,
            ref_list: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            tags: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
            author: row.get(7)?,
            date: row.get(8)?,
            logsource: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
            detection: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
            fields: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
            falsepositives: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
            level: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or(Levels::Low),
            enabled: row.get(14)?,
            created_at: row.get(15)?,
            updated_at: row.get(16)?,
        })
    }).optional()?;

    Ok(rule)
}

pub fn update_rule(rule: &Rule) -> Result<(), RuleError> {
    rule.validate()?;
    let conn = establish_connection()?;

    conn.execute(
        "UPDATE rules SET 
         account_id = ?1, title = ?2, status = ?3, description = ?4, ref_list = ?5,
         tags = ?6, author = ?7, date = ?8, logsource = ?9, detection = ?10, 
         fields = ?11, falsepositives = ?12, level = ?13, enabled = ?14,
         updated_at = ?15 
         WHERE id = ?16",
        params![
            rule.account_id,
            rule.title,
            rule.status,
            rule.description,
            serde_json::to_string(&rule.ref_list)?,
            serde_json::to_string(&rule.tags)?,
            rule.author,
            rule.date,
            serde_json::to_string(&rule.logsource)?,
            serde_json::to_string(&rule.detection)?,
            serde_json::to_string(&rule.fields)?,
            serde_json::to_string(&rule.falsepositives)?,
            serde_json::to_string(&rule.level)?,
            rule.enabled,
            Utc::now().to_rfc3339(),
            rule.id,
        ],
    )?;

    Ok(())
}

pub fn delete_rule(id: &String) -> Result<(), RuleError> {
    if id.is_empty() {
        return Err(RuleError::ValidationError("Rule ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    conn.execute("DELETE FROM rules WHERE id = ?1", params![id])?;

    Ok(())
}

pub fn list_rules(account_id: &String) -> Result<Vec<Rule>, RuleError> {
    if account_id.is_empty() {
        return Err(RuleError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare("SELECT * FROM rules WHERE account_id = ?1")?;

    let rules_iter = stmt.query_map(params![account_id], |row| {
        Ok(Rule {
            id: row.get(0)?,
            account_id: row.get(1)?,
            title: row.get(2)?,
            status: row.get(3)?,
            description: row.get(4)?,
            ref_list: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            tags: serde_json::from_str(&row.get::<_, String>(6)?).unwrap_or_default(),
            author: row.get(7)?,
            date: row.get(8)?,
            logsource: serde_json::from_str(&row.get::<_, String>(9)?).unwrap_or_default(),
            detection: serde_json::from_str(&row.get::<_, String>(10)?).unwrap_or_default(),
            fields: serde_json::from_str(&row.get::<_, String>(11)?).unwrap_or_default(),
            falsepositives: serde_json::from_str(&row.get::<_, String>(12)?).unwrap_or_default(),
            level: serde_json::from_str(&row.get::<_, String>(13)?).unwrap_or(Levels::Low),
            enabled: row.get(14)?,
            created_at: row.get(15)?,
            updated_at: row.get(16)?,
        })
    })?;

    let rules: Result<Vec<Rule>, SqliteError> = rules_iter.collect();
    Ok(rules?)
}

pub async fn evaluate_log_against_rules(log: &NormalizedLog, account_id: &String) -> Result<Vec<Alert>, RuleError> {
    let rules = list_rules(account_id)?;
    let mut triggered_alerts = Vec::new();

    for rule in rules {
        if !rule.enabled || &rule.account_id != account_id {
            continue;
        }

        if matches_detection(&rule.detection, log) {
            let new_alert = Alert {
                id: Uuid::new_v4().to_string(),
                rule_id: rule.id.clone(),
                account_id: rule.account_id.clone(),
                severity: rule.level.to_string(),
                message: format!("Alert triggered: {} - {}", rule.title, rule.description),
                acknowledged: false,
                case_id: None,
                created_at: Utc::now().to_rfc3339(),
            };
            create_alert(&new_alert)
                .map_err(|e| RuleError::AlertCreationError(e.to_string()))?;
            info!("Alert created: {:?}", new_alert.message);
            triggered_alerts.push(new_alert);
        }
    }

    Ok(triggered_alerts)
}

fn matches_detection(detection: &Detection, log: &NormalizedLog) -> bool {
    info!("Evaluating detection: {:?}", detection);
    for (field, expected_value) in &detection.selection {
        let actual_value = match field.as_str() {
            "event_type" => log.event_type.as_ref().map_or("", String::as_str),
            "src_ip" => log.src_ip.as_ref().map_or("", String::as_str),
            "dst_ip" => log.dst_ip.as_ref().map_or("", String::as_str),
            "timestamp" => log.timestamp.as_ref().map_or("", String::as_str),
            _ => log.extensions.get(field).map_or("", String::as_str),
        };

        let expected_str = expected_value.as_str().unwrap_or("");
        if actual_value != expected_str {
            info!(
                "Detection mismatch: field '{}' - actual: '{}', expected: '{}'",
                field, actual_value, expected_str
            );
            return false;
        }
    }
    info!("Detection match found for log: {:?}", log.raw);
    true
}