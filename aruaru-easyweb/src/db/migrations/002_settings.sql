-- アプリ設定をDBに保存（VI/.env操作不要にする）
CREATE TABLE IF NOT EXISTS app_settings (
    key        TEXT PRIMARY KEY,
    value      TEXT NOT NULL,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- セットアップ完了フラグ（0=未完了、1=完了）
INSERT OR IGNORE INTO app_settings (key, value) VALUES ('setup_completed', '0');
