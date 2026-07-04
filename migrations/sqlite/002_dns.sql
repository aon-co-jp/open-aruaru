-- aruaru-dns v0.1

CREATE TABLE IF NOT EXISTS dns_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    zone TEXT NOT NULL,
    name TEXT NOT NULL,
    record_type TEXT NOT NULL,
    value TEXT NOT NULL,
    ttl INTEGER NOT NULL DEFAULT 3600,
    created_at TEXT NOT NULL,
    UNIQUE (zone, name, record_type, value)
);

CREATE INDEX IF NOT EXISTS idx_dns_records_zone ON dns_records (zone);
