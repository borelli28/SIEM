use sqlx::sqlite::SqlitePool;
use sqlx::{Result, query, query_as};
use uuid::Uuid;

#[derive(Debug)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub password: String,
}

impl Account {
    pub fn new(name: &str, password: &str) -> Self {
        Account {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            password: password.to_string(),
        }
    }
}

pub async fn create_account(pool: &SqlitePool, name: &str, password: &str) -> Result<Account> {
    let account = Account::new(name, password);
    query(
        "INSERT INTO accounts (id, name, password) VALUES (?, ?, ?)"
    )
    .bind(&account.id)
    .bind(&account.name)
    .bind(&account.password)
    .execute(pool)
    .await?;
    Ok(account)
}

pub async fn get_account(pool: &SqlitePool, id: &str) -> Result<Option<Account>> {
    let account = query_as::<sqlx::Sqlite, Account>(
        "SELECT id, name, password FROM accounts WHERE id = ?"
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

    if let Some(new_password) = password {
        query.push_str(if first { " password = ?" } else { ", password = ?" });
        first = false;
    }

    query.push_str(" WHERE id = ?");

    let mut query_builder = query(&query).bind(id);

    if let Some(new_name) = name {
        query_builder = query_builder.bind(new_name);
    }

    if let Some(new_password) = password {
        query_builder = query_builder.bind(new_password);
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