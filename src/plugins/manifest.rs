use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub plugins: Vec<String>,
    #[serde(default)]
    pub provides: Vec<String>,
    #[serde(default, rename = "detect_conflicts")]
    pub detect_conflicts: Vec<String>,
}

impl PluginManifest {
    pub fn is_bundle(&self) -> bool {
        self.kind == "bundle" || !self.plugins.is_empty()
    }
}

pub fn load(path: &Path) -> anyhow::Result<PluginManifest> {
    Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
}

pub fn path(plugins_dir: &Path, id: &str) -> std::path::PathBuf {
    plugins_dir.join(id).join("manifest.toml")
}
