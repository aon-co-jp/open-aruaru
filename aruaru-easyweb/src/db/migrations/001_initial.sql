CREATE TABLE IF NOT EXISTS users (
    id         TEXT PRIMARY KEY,
    email      TEXT NOT NULL UNIQUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_login DATETIME
);

CREATE TABLE IF NOT EXISTS login_tokens (
    id         TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id),
    token      TEXT NOT NULL UNIQUE,
    expires_at DATETIME NOT NULL,
    used       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS domains (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL UNIQUE,   -- bare domain (www なし)
    www_redirect    INTEGER NOT NULL DEFAULT 1,  -- www→bare 自動リダイレクト
    is_app_domain   INTEGER NOT NULL DEFAULT 0,
    webserver       TEXT NOT NULL DEFAULT 'nginx',
    proxy_port      INTEGER,                -- リバースプロキシ先ポート
    https_enabled   INTEGER NOT NULL DEFAULT 0,
    cert_expires_at DATETIME,
    gzip_enabled    INTEGER NOT NULL DEFAULT 1,
    created_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
