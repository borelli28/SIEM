use rusqlite::{Error as SqliteError, OptionalExtension, params};
use crate::database::establish_connection;
use std::{fmt, net::IpAddr, str::FromStr};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub enum AgentError {
    DatabaseError(SqliteError),
    ValidationError(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AgentError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl From<SqliteError> for AgentError {
    fn from(err: SqliteError) -> Self {
        AgentError::DatabaseError(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub api_key: String,
    pub host_id: String,
    pub account_id: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
    pub status: AgentStatus,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Error,
}

impl Agent {
    fn validate(&self) -> Result<(), AgentError> {
        if self.account_id.is_empty() {
            return Err(AgentError::ValidationError("Account ID cannot be empty".to_string()));
        }

        if self.host_id.is_empty() {
            return Err(AgentError::ValidationError("Host ID cannot be empty".to_string()));
        }

        if let Some(ref ip) = self.ip_address {
            if IpAddr::from_str(ip).is_err() {
                return Err(AgentError::ValidationError(format!("Invalid IP address: {}", ip)));
            }
        }

        Ok(())
    }
}

pub fn register_agent(agent: &Agent) -> Result<(String, String), AgentError> {
    agent.validate()?;

    let conn = establish_connection()?;
    let id = Uuid::new_v4().to_string();
    let api_key = Uuid::new_v4().to_string();

    if agent_exists(&agent.host_id)? {
        return Err(AgentError::ValidationError(
            format!("An agent for host '{}' already exists.", agent.host_id)
        ));
    }

    let new_agent = Agent {
        id: id.clone(),
        api_key: api_key.clone(),
        host_id: agent.host_id.clone(),
        account_id: agent.account_id.clone(),
        ip_address: agent.ip_address.clone(),
        hostname: agent.hostname.clone(),
        status: AgentStatus::Active,
        last_seen: Some(Utc::now()),
    };

    conn.execute(
        "INSERT INTO agents (id, api_key, host_id, account_id, ip_address, hostname, status, last_seen) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            new_agent.id,
            new_agent.api_key,
            new_agent.host_id,
            new_agent.account_id,
            new_agent.ip_address,
            new_agent.hostname,
            "Active",
            new_agent.last_seen.map(|dt| dt.to_rfc3339()),
        ],
    )?;

    Ok((id, api_key))
}

pub fn verify_agent_api_key(api_key: &str) -> Result<bool, AgentError> {
    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT status FROM agents WHERE api_key = ?1 AND status = 'Active'"
    )?;

    let result = stmt.query_row(
        params![api_key],
        |_| Ok(true)
    ).optional()?;

    Ok(result.unwrap_or(false))
}

fn agent_exists(host_id: &String) -> Result<bool, AgentError> {
    if host_id.is_empty() {
        return Err(AgentError::ValidationError("Host ID cannot be empty".to_string()));
    }

    let conn = establish_connection()?;
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) FROM agents WHERE host_id = ?1"
    )?;

    let count: i64 = stmt.query_row(params![host_id], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn update_agent_last_seen(agent_id: &str) -> Result<(), AgentError> {
    let conn = establish_connection()?;
    conn.execute(
        "UPDATE agents SET last_seen = ?1 WHERE id = ?2",
        params![Utc::now().to_rfc3339(), agent_id],
    )?;
    Ok(())
}