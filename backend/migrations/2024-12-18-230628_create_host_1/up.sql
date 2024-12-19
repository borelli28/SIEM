CREATE TABLE host (
    id TEXT PRIMARY KEY NOT NULL,
    account_id TEXT NOT NULL,
    ip_address TEXT,
    hostname TEXT,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);