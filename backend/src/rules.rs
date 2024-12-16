use evalexpr::{context_map, eval_boolean_with_context, ContextWithMutableVariables, HashMapContext};
use sqlx::{SqlitePool, Result, FromRow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AlertRule {
    pub id: Uuid,
    pub account_id: String,
    pub name: String,
    pub description: String,
    pub condition: String,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "alert_severity", rename_all = "lowercase")]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl AlertRule {
    pub fn new(account_id: String, name: String, description: String, condition: String, severity: AlertSeverity) -> Self {
        let now = chrono::Utc::now();
        AlertRule {
            id: Uuid::new_v4(),
            account_id,
            name,
            description,
            condition,
            severity,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

pub async fn create_rule(pool: &SqlitePool, rule: &AlertRule) -> Result<Uuid> {
    let id = sqlx::query!(
        "INSERT INTO alert_rules (id, account_id, name, description, condition, severity, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        rule.id,
        rule.account_id,
        rule.name,
        rule.description,
        rule.condition,
        rule.severity as AlertSeverity,
        rule.enabled,
        rule.created_at,
        rule.updated_at
    )
    .execute(pool)
    .await?
    .last_insert_rowid();

    Ok(rule.id)
}

pub async fn get_rule(pool: &SqlitePool, id: Uuid) -> Result<Option<AlertRule>> {
    sqlx::query_as!(
        AlertRule,
        "SELECT * FROM alert_rules WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn update_rule(pool: &SqlitePool, rule: &AlertRule) -> Result<bool> {
    let result = sqlx::query!(
        "UPDATE alert_rules SET name = ?, description = ?, condition = ?, severity = ?, enabled = ?, updated_at = ? WHERE id = ?",
        rule.name,
        rule.description,
        rule.condition,
        rule.severity as AlertSeverity,
        rule.enabled,
        chrono::Utc::now(),
        rule.id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_rule(pool: &SqlitePool, id: Uuid) -> Result<bool> {
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

pub async fn evaluate_log_against_rules(pool: &SqlitePool, log: &Log, account_id: &str) -> Result<Vec<Alert>> {
    let rules = list_rules(pool, account_id).await?;
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