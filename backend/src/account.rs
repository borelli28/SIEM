use crate::database::establish_connection;
use diesel::result::Error as DieselError;
use serde::{Serialize, Deserialize};
use crate::schema::accounts;
use diesel::prelude::*;
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
    DieselError(DieselError),
    PasswordHashError(String),
    ExpectedField(String),
}

impl From<DieselError> for AccountError {
    fn from(error: DieselError) -> Self {
        AccountError::DieselError(error)
    }
}

impl fmt::Display for AccountError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccountError::InvalidRole => write!(f, "Invalid role provided"),
            AccountError::DieselError(err) => write!(f, "Database error: {}", err),
            AccountError::PasswordHashError(err) => write!(f, "Password hash error: {}", err),
            AccountError::ExpectedField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

#[derive(Queryable, Insertable, AsChangeset, Debug, Serialize, Deserialize)]
#[diesel(table_name = accounts)]
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
}

pub fn create_account(name: String, password: String, role: String) -> Result<usize, AccountError> {
    let mut conn = establish_connection();
    let id = Uuid::new_v4().to_string();

    if !Account::is_valid_role(&role) {
        return Err(AccountError::InvalidRole);
    }

    let hashed_password = Account::hash_password(&password)?;

    let new_account = Account {
        id,
        name,
        password: hashed_password,
        role,
    };

    diesel::insert_into(accounts::table)
        .values(&new_account)
        .execute(&mut conn)
        .map_err(AccountError::from)
}

pub fn get_account(id: &String) -> Result<Option<Account>, AccountError> {
    let mut conn = establish_connection();
    accounts::table.filter(accounts::id.eq(id))
        .first(&mut conn)
        .optional()
        .map_err(AccountError::from)
}

pub fn update_account(account: &Account) -> Result<bool, AccountError> {
    if account.id.is_empty() {
        return Err(AccountError::ExpectedField("id".to_string()));
    }
    if account.name.is_empty() {
        return Err(AccountError::ExpectedField("name".to_string()));
    }
    if account.password.is_empty() {
        return Err(AccountError::ExpectedField("password".to_string()));
    }
    if account.role.is_empty() {
        return Err(AccountError::ExpectedField("role".to_string()));
    }
    if !Account::is_valid_role(&account.role) {
        return Err(AccountError::InvalidRole);
    }

    let mut conn = establish_connection();
    let affected_rows = diesel::update(accounts::table.find(&account.id))
        .set(account)
        .execute(&mut conn)?;
    Ok(affected_rows > 0)
}

pub fn delete_account(id: &String) -> Result<bool, AccountError> {
    let mut conn = establish_connection();
    let affected_rows = diesel::delete(accounts::table.filter(accounts::id.eq(id)))
        .execute(&mut conn)?;
    Ok(affected_rows > 0)
}

pub fn verify_login(name: &String, password: &String) -> Result<Option<Account>, AccountError> {
    let mut conn = establish_connection();

    let account: Option<Account> = accounts::table
        .filter(accounts::name.eq(name))
        .first(&mut conn)
        .optional()?;

    if let Some(account) = account {
        if account.verify_password(password) {
            Ok(Some(account))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}