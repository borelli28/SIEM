use crate::database::establish_connection;
use serde::{Serialize, Deserialize};
use crate::schema::logs;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Insertable, AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = logs)]
pub struct Log {
    pub id: String,
    pub account_id: String,
    pub host_id: String,
    pub version: Option<String>,
    pub device_vendor: Option<String>,
    pub device_product: Option<String>,
    pub device_version: Option<String>,
    pub signature_id: Option<String>,
    pub name: Option<String>,
    pub severity: Option<String>,
    pub extensions: Option<String>,
}

impl Log {
    pub fn new(account_id: String, host_id: String) -> Self {
        Log {
            id: Uuid::new_v4().to_string(),
            account_id,
            host_id,
            version: None,
            device_vendor: None,
            device_product: None,
            device_version: None,
            signature_id: None,
            name: None,
            severity: None,
            extensions: None,
        }
    }
}

pub fn create_log(log: &Log) -> Result<Log, diesel::result::Error> {
    let mut conn = establish_connection();
    diesel::insert_into(logs::table)
        .values(log)
        .execute(&mut conn)?;
    logs::table.order(logs::id.desc()).first(&mut conn)
}

pub fn get_log(log_id: &String) -> Result<Option<Log>, diesel::result::Error> {
    let mut conn = establish_connection();
    logs::table.find(log_id).first(&mut conn).optional()
}

pub fn get_all_logs(account_id: &String) -> Result<Vec<Log>, diesel::result::Error> {
    let mut conn = establish_connection();
    logs::table
        .filter(logs::account_id.eq(account_id))
        .load::<Log>(&mut conn)
}

pub fn update_log(log_id: &String, updated_log: &Log) -> Result<Log, diesel::result::Error> {
    let mut conn = establish_connection();
    diesel::update(logs::table.find(log_id))
        .set(updated_log)
        .execute(&mut conn)?;
    logs::table.find(log_id).first(&mut conn)
}

pub fn delete_log(log_id: &String) -> Result<bool, diesel::result::Error> {
    let mut conn = establish_connection();
    let affected_rows = diesel::delete(logs::table.find(log_id))
        .execute(&mut conn)?;
    Ok(affected_rows > 0)
}