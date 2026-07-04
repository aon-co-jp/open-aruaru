//! Nginx / Apache 選択アドバイザー
//! - 言語・フレームワーク別の推薦ロジック
//! - 特徴・設定難易度・設定ファイル運用の日英解説
//! - AI 自動設定（vhost テンプレート生成）

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────
// 1. データ型
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebServerInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub summary_ja: &'static str,
    pub summary_en: &'static str,
    pub config_difficulty_ja: &'static str,
    pub config_difficulty_en: &'static str,
    pub config_file_ops_ja: &'static str,
    pub config_file_ops_en: &'static str,
    pub strengths_ja: Vec<&'static str>,
    pub strengths_en: Vec<&'static str>,
    pub weaknesses_ja: Vec<&'static str>,
    pub weaknesses_en: Vec<&'static str>,
    pub config_example: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LangFrameworkEntry {
    pub id: &'static str,
    pub label_ja: &'static str,
    pub label_en: &'static str,
    pub recommended_server: &'static str, // "nginx" | "apache" | "either"
    pub reason_ja: &'static str,
    pub reason_en: &'static str,
    pub compat_nginx: CompatLevel,
    pub compat_apache: CompatLevel,
    pub tips_ja: &'static str,
    pub tips_en: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatLevel {
    Excellent,
    Good,
    Fair,
    Limited,
}

impl CompatLevel {
    pub fn label_ja(self) -> &'static str {
        match self {
            Self::Excellent => "◎ 最適",
            Self::Good => "○ 良好",
            Self::Fair => "△ 可",
            Self::Limited => "✕ 不向き",
        }
    }
    pub fn label_en(self) -> &'static str {
        match self {
            Self::Excellent => "Excellent",
            Self::Good => "Good",
            Self::Fair => "Fair",
            Self::Limited => "Limited",
        }
    }
    pub fn score(self) -> u8 {
        match self {
            Self::Excellent => 4,
            Self::Good => 3,
            Self::Fair => 2,
            Self::Limited => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Recommendation {
    pub recommended: &'static str,
    pub reason_ja: String,
    pub reason_en: String,
    pub nginx_compat_ja: &'static str,
    pub nginx_compat_en: &'static str,
    pub apache_compat_ja: &'static str,
    pub apache_compat_en: &'static str,
    pub tips_ja: String,
    pub tips_en: String,
    pub config_preview: String,
}

// ─────────────────────────────────────────────────────────
// 2. Nginx / Apache 基本情報
// ─────────────────────────────────────────────────────────

pub fn nginx_info() -> WebServerInfo {
    WebServerInfo {
        id: "nginx",
        name: "Nginx",
        summary_ja: "高速・軽量なイベント駆動型 Web サーバー。リバースプロキシ・ロードバランサーとして特に強力。",
        summary_en: "High-performance, event-driven web server. Excels as a reverse proxy and load balancer.",
        config_difficulty_ja: "中級（設定ブロック構文を覚えると直感的）",
        config_difficulty_en: "Intermediate — block syntax is intuitive once learned.",
        config_file_ops_ja: "単一ファイルまたは sites-available / sites-enabled パターン。\
            `nginx -t` でテスト、`systemctl reload nginx` で無停止反映。\
            `include` ディレクティブでドメインごとに分割管理が容易。",
        config_file_ops_en: "Single file or sites-available/sites-enabled pattern. \
            Test with `nginx -t`, apply with `systemctl reload nginx`. \
            Use `include` directives to split per-domain configs cleanly.",
        strengths_ja: vec![
            "静的ファイルの配信が極めて高速",
            "高並列接続を少ないメモリで処理",
            "リバースプロキシ・WebSocket に最適",
            "HTTP/2・HTTP/3 の標準対応",
            "設定ファイルが読みやすくコンパクト",
        ],
        strengths_en: vec![
            "Extremely fast static file serving",
            "Handles high concurrency with low memory",
            "Best-in-class reverse proxy and WebSocket support",
            "Native HTTP/2 and HTTP/3 support",
            "Clean, readable config files",
        ],
        weaknesses_ja: vec![
            ".htaccess 非対応（セキュリティ上は利点でもある）",
            "PHP は php-fpm 経由が必須",
            "動的な設定変更は reload が必要",
        ],
        weaknesses_en: vec![
            "No .htaccess support (often a security advantage)",
            "PHP requires php-fpm",
            "Config changes need a reload",
        ],
        config_example: r#"server {
    listen 80;
    server_name example.com www.example.com;

    # アプリへのリバースプロキシ
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # 静的ファイル直接配信
    location /static/ {
        alias /var/www/myapp/static/;
        expires 30d;
        add_header Cache-Control "public, immutable";
    }

    # ACME チャレンジ (Let's Encrypt)
    location /.well-known/acme-challenge/ {
        root /var/www/acme;
    }
}"#,
    }
}

pub fn apache_info() -> WebServerInfo {
    WebServerInfo {
        id: "apache",
        name: "Apache HTTP Server",
        summary_ja: "世界シェアトップクラスの老舗 Web サーバー。.htaccess によるディレクトリ単位の設定が特徴。",
        summary_en: "World's most widely deployed web server. Known for per-directory .htaccess configuration.",
        config_difficulty_ja: "初〜中級（.htaccess で手軽に設定、mod_* モジュールが豊富）",
        config_difficulty_en: "Beginner to Intermediate — .htaccess makes quick changes easy; rich mod_* ecosystem.",
        config_file_ops_ja: "sites-available / sites-enabled パターン（Debian 系）。\
            `a2ensite` / `a2dissite` コマンドでサイト有効化。\
            .htaccess でディレクトリごとに上書き可能（共有ホスティング向け）。\
            `apachectl configtest` でテスト後 `systemctl reload apache2`。",
        config_file_ops_en: "sites-available/sites-enabled on Debian. \
            Enable sites with `a2ensite`. \
            .htaccess allows per-directory overrides — great for shared hosting. \
            Test with `apachectl configtest`, apply with `systemctl reload apache2`.",
        strengths_ja: vec![
            ".htaccess でディレクトリ単位の設定が可能",
            "mod_php で PHP をシンプルに動作",
            "mod_rewrite による URL 書き換えが強力",
            "モジュールが豊富（mod_ssl, mod_security 等）",
            "WordPress など PHP 系 CMS と最も相性が良い",
        ],
        strengths_en: vec![
            "Per-directory config via .htaccess",
            "mod_php makes PHP setup trivial",
            "Powerful URL rewriting with mod_rewrite",
            "Rich module ecosystem (mod_ssl, mod_security, etc.)",
            "Best compatibility with PHP CMS like WordPress",
        ],
        weaknesses_ja: vec![
            "高負荷時のメモリ使用量が Nginx より多い",
            "WebSocket プロキシは設定が複雑",
            ".htaccess の多用は パフォーマンスに影響",
        ],
        weaknesses_en: vec![
            "Higher memory usage under load compared to Nginx",
            "WebSocket proxying is more complex",
            "Excessive .htaccess use can hurt performance",
        ],
        config_example: r#"<VirtualHost *:80>
    ServerName example.com
    ServerAlias www.example.com

    # アプリへのリバースプロキシ
    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:3000/
    ProxyPassReverse / http://127.0.0.1:3000/

    # ヘッダー設定
    RequestHeader set X-Forwarded-Proto "http"
    RequestHeader set X-Real-IP %{REMOTE_ADDR}e

    # 静的ファイル
    Alias /static/ /var/www/myapp/static/
    <Directory /var/www/myapp/static/>
        Options -Indexes
        AllowOverride None
        Require all granted
        ExpiresActive On
        ExpiresDefault "access plus 30 days"
    </Directory>

    ErrorLog ${APACHE_LOG_DIR}/myapp_error.log
    CustomLog ${APACHE_LOG_DIR}/myapp_access.log combined
</VirtualHost>"#,
    }
}

// ─────────────────────────────────────────────────────────
// 3. 言語 / フレームワーク データ
// ─────────────────────────────────────────────────────────

pub fn lang_framework_entries() -> Vec<LangFrameworkEntry> {
    vec![
        LangFrameworkEntry {
            id: "rust-poem",
            label_ja: "Rust / Poem",
            label_en: "Rust / Poem",
            recommended_server: "nginx",
            reason_ja: "Poem はポート直接バインドするので Nginx リバースプロキシが最適。\
                高並列を低メモリで捌く Nginx と Rust の組み合わせは最強構成。",
            reason_en: "Poem binds directly to a port; Nginx reverse proxy is ideal. \
                Nginx + Rust is an elite pairing for high concurrency with minimal memory.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "proxy_pass http://127.0.0.1:3000; を設定し、\
                X-Forwarded-Proto ヘッダーを渡してください。\
                WebSocket を使う場合は proxy_http_version 1.1; も必要です。",
            tips_en: "Set `proxy_pass http://127.0.0.1:3000;` and pass X-Forwarded-Proto. \
                Add `proxy_http_version 1.1;` if using WebSockets.",
        },
        LangFrameworkEntry {
            id: "rust-axum",
            label_ja: "Rust / Axum",
            label_en: "Rust / Axum",
            recommended_server: "nginx",
            reason_ja: "Axum も TCP バインド型。Nginx でのリバースプロキシが定番構成。",
            reason_en: "Axum also binds TCP directly. Nginx reverse proxy is the standard setup.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "Poem と同様に proxy_pass でポート転送。\
                Tower ミドルウェアで IP 取得する場合は X-Forwarded-For の転送を忘れずに。",
            tips_en: "Same as Poem: proxy_pass to the bound port. \
                Remember to forward X-Forwarded-For for Tower IP-extraction middleware.",
        },
        LangFrameworkEntry {
            id: "php-laravel",
            label_ja: "PHP / Laravel",
            label_en: "PHP / Laravel",
            recommended_server: "nginx",
            reason_ja: "Laravel 公式ドキュメントが Nginx を推奨。php-fpm との組み合わせで高速。\
                .env ファイルの保護も Nginx の方が設定しやすい。",
            reason_en: "Laravel's official docs recommend Nginx with php-fpm for best performance. \
                Protecting .env is simpler with Nginx location blocks.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "try_files $uri $uri/ /index.php?$query_string; が Laravel の基本設定。\
                php-fpm のソケットパスを確認してください（/var/run/php/php8.x-fpm.sock）。",
            tips_en: "Key directive: `try_files $uri $uri/ /index.php?$query_string;`. \
                Match php-fpm socket path (/var/run/php/php8.x-fpm.sock).",
        },
        LangFrameworkEntry {
            id: "php-wordpress",
            label_ja: "PHP / WordPress",
            label_en: "PHP / WordPress",
            recommended_server: "apache",
            reason_ja: "WordPress は .htaccess による URL 書き換えを標準利用。\
                mod_php / mod_rewrite との組み合わせが最も簡単で互換性が高い。",
            reason_en: "WordPress relies on .htaccess for URL rewriting by default. \
                mod_php + mod_rewrite is the simplest and most compatible setup.",
            compat_nginx: CompatLevel::Good,
            compat_apache: CompatLevel::Excellent,
            tips_ja: "Apache: AllowOverride All で .htaccess を有効化。\
                Nginx を選ぶ場合は WordPress 用の try_files ルールを手動追加してください。",
            tips_en: "Apache: set AllowOverride All to enable .htaccess. \
                If using Nginx, manually add WordPress try_files rules.",
        },
        LangFrameworkEntry {
            id: "python-django",
            label_ja: "Python / Django",
            label_en: "Python / Django",
            recommended_server: "nginx",
            reason_ja: "Django + Gunicorn/uvicorn を Nginx でリバースプロキシするのが本番標準。\
                静的ファイルは collectstatic → Nginx 直接配信で高速化。",
            reason_en: "Django + Gunicorn/uvicorn behind Nginx is the production standard. \
                Serve static files directly from Nginx after `collectstatic`.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "upstream { server 127.0.0.1:8000; } でバックエンドを定義し、\
                /static/ は alias /var/www/myapp/staticfiles/; で直接配信。",
            tips_en: "Define `upstream { server 127.0.0.1:8000; }` and serve \
                /static/ directly via `alias /var/www/myapp/staticfiles/;`.",
        },
        LangFrameworkEntry {
            id: "python-flask",
            label_ja: "Python / Flask",
            label_en: "Python / Flask",
            recommended_server: "nginx",
            reason_ja: "Flask + Gunicorn + Nginx が鉄板構成。軽量なので Nginx の低メモリ特性と相性抜群。",
            reason_en: "Flask + Gunicorn + Nginx is the go-to stack. \
                Flask's lightweight nature pairs perfectly with Nginx's low memory footprint.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "Gunicorn を systemd で管理し、Nginx で proxy_pass。\
                WSGI / ASGI どちらにも対応。",
            tips_en: "Manage Gunicorn with systemd and proxy with Nginx. \
                Works with both WSGI and ASGI.",
        },
        LangFrameworkEntry {
            id: "nodejs-express",
            label_ja: "Node.js / Express",
            label_en: "Node.js / Express",
            recommended_server: "nginx",
            reason_ja: "Node は自前 HTTP サーバーを持つため Nginx リバースプロキシが定番。\
                WebSocket も Nginx が簡単に対応できる。",
            reason_en: "Node runs its own HTTP server; Nginx reverse proxy is standard. \
                Nginx handles WebSocket upgrades with minimal config.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Fair,
            tips_ja: "proxy_http_version 1.1; と Upgrade/Connection ヘッダーで WebSocket を有効化。\
                PM2 で Node を管理すると再起動・ログが楽。",
            tips_en: "Enable WebSocket with `proxy_http_version 1.1;` and Upgrade/Connection headers. \
                Use PM2 to manage Node processes.",
        },
        LangFrameworkEntry {
            id: "ruby-rails",
            label_ja: "Ruby / Rails",
            label_en: "Ruby on Rails",
            recommended_server: "nginx",
            reason_ja: "Puma + Nginx が Rails 本番のデファクトスタンダード。\
                Asset Pipeline の静的配信も Nginx が高速。",
            reason_en: "Puma + Nginx is the Rails production standard. \
                Nginx serves Asset Pipeline files efficiently.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "Unix ソケット（/var/run/puma.sock）経由で接続するとパフォーマンス向上。\
                proxy_pass unix:/var/run/puma.sock; で設定。",
            tips_en: "Use Unix socket for better performance: \
                `proxy_pass unix:/var/run/puma.sock;`",
        },
        LangFrameworkEntry {
            id: "java-spring",
            label_ja: "Java / Spring Boot",
            label_en: "Java / Spring Boot",
            recommended_server: "nginx",
            reason_ja: "Spring Boot 組み込み Tomcat をポートバインド → Nginx リバースプロキシが標準。\
                Apache でも可だが Nginx の方が設定が簡潔。",
            reason_en: "Spring Boot embedded Tomcat on port → Nginx reverse proxy is standard. \
                Apache also works but Nginx config is simpler.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "X-Forwarded-For / X-Forwarded-Proto の転送と \
                server.forward-headers-strategy=NATIVE の Spring 側設定を合わせてください。",
            tips_en: "Forward X-Forwarded-For/Proto and set \
                `server.forward-headers-strategy=NATIVE` in Spring.",
        },
        LangFrameworkEntry {
            id: "go-gin",
            label_ja: "Go / Gin・Echo",
            label_en: "Go / Gin or Echo",
            recommended_server: "nginx",
            reason_ja: "Go もポートバインド型。Nginx + Go は超高速な組み合わせ。\
                メモリ効率が両者ともに優秀。",
            reason_en: "Go also binds to a port. Nginx + Go is an ultra-fast combination. \
                Both are memory-efficient.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "シンプルな proxy_pass 設定のみで動作。\
                静的ファイルは Go 側でなく Nginx で配信するとさらに高速。",
            tips_en: "Simple proxy_pass is all you need. \
                Serve static files from Nginx instead of Go for maximum speed.",
        },
        LangFrameworkEntry {
            id: "php-bare",
            label_ja: "PHP（フレームワークなし）",
            label_en: "PHP (no framework)",
            recommended_server: "apache",
            reason_ja: "フレームワークなし PHP は .htaccess での URL 制御が多い。\
                Apache + mod_php が最も手間が少ない。",
            reason_en: "Plain PHP often relies on .htaccess for URL control. \
                Apache + mod_php is the lowest-friction setup.",
            compat_nginx: CompatLevel::Good,
            compat_apache: CompatLevel::Excellent,
            tips_ja: "Apache では AllowOverride All と mod_rewrite を有効化するだけ。\
                Nginx を選ぶ場合は .htaccess ルールを nginx.conf に移植する作業が必要。",
            tips_en: "Apache: just enable AllowOverride All and mod_rewrite. \
                Nginx requires porting .htaccess rules manually.",
        },
        LangFrameworkEntry {
            id: "python-fastapi",
            label_ja: "Python / FastAPI",
            label_en: "Python / FastAPI",
            recommended_server: "nginx",
            reason_ja: "FastAPI は Uvicorn / Gunicorn+Uvicorn Worker でポートバインドするため \
                Nginx リバースプロキシが定番。WebSocket・SSE（Server-Sent Events）も \
                Nginx で透過的に通過できる。非同期 ASGI 特性を最大限に活かせる構成。",
            reason_en: "FastAPI runs on Uvicorn (or Gunicorn+Uvicorn workers), binding to a port — \
                Nginx reverse proxy is the standard production setup. \
                WebSocket and SSE pass through Nginx transparently. \
                Best setup to maximize async ASGI performance.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Fair,
            tips_ja: "本番は `gunicorn -k uvicorn.workers.UvicornWorker main:app` で複数ワーカー起動。\
                Nginx で proxy_pass http://127.0.0.1:8000; を設定し、\
                WebSocket 利用時は proxy_http_version 1.1; と Upgrade/Connection ヘッダーを追加。\
                Apache は mod_proxy_wstunnel が必要で設定が複雑になるため Nginx を強く推奨。",
            tips_en: "In production run `gunicorn -k uvicorn.workers.UvicornWorker main:app` \
                for multiple workers. Set `proxy_pass http://127.0.0.1:8000;` in Nginx. \
                Add `proxy_http_version 1.1;` and Upgrade/Connection headers for WebSocket. \
                Apache needs mod_proxy_wstunnel and is significantly more complex — Nginx strongly recommended.",
        },
        LangFrameworkEntry {
            id: "mojo",
            label_ja: "Mojo（Modular / MAX Engine）",
            label_en: "Mojo (Modular / MAX Engine)",
            recommended_server: "nginx",
            reason_ja: "Mojo は Python 互換の高速システム言語（C/Rust 並みの速度）。\
                HTTP サーバーは Uvicorn 互換 ASGI または独自バインドで起動するため \
                Nginx リバースプロキシが最適。FastAPI との親和性も高く同じ構成で運用可能。\
                MAX Engine デプロイ時は GPU/AI 推論エンドポイントを Nginx で分散もできる。",
            reason_en: "Mojo is a high-performance Python-compatible language (C/Rust-level speed). \
                It runs via Uvicorn-compatible ASGI or its own port binding — Nginx reverse proxy is ideal. \
                Shares the same production setup as FastAPI. \
                With MAX Engine, Nginx can distribute GPU/AI inference endpoints.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Fair,
            tips_ja: "現時点（2024〜2025）では Mojo の Web フレームワークは発展途上のため \
                Python/FastAPI エコシステムとの併用が主流。\
                `uvicorn app:app --host 127.0.0.1 --port 8000` でバインドし \
                Nginx で proxy_pass。WebSocket 利用時は proxy_http_version 1.1; を追加。\
                MAX Engine の AI 推論サーバーは別ポート（例: 8001）で起動し \
                Nginx で /api/infer/ → 8001 のようにルーティングすると整理しやすい。",
            tips_en: "As of 2024–2025, Mojo web frameworks are still evolving — \
                Python/FastAPI ecosystem integration is the mainstream approach. \
                Bind with `uvicorn app:app --host 127.0.0.1 --port 8000` and proxy with Nginx. \
                Add `proxy_http_version 1.1;` for WebSocket support. \
                Run MAX Engine inference on a separate port (e.g. 8001) and route \
                /api/infer/ → 8001 in Nginx for clean separation.",
        },
        LangFrameworkEntry {
            id: "static",
            label_ja: "静的サイト（HTML/CSS/JS）",
            label_en: "Static site (HTML/CSS/JS)",
            recommended_server: "nginx",
            reason_ja: "Nginx は静的ファイル配信が世界最速クラス。CDN 代わりにもなる。",
            reason_en: "Nginx is among the fastest static file servers in the world. \
                Can substitute for a CDN on smaller sites.",
            compat_nginx: CompatLevel::Excellent,
            compat_apache: CompatLevel::Good,
            tips_ja: "root /var/www/html; と try_files $uri $uri/ =404; だけで完結。\
                gzip_static on; で事前圧縮ファイルも自動配信。",
            tips_en: "Just `root /var/www/html;` and `try_files $uri $uri/ =404;`. \
                Enable `gzip_static on;` to serve pre-compressed files automatically.",
        },
    ]
}

// ─────────────────────────────────────────────────────────
// 4. データベース・追加サービス データ
// ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseEntry {
    pub id: &'static str,
    pub label_ja: &'static str,
    pub label_en: &'static str,
    pub category: &'static str, // "rdbms" | "nosql" | "cache" | "search" | "queue"
    pub default_port: u16,
    pub nginx_note_ja: &'static str,
    pub nginx_note_en: &'static str,
    pub apache_note_ja: &'static str,
    pub apache_note_en: &'static str,
    /// Nginxへの影響スコア差分（正=Nginx有利, 負=Apache有利）
    pub nginx_score_delta: i8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalService {
    pub id: &'static str,
    pub label_ja: &'static str,
    pub label_en: &'static str,
    pub nginx_note_ja: &'static str,
    pub nginx_note_en: &'static str,
    pub apache_note_ja: &'static str,
    pub apache_note_en: &'static str,
    pub nginx_score_delta: i8,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtendedRecommendation {
    pub recommended: String,
    pub nginx_score: i32,
    pub apache_score: i32,
    pub reason_ja: String,
    pub reason_en: String,
    pub nginx_compat_ja: &'static str,
    pub nginx_compat_en: &'static str,
    pub apache_compat_ja: &'static str,
    pub apache_compat_en: &'static str,
    pub tips_ja: String,
    pub tips_en: String,
    pub db_notes_ja: Vec<String>,
    pub db_notes_en: Vec<String>,
    pub service_notes_ja: Vec<String>,
    pub service_notes_en: Vec<String>,
    pub config_content: String,
    pub filename: String,
}

pub fn list_databases() -> Vec<DatabaseEntry> {
    vec![
        DatabaseEntry {
            id: "postgresql",
            label_ja: "PostgreSQL",
            label_en: "PostgreSQL",
            category: "rdbms",
            default_port: 5432,
            nginx_note_ja: "PostgreSQL はアプリ経由でアクセス。Nginx の設定に直接影響なし。\
                pgBouncer（コネクションプール）を挟む場合は別ポート（5433等）をアプリ側で指定。",
            nginx_note_en: "PostgreSQL is accessed via the app layer — no direct Nginx impact. \
                If using pgBouncer, configure the alternate port (e.g. 5433) in the app.",
            apache_note_ja: "Apache 経由でも直接影響なし。mod_dbd で Apache から直接接続する構成も可能（非推奨）。",
            apache_note_en: "No direct Apache impact. mod_dbd can connect directly but is not recommended.",
            nginx_score_delta: 0,
        },
        DatabaseEntry {
            id: "mysql",
            label_ja: "MySQL / MariaDB",
            label_en: "MySQL / MariaDB",
            category: "rdbms",
            default_port: 3306,
            nginx_note_ja: "PHP + MySQL の構成では Nginx + php-fpm が高速で推奨。",
            nginx_note_en: "For PHP + MySQL, Nginx + php-fpm is the fastest recommended setup.",
            apache_note_ja: "Apache + mod_php + MySQL は WordPress 等 PHP CMS の伝統的な定番構成（LAMP スタック）。",
            apache_note_en: "Apache + mod_php + MySQL is the classic LAMP stack — ideal for WordPress and PHP CMSes.",
            nginx_score_delta: 0,
        },
        DatabaseEntry {
            id: "mongodb",
            label_ja: "MongoDB",
            label_en: "MongoDB",
            category: "nosql",
            default_port: 27017,
            nginx_note_ja: "MongoDB は Node.js/Python 等と組み合わせることが多い。Nginx リバースプロキシが最適。",
            nginx_note_en: "MongoDB pairs well with Node.js/Python. Nginx reverse proxy is optimal.",
            apache_note_ja: "Apache でも動作するが WebSocket・非同期 API との組み合わせで Nginx の方が設定が楽。",
            apache_note_en: "Apache works but Nginx is simpler for WebSocket/async API setups common with MongoDB.",
            nginx_score_delta: 1,
        },
        DatabaseEntry {
            id: "redis",
            label_ja: "Redis（キャッシュ・セッション）",
            label_en: "Redis (cache / session)",
            category: "cache",
            default_port: 6379,
            nginx_note_ja: "Redis はアプリ内キャッシュ。Nginx の proxy_cache と併用すると二重キャッシュで高速化。\
                nginx_cache_bypass 設定でキャッシュ制御が細かくできる。",
            nginx_note_en: "Redis is an in-app cache. Combined with Nginx proxy_cache, double-layered caching is possible. \
                Use cache_bypass directives for fine-grained control.",
            apache_note_ja: "Apache でも利用可能だが proxy キャッシュ設定は Nginx の方が直感的。",
            apache_note_en: "Works with Apache but proxy cache config is more intuitive in Nginx.",
            nginx_score_delta: 2,
        },
        DatabaseEntry {
            id: "sqlite",
            label_ja: "SQLite",
            label_en: "SQLite",
            category: "rdbms",
            default_port: 0,
            nginx_note_ja: "SQLite はファイルベースのため Web サーバー選択に影響なし。\
                小規模・開発環境向け。本番は PostgreSQL / MySQL への移行を推奨。",
            nginx_note_en: "SQLite is file-based — no web server impact. \
                Recommended for development only; migrate to PostgreSQL/MySQL for production.",
            apache_note_ja: "Web サーバー選択に影響なし。",
            apache_note_en: "No impact on web server choice.",
            nginx_score_delta: 0,
        },
        DatabaseEntry {
            id: "elasticsearch",
            label_ja: "Elasticsearch / OpenSearch",
            label_en: "Elasticsearch / OpenSearch",
            category: "search",
            default_port: 9200,
            nginx_note_ja: "Elasticsearch の REST API を直接公開する場合、Nginx の location ブロックで \
                認証プロキシを挟むと安全。Nginx の auth_basic や JWT 検証と組み合わせ可能。",
            nginx_note_en: "When exposing Elasticsearch REST API publicly, use Nginx as an auth proxy. \
                Combine with auth_basic or JWT validation in Nginx.",
            apache_note_ja: "Apache でも auth プロキシは可能だが、設定が複雑になりやすい。",
            apache_note_en: "Auth proxy works with Apache but configuration tends to be more complex.",
            nginx_score_delta: 2,
        },
        DatabaseEntry {
            id: "dynamodb",
            label_ja: "DynamoDB（AWS）",
            label_en: "DynamoDB (AWS)",
            category: "nosql",
            default_port: 443,
            nginx_note_ja: "AWS DynamoDB は HTTPS で外部アクセス。Nginx の upstream 設定に影響なし。",
            nginx_note_en: "DynamoDB is accessed via AWS HTTPS APIs — no Nginx upstream config needed.",
            apache_note_ja: "同上。Web サーバーの選択に影響なし。",
            apache_note_en: "Same — no web server impact.",
            nginx_score_delta: 0,
        },
        DatabaseEntry {
            id: "rabbitmq",
            label_ja: "RabbitMQ（メッセージキュー）",
            label_en: "RabbitMQ (message queue)",
            category: "queue",
            default_port: 5672,
            nginx_note_ja: "RabbitMQ Management UI（ポート 15672）を Nginx でリバースプロキシすると \
                HTTPS 化・アクセス制限が容易。STOMP over WebSocket も Nginx で透過。",
            nginx_note_en: "Proxy RabbitMQ Management UI (port 15672) through Nginx for easy HTTPS and access control. \
                STOMP over WebSocket passes through Nginx transparently.",
            apache_note_ja: "Apache でも可能だが WebSocket プロキシ設定が複雑。",
            apache_note_en: "Possible with Apache but WebSocket proxy config is more complex.",
            nginx_score_delta: 2,
        },
        DatabaseEntry {
            id: "supabase",
            label_ja: "Supabase（PostgreSQL + Auth + Storage）",
            label_en: "Supabase (PostgreSQL + Auth + Storage)",
            category: "nosql",
            default_port: 54321,
            nginx_note_ja: "Supabase は内部的に Nginx を使用。セルフホスト時は Nginx でリバースプロキシを追加。\
                WebSocket（Realtime）が必須のため Nginx が最適。",
            nginx_note_en: "Supabase uses Nginx internally. For self-hosted, add Nginx reverse proxy. \
                WebSocket (Realtime) is required — Nginx is the best fit.",
            apache_note_ja: "Supabase の Realtime（WebSocket）を Apache で通すには追加モジュールと複雑な設定が必要。",
            apache_note_en: "Routing Supabase Realtime (WebSocket) through Apache needs extra modules and complex config.",
            nginx_score_delta: 3,
        },
    ]
}

pub fn list_additional_services() -> Vec<AdditionalService> {
    vec![
        AdditionalService {
            id: "websocket",
            label_ja: "WebSocket（リアルタイム通信）",
            label_en: "WebSocket (real-time)",
            nginx_note_ja: "proxy_http_version 1.1; と Upgrade/Connection ヘッダーで簡単に対応。",
            nginx_note_en: "Supported with `proxy_http_version 1.1;` and Upgrade/Connection headers.",
            apache_note_ja: "mod_proxy_wstunnel が必要。設定が増えるが対応可能。",
            apache_note_en: "Requires mod_proxy_wstunnel — more config but supported.",
            nginx_score_delta: 2,
        },
        AdditionalService {
            id: "ssl",
            label_ja: "HTTPS / SSL（Let's Encrypt）",
            label_en: "HTTPS / SSL (Let's Encrypt)",
            nginx_note_ja: "certbot --nginx で自動設定。aruaru の自動更新と連携済み。",
            nginx_note_en: "Auto-configured with `certbot --nginx`. Integrated with aruaru auto-renewal.",
            apache_note_ja: "certbot --apache でも自動設定可。どちらも同等に対応。",
            apache_note_en: "Auto-configured with `certbot --apache`. Both servers are equally supported.",
            nginx_score_delta: 0,
        },
        AdditionalService {
            id: "fileupload",
            label_ja: "ファイルアップロード（大容量）",
            label_en: "Large file upload",
            nginx_note_ja: "client_max_body_size を設定するだけ。デフォルト 1MB → 必要サイズに変更。",
            nginx_note_en: "Just set `client_max_body_size`. Default is 1MB — increase as needed.",
            apache_note_ja: "LimitRequestBody ディレクティブで設定。同様に簡単。",
            apache_note_en: "Set `LimitRequestBody` directive — equally simple.",
            nginx_score_delta: 0,
        },
        AdditionalService {
            id: "loadbalance",
            label_ja: "ロードバランサー（複数サーバー）",
            label_en: "Load balancing (multiple servers)",
            nginx_note_ja: "upstream ブロックで複数バックエンドを定義。least_conn・ip_hash 等の戦略が豊富。",
            nginx_note_en: "Define multiple backends in an `upstream` block. Rich strategies: least_conn, ip_hash, etc.",
            apache_note_ja: "mod_proxy_balancer で可能だが設定が煩雑。Nginx の方が直感的。",
            apache_note_en: "Possible with mod_proxy_balancer but verbose. Nginx is more intuitive.",
            nginx_score_delta: 3,
        },
        AdditionalService {
            id: "cache",
            label_ja: "プロキシキャッシュ（高速化）",
            label_en: "Proxy cache (speed boost)",
            nginx_note_ja: "proxy_cache_path で強力なディスクキャッシュ。Cache-Control ヘッダーとも連携。",
            nginx_note_en: "Powerful disk cache via `proxy_cache_path`. Works with Cache-Control headers.",
            apache_note_ja: "mod_cache で可能。設定はやや複雑。",
            apache_note_en: "Possible with mod_cache but configuration is more complex.",
            nginx_score_delta: 2,
        },
        AdditionalService {
            id: "gzip",
            label_ja: "Gzip 圧縮",
            label_en: "Gzip compression",
            nginx_note_ja: "gzip on; と gzip_types で即座に有効化。軽量で高速。",
            nginx_note_en: "Enable instantly with `gzip on;` and `gzip_types`. Lightweight and fast.",
            apache_note_ja: "mod_deflate で同等に対応。どちらも簡単。",
            apache_note_en: "Equally easy with mod_deflate.",
            nginx_score_delta: 0,
        },
        AdditionalService {
            id: "ratelimit",
            label_ja: "レート制限（API 保護）",
            label_en: "Rate limiting (API protection)",
            nginx_note_ja: "limit_req_zone と limit_req で IP 単位のレート制限が簡単。",
            nginx_note_en: "`limit_req_zone` + `limit_req` enable per-IP rate limiting with minimal config.",
            apache_note_ja: "mod_ratelimit や mod_evasive で対応可能。設定がやや複雑。",
            apache_note_en: "Possible with mod_ratelimit or mod_evasive but more complex.",
            nginx_score_delta: 2,
        },
        AdditionalService {
            id: "grpc",
            label_ja: "gRPC",
            label_en: "gRPC",
            nginx_note_ja: "Nginx 1.13+ から grpc_pass で gRPC プロキシを標準サポート。HTTP/2 が必要。",
            nginx_note_en: "Native gRPC proxy support since Nginx 1.13+ via `grpc_pass`. Requires HTTP/2.",
            apache_note_ja: "Apache での gRPC プロキシは設定が困難。Nginx を強く推奨。",
            apache_note_en: "gRPC proxying is difficult with Apache. Nginx is strongly recommended.",
            nginx_score_delta: 4,
        },
    ]
}

// ─────────────────────────────────────────────────────────
// 5. 拡張推薦ロジック（DB・サービス考慮）
// ─────────────────────────────────────────────────────────

pub fn recommend_full(
    lang_id: &str,
    db_ids: &[String],
    service_ids: &[String],
    port: u16,
    domain: &str,
) -> ExtendedRecommendation {
    let entries   = lang_framework_entries();
    let databases = list_databases();
    let services  = list_additional_services();

    let entry = entries.iter().find(|e| e.id == lang_id);

    // 基本スコア（言語/FW から）
    let mut nginx_score: i32 = match entry {
        Some(e) => e.compat_nginx.score() as i32,
        None    => 3,
    };
    let mut apache_score: i32 = match entry {
        Some(e) => e.compat_apache.score() as i32,
        None    => 3,
    };

    // DB スコア加算
    let mut db_notes_ja = Vec::new();
    let mut db_notes_en = Vec::new();
    for db_id in db_ids {
        if let Some(db) = databases.iter().find(|d| d.id == db_id.as_str()) {
            nginx_score  += db.nginx_score_delta as i32;
            apache_score -= db.nginx_score_delta as i32;
            db_notes_ja.push(format!("[{}] {}", db.label_ja, db.nginx_note_ja));
            db_notes_en.push(format!("[{}] {}", db.label_en, db.nginx_note_en));
        }
    }

    // 追加サービス スコア加算
    let mut service_notes_ja = Vec::new();
    let mut service_notes_en = Vec::new();
    for svc_id in service_ids {
        if let Some(svc) = services.iter().find(|s| s.id == svc_id.as_str()) {
            nginx_score  += svc.nginx_score_delta as i32;
            apache_score -= svc.nginx_score_delta as i32;
            service_notes_ja.push(format!("[{}] {}", svc.label_ja, svc.nginx_note_ja));
            service_notes_en.push(format!("[{}] {}", svc.label_en, svc.nginx_note_en));
        }
    }

    let recommended = if nginx_score >= apache_score { "nginx" } else { "apache" }.to_string();

    let (reason_ja, reason_en) = build_reason(entry, &recommended, nginx_score, apache_score);
    let tips_ja = entry.map(|e| e.tips_ja.to_string()).unwrap_or_default();
    let tips_en = entry.map(|e| e.tips_en.to_string()).unwrap_or_default();

    let config_content = generate_config_content(
        &recommended, domain, port, lang_id, db_ids, service_ids,
    );
    let filename = format!(
        "{}-{}-{}.conf",
        recommended,
        domain.replace('.', "_"),
        lang_id.replace('-', "_"),
    );

    ExtendedRecommendation {
        recommended,
        nginx_score,
        apache_score,
        reason_ja,
        reason_en,
        nginx_compat_ja: entry.map(|e| e.compat_nginx.label_ja()).unwrap_or("○ 良好"),
        nginx_compat_en: entry.map(|e| e.compat_nginx.label_en()).unwrap_or("Good"),
        apache_compat_ja: entry.map(|e| e.compat_apache.label_ja()).unwrap_or("○ 良好"),
        apache_compat_en: entry.map(|e| e.compat_apache.label_en()).unwrap_or("Good"),
        tips_ja,
        tips_en,
        db_notes_ja,
        db_notes_en,
        service_notes_ja,
        service_notes_en,
        config_content,
        filename,
    }
}

fn build_reason(
    entry: Option<&LangFrameworkEntry>,
    recommended: &str,
    nginx_score: i32,
    apache_score: i32,
) -> (String, String) {
    let lang_reason_ja = entry.map(|e| e.reason_ja).unwrap_or("汎用構成");
    let lang_reason_en = entry.map(|e| e.reason_en).unwrap_or("General-purpose setup");
    let diff = (nginx_score - apache_score).abs();
    let strength_ja = if diff >= 4 { "強く" } else if diff >= 2 { "" } else { "わずかに" };
    let strength_en = if diff >= 4 { "strongly" } else if diff >= 2 { "" } else { "slightly" };

    let ja = format!(
        "言語/FW: {lang_reason_ja} \
        スコア → Nginx {nginx_score} / Apache {apache_score}。\
        DB・サービスの組み合わせから {recommended} を{strength_ja}推薦します。",
        recommended = recommended.to_uppercase(),
    );
    let en = format!(
        "Language/FW: {lang_reason_en} \
        Score → Nginx {nginx_score} / Apache {apache_score}. \
        Based on your DB and service selections, {recommended} is {strength_en}recommended.",
        recommended = recommended.to_uppercase(),
    );
    (ja, en)
}

/// ダウンロード用設定ファイル本文を生成（ファイル保存なし）
pub fn generate_config_content(
    server: &str,
    domain: &str,
    port: u16,
    lang_id: &str,
    db_ids: &[String],
    service_ids: &[String],
) -> String {
    let databases = list_databases();
    let services  = list_additional_services();
    let ts        = chrono::Utc::now().to_rfc3339();

    // DB コメント
    let db_comments: String = db_ids.iter().filter_map(|id| {
        databases.iter().find(|d| d.id == id.as_str())
    }).map(|d| {
        if server == "nginx" {
            format!("    # [DB: {}] {}\n", d.label_en, d.nginx_note_en)
        } else {
            format!("    # [DB: {}] {}\n", d.label_en, d.apache_note_en)
        }
    }).collect();

    let has_websocket = service_ids.iter().any(|s| s == "websocket" || s == "grpc");
    let has_cache     = service_ids.iter().any(|s| s == "cache");
    let has_ratelimit = service_ids.iter().any(|s| s == "ratelimit");
    let has_gzip      = service_ids.iter().any(|s| s == "gzip");
    let has_lb        = service_ids.iter().any(|s| s == "loadbalance");

    let svc_comments: String = service_ids.iter().filter_map(|id| {
        services.iter().find(|s| s.id == id.as_str())
    }).map(|s| {
        if server == "nginx" {
            format!("    # [Service: {}] {}\n", s.label_en, s.nginx_note_en)
        } else {
            format!("    # [Service: {}] {}\n", s.label_en, s.apache_note_en)
        }
    }).collect();

    if server == "apache" {
        let file_limit = if service_ids.iter().any(|s| s == "fileupload") {
            "\n    LimitRequestBody 104857600  # 100MB — adjust as needed"
        } else { "" };
        format!(
r#"# ============================================================
# aruaru 自動生成 Apache 設定
# 言語: {lang}  生成: {ts}
# ============================================================

{db_comments}{svc_comments}
<VirtualHost *:80>
    ServerName {domain}
    ServerAlias www.{domain}

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:{port}/
    ProxyPassReverse / http://127.0.0.1:{port}/

    RequestHeader set X-Forwarded-Proto "http"
    RequestHeader set X-Real-IP %{{REMOTE_ADDR}}e{file_limit}

    ErrorLog ${{APACHE_LOG_DIR}}/{safe}_error.log
    CustomLog ${{APACHE_LOG_DIR}}/{safe}_access.log combined
</VirtualHost>
"#,
            lang = lang_id, ts = ts, domain = domain, port = port,
            safe = lang_id.replace('-', "_"),
            db_comments = db_comments, svc_comments = svc_comments,
        )
    } else {
        // Nginx
        let upstream_block = if has_lb {
            format!(
r#"# ロードバランサー設定 — サーバーを追加してください
upstream {safe}_backend {{
    least_conn;
    server 127.0.0.1:{port};
    # server 127.0.0.1:{port2};   # 追加サーバー
    keepalive 32;
}}

"#,
                safe = lang_id.replace('-', "_"), port = port,
                port2 = port + 1,
            )
        } else { String::new() };

        let cache_zone = if has_cache {
            format!(
                "proxy_cache_path /var/cache/nginx/{safe} levels=1:2 keys_zone={safe}_cache:10m max_size=1g inactive=60m use_temp_path=off;\n\n",
                safe = lang_id.replace('-', "_"),
            )
        } else { String::new() };

        let proxy_target = if has_lb {
            format!("http://{safe}_backend", safe = lang_id.replace('-', "_"))
        } else {
            format!("http://127.0.0.1:{port}")
        };

        let ws_headers = if has_websocket {
            "        proxy_http_version 1.1;\n\
                     proxy_set_header Upgrade $http_upgrade;\n\
                     proxy_set_header Connection \"upgrade\";\n"
        } else { "" };

        let cache_directives = if has_cache {
            format!(
                "        proxy_cache {safe}_cache;\n\
                         proxy_cache_valid 200 1m;\n\
                         proxy_cache_bypass $http_pragma;\n\
                         add_header X-Cache-Status $upstream_cache_status;\n",
                safe = lang_id.replace('-', "_"),
            )
        } else { String::new() };

        let rl_zone = if has_ratelimit {
            format!(
                "limit_req_zone $binary_remote_addr zone={safe}_limit:10m rate=30r/s;\n\n",
                safe = lang_id.replace('-', "_"),
            )
        } else { String::new() };

        let rl_directive = if has_ratelimit {
            format!(
                "        limit_req zone={safe}_limit burst=50 nodelay;\n",
                safe = lang_id.replace('-', "_"),
            )
        } else { String::new() };

        let gzip_block = if has_gzip {
r#"
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;
    gzip_comp_level 6;
    gzip_vary on;
"#
        } else { "" };

        let file_limit = if service_ids.iter().any(|s| s == "fileupload") {
            "\n    client_max_body_size 100M;  # ファイルアップロード上限 — 必要に応じて変更"
        } else { "" };

        format!(
r#"# ============================================================
# aruaru 自動生成 Nginx 設定
# 言語: {lang}  生成: {ts}
# ============================================================

{cache_zone}{rl_zone}{upstream_block}{db_comments}{svc_comments}
server {{
    listen 80;
    server_name {domain} www.{domain};{file_limit}{gzip_block}
    # Let's Encrypt ACME チャレンジ
    location /.well-known/acme-challenge/ {{ root /var/www/acme; }}

    location / {{
        proxy_pass {proxy_target};
{ws_headers}{rl_directive}{cache_directives}        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_read_timeout 60s;
        proxy_connect_timeout 10s;
    }}
}}
"#,
            lang = lang_id, ts = ts, domain = domain,
            cache_zone = cache_zone, rl_zone = rl_zone,
            upstream_block = upstream_block,
            db_comments = db_comments, svc_comments = svc_comments,
            file_limit = file_limit, gzip_block = gzip_block,
            proxy_target = proxy_target,
            ws_headers = ws_headers,
            rl_directive = rl_directive,
            cache_directives = cache_directives,
        )
    }
}

// ─────────────────────────────────────────────────────────
// 6. 推薦ロジック（既存・互換）
// ─────────────────────────────────────────────────────────

pub fn recommend(lang_id: &str, port: u16) -> Recommendation {
    let entries = lang_framework_entries();
    let entry = entries.iter().find(|e| e.id == lang_id);

    match entry {
        Some(e) => {
            let config_preview = auto_config_preview(e, port);
            Recommendation {
                recommended: e.recommended_server,
                reason_ja: e.reason_ja.to_string(),
                reason_en: e.reason_en.to_string(),
                nginx_compat_ja: e.compat_nginx.label_ja(),
                nginx_compat_en: e.compat_nginx.label_en(),
                apache_compat_ja: e.compat_apache.label_ja(),
                apache_compat_en: e.compat_apache.label_en(),
                tips_ja: e.tips_ja.to_string(),
                tips_en: e.tips_en.to_string(),
                config_preview,
            }
        }
        None => {
            // 不明な言語 → Nginx を汎用推薦
            Recommendation {
                recommended: "nginx",
                reason_ja: "汎用構成として Nginx のリバースプロキシを推薦します。".to_string(),
                reason_en: "Nginx reverse proxy recommended as a general-purpose setup.".to_string(),
                nginx_compat_ja: CompatLevel::Good.label_ja(),
                nginx_compat_en: CompatLevel::Good.label_en(),
                apache_compat_ja: CompatLevel::Good.label_ja(),
                apache_compat_en: CompatLevel::Good.label_en(),
                tips_ja: "アプリのポート番号を確認し、proxy_pass に設定してください。".to_string(),
                tips_en: "Check your app's port and set it in proxy_pass.".to_string(),
                config_preview: generic_nginx_preview(port),
            }
        }
    }
}

fn auto_config_preview(entry: &LangFrameworkEntry, port: u16) -> String {
    let domain = "example.com";
    match entry.recommended_server {
        "apache" => format!(
            r#"# Apache — {label} 向け自動設定プレビュー
<VirtualHost *:80>
    ServerName {domain}
    ServerAlias www.{domain}

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:{port}/
    ProxyPassReverse / http://127.0.0.1:{port}/

    RequestHeader set X-Forwarded-Proto "http"

    ErrorLog ${{APACHE_LOG_DIR}}/{eid}_error.log
    CustomLog ${{APACHE_LOG_DIR}}/{eid}_access.log combined
</VirtualHost>"#,
            label = entry.label_en,
            domain = domain,
            port = port,
            eid = entry.id,
        ),
        _ => format!(
            r#"# Nginx — {label} 向け自動設定プレビュー
server {{
    listen 80;
    server_name {domain} www.{domain};

    location /.well-known/acme-challenge/ {{ root /var/www/acme; }}

    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }}
}}"#,
            label = entry.label_en,
            domain = domain,
            port = port,
        ),
    }
}

