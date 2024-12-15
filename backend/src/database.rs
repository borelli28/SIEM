use sqlx::sqlite::SqlitePool;
use sqlx::{Result};

pub async fn initialize_db(db_path: &str) -> Result<SqlitePool> {
    let pool = SqlitePool::connect(&format!("file:{}", db_path)).await?;
    init_db(&pool).await?;
    Ok(pool)
}

async fn init_db(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            version TEXT,
            device_vendor TEXT,
            device_product TEXT,
            device_version TEXT,
            signature_id TEXT,
            name TEXT,
            severity TEXT,
            extensions TEXT
        )"
    )
    .execute(pool)
    .await?;

    Ok(())
}
