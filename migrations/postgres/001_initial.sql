-- aruaru Ver 0.1c — PostgreSQL 初期スキーマ（SQLite 互換 TEXT 日時）

CREATE TABLE IF NOT EXISTS schema_migrations (
    version BIGINT PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
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
    used SMALLINT NOT NULL DEFAULT 0
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
    https_enabled SMALLINT NOT NULL DEFAULT 0,
    auto_https SMALLINT NOT NULL DEFAULT 1,
    last_checked TEXT
);

CREATE TABLE IF NOT EXISTS platform_domains (
    name TEXT PRIMARY KEY,
    is_primary SMALLINT NOT NULL DEFAULT 0,
    https_enabled SMALLINT NOT NULL DEFAULT 0,
    auto_https SMALLINT NOT NULL DEFAULT 1,
    last_checked TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS reminder_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
