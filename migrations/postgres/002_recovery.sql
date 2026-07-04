-- aruaru Ver 0.1d — OS-Recovery プラグイン用テーブル

CREATE TABLE IF NOT EXISTS recovery_footprints (
    id TEXT PRIMARY KEY,
    data_dir TEXT NOT NULL,
    app_dir TEXT NOT NULL,
    drive_letter TEXT,
    marker_path TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recovery_jobs (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    status TEXT NOT NULL,
    source_path TEXT,
    target_path TEXT,
    message TEXT,
    created_at TEXT NOT NULL,
    finished_at TEXT
);
