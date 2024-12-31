use evalexpr::{eval_boolean_with_context, ContextWithMutableVariables, HashMapContext, 
    DefaultNumericTypes
};
use diesel::{deserialize::FromSqlRow, sql_types::Text, AsExpression, prelude::*};
use crate::database::establish_connection;
use crate::alert::{create_alert, Alert};
use serde::{Serialize, Deserialize};
use crate::collector::LogEntry;
use chrono::{DateTime, Utc};
use crate::schema::rules;
use uuid::Uuid;
use serde_json;
use std::fmt;

#[derive(Debug)]
pub enum RuleError {
    DatabaseError(diesel::result::Error),
    ValidationError(String),
    AlertCreationError(String),
}

impl fmt::Display for RuleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuleError::DatabaseError(err) => write!(f, "Database error: {}", err),
            RuleError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            RuleError::AlertCreationError(msg) => write!(f, "Alert creation error: {}", msg),
        }
    }
}

impl std::error::Error for RuleError {}

impl From<diesel::result::Error> for RuleError {
    fn from(err: diesel::result::Error) -> Self {
        RuleError::DatabaseError(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSource {
    pub category: String,
    pub product: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub selection: std::collections::HashMap<String, serde_json::Value>,
    pub condition: String,
}

impl fmt::Display for Detection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Selection: {{\n")?;
        for (key, value) in &self.selection {
            write!(f, "  {}: {},\n", key, value)?;
        }
        write!(f, "}}\nCondition: {}", self.condition)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = Text)]
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

#[derive(Debug, Queryable, Insertable, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = rules)]
pub struct Rule {
    pub id: String,
    pub account_id: String,
    pub title: String,
    pub status: String,
    pub description: String,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub references: Vec<String>,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub tags: Vec<String>,
    pub author: String,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub date: DateTime<Utc>,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub logsource: LogSource,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub detection: Detection,
    #[diesel(serialize_as = String, deserialize_as = String)]
    pub fields: Vec<String>,
    #[diesel(serialize_as = String, deserialize_as = String)]
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
        if self.name.is_empty() {
            return Err(RuleError::ValidationError("Rule name cannot be empty".to_string()));
        }
        if self.condition.is_empty() {
            return Err(RuleError::ValidationError("Rule condition cannot be empty".to_string()));
        }
        if !["Low", "Medium", "High"].contains(&self.severity.as_str()) {
            return Err(RuleError::ValidationError("Invalid severity level".to_string()));
        }
        Ok(())
    }
}

pub fn create_rule(rule: &Rule) -> Result<(), RuleError> {
    rule.validate()?;
    let mut conn = establish_connection();
    let now = Utc::now();
    let new_rule = Rule {
        id: Uuid::new_v4().to_string(),
        account_id: rule.account_id.clone(),
        title: rule.title.clone(),
        status: rule.status.clone(),
        description: rule.description.clone(),
        references: rule.references.clone(),
        tags: rule.tags.clone(),
        author: rule.author.clone(),
        date: now,
        logsource: rule.logsource.clone(),
        detection: rule.detection.clone(),
        fields: rule.fields.clone(),
        falsepositives: rule.falsepositives.clone(),
        level: rule.level.clone(),
        enabled: true,
        created_at: now.to_rfc3339(),
        updated_at: now.to_rfc3339(),
    };

    diesel::insert_into(rules::table)
        .values(&new_rule)
        .execute(&mut conn)?;
    Ok(())
}

pub fn get_rule(id: &String) -> Result<Option<Rule>, RuleError> {
    if id.is_empty() {
        return Err(RuleError::ValidationError("Rule ID cannot be empty".to_string()));
    }
    let mut conn = establish_connection();
    let result = rules::table.find(id).first(&mut conn).optional()?;
    Ok(result)
}

pub fn update_rule(rule: &Rule) -> Result<(), RuleError> {
    rule.validate()?;
    let mut conn = establish_connection();
    diesel::update(rules::table.find(&rule.id))
        .set(rule)
        .execute(&mut conn)?;
    Ok(())
}

pub fn delete_rule(id: &String) -> Result<(), RuleError> {
    if id.is_empty() {
        return Err(RuleError::ValidationError("Rule ID cannot be empty".to_string()));
    }
    let mut conn = establish_connection();
    diesel::delete(rules::table.find(id)).execute(&mut conn)?;
    Ok(())
}

pub fn list_rules(account_id: &String) -> Result<Vec<Rule>, RuleError> {
    if account_id.is_empty() {
        return Err(RuleError::ValidationError("Account ID cannot be empty".to_string()));
    }
    let mut conn = establish_connection();
    let results = rules::table
        .filter(rules::account_id.eq(account_id))
        .load::<Rule>(&mut conn)?;
    Ok(results)
}

pub async fn evaluate_log_against_rules(log: &LogEntry, account_id: &String) -> Result<Vec<Alert>, RuleError> {
    if account_id.is_empty() {
        return Err(RuleError::ValidationError("Account ID cannot be empty".to_string()));
    }
    let rules = list_rules(&account_id)?;
    let mut triggered_alerts = Vec::new();

    for rule in rules {
        // Added "&" to rule.account_id so we can compare equals types(&String)
        if (rule.enabled) && (&rule.account_id == account_id) && evaluate_detection(&rule.detection, log) {
            let new_alert = Alert {
                id: Uuid::new_v4().to_string(),
                rule_id: rule.id.clone(),
                account_id: rule.account_id.clone(),
                severity: rule.severity.clone(),
                message: format!("Alert triggered: {} - {}", rule.title, rule.description),
                acknowledged: false,
                created_at: Utc::now().to_rfc3339(),
            };
            create_alert(&new_alert).map_err(|e| RuleError::AlertCreationError(e.to_string()))?;
            triggered_alerts.push(new_alert);
        }
    }

    Ok(triggered_alerts)
}

// Evaluate a condition string against a log entry
fn evaluate_detection(detection: &Detection, log: &LogEntry) -> bool {
    let mut context: HashMapContext<DefaultNumericTypes> = HashMapContext::new();

    // Context = Key/Value pairs like Dictionaries
    context.set_value("severity".to_string(), log.severity.clone().into()).unwrap();
    context.set_value("name".to_string(), log.name.clone().into()).unwrap();
    context.set_value("device_vendor".to_string(), log.device_vendor.clone().into()).unwrap();
    context.set_value("device_product".to_string(), log.device_product.clone().into()).unwrap();

    // Insert extensions
    for (key, value) in &log.extensions {
        context.set_value(key.to_string(), value.clone().into()).unwrap();
    }

    // Evaluate the condition as a boolean expression using the context
    match eval_boolean_with_context(detection, &context) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to evaluate condition: {}. Error: {:?}", detection, e);
            false  // Treat errors as a non-match
        }
    }
}