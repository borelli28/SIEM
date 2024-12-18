CREATE TABLE alert_rules (
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
);