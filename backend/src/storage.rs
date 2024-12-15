use rusqlite::{params, Connection, Result};
use crate::collector::LogEntry;
use std::sync::Mutex;

pub struct Storage {
    conn: Mutex<Connection>
}

impl Storage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let storage = Storage { conn: conn.into() };
        storage.init_db()?;
        Ok(storage)
    }

    fn init_db(&self) -> Result<()> {
        self.conn.lock().unwrap().execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY,
                version TEXT,
                device_vendor TEXT,
                device_product TEXT,
                device_version TEXT,
                signature_id TEXT,
                name TEXT,
                severity TEXT,
                extensions TEXT
            )",
            [],
        )?;
        Ok(())
    }

    pub fn insert_log(&self, log: &LogEntry) -> Result<()> {
        let extensions_json = serde_json::to_string(&log.extensions).unwrap_or_default();
        self.conn.lock().unwrap().execute(
            "INSERT INTO logs (version, device_vendor, device_product, device_version, signature_id, name, severity, extensions)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                log.version,
                log.device_vendor,
                log.device_product,
                log.device_version,
                log.signature_id,
                log.name,
                log.severity,
                extensions_json,
            ],
        )?;
        Ok(())
    }
}