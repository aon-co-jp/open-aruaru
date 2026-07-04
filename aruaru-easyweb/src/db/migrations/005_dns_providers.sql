-- DNS プロバイダー設定
CREATE TABLE IF NOT EXISTS dns_providers (
    id         TEXT PRIMARY KEY,
    provider   TEXT NOT NULL,  -- cloudflare / conoha / value_domain
    label      TEXT NOT NULL,
    api_token  TEXT NOT NULL DEFAULT '',
    api_key    TEXT NOT NULL DEFAULT '',
    email      TEXT NOT NULL DEFAULT '',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
