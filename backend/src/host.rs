use crate::database::establish_connection;
use std::{fmt, net::IpAddr, str::FromStr};
use serde::{Serialize, Deserialize};
use crate::schema::host;
use diesel::prelude::*;
use uuid::Uuid;

#[derive(Debug)]
pub enum HostError {
    DatabaseError(diesel::result::Error),
    ValidationError(String),
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HostError::DatabaseError(err) => write!(f, "Database error: {}", err),
            HostError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl From<diesel::result::Error> for HostError {
    fn from(err: diesel::result::Error) -> Self {
        HostError::DatabaseError(err)
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = host)]
pub struct Host {
    pub id: String,
    pub account_id: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
}

impl Host {
    fn validate(&self) -> Result<(), HostError> {
        if self.account_id.is_empty() {
            return Err(HostError::ValidationError("Account ID cannot be empty".to_string()));
        }

        if let Some(ref ip) = self.ip_address {
            if IpAddr::from_str(ip).is_err() {
                return Err(HostError::ValidationError(format!("Invalid IP address: {}", ip)));
            }
        }

        Ok(())
    }
}

pub fn create_host(host: &Host, account_id: &String) -> Result<(), HostError> {
    host.validate()?;

    let mut conn = establish_connection();
    let id = Uuid::new_v4().to_string();

    if let Some(ref hostname) = host.hostname {
        if hostname_exists(account_id, hostname)? {
            return Err(HostError::ValidationError(
                format!("A host with the hostname '{}' already exists.", hostname)
            ));
        }
    }

    let new_host = Host {
        id,
        account_id: account_id.clone(),
        ip_address: host.ip_address.clone(),
        hostname: host.hostname.clone(),
    };

    diesel::insert_into(host::table)
        .values(new_host)
        .execute(&mut conn)?;
    Ok(())
}

pub fn get_host(host_id: &String) -> Result<Option<Host>, HostError> {
    if host_id.is_empty() {
        return Err(HostError::ValidationError("Host ID cannot be empty".to_string()));
    }

    let mut conn = establish_connection();
    Ok(host::table.find(host_id).first(&mut conn).optional()?)
}

pub fn get_all_hosts(account_id: &String) -> Result<Vec<Host>, HostError> {
    if account_id.is_empty() {
        return Err(HostError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let mut conn = establish_connection();
    Ok(host::table
        .filter(host::account_id.eq(account_id))
        .load::<Host>(&mut conn)?)
}

pub fn update_host(host: &Host) -> Result<(), HostError> {
    host.validate()?;

    let mut conn = establish_connection();
    diesel::update(host::table.find(&host.id))
        .set(host)
        .execute(&mut conn)?;
    Ok(())
}

pub fn delete_host(host_id: &String) -> Result<bool, HostError> {
    if host_id.is_empty() {
        return Err(HostError::ValidationError("Host ID cannot be empty".to_string()));
    }

    let mut conn = establish_connection();
    let num_deleted = diesel::delete(host::table.find(host_id))
        .execute(&mut conn)?;
    Ok(num_deleted > 0)
}

fn hostname_exists(account_id: &String, hostname: &String) -> Result<bool, HostError> {
    if account_id.is_empty() || hostname.is_empty() {
        return Err(HostError::ValidationError("Account ID and/or hostname cannot be empty".to_string()));
    }

    let mut conn = establish_connection();
    let count: i64 = host::table
        .filter(host::account_id.eq(account_id))
        .filter(host::hostname.eq(hostname))
        .count()
        .get_result(&mut conn)?;
    Ok(count > 0)
}