CREATE TABLE alerts (
    id TEXT PRIMARY KEY,
    rule_id TEXT NOT NULL,
    account_id TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES alert_rules(id)
);