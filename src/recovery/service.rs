use std::path::{Path, PathBuf};

use chrono::Utc;
use uuid::Uuid;

use super::config::{self, RecoveryConfig};
use super::footprint::{self, Footprint};
use super::jobs::{JobKind, JobStatus, RecoveryJob};
use crate::state::AppState;

pub struct RecoveryService<'a> {
    app: &'a AppState,
}

impl<'a> RecoveryService<'a> {
    pub fn new(app: &'a AppState) -> Self {
        Self { app }
    }

    pub fn recovery_dir(&self) -> PathBuf {
        self.app.data_dir.join("recovery")
    }

    pub fn plugin_data_dir(&self) -> PathBuf {
        self.app.plugins_data().join(super::PLUGIN_ID)
    }

    pub async fn ensure_plugin(&self) -> anyhow::Result<()> {
        if !self.app.has_plugin(super::PLUGIN_ID).await? {
            anyhow::bail!("plugin not installed: {}", super::PLUGIN_ID);
        }
        Ok(())
    }

    pub fn load_config(&self) -> anyhow::Result<RecoveryConfig> {
        let defaults = self.plugin_data_dir();
        config::load_or_create(&self.recovery_dir(), Some(&defaults))
    }

    pub fn save_config(&self, cfg: &RecoveryConfig) -> anyhow::Result<()> {
        config::save(&config::config_path(&self.recovery_dir()), cfg)
    }

    pub async fn bind_drive(&self, data_dir: &Path, app_dir: &Path) -> anyhow::Result<Footprint> {
        self.ensure_plugin().await?;
        let cfg = self.load_config()?;
        std::fs::create_dir_all(data_dir)?;

        let fp = Footprint::new(data_dir, app_dir);
        let marker = footprint::marker_path(data_dir, &cfg.footprint.marker_file);
        fp.write(&marker)?;

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO recovery_footprints (id, data_dir, app_dir, drive_letter, marker_path, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT(id) DO UPDATE SET data_dir=excluded.data_dir, app_dir=excluded.app_dir,
               drive_letter=excluded.drive_letter, marker_path=excluded.marker_path, updated_at=excluded.updated_at",
        )
        .bind(&id)
        .bind(&fp.data_dir)
        .bind(&fp.app_dir)
        .bind(&fp.data_drive)
        .bind(marker.to_string_lossy().as_ref())
        .bind(&now)
        .bind(&now)
        .execute(&self.app.pool)
        .await?;

        self.record_job(
            JobKind::BindDrive,
            JobStatus::Succeeded,
            Some(data_dir),
            Some(app_dir),
            Some("footprint written"),
        )
        .await?;

        let mut cfg = cfg;
        cfg.drive.data_drive = fp.data_drive.clone();
        self.save_config(&cfg)?;

        Ok(fp)
    }

    pub fn scan_footprints(&self) -> anyhow::Result<Vec<footprint::FootprintScan>> {
        let cfg = self.load_config().unwrap_or_default();
        Ok(footprint::scan_drives(&cfg.footprint.marker_file))
    }

    pub async fn restore_access(&self, marker_path: &Path) -> anyhow::Result<PathBuf> {
        self.ensure_plugin().await?;
        let fp = Footprint::read(marker_path)?;
        let desktop_dir = desktop_config_dir();
        std::fs::create_dir_all(&desktop_dir)?;
        let config_path = desktop_dir.join("desktop.toml");

        let body = format!(
            "# aruaru desktop — restored by aruaru-OS-Recovery\n\
             data_dir = \"{}\"\n\
             root_dir = \"{}\"\n\
             footprint = \"{}\"\n\
             restored_at = \"{}\"\n",
            escape_toml(&fp.data_dir),
            escape_toml(&fp.app_dir),
            escape_toml(&marker_path.to_string_lossy()),
            Utc::now().to_rfc3339()
        );
        std::fs::write(&config_path, body)?;

        self.record_job(
            JobKind::RestoreAccess,
            JobStatus::Succeeded,
            Some(marker_path),
            Some(&config_path),
            Some("desktop.toml restored"),
        )
        .await?;

        Ok(config_path)
    }

    pub async fn list_jobs(&self, limit: i64) -> anyhow::Result<Vec<RecoveryJob>> {
        self.ensure_plugin().await?;
        Ok(sqlx::query_as::<_, RecoveryJob>(
            "SELECT id, kind, status, source_path, target_path, message, created_at, finished_at
             FROM recovery_jobs ORDER BY created_at DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(&self.app.pool)
        .await?)
    }

    pub async fn list_footprints_db(&self) -> anyhow::Result<Vec<RecoveryFootprintRow>> {
        self.ensure_plugin().await?;
        Ok(sqlx::query_as::<_, RecoveryFootprintRow>(
            "SELECT id, data_dir, app_dir, drive_letter, marker_path, created_at, updated_at
             FROM recovery_footprints ORDER BY updated_at DESC",
        )
        .fetch_all(&self.app.pool)
        .await?)
    }

    async fn record_job(
        &self,
        kind: JobKind,
        status: JobStatus,
        source: Option<&Path>,
        target: Option<&Path>,
        message: Option<&str>,
    ) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let finished = if status == JobStatus::Succeeded || status == JobStatus::Failed {
            Some(now.clone())
        } else {
            None
        };
        sqlx::query(
            "INSERT INTO recovery_jobs (id, kind, status, source_path, target_path, message, created_at, finished_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(&id)
        .bind(kind.as_str())
        .bind(status.as_str())
        .bind(source.map(|p| p.to_string_lossy().to_string()))
        .bind(target.map(|p| p.to_string_lossy().to_string()))
        .bind(message)
        .bind(&now)
        .bind(finished)
        .execute(&self.app.pool)
        .await?;
        Ok(id)
    }
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct RecoveryFootprintRow {
    pub id: String,
    pub data_dir: String,
    pub app_dir: String,
    pub drive_letter: Option<String>,
    pub marker_path: String,
    pub created_at: String,
    pub updated_at: String,
}

pub fn desktop_config_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("aruaru")
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".config")
            .join("aruaru")
    }
}

fn escape_toml(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
