use diesel::AsExpression;
use evalexpr::{eval_boolean_with_context, ContextWithMutableVariables, HashMapContext};
use crate::collector::LogEntry;
use crate::database::establish_connection;
use diesel::prelude::*;
use uuid::Uuid;
use chrono::Utc;
use std::error::Error;

use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql};
use diesel::sqlite::Sqlite;
use diesel::FromSqlRow;

table! {
    alert_rules (id) {
        id -> Text,
        account_id -> Text,
        name -> Text,
        description -> Text,
        condition -> Text,
        severity -> Text,
        enabled -> Bool,
        created_at -> Text,
        updated_at -> Text,
    }
}

#[derive(Queryable, Insertable, Clone, AsChangeset)]
#[diesel(table_name = alert_rules)]
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

impl AlertRule {
    pub fn new(account_id: String, name: String, description: String, condition: String, severity: AlertSeverity) -> Self {
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

#[derive(Debug, Clone, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl ToSql<diesel::sql_types::Text, Sqlite> for AlertSeverity {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Sqlite>) -> serialize::Result {
        let s = match *self {
            AlertSeverity::Low => "Low",
            AlertSeverity::Medium => "Medium",
            AlertSeverity::High => "High",
            AlertSeverity::Critical => "Critical",
        };
        out.write_all(s.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<diesel::sql_types::Text, Sqlite> for AlertSeverity {
    fn from_sql(bytes: Option<&<Sqlite as diesel::backend::Backend>::RawValue>) -> deserialize::Result<Self> {
        let s = String::from_utf8(bytes.unwrap().to_vec())?;
        match s.as_str() {
            "Low" => Ok(AlertSeverity::Low),
            "Medium" => Ok(AlertSeverity::Medium),
            "High" => Ok(AlertSeverity::High),
            "Critical" => Ok(AlertSeverity::Critical),
            _ => Err("Unrecognized severity".into()),
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

pub fn evaluate_log_against_rules(log: &LogEntry, account_id: &str) -> Result<Vec<AlertRule>, Box<dyn Error>> {
    let rules = list_rules()?;
    let mut triggered_alerts = Vec::new();

    for rule in rules {
        if rule.enabled && rule.account_id == account_id && evaluate_condition(&rule.condition, log) {
            triggered_alerts.push(rule);
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