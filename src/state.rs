use std::path::{Path, PathBuf};
use std::sync::Arc;

use sqlx::AnyPool;

use crate::config::AppConfig;
use crate::storage::{self, DbBackend};

pub type SharedState = Arc<AppState>;

#[derive(Clone)]
pub struct AppState {
    pub pool: AnyPool,
    pub db_backend: DbBackend,
    pub config: AppConfig,
    pub data_dir: PathBuf,
    pub root_dir: PathBuf,
}

impl AppState {
    pub async fn new(data_dir: &Path, root_dir: &Path) -> anyhow::Result<Self> {
        std::fs::create_dir_all(data_dir)?;
        let config = AppConfig::load_or_create(data_dir, root_dir)?;
        let (pool, db_backend) = storage::connect(&config.database, data_dir).await?;
        storage::run_migrations(&pool, db_backend, root_dir).await?;

        Ok(Self {
            pool,
            db_backend,
            config,
            data_dir: data_dir.to_path_buf(),
            root_dir: root_dir.to_path_buf(),
        })
    }

    pub fn plugins_dir(&self) -> PathBuf {
        self.root_dir.join("plugins")
    }

    pub fn plugins_data(&self) -> PathBuf {
        self.data_dir.join("plugins")
    }

    pub fn recovery_dir(&self) -> PathBuf {
        self.data_dir.join("recovery")
    }

    pub async fn has_plugin(&self, id: &str) -> anyhow::Result<bool> {
        crate::plugins::is_installed(&self.pool, id).await
    }

    pub async fn install(&self, target: &str) -> anyhow::Result<Vec<String>> {
        crate::plugins::install(
            &self.pool,
            &self.plugins_dir(),
            &self.plugins_data(),
            target,
        )
        .await
    }

    pub async fn uninstall(&self, id: &str, purge_data: bool) -> anyhow::Result<()> {
        crate::plugins::uninstall(
            &self.pool,
            &self.plugins_dir(),
            &self.plugins_data(),
            id,
            purge_data,
        )
        .await?;
        if purge_data && id == crate::recovery::PLUGIN_ID {
            let dir = self.recovery_dir();
            if dir.exists() {
                std::fs::remove_dir_all(&dir)?;
            }
        }
        Ok(())
    }

    pub async fn list_plugins(&self) -> anyhow::Result<Vec<crate::plugins::InstalledPlugin>> {
        crate::plugins::list_installed(&self.pool).await
    }

    pub async fn list_domains(&self) -> anyhow::Result<Vec<DomainRow>> {
        Ok(sqlx::query_as::<_, DomainRow>(
            "SELECT name, https_enabled, auto_https, last_checked FROM domains ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?)
    }

    pub async fn add_domain(&self, name: &str) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO domains (name, https_enabled, auto_https, last_checked) VALUES ($1, 0, 1, NULL)
             ON CONFLICT (name) DO NOTHING",
        )
        .bind(name.trim().to_lowercase())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_domain(&self, name: &str) -> anyhow::Result<()> {
        let name = name.trim().to_lowercase();
        let affected = sqlx::query("DELETE FROM domains WHERE name = $1")
            .bind(&name)
            .execute(&self.pool)
            .await?
            .rows_affected();
        if affected == 0 {
            anyhow::bail!("domain not found: {name}");
        }
        Ok(())
    }

    /// ドメインの HTTPS 状態をリセットして再取得キューに戻す
    pub async fn reregister_domain(&self, name: &str) -> anyhow::Result<()> {
        let name = name.trim().to_lowercase();
        let affected = sqlx::query(
            "UPDATE domains SET https_enabled = 0, last_checked = NULL WHERE name = $1",
        )
        .bind(&name)
        .execute(&self.pool)
        .await?
        .rows_affected();
        if affected == 0 {
            anyhow::bail!("domain not found: {name}");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)]
pub struct DomainRow {
    pub name: String,
    pub https_enabled: i64,
    pub auto_https: i64,
    pub last_checked: Option<String>,
}
