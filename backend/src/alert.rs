use crate::rules::AlertRule;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use serde::{Deserialize, Serialize};
use chrono::{Utc};
use uuid::Uuid;
use crate::schema::alerts; 
use crate::database::establish_connection;
use crate::schema::alerts::dsl::*;

fn db_conn() -> SqliteConnection {
    let conn: SqliteConnection = establish_connection();
    conn
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = alerts)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub account_id: String,
    pub severity: String,
    pub message: String,
    pub acknowledged: bool,
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
            acknowledged: false,
            created_at: Utc::now().to_string(),
        }
    }
}

pub fn create_alert(alert: &Alert) -> Result<String, diesel::result::Error> {
    let mut conn: SqliteConnection = db_conn();
    diesel::insert_into(alerts::table)
        .values(alert)
        .execute(&mut conn)?;

    Ok(alert.id.clone())
}

pub fn get_alert(alert_id: &String) -> Result<Option<Alert>, diesel::result::Error> {
    let mut conn: SqliteConnection = db_conn();
    alerts::table.find(alert_id).first(&mut conn).optional()
}

pub fn list_alerts(acct_id: &String) -> Result<Vec<Alert>, diesel::result::Error> {
    let mut conn: SqliteConnection = db_conn();
    alerts::table
        .filter(alerts::account_id.eq(acct_id))
        .order(alerts::created_at.desc())
        .load::<Alert>(&mut conn)
}

pub fn delete_alert(alert_id: &String) -> Result<bool, diesel::result::Error> {
    let mut conn: SqliteConnection = db_conn();
    let result = diesel::delete(alerts::table.find(alert_id)).execute(&mut conn)?;
    Ok(result > 0)
}

pub fn acknowledge_alert(alert_id: &String) -> Result<bool, diesel::result::Error> {
    let mut conn: SqliteConnection = db_conn();
    let updated_rows = diesel::update(alerts.find(alert_id))
        .set(acknowledged.eq(true))
        .execute(&mut conn)?;

    Ok(updated_rows > 0)
}