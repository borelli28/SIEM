use crate::collector::LogEntry;
use evalexpr::{eval_boolean_with_context, ContextWithMutableVariables, HashMapContext};
use sqlx::{SqlitePool, Result, FromRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AlertRule {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub description: String,
    pub condition: String,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone)]
#[sqlx(type_name = "alert_severity", rename_all = "lowercase")]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AlertRule {
    pub fn new(account_id: String, name: String, description: String, condition: String, severity: AlertSeverity) -> Self {
        let now = chrono::Utc::now().to_string();
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

pub async fn create_rule(pool: &SqlitePool, rule: &AlertRule) -> Result<String> {
    let id_str = rule.id.to_string();
    let severity = rule.severity.clone() as AlertSeverity;
    sqlx::query!(
        "INSERT INTO alert_rules (id, account_id, name, description, condition, severity, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        id_str,
        rule.account_id,
        rule.name,
        rule.description,
        rule.condition,
        severity,
        rule.enabled,
        rule.created_at,
        rule.updated_at
    )
    .execute(pool)
    .await?;

    Ok(id_str)
}

pub async fn get_rule(pool: &SqlitePool, id: Uuid) -> Result<Option<AlertRule>> {
    sqlx::query_as!(
        AlertRule,
        "SELECT * FROM alert_rules WHERE id = ?",
        id.to_string()
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_rule(pool: &SqlitePool, rule: &AlertRule) -> Result<bool> {
    let severity = rule.severity.clone() as AlertSeverity;
    let timestamp = chrono::Utc::now().to_string();
    let rule_id = rule.id.to_string();
    let result = sqlx::query!(
        "UPDATE alert_rules SET name = ?, description = ?, condition = ?, severity = ?, enabled = ?, updated_at = ? WHERE id = ?",
        rule.name,
        rule.description,
        rule.condition,
        severity,
        rule.enabled,
        timestamp,
        rule_id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_rule(pool: &SqlitePool, id: Uuid) -> Result<bool> {
    let id = id.to_string();
    let result = sqlx::query!(
        "DELETE FROM alert_rules WHERE id = ?",
        id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_rules(pool: &SqlitePool) -> Result<Vec<AlertRule>> {
    sqlx::query_as!(
        AlertRule,
        "SELECT * FROM alert_rules"
    )
    .fetch_all(pool)
    .await
}

pub async fn evaluate_log_against_rules<Alert>(pool: &SqlitePool, log: &LogEntry, account_id: &str) -> Result<Vec<Alert>> {
    let rules = list_rules(pool).await?;
    let mut triggered_alerts = Vec::new();

    for rule in rules {
        if rule.enabled && evaluate_condition(&rule.condition, log) {
            let alert = generate_alert(log, &rule).await?;
            triggered_alerts.push(alert);
        }
    }

    Ok(triggered_alerts)
}

// Evaluate a condition string against a log entry
fn evaluate_condition(condition: &str, log: &LogEntry) -> bool {
    let mut context = HashMapContext::new();

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