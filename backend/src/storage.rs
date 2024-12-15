use sqlx::sqlite::SqlitePool;
use sqlx::{Pool, Result};
use crate::collector::LogEntry;
use std::sync::Arc;

pub struct Storage {
    pool: Arc<Pool<sqlx::Sqlite>>
}

impl Storage {
    pub async fn new(db_path: &str) -> Result<Self> {
        let pool = SqlitePool::connect(&format!("sqlite://{}", db_path)).await?;
        let storage = Storage {
            pool: Arc::new(pool),
        };
        storage.init_db().await?;
        Ok(storage)
    }

    async fn init_db(&self) -> Result<()> {
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
        .execute(self.pool.as_ref()).await?;
        Ok(())
    }

    pub async fn insert_log(&self, log: &LogEntry) -> Result<()> {
        let extensions_json = serde_json::to_string(&log.extensions).unwrap_or_default();
        sqlx::query(
            "INSERT INTO logs (version, device_vendor, device_product, device_version, signature_id, name, severity, extensions)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&log.version)
        .bind(&log.device_vendor)
        .bind(&log.device_product)
        .bind(&log.device_version)
        .bind(&log.signature_id)
        .bind(&log.name)
        .bind(&log.severity)
        .bind(extensions_json)
        .execute(self.pool.as_ref()).await?;
        Ok(())
    }
}