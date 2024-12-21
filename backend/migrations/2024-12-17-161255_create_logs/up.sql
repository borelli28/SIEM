CREATE TABLE logs (
    id TEXT PRIMARY KEY NOT NULL,
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
    FOREIGN KEY (account_id) REFERENCES accounts(id),
    FOREIGN KEY (host_id) REFERENCES host(id)
);