CREATE TABLE logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    host TEXT,
    account_id TEXT NOT NULL,
    version TEXT,
    device_vendor TEXT,
    device_product TEXT,
    device_version TEXT,
    signature_id TEXT,
    name TEXT,
    severity TEXT,
    extensions TEXT,
    FOREIGN KEY (account_id) REFERENCES accounts(id)
);