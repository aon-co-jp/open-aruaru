-- aruaru Ver 0.1c — SQLite 開発用スキーマ

CREATE TABLE IF NOT EXISTS schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS admin_users (
    email TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS magic_tokens (
    token_hash TEXT PRIMARY KEY,
    email TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    used INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS installed_plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    kind TEXT NOT NULL,
    installed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS domains (
    name TEXT PRIMARY KEY,
    https_enabled INTEGER NOT NULL DEFAULT 0,
    auto_https INTEGER NOT NULL DEFAULT 1,
    last_checked TEXT
);

CREATE TABLE IF NOT EXISTS platform_domains (
    name TEXT PRIMARY KEY,
    is_primary INTEGER NOT NULL DEFAULT 0,
    https_enabled INTEGER NOT NULL DEFAULT 0,
    auto_https INTEGER NOT NULL DEFAULT 1,
    last_checked TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS reminder_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
