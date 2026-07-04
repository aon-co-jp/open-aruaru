use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryConfig {
    #[serde(default)]
    pub footprint: FootprintSection,
    #[serde(default)]
    pub drive: DriveSection,
    #[serde(default)]
    pub isolation: IsolationSection,
    #[serde(default)]
    pub backup: BackupSection,
    #[serde(default)]
    pub sync: SyncSection,
    #[serde(default)]
    pub migration: MigrationSection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FootprintSection {
    #[serde(default = "default_marker")]
    pub marker_file: String,
}

fn default_marker() -> String {
    ".aruaru-footprint.json".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DriveSection {
    #[serde(default = "default_os_drive")]
    pub os_drive: String,
    #[serde(default)]
    pub data_drive: String,
}

fn default_os_drive() -> String {
    "C:".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IsolationSection {
    #[serde(default = "default_true")]
    pub app_on_data_drive: bool,
    #[serde(default = "default_true")]
    pub auto_restore_access: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackupSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_schedule")]
    pub schedule: String,
    #[serde(default = "default_retain")]
    pub retain_days: u32,
    #[serde(default = "default_backup_dir")]
    pub local_dir: String,
}

fn default_schedule() -> String {
    "daily".to_string()
}
fn default_retain() -> u32 {
    30
}
fn default_backup_dir() -> String {
    "recovery/backups".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MigrationSection {
    #[serde(default)]
    pub allowed_targets: Vec<String>,
}

impl Default for RecoveryConfig {
    fn default() -> Self {
        Self {
            footprint: FootprintSection {
                marker_file: default_marker(),
            },
            drive: DriveSection {
                os_drive: default_os_drive(),
                data_drive: String::new(),
            },
            isolation: IsolationSection {
                app_on_data_drive: true,
                auto_restore_access: true,
            },
            backup: BackupSection {
                enabled: true,
                schedule: default_schedule(),
                retain_days: default_retain(),
                local_dir: default_backup_dir(),
            },
            sync: SyncSection {
                enabled: false,
                targets: vec![],
            },
            migration: MigrationSection {
                allowed_targets: vec![
                    "local_drive".into(),
                    "nas".into(),
                    "cloud_stub".into(),
                ],
            },
        }
    }
}

pub fn config_path(recovery_dir: &Path) -> PathBuf {
    recovery_dir.join("config.toml")
}

pub fn load_or_create(recovery_dir: &Path, plugin_defaults: Option<&Path>) -> anyhow::Result<RecoveryConfig> {
    std::fs::create_dir_all(recovery_dir)?;
    let path = config_path(recovery_dir);
    if path.exists() {
        return Ok(toml::from_str(&std::fs::read_to_string(&path)?)?);
    }
    if let Some(defaults) = plugin_defaults {
        let src = defaults.join("recovery.default.toml");
        if src.exists() {
            std::fs::copy(&src, &path)?;
            return Ok(toml::from_str(&std::fs::read_to_string(&path)?)?);
        }
    }
    let cfg = RecoveryConfig::default();
    save(&path, &cfg)?;
    Ok(cfg)
}

pub fn save(path: &Path, cfg: &RecoveryConfig) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, toml::to_string_pretty(cfg)?)?;
    Ok(())
}
