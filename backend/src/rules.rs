use evalexpr::{
    eval_boolean_with_context, 
    ContextWithMutableVariables, 
    HashMapContext, 
    DefaultNumericTypes
};
use crate::database::establish_connection;
use crate::alert::{create_alert, Alert};
use serde::{Serialize, Deserialize};
use crate::schema::alert_rules;
use crate::collector::LogEntry;
use diesel::prelude::*;
use chrono::Utc;
use uuid::Uuid;
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

#[derive(Debug, Queryable, Insertable, Clone, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = alert_rules)]
pub struct Rule {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub description: String,
    pub condition: String,
    pub severity: String,
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
    let now = Utc::now().to_rfc3339();
    let new_rule = Rule {
        id: Uuid::new_v4().to_string(),
        account_id: rule.account_id.clone(),
        name: rule.name.clone(),
        description: rule.description.clone(),
        condition: rule.condition.clone(),
        severity: rule.severity.clone(),
        enabled: true,
        created_at: now.clone(),
        updated_at: now,
    };

    diesel::insert_into(alert_rules::table)
        .values(&new_rule)
        .execute(&mut conn)?;
    Ok(())
}

pub fn get_rule(id: &String) -> Result<Option<Rule>, RuleError> {
    if id.is_empty() {
        return Err(RuleError::ValidationError("Rule ID cannot be empty".to_string()));
    }
    let mut conn = establish_connection();
    let result = alert_rules::table.find(id).first(&mut conn).optional()?;
    Ok(result)
}

pub fn update_rule(rule: &Rule) -> Result<(), RuleError> {
    rule.validate()?;
    let mut conn = establish_connection();
    diesel::update(alert_rules::table.find(&rule.id))
        .set(rule)
        .execute(&mut conn)?;
    Ok(())
}

pub fn delete_rule(id: &String) -> Result<(), RuleError> {
    if id.is_empty() {
        return Err(RuleError::ValidationError("Rule ID cannot be empty".to_string()));
    }
    let mut conn = establish_connection();
    diesel::delete(alert_rules::table.find(id)).execute(&mut conn)?;
    Ok(())
}

pub fn list_rules(account_id: &String) -> Result<Vec<Rule>, RuleError> {
    if account_id.is_empty() {
        return Err(RuleError::ValidationError("Account ID cannot be empty".to_string()));
    }
    let mut conn = establish_connection();
    let results = alert_rules::table
        .filter(alert_rules::account_id.eq(account_id))
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
        if (rule.enabled) && (&rule.account_id == account_id) && evaluate_condition(&rule.condition, log) {
            let new_alert = Alert {
                id: Uuid::new_v4().to_string(),
                rule_id: rule.id.clone(),
                account_id: rule.account_id.clone(),
                severity: rule.severity.clone(),
                message: format!("Alert triggered: {} - {}", rule.name, rule.description),
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
fn evaluate_condition(condition: &String, log: &LogEntry) -> bool {
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
    match eval_boolean_with_context(condition, &context) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed to evaluate condition: {}. Error: {:?}", condition, e);
            false  // Treat errors as a non-match
        }
    }
}