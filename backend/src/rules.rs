use evalexpr::{eval_boolean_with_context, ContextWithMutableVariables, HashMapContext, DefaultNumericTypes};
use crate::database::establish_connection;
use crate::collector::LogEntry;
use crate::alert::{create_alert, Alert};
use diesel::prelude::*;
use std::error::Error;
use chrono::Utc;
use uuid::Uuid;
use crate::schema::alert_rules;

#[derive(Debug, Queryable, Insertable, Clone, AsChangeset)]
#[diesel(table_name = alert_rules)]
pub struct AlertRule {
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

impl AlertRule {
    pub fn new(account_id: String, name: String, description: String, condition: String, severity: String) -> Self {
        let now = Utc::now().to_rfc3339();
        AlertRule {
            id: Uuid::new_v4().to_string(),
            account_id,
            name,
            description,
            condition,
            severity,
            enabled: true,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

pub fn create_rule(rule: &AlertRule) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection();
    diesel::insert_into(alert_rules::table)
        .values(rule)
        .execute(&mut conn)?;
    Ok(())
}

pub fn get_rule(id: &str) -> Result<Option<AlertRule>, Box<dyn Error>> {
    let mut conn = establish_connection();
    let result = alert_rules::table.find(id).first(&mut conn).optional()?;
    Ok(result)
}

pub fn update_rule(rule: &AlertRule) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection();
    diesel::update(alert_rules::table.find(&rule.id))
        .set(rule)
        .execute(&mut conn)?;
    Ok(())
}

pub fn delete_rule(id: &str) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection();
    diesel::delete(alert_rules::table.find(id)).execute(&mut conn)?;
    Ok(())
}

pub fn list_rules() -> Result<Vec<AlertRule>, Box<dyn Error>> {
    let mut conn = establish_connection();
    let results = alert_rules::table.load::<AlertRule>(&mut conn)?;
    Ok(results)
}

pub async fn evaluate_log_against_rules(log: &LogEntry, account_id: &String) -> Result<Vec<Alert>, Box<dyn Error>> {
    let mut conn = establish_connection();
    let rules = list_rules()?;
    let mut triggered_alerts = Vec::new();

    for rule in rules {
        // Added "&" to rule.account_id so we can compare equals types(&String)
        if rule.enabled && &rule.account_id == account_id && evaluate_condition(&rule.condition, log) {
            let alert = Alert::new(&rule);
            create_alert(&mut conn, &alert).expect("Failed to create alert");
            triggered_alerts.push(alert);
        }
    }

    Ok(triggered_alerts)
}

// Evaluate a condition string against a log entry
fn evaluate_condition(condition: &str, log: &LogEntry) -> bool {
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