-- VPS サーバー管理テーブル
CREATE TABLE IF NOT EXISTS vps_servers (
    id             TEXT PRIMARY KEY,
    name           TEXT NOT NULL,           -- 表示名（例: "本番サーバー"）
    ip_address     TEXT NOT NULL,           -- IPアドレス
    ssh_port       INTEGER NOT NULL DEFAULT 22,
    ssh_user       TEXT NOT NULL DEFAULT 'root',
    auth_method    TEXT NOT NULL DEFAULT 'key',  -- "key" or "password"
    ssh_key_id     TEXT,                    -- ssh_keys テーブルの FK
    ssh_password   TEXT,                    -- パスワード認証の場合（暗号化推奨）
    status         TEXT NOT NULL DEFAULT 'unknown',  -- online/offline/unknown
    last_checked   DATETIME,
    created_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- SSH 鍵ペア管理テーブル
CREATE TABLE IF NOT EXISTS ssh_keys (
    id           TEXT PRIMARY KEY,
    name         TEXT NOT NULL,             -- 鍵の名前（例: "aruaru-key"）
    key_type     TEXT NOT NULL DEFAULT 'ed25519',  -- "ed25519" or "rsa"
    public_key   TEXT NOT NULL,             -- 公開鍵（OpenSSH形式）
    private_key  TEXT NOT NULL,             -- 秘密鍵（OpenSSH形式）
    fingerprint  TEXT NOT NULL,             -- フィンガープリント
    created_at   DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
