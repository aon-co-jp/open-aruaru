//! aruaru-easyweb 設定
//!
//! 読み込み優先順位:
//!   1. data_dir/config.toml  （ユーザー設定 — VPS の ./data/config.toml）
//!   2. root_dir/config/default.toml  （デフォルト値）
//!   3. 環境変数でオーバーライド可能

use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

// ─── VPSサーバー用設定 ─────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_public_url")]
    pub public_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            public_url: default_public_url(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_ttl_minutes")]
    pub magic_link_ttl_minutes: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self { magic_link_ttl_minutes: default_ttl_minutes() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SmtpConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    #[serde(default)]
    pub user: String,
    /// パスワードは設定ファイルに書かず環境変数 ARUARU_SMTP_PASSWORD を推奨
    #[serde(default)]
    pub password: String,
    #[serde(default = "default_smtp_from")]
    pub from: String,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: default_smtp_port(),
            user: String::new(),
            password: String::new(),
            from: default_smtp_from(),
        }
    }
}

impl SmtpConfig {
    /// 環境変数 ARUARU_SMTP_PASSWORD があればそちらを優先する
    pub fn resolved_password(&self) -> String {
        env::var("ARUARU_SMTP_PASSWORD").unwrap_or_else(|_| self.password.clone())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_primary")]
    pub primary: String,
    #[serde(default = "default_postgres_url")]
    pub postgres_url: String,
    #[serde(default)]
    pub cockroach_url: String,
    #[serde(default)]
    pub cockroach_sync_enabled: bool,
    #[serde(default = "default_cockroach_sync_interval")]
    pub cockroach_sync_interval_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            primary: default_db_primary(),
            postgres_url: default_postgres_url(),
            cockroach_url: String::new(),
            cockroach_sync_enabled: false,
            cockroach_sync_interval_secs: default_cockroach_sync_interval(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReminderConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_reminder_interval")]
    pub interval_days: i64,
}

impl Default for ReminderConfig {
    fn default() -> Self {
        Self { enabled: true, interval_days: default_reminder_interval() }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HttpsMonitorConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_https_scan_interval")]
    pub scan_interval_secs: u64,
    #[serde(default = "default_true")]
    pub auto_configure: bool,
    #[serde(default)]
    pub certbot_email: String,
    #[serde(default = "default_web_server")]
    pub web_server: String,
    #[serde(default = "default_acme_webroot")]
    pub acme_webroot: String,
    #[serde(default = "default_renewal_interval")]
    pub renewal_interval_secs: u64,
}

impl Default for HttpsMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_interval_secs: default_https_scan_interval(),
            auto_configure: true,
            certbot_email: String::new(),
            web_server: default_web_server(),
            acme_webroot: default_acme_webroot(),
            renewal_interval_secs: default_renewal_interval(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinuxOsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_linux_manifest_url")]
    pub manifest_url: String,
    #[serde(default = "default_crawl_interval")]
    pub crawl_interval_secs: u64,
    #[serde(default)]
    pub auto_apply: bool,
}

impl Default for LinuxOsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            manifest_url: default_linux_manifest_url(),
            crawl_interval_secs: default_crawl_interval(),
            auto_apply: false,
        }
    }
}

/// GitHub 連携設定
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GithubConfig {
    /// Personal Access Token（repo スコープ）
    pub token: Option<String>,
    /// 組織リポジトリを取得する場合の Org 名
    pub org: Option<String>,
}

/// VPSサーバー全体の設定（config/default.toml から読み込む）
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub smtp: SmtpConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub reminders: ReminderConfig,
    #[serde(default)]
    pub https_monitor: HttpsMonitorConfig,
    #[serde(default)]
    pub linux_os: LinuxOsConfig,
    #[serde(default)]
    pub github: Option<GithubConfig>,
}

