use crate::database::establish_connection;
use crate::schema::host;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Queryable, Insertable, AsChangeset, Debug)]
#[diesel(table_name = host)]
pub struct Host {
    pub id: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
}

impl Host {
    pub fn new(ip_address: Option<String>, hostname: Option<String>) -> Self {
        Host {
            id: Uuid::new_v4().to_string(),
            ip_address,
            hostname,
        }
    }
}

pub fn create_host(host: &Host) -> Result<(), diesel::result::Error> {
    let mut conn = establish_connection();
    diesel::insert_into(host::table)
        .values(host)
        .execute(&mut conn)?;
    Ok(())
}

pub fn get_host(host_id: &str) -> Result<Option<Host>, diesel::result::Error> {
    let mut conn = establish_connection();
    host::table.find(host_id).first(&mut conn).optional()
}

pub fn get_all_hosts() -> Result<Vec<Host>, diesel::result::Error> {
    let mut conn = establish_connection();
    host::table.load::<Host>(&mut conn)
}

pub fn update_host(host: &Host) -> Result<(), diesel::result::Error> {
    let mut conn = establish_connection();
    diesel::update(host::table.find(&host.id))
        .set(host)
        .execute(&mut conn)?;
    Ok(())
}

pub fn delete_host(host_id: &str) -> Result<bool, diesel::result::Error> {
    let mut conn = establish_connection();
    let num_deleted = diesel::delete(host::table.find(host_id))
        .execute(&mut conn)?;
    Ok(num_deleted > 0)
}