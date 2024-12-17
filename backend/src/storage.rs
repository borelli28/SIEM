use crate::database::establish_connection;
use crate::collector::LogEntry;
use diesel::prelude::*;
use std::error::Error;
use serde_json;

table! {
    logs (id) {
        id -> Integer,
        version -> Nullable<Text>,
        device_vendor -> Nullable<Text>,
        device_product -> Nullable<Text>,
        device_version -> Nullable<Text>,
        signature_id -> Nullable<Text>,
        name -> Nullable<Text>,
        severity -> Nullable<Text>,
        extensions -> Text,
    }
}

#[derive(Insertable)]
#[diesel(table_name = logs)]
struct NewLog {
    version: Option<String>,
    device_vendor: Option<String>,
    device_product: Option<String>,
    device_version: Option<String>,
    signature_id: Option<String>,
    name: Option<String>,
    severity: Option<String>,
    extensions: String,
}

pub fn insert_log(log: &LogEntry) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection();
    let extensions_json = serde_json::to_string(&log.extensions)?;

    let new_log = NewLog {
        version: Some(log.version.clone()),
        device_vendor: Some(log.device_vendor.clone()),
        device_product: Some(log.device_product.clone()),
        device_version: Some(log.device_version.clone()),
        signature_id: Some(log.signature_id.clone()),
        name: Some(log.name.clone()),
        severity: Some(log.severity.clone()),
        extensions: extensions_json,
    };

    diesel::insert_into(logs::table)
        .values(&new_log)
        .execute(&mut conn)?;

    Ok(())
}