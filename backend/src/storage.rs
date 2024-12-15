use sqlx::sqlite::SqlitePool;
use sqlx::{Pool, Result};
use crate::collector::LogEntry;
use std::sync::Arc;

pub struct Storage {
    pool: Arc<Pool<sqlx::Sqlite>>,
}

impl Storage {
    pub fn new(pool: SqlitePool) -> Self {
        Storage {
            pool: Arc::new(pool),
        }
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
        .execute(self.pool.as_ref())
        .await?;
        Ok(())
    }
}