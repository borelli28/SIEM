use rusqlite::{Connection, Result};

pub struct Schema;

impl Schema {
    pub fn create_all(conn: &Connection) -> Result<()> {
        Self::create_accounts_table(conn)?;
        Self::create_alerts_table(conn)?;
        Self::create_hosts_table(conn)?;
        Self::create_rules_table(conn)?;
        Self::create_logs_table(conn)?;
        Ok(())
    }

    fn create_accounts_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                email TEXT UNIQUE NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
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

    fn create_rules_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS rules (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                description TEXT NOT NULL,
                references TEXT NOT NULL,
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
            )",
            [],
        )?;
        Ok(())
    }

    fn create_logs_table(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS logs (
                id TEXT PRIMARY KEY,
                host_id TEXT NOT NULL,
                account_id TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(host_id) REFERENCES hosts(id),
                FOREIGN KEY(account_id) REFERENCES accounts(id)
            )",
            [],
        )?;
        Ok(())
    }
}