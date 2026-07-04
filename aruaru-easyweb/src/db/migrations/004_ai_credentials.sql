-- Anthropic API Key 管理（ブラウザから登録・.env 不要）
CREATE TABLE IF NOT EXISTS ai_credentials (
    id           TEXT PRIMARY KEY DEFAULT 'anthropic',
    api_key      TEXT NOT NULL DEFAULT '',
    plan         TEXT NOT NULL DEFAULT '',   -- free / pro / team / enterprise
    billing      TEXT NOT NULL DEFAULT '',   -- monthly / annual / —
    account      TEXT NOT NULL DEFAULT '',   -- メールアドレス
    verified     INTEGER NOT NULL DEFAULT 0,
    connected_at DATETIME
);
INSERT OR IGNORE INTO ai_credentials (id) VALUES ('anthropic');
