CREATE TABLE IF NOT EXISTS ai_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- API KEY 未登録フラグ
INSERT OR IGNORE INTO ai_settings (key, value) VALUES ('anthropic_api_key', '');
INSERT OR IGNORE INTO ai_settings (key, value) VALUES ('plan_type', 'free');
