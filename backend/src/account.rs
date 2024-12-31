use crate::database::establish_connection;
use rusqlite::{Error as SqliteError, params};
use serde::{Serialize, Deserialize};
use actix_session::Session;
use actix_web::HttpRequest;
use regex::Regex;
use uuid::Uuid;
use std::fmt;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

#[derive(Debug)]
pub enum AccountError {
    InvalidRole,
    DatabaseError(SqliteError),
    PasswordHashError(String),
    ExpectedField(String),
    SessionError(String),
    ValidationError(String),
}

impl From<SqliteError> for AccountError {
    fn from(error: SqliteError) -> Self {
        AccountError::DatabaseError(error)
    }
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountError::InvalidRole => write!(f, "Invalid role provided"),
            AccountError::DatabaseError(err) => write!(f, "Database error: {}", err),
            AccountError::PasswordHashError(err) => write!(f, "Password hash error: {}", err),
            AccountError::ExpectedField(field) => write!(f, "Missing required field: {}", field),
            AccountError::SessionError(err) => write!(f, "Session Error: {}", err),
            AccountError::ValidationError(err) => write!(f, "Validation Error: {}", err),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub password: String,
    pub role: String,
}

impl Account {
    fn verify_password(&self, password: &String) -> bool {
        let parsed_hash = PasswordHash::new(&self.password).unwrap();
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }

    pub fn is_valid_role(role: &str) -> bool {
        matches!(role, "Admin" | "Analyst")
    }

    fn hash_password(password: &str) -> Result<String, AccountError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        argon2.hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| AccountError::PasswordHashError(e.to_string()))
    }

    fn validate_name(name: &str) -> Result<(), AccountError> {
        if name.len() < 3 || name.len() > 50 {
            return Err(AccountError::ValidationError("Name must be between 3 and 50 characters".to_string()));
        }
        let name_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
        if !name_regex.is_match(name) {
            return Err(AccountError::ValidationError("Name can only contain alphanumeric characters, underscores, and hyphens".to_string()));
        }
        Ok(())
    }

    fn validate_password(password: &str) -> Result<(), AccountError> {
        if password.len() < 15 {
            return Err(AccountError::ValidationError("Password must be at least 15 characters long".to_string()));
        }
        if !password.chars().any(|c| c.is_uppercase()) {
            return Err(AccountError::ValidationError("Password must contain at least one uppercase letter".to_string()));
        }
        if !password.chars().any(|c| c.is_lowercase()) {
            return Err(AccountError::ValidationError("Password must contain at least one lowercase letter".to_string()));
        }
        if !password.chars().any(|c| c.is_digit(10)) {
            return Err(AccountError::ValidationError("Password must contain at least one number".to_string()));
        }
        Ok(())
    }
}

pub fn create_account(name: String, password: String, role: String) -> Result<usize, AccountError> {
    let conn = establish_connection()?;
    let id = Uuid::new_v4().to_string();

    Account::validate_name(&name)?;
    Account::validate_password(&password)?;

    if !Account::is_valid_role(&role) {
        return Err(AccountError::InvalidRole);
    }

    if account_exists(&name)? {
        return Err(AccountError::ValidationError(format!("Account name '{}' already exists.", name)));
    }

    let hashed_password = Account::hash_password(&password)?;

    conn.execute(
        "INSERT INTO accounts (id, name, password, role) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, hashed_password, role],
    ).map_err(AccountError::from)
}

pub fn get_account(id: &String) -> Result<Option<Account>, AccountError> {
    if id.is_empty() {
        return Err(AccountError::ValidationError("Account ID cannot be empty".to_string()));
    }
    
    let conn = establish_connection()?;
    let mut stmt = conn.prepare("SELECT id, name, password, role FROM accounts WHERE id = ?1")?;
    
    let account = stmt.query_row(params![id], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            password: row.get(2)?,
            role: row.get(3)?,
        })
    }).optional()?;

    Ok(account)
}

pub fn update_account(account: &Account) -> Result<bool, AccountError> {
    if account.id.is_empty() {
        return Err(AccountError::ExpectedField("id".to_string()));
    }
    Account::validate_name(&account.name)?;
    Account::validate_password(&account.password)?;
    if !Account::is_valid_role(&account.role) {
        return Err(AccountError::InvalidRole);
    }

    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "UPDATE accounts SET name = ?1, password = ?2, role = ?3 WHERE id = ?4",
        params![account.name, account.password, account.role, account.id],
    )?;

    Ok(affected_rows > 0)
}

pub fn delete_account(id: &String) -> Result<bool, AccountError> {
    if id.is_empty() {
        return Err(AccountError::ValidationError("Account ID cannot be empty".to_string()));
    }
    
    let conn = establish_connection()?;
    let affected_rows = conn.execute(
        "DELETE FROM accounts WHERE id = ?1",
        params![id],
    )?;

    Ok(affected_rows > 0)
}

pub fn verify_login(session: &Session, name: &String, password: &String, req: &HttpRequest) -> Result<Option<Account>, AccountError> {
    Account::validate_name(name)?;
    Account::validate_password(password)?;

    let conn = establish_connection()?;
    let mut stmt = conn.prepare("SELECT id, name, password, role FROM accounts WHERE name = ?1")?;
    
    let account = stmt.query_row(params![name], |row| {
        Ok(Account {
            id: row.get(0)?,
            name: row.get(1)?,
            password: row.get(2)?,
            role: row.get(3)?,
        })
    }).optional()?;

    if let Some(account) = account {
        if account.verify_password(password) {
            // Store account ID
            session.insert("account_id", account.id.clone())
                .map_err(|e| AccountError::SessionError(e.to_string()))?;

            let user_agent = req.headers().get(actix_web::http::header::USER_AGENT)
                .and_then(|h| h.to_str().ok())
                .unwrap_or("unknown")
                .to_string();
            session.insert("user_agent", user_agent)
                .map_err(|e| AccountError::SessionError(e.to_string()))?;

            // Store last activity time
            session.insert("last_activity", std::time::SystemTime::now())
                .map_err(|e| AccountError::SessionError(e.to_string()))?;
            session.renew();
            return Ok(Some(account)); // Login successful
        } else {
            return Ok(None); // Password incorrect
        }
    }

    Ok(None) // No account found with the provided name
}

fn account_exists(name: &String) -> Result<bool, AccountError> {
    Account::validate_name(name)?;
    let conn = establish_connection()?;
    
    let mut stmt = conn.prepare("SELECT COUNT(*) FROM accounts WHERE name = ?1")?;
    let count: i64 = stmt.query_row(params![name], |row| row.get(0))?;

    Ok(count > 0)
}