use crate::database::establish_connection;
use crate::collector::LogEntry;
use crate::schema::logs;
use diesel::prelude::*;
use std::error::Error;
use serde_json;

#[derive(Insertable)]
#[diesel(table_name = logs)]
struct NewLog {
    account_id: String,
    host_id: String,
    version: Option<String>,
    device_vendor: Option<String>,
    device_product: Option<String>,
    device_version: Option<String>,
    signature_id: Option<String>,
    name: Option<String>,
    severity: Option<String>,
    extensions: String,
}

pub async fn insert_log(log: &LogEntry) -> Result<(), Box<dyn Error>> {
    let mut conn = establish_connection();
    let extensions_json = serde_json::to_string(&log.extensions)?;

    let new_log = NewLog {
        account_id: log.account_id.clone(),
        host_id: log.account_id.clone(),
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