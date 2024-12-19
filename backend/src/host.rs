use crate::database::establish_connection;
use serde::{Serialize, Deserialize};
use crate::schema::host;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Insertable, AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = host)]
pub struct Host {
    pub id: String,
    pub account_id: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
}

impl Host {
    pub fn new(account_id: String, ip_address: Option<String>, hostname: Option<String>) -> Self {
        Host {
            id: Uuid::new_v4().to_string(),
            account_id,
            ip_address,
            hostname,
        }
    }
}

pub fn create_host(host: &Host, _account_id: &String) -> Result<(), diesel::result::Error> {
    let mut conn = establish_connection();
    diesel::insert_into(host::table)
        .values(host)
        .execute(&mut conn)?;
    Ok(())
}

pub fn get_host(host_id: &String) -> Result<Option<Host>, diesel::result::Error> {
    let mut conn = establish_connection();
    host::table.find(host_id).first(&mut conn).optional()
}

pub fn get_all_hosts(account_id: &String) -> Result<Vec<Host>, diesel::result::Error> {
    let mut conn = establish_connection();
    host::table
        .filter(host::account_id.eq(account_id))
        .load::<Host>(&mut conn)
}

pub fn update_host(host: &Host) -> Result<(), diesel::result::Error> {
    let mut conn = establish_connection();
    diesel::update(host::table.find(&host.id))
        .set(host)
        .execute(&mut conn)?;
    Ok(())
}

pub fn delete_host(host_id: &String) -> Result<bool, diesel::result::Error> {
    let mut conn = establish_connection();
    let num_deleted = diesel::delete(host::table.find(host_id))
        .execute(&mut conn)?;
    Ok(num_deleted > 0)
}