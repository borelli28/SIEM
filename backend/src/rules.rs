use sqlx::{SqlitePool, Result, FromRow};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AlertRule {
    pub id: Uuid,
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
    pub fn new(name: String, description: String, condition: String, severity: AlertSeverity) -> Self {
        let now = chrono::Utc::now();
        AlertRule {
            id: Uuid::new_v4(),
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
        "INSERT INTO alert_rules (id, name, description, condition, severity, enabled, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        rule.id,
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

    Ok(Uuid::from_u64(id as u64))
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