impl AppConfig {
    /// data_dir/config.toml → root_dir/config/default.toml の順で読み込む
    pub fn load_or_create(data_dir: &Path, root_dir: &Path) -> Result<Self> {
        let user_path = data_dir.join("config.toml");
        let default_path = root_dir.join("config").join("default.toml");

        let mut cfg: AppConfig = if user_path.exists() {
            let raw = std::fs::read_to_string(&user_path)
                .with_context(|| format!("設定ファイル読み込み失敗: {}", user_path.display()))?;
            toml::from_str(&raw)
                .with_context(|| format!("設定ファイルパース失敗: {}", user_path.display()))?
        } else if default_path.exists() {
            let raw = std::fs::read_to_string(&default_path)
                .with_context(|| format!("デフォルト設定読み込み失敗: {}", default_path.display()))?;
            toml::from_str(&raw)
                .with_context(|| format!("デフォルト設定パース失敗: {}", default_path.display()))?
        } else {
            tracing::warn!("設定ファイルが見つかりません。デフォルト値を使用します。");
            AppConfig::default()
        };

        // 環境変数でオーバーライド
        if let Ok(v) = env::var("ARUARU_PUBLIC_URL") {
            cfg.server.public_url = v;
        }
        if let Ok(v) = env::var("ARUARU_DATABASE_URL") {
            cfg.database.postgres_url = v;
            cfg.database.primary = "postgres".to_string();
        }
        if let Ok(v) = env::var("ARUARU_SMTP_HOST") {
            cfg.smtp.host = v;
            cfg.smtp.enabled = true;
        }
        if let Ok(v) = env::var("ARUARU_SMTP_USER") {
            cfg.smtp.user = v;
        }
        if let Ok(v) = env::var("ARUARU_SMTP_FROM") {
            cfg.smtp.from = v;
        }

        Ok(cfg)
    }
}

// ─── READMEウォッチャー用設定（既存機能を維持）───────────────

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OutputMode {
    Rs,
    Html,
    Both,
}

impl OutputMode {
    pub fn parse(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "rs" | "readme.rs" => Ok(Self::Rs),
            "html" | "readme.html" => Ok(Self::Html),
            "both" | "all" | "rs_and_html" => Ok(Self::Both),
            other => Err(anyhow::anyhow!("unknown output mode: {other}. Use rs, html, or both.")),
        }
    }

    pub fn writes_rs(&self) -> bool {
        matches!(self, Self::Rs | Self::Both)
    }

    pub fn writes_html(&self) -> bool {
        matches!(self, Self::Html | Self::Both)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rs => "rs",
            Self::Html => "html",
            Self::Both => "both",
        }
    }
}

/// READMEウォッチャー専用設定
#[derive(Clone, Debug)]
pub struct WatcherConfig {
    pub root: PathBuf,
    pub listen: SocketAddr,
    pub output_mode: OutputMode,
    pub scan_interval_secs: u64,
    pub stable_wait_millis: u64,
    pub max_readme_bytes: u64,
    pub title_prefix: String,
}

impl WatcherConfig {
    pub fn from_env_and_args() -> Result<Self> {
        let mut root = env::var_os("ARUARU_README_ROOT")
            .map(PathBuf::from)
            .unwrap_or(env::current_dir().context("failed to read current dir")?);
        let mut listen: SocketAddr = env::var("ARUARU_README_LISTEN")
            .unwrap_or_else(|_| "127.0.0.1:7878".to_string())
            .parse()
            .context("invalid ARUARU_README_LISTEN")?;
        let mut output_mode = OutputMode::parse(
            &env::var("ARUARU_README_OUTPUT").unwrap_or_else(|_| "rs".to_string()),
        )?;
        let mut scan_interval_secs: u64 = env::var("ARUARU_README_SCAN_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        let mut stable_wait_millis: u64 = env::var("ARUARU_README_STABLE_WAIT_MILLIS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_200);
        let mut max_readme_bytes: u64 = env::var("ARUARU_README_MAX_BYTES")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1_048_576);
        let mut title_prefix = env::var("ARUARU_README_TITLE_PREFIX")
            .unwrap_or_else(|_| "aruaru README".to_string());

