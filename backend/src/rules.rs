use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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