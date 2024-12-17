use sqlx::sqlite::SqlitePool;
use sqlx::{Result};

pub async fn initialize_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(&format!("file:{}", db_path)).await?;
    init_db(&pool).await?;
    Ok(pool)
}

async fn init_db(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS accounts (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            password TEXT NOT NULL
        )"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_id TEXT NOT NULL,

            version TEXT,
            device_vendor TEXT,
            device_product TEXT,
            device_version TEXT,
            signature_id TEXT,
            name TEXT,
            severity TEXT,
            extensions TEXT,
            FOREIGN KEY (account_id) REFERENCES accounts(id)
        )"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS alert_rules (
            id TEXT PRIMARY KEY NOT NULL,
            account_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            condition TEXT NOT NULL,
            severity TEXT NOT NULL,
            enabled BOOLEAN NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (account_id) REFERENCES accounts(id)
        )"
    )
    .execute(pool)
    .await?;

    Ok(())
}