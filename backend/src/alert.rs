use crate::rules::AlertRule;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::{Utc};
use uuid::Uuid;
use crate::schema::alerts; 

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = alerts)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub account_id: String,
    pub severity: String,
    pub message: String,
    pub created_at: String,
}

impl Alert {
    pub fn new(rule: &AlertRule) -> Self {
        Alert {
            id: Uuid::new_v4().to_string(),
            rule_id: rule.id.clone(),
            account_id: rule.account_id.clone(),
            severity: rule.severity.clone(),
            message: format!("Alert triggered: {} - {}", rule.name, rule.description),
            created_at: Utc::now().to_string(),
        }
    }
}

pub fn create_alert(conn: &mut SqliteConnection, alert: &AlertRule) -> Result<Uuid, diesel::result::Error> {
    diesel::insert_into(alerts::table)
        .values(alert)
        .execute(conn)?;

    Ok(alert.id)
}

pub fn get_alert(conn: &mut SqliteConnection, id: String) -> Result<Option<Alert>, diesel::result::Error> {
    alerts::table.find(id).first(conn).optional()
}

pub fn list_alerts(conn: &mut SqliteConnection, account_id: &String) -> Result<Vec<Alert>, diesel::result::Error> {
    alerts::table
        .filter(alerts::account_id.eq(account_id))
        .order(alerts::created_at.desc())
        .load::<Alert>(conn)
}

pub fn delete_alert(conn: &mut SqliteConnection, id: String) -> Result<bool, diesel::result::Error> {
    let result = diesel::delete(alerts::table.find(id)).execute(conn)?;
    Ok(result > 0)
}