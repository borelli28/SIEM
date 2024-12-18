CREATE TABLE log_hosts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    log_id INTEGER NOT NULL,
    host_id TEXT NOT NULL,
    FOREIGN KEY (log_id) REFERENCES logs(id),
    FOREIGN KEY (host_id) REFERENCES hosts(id)
);