fn generic_nginx_preview(port: u16) -> String {
    format!(
        r#"server {{
    listen 80;
    server_name example.com www.example.com;

    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }}
}}"#
    )
}

// ─────────────────────────────────────────────────────────
// 5. 自動設定実行（設定ファイル生成）
// ─────────────────────────────────────────────────────────

/// 選択した Web サーバーとドメイン・ポートを元に設定ファイルを生成して保存
pub fn generate_config(
    data_dir: &std::path::Path,
    server: &str,       // "nginx" | "apache"
    domain: &str,
    port: u16,
    lang_id: &str,
) -> anyhow::Result<String> {
    let entries = lang_framework_entries();
    let entry = entries.iter().find(|e| e.id == lang_id);

    let content = match (server, entry) {
        ("apache", Some(e)) => format!(
            r#"# aruaru 自動生成 Apache 設定 — {label}
# 生成: {ts}
<VirtualHost *:80>
    ServerName {domain}
    ServerAlias www.{domain}

    ProxyPreserveHost On
    ProxyPass / http://127.0.0.1:{port}/
    ProxyPassReverse / http://127.0.0.1:{port}/

    RequestHeader set X-Forwarded-Proto "http"
    RequestHeader set X-Real-IP %{{REMOTE_ADDR}}e

    ErrorLog ${{APACHE_LOG_DIR}}/{eid}_error.log
    CustomLog ${{APACHE_LOG_DIR}}/{eid}_access.log combined
</VirtualHost>
"#,
            label = e.label_en,
            ts = chrono::Utc::now().to_rfc3339(),
            domain = domain,
            port = port,
            eid = e.id,
        ),
        (_, _) => format!(
            r#"# aruaru 自動生成 Nginx 設定 — {lang}
# 生成: {ts}
server {{
    listen 80;
    server_name {domain} www.{domain};

    location /.well-known/acme-challenge/ {{ root /var/www/acme; }}

    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 60s;
        proxy_connect_timeout 10s;
    }}
}}
"#,
            lang = lang_id,
            ts = chrono::Utc::now().to_rfc3339(),
            domain = domain,
            port = port,
        ),
    };

    let dir = data_dir.join("vhost").join(server);
    std::fs::create_dir_all(&dir)?;
    let filename = format!(
        "{}.conf",
        domain.replace('.', "_").replace('-', "_")
    );
    let path = dir.join(&filename);
    std::fs::write(&path, &content)?;

    Ok(path.to_string_lossy().to_string())
}
