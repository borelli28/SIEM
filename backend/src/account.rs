use sqlx::{Result, query, query_as};
use sqlx::sqlite::SqlitePool;
use uuid::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2.hash_password(password.as_bytes(), &salt).map(|hash| hash.to_string())
}

#[derive(Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub password_hash: String,
}

impl Account {
    pub fn new(name: &str, password: &str) -> Result<Self, argon2::password_hash::Error> {
        let password_hash = hash_password(password)?;

        Ok(Account {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            password_hash,
        })
    }

    pub fn verify_password(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(&self.password_hash).unwrap();
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
}

pub async fn create_account(pool: &SqlitePool, name: &str, password: &str) -> Result<Account> {
    let account = Account::new(name, password).map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
    query(
        "INSERT INTO accounts (id, name, password_hash) VALUES (?, ?, ?)"
    )
    .bind(&account.id)
    .bind(&account.name)
    .bind(&account.password_hash)
    .execute(pool)
    .await?;
    Ok(account)
}

pub async fn get_account(pool: &SqlitePool, id: &str) -> Result<Option<Account>> {
    let account = query_as::<sqlx::Sqlite, Account>(
        "SELECT id, name, password_hash FROM accounts WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(account)
}

pub async fn update_account(pool: &SqlitePool, id: &str, name: Option<&str>, password: Option<&str>) -> Result<bool> {
    let mut query = String::from("UPDATE accounts SET");
    let mut first = true;

    if let Some(new_name) = name {
        query.push_str(if first { " name = ?" } else { ", name = ?" });
        first = false;
    }

    if password.is_some() {
        query.push_str(if first { " password_hash = ?" } else { ", password_hash = ?" });
        first = false;
    }

    query.push_str(" WHERE id = ?");

    let mut query_builder = query(&query).bind(id);

    if let Some(new_name) = name {
        query_builder = query_builder.bind(new_name);
    }

    if let Some(new_password) = password {
        let password_hash = hash_password(new_password)
            .map_err(|e| sqlx::Error::Protocol(e.to_string()))?;
        query_builder = query_builder.bind(password_hash);
    }

    let rows_affected = query_builder.execute(pool).await?.rows_affected();
    Ok(rows_affected > 0)
}

pub async fn delete_account(pool: &SqlitePool, id: &str) -> Result<bool> {
    let result = query("DELETE FROM accounts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn verify_login(pool: &SqlitePool, name: &str, password: &str) -> Result<Option<Account>> {
    let account = query_as::<sqlx::Sqlite, Account>(
        "SELECT id, name, password_hash FROM accounts WHERE name = ?"
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

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