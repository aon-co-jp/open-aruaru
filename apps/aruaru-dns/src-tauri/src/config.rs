use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub postgres: PostgresConfig,
    pub aruaru_db: AruaruDbConfig,
    #[serde(default)]
    pub nameservers: NameserverConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AruaruDbConfig {
    pub url: String,
    pub source_path: String,
    #[serde(default = "default_true")]
    pub sync_enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NameserverConfig {
    #[serde(default)]
    pub list: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let user = config_path();
        if user.exists() {
            return Ok(toml::from_str(&std::fs::read_to_string(&user)?)?);
        }
        let embedded = include_str!("../config/default.toml");
        Ok(toml::from_str(embedded)?)
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let p = config_path();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(p, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("aruaru-dns")
        .join("config.toml")
}
