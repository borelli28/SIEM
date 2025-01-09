use rusqlite::{Connection, Result};
use log::{debug, info, error};

pub struct Schema;

impl Schema {
    pub fn create_all(conn: &Connection) -> Result<()> {
        debug!("Starting schema creation");

        info!("Creating accounts table");
        Self::create_accounts_table(conn)?;

        info!("Creating rules table");
        Self::create_rules_table(conn)?;

        info!("Creating hosts table");
        Self::create_hosts_table(conn)?;

        info!("Creating alerts table");
        Self::create_alerts_table(conn)?;

        info!("Creating logs table");
        Self::create_logs_table(conn)?;

        info!("Creating agents table");
        Self::create_agents_table(conn)?;

        info!("All tables created successfully");
        Ok(())
    }

    fn create_accounts_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                name TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                role TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        Ok(())
    }

    fn create_rules_table(conn: &Connection) -> Result<()> {        
        let sql = "CREATE TABLE IF NOT EXISTS rules (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            description TEXT NOT NULL,
            ref_list TEXT NOT NULL,
            tags TEXT NOT NULL,
            author TEXT NOT NULL,
            date TEXT NOT NULL,
            logsource TEXT NOT NULL,
            detection TEXT NOT NULL,
            fields TEXT NOT NULL,
            falsepositives TEXT NOT NULL,
            level TEXT NOT NULL,
            enabled BOOLEAN NOT NULL DEFAULT true,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(account_id) REFERENCES accounts(id)
        )";
        
        match conn.execute(sql, []) {
            Ok(_) => {
                info!("Rules table created successfully");
                Ok(())
            },
            Err(e) => {
                error!("Failed to create rules table: {}", e);
                Err(e)
            }
        }
    }

    fn create_hosts_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS hosts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                hostname TEXT NOT NULL,
                ip_address TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(account_id) REFERENCES accounts(id),
                UNIQUE(account_id, hostname),
                UNIQUE(account_id, ip_address)
            )",
            [],
        )?;
        Ok(())
    }

    fn create_alerts_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS alerts (
                id TEXT PRIMARY KEY,
                rule_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                acknowledged BOOLEAN NOT NULL DEFAULT false,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(rule_id) REFERENCES rules(id),
                FOREIGN KEY(account_id) REFERENCES accounts(id)
            )",
            [],
        )?;
        Ok(())
    }

    fn create_logs_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id TEXT PRIMARY KEY,
                hash TEXT NOT NULL UNIQUE,
                account_id TEXT NOT NULL,
                host_id TEXT NOT NULL,
                version TEXT,
                device_vendor TEXT,
                device_product TEXT,
                device_version TEXT,
                signature_id TEXT,
                name TEXT,
                severity TEXT,
                extensions TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(host_id) REFERENCES hosts(id),
                FOREIGN KEY(account_id) REFERENCES accounts(id)
            )",
            [],
        )?;
        Ok(())
    }

    fn create_agents_table(conn: &Connection) -> Result<()> {
        conn.execute(
            " CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                api_key TEXT NOT NULL UNIQUE,
                host_id TEXT NOT NULL UNIQUE,
                account_id TEXT NOT NULL,
                ip_address TEXT,
                hostname TEXT,
                status TEXT NOT NULL,
                last_seen DATETIME,
                FOREIGN KEY(host_id) REFERENCES hosts(id),
                FOREIGN KEY(account_id) REFERENCES accounts(id)
            );",
            [],
        )?;
        Ok(())
    }
}