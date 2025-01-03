use rusqlite::{Error as SqliteError, params};
use crate::database::establish_connection;
use std::{fmt, net::IpAddr, str::FromStr};
use serde::{Serialize, Deserialize};
use rusqlite::OptionalExtension;
use uuid::Uuid;

#[derive(Debug)]
pub enum HostError {
    DatabaseError(SqliteError),
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

impl From<SqliteError> for HostError {
    fn from(err: SqliteError) -> Self {
        HostError::DatabaseError(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

    let conn = establish_connection()?;
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

    conn.execute(
        "INSERT INTO hosts (id, account_id, ip_address, hostname) VALUES (?1, ?2, ?3, ?4)",
        params![
            new_host.id,
            new_host.account_id,
            new_host.ip_address,
            new_host.hostname,
        ],
    )?;
    
    Ok(())
}

pub fn get_host(host_id: &String) -> Result<Option<Host>, HostError> {
    if host_id.is_empty() {
        return Err(HostError::ValidationError("Host ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, ip_address, hostname FROM hosts WHERE id = ?1"
    )?;

    let host = stmt.query_row(params![host_id], |row| {
        Ok(Host {
            id: row.get(0)?,
            account_id: row.get(1)?,
            ip_address: row.get(2)?,
            hostname: row.get(3)?,
        })
    }).optional()?;

    Ok(host)
}

pub fn get_all_hosts(account_id: &String) -> Result<Vec<Host>, HostError> {
    if account_id.is_empty() {
        return Err(HostError::ValidationError("Account ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT id, account_id, ip_address, hostname FROM hosts WHERE account_id = ?1"
    )?;

    let hosts_iter = stmt.query_map(params![account_id], |row| {
        Ok(Host {
            id: row.get(0)?,
            account_id: row.get(1)?,
            ip_address: row.get(2)?,
            hostname: row.get(3)?,
        })
    })?;

    let hosts: Result<Vec<Host>, SqliteError> = hosts_iter.collect();
    Ok(hosts?)
}

pub fn update_host(host: &Host) -> Result<(), HostError> {
    host.validate()?;

    let conn = establish_connection()?;
    conn.execute(
        "UPDATE hosts SET account_id = ?1, ip_address = ?2, hostname = ?3 WHERE id = ?4",
        params![
            host.account_id,
            host.ip_address,
            host.hostname,
            host.id,
        ],
    )?;

    Ok(())
}

pub fn delete_host(host_id: &String) -> Result<bool, HostError> {
    if host_id.is_empty() {
        return Err(HostError::ValidationError("Host ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "DELETE FROM hosts WHERE id = ?1",
        params![host_id],
    )?;

    Ok(affected_rows > 0)
}

fn hostname_exists(account_id: &String, hostname: &String) -> Result<bool, HostError> {
    if account_id.is_empty() || hostname.is_empty() {
        return Err(HostError::ValidationError("Account ID and/or hostname cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) FROM hosts WHERE account_id = ?1 AND hostname = ?2"
    )?;

    let count: i64 = stmt.query_row(params![account_id, hostname], |row| row.get(0))?;
    Ok(count > 0)
}