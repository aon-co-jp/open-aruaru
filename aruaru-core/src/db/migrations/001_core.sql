-- VPS サーバー登録
CREATE TABLE IF NOT EXISTS vps_servers (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,           -- 表示名
    ip_address    TEXT NOT NULL,           -- IPアドレス（Webフォームで入力）
    port          INTEGER NOT NULL DEFAULT 22,
    username      TEXT NOT NULL DEFAULT 'root',
    auth_method   TEXT NOT NULL DEFAULT 'key',  -- 'key' or 'password'
    key_id        TEXT,                    -- ssh_keys.id への参照
    password_enc  TEXT,                    -- 暗号化パスワード（鍵認証推奨）
    status        TEXT NOT NULL DEFAULT 'unknown',  -- online/offline/unknown
    last_checked  DATETIME,
    notes         TEXT,
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- SSH 鍵ペア管理
CREATE TABLE IF NOT EXISTS ssh_keys (
    id            TEXT PRIMARY KEY,
    name          TEXT NOT NULL,           -- 表示名
    key_type      TEXT NOT NULL DEFAULT 'ed25519',  -- ed25519 / rsa
    public_key    TEXT NOT NULL,
    private_key   TEXT NOT NULL,           -- 本番では暗号化推奨
    fingerprint   TEXT NOT NULL,
    created_at    DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- SSH セッション履歴
CREATE TABLE IF NOT EXISTS ssh_sessions (
    id          TEXT PRIMARY KEY,
    vps_id      TEXT NOT NULL REFERENCES vps_servers(id),
    started_at  DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at    DATETIME,
    initiated_by TEXT NOT NULL DEFAULT 'user'  -- 'user' or 'ai'
);

-- SFTP 転送ログ
CREATE TABLE IF NOT EXISTS sftp_transfers (
    id           TEXT PRIMARY KEY,
    vps_id       TEXT NOT NULL REFERENCES vps_servers(id),
    direction    TEXT NOT NULL,   -- 'upload' or 'download'
    remote_path  TEXT NOT NULL,
    local_name   TEXT NOT NULL,
    size_bytes   INTEGER,
    status       TEXT NOT NULL DEFAULT 'pending',  -- pending/done/error
    initiated_by TEXT NOT NULL DEFAULT 'user',
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- AI 自動操作タスクキュー
CREATE TABLE IF NOT EXISTS ai_tasks (
    id           TEXT PRIMARY KEY,
    vps_id       TEXT NOT NULL,
    task_type    TEXT NOT NULL,   -- 'deploy' / 'setup' / 'command' / 'upload'
    payload      TEXT NOT NULL,   -- JSON
    status       TEXT NOT NULL DEFAULT 'pending',
    result       TEXT,
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at DATETIME
);

-- Anthropic API Key 管理（.env 不要、ブラウザから登録）
CREATE TABLE IF NOT EXISTS ai_credentials (
    id          TEXT PRIMARY KEY DEFAULT 'anthropic',
    api_key     TEXT NOT NULL DEFAULT '',
    plan        TEXT NOT NULL DEFAULT '',   -- free / pro / team / enterprise
    billing     TEXT NOT NULL DEFAULT '',   -- monthly / annual
    account     TEXT NOT NULL DEFAULT '',   -- メールアドレス
    verified    INTEGER NOT NULL DEFAULT 0,
    connected_at DATETIME
);
INSERT OR IGNORE INTO ai_credentials (id) VALUES ('anthropic');
