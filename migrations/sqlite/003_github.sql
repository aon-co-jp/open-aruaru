-- GitHub プロジェクト連携テーブル
CREATE TABLE IF NOT EXISTS github_projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gh_id INTEGER NOT NULL UNIQUE,
    name TEXT NOT NULL,
    full_name TEXT NOT NULL,
    description TEXT,
    html_url TEXT NOT NULL,
    language TEXT,
    stargazers_count INTEGER NOT NULL DEFAULT 0,
    updated_at_gh TEXT NOT NULL,
    selected INTEGER NOT NULL DEFAULT 0,
    readme_content TEXT,
    ext_summary TEXT,          -- JSON: [["rs",42],["ts",18],...]
    saved_at TEXT NOT NULL,
    readme_fetched_at TEXT     -- README 最終取得日時
);

-- PPTX 生成ジョブの履歴
CREATE TABLE IF NOT EXISTS pptx_jobs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    gh_ids TEXT NOT NULL,          -- JSON配列 e.g. "[123,456]"
    status TEXT NOT NULL DEFAULT 'pending',  -- pending | running | done | error
    filename TEXT,
    error_msg TEXT,
    created_at TEXT NOT NULL,
    finished_at TEXT
);
