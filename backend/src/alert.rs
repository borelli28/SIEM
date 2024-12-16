use sqlx::{SqlitePool, Result, FromRow};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Alert {
    pub id: Uuid,
    pub rule_id: Uuid,
    pub account_id: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub created_at: DateTime<Utc>,
}

impl Alert {
    pub fn new(rule: &AlertRule, log: &Log) -> Self {
        Alert {
            id: Uuid::new_v4(),
            rule_id: rule.id,
            log_id: log.id,
            account_id: rule.account_id.clone(),
            severity: rule.severity,
            message: format!("Alert triggered: {} - {}", rule.name, rule.description),
            created_at: Utc::now(),
        }
    }
}

pub async fn create_alert(pool: &SqlitePool, alert: &Alert) -> Result<Uuid> {
    sqlx::query!(
        "INSERT INTO alerts (id, rule_id, account_id, severity, message, created_at) VALUES (?, ?, ?, ?, ?, ?)",
        alert.id,
        alert.rule_id,
        alert.account_id,
        alert.severity as AlertSeverity,
        alert.message,
        alert.created_at
    )
    .execute(pool)
    .await?;

    Ok(alert.id)
}

pub async fn get_alert(pool: &SqlitePool, id: Uuid) -> Result<Option<Alert>> {
    sqlx::query_as!(
        Alert,
        "SELECT * FROM alerts WHERE id = ?",
        id
    )
    .fetch_optional(pool)
    .await
}

pub async fn list_alerts(pool: &SqlitePool, account_id: &str) -> Result<Vec<Alert>> {
    sqlx::query_as!(
        Alert,
        "SELECT * FROM alerts WHERE account_id = ? ORDER BY created_at DESC",
        account_id
    )
    .fetch_all(pool)
    .await
}

pub async fn delete_alert(pool: &SqlitePool, id: Uuid) -> Result<bool> {
    let result = sqlx::query!(
        "DELETE FROM alerts WHERE id = ?",
        id
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}