        let args: Vec<String> = env::args().collect();
        let mut i = 1usize;
        while i < args.len() {
            match args[i].as_str() {
                "--root" => {
                    i += 1;
                    root = PathBuf::from(args.get(i).context("--root requires a value")?);
                }
                "--listen" => {
                    i += 1;
                    listen = args
                        .get(i)
                        .context("--listen requires a value")?
                        .parse()
                        .context("invalid --listen value")?;
                }
                "--output" => {
                    i += 1;
                    output_mode =
                        OutputMode::parse(args.get(i).context("--output requires a value")?)?;
                }
                "--scan-interval-secs" => {
                    i += 1;
                    scan_interval_secs = args
                        .get(i)
                        .context("--scan-interval-secs requires a value")?
                        .parse()
                        .context("invalid --scan-interval-secs value")?;
                }
                "--stable-wait-millis" => {
                    i += 1;
                    stable_wait_millis = args
                        .get(i)
                        .context("--stable-wait-millis requires a value")?
                        .parse()
                        .context("invalid --stable-wait-millis value")?;
                }
                "--max-readme-bytes" => {
                    i += 1;
                    max_readme_bytes = args
                        .get(i)
                        .context("--max-readme-bytes requires a value")?
                        .parse()
                        .context("invalid --max-readme-bytes value")?;
                }
                "--title-prefix" => {
                    i += 1;
                    title_prefix = args.get(i).context("--title-prefix requires a value")?.clone();
                }
                "--help" | "-h" => {
                    print_help();
                    std::process::exit(0);
                }
                other => return Err(anyhow::anyhow!("unknown argument: {other}")),
            }
            i += 1;
        }

        if scan_interval_secs == 0 {
            return Err(anyhow::anyhow!("scan interval must be 1 second or more"));
        }
        if stable_wait_millis < 200 {
            return Err(anyhow::anyhow!("stable wait should be 200ms or more"));
        }
        if max_readme_bytes < 1024 {
            return Err(anyhow::anyhow!("max README size should be 1024 bytes or more"));
        }

        Ok(Self {
            root,
            listen,
            output_mode,
            scan_interval_secs,
            stable_wait_millis,
            max_readme_bytes,
            title_prefix,
        })
    }
}

pub fn print_help() {
    println!(
        "aruaru-easyweb\n\n\
         Usage: aruaru [SUBCOMMAND]\n\n\
         Subcommands:\n\
           serve          VPSサーバーを起動（デフォルト）\n\
           install        プラグインをインストール\n\
           uninstall      プラグインをアンインストール\n\
           reminders      リマインダーリンクを再送\n\n\
         Environment:\n\
           ARUARU_DATA_DIR          データディレクトリ（デフォルト: ./data）\n\
           ARUARU_ROOT              ルートディレクトリ（デフォルト: .）\n\
           ARUARU_PUBLIC_URL        公開URL（例: https://example.com）\n\
           ARUARU_DATABASE_URL      PostgreSQL接続URL\n\
           ARUARU_SMTP_HOST         SMTPホスト\n\
           ARUARU_SMTP_USER         SMTPユーザー\n\
           ARUARU_SMTP_PASSWORD     SMTPパスワード（推奨: 環境変数で設定）\n\
           ARUARU_SMTP_FROM         送信者アドレス\n"
    );
}

// ─── デフォルト値ヘルパー ─────────────────────────────────────

fn default_linux_manifest_url() -> String { String::new() }
fn default_crawl_interval() -> u64 { 3600 }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 3000 }
fn default_public_url() -> String { "http://localhost:3000".to_string() }
fn default_ttl_minutes() -> i64 { 60 }
fn default_smtp_port() -> u16 { 587 }
fn default_smtp_from() -> String { "noreply@example.com".to_string() }
fn default_db_primary() -> String { "sqlite".to_string() }
fn default_postgres_url() -> String {
    "postgres://aruaru:aruaru@127.0.0.1:5432/aruaru".to_string()
}
fn default_cockroach_sync_interval() -> u64 { 300 }
fn default_reminder_interval() -> i64 { 7 }
fn default_true() -> bool { true }
fn default_https_scan_interval() -> u64 { 300 }
fn default_web_server() -> String { "nginx".to_string() }
fn default_acme_webroot() -> String { "acme-webroot".to_string() }
fn default_renewal_interval() -> u64 { 86400 }
