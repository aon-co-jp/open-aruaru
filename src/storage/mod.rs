//! VersionLess Storage — DB 抽象層（Ver 0.1 仕様）
//!
//! PostgreSQL（主）/ CockroachDB（同期レプリカ）/ SQLite（開発）を
//! 同一 trait で操作する。詳細: `docs/SPEC_v0.1.md`

mod migrate;
mod pool;

pub use migrate::run as run_migrations;
pub use pool::connect;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbBackend {
    Sqlite,
    Postgres,
    Cockroach,
}

impl DbBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Cockroach => "cockroach",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCaps {
    pub backend: DbBackend,
    pub schema_version: i64,
    pub supports_returning: bool,
}

/// サイトドメイン（easyweb 管理対象）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRecord {
    pub name: String,
    pub https_enabled: bool,
}

/// VersionLess Storage の共通契約（ドメイン操作）
#[async_trait]
pub trait DomainStore: Send + Sync {
    async fn list(&self) -> anyhow::Result<Vec<DomainRecord>>;
    async fn add(&self, name: &str) -> anyhow::Result<()>;
    async fn set_https(&self, name: &str, enabled: bool) -> anyhow::Result<()>;
}

/// 認証ストア（e-mail は利用者 VPS 内のみ — 作者には送信しない）
#[async_trait]
pub trait AuthStore: Send + Sync {
    async fn register_email(&self, email: &str) -> anyhow::Result<()>;
    async fn has_admin(&self) -> anyhow::Result<bool>;
}

/// プラグインストア
#[async_trait]
pub trait PluginStore: Send + Sync {
    async fn list_installed(&self) -> anyhow::Result<Vec<String>>;
    async fn register(&self, id: &str, version: &str) -> anyhow::Result<()>;
}
