CREATE TABLE alerts (
    id TEXT PRIMARY KEY NOT NULL,
    rule_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    acknowledged BOOL NOT NUll,
    created_at TEXT NOT NULL,
    FOREIGN KEY (rule_id) REFERENCES rules(id)
);