use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footprint {
    pub version: u32,
    pub aruaru_version: String,
    pub plugin: String,
    pub data_dir: String,
    pub app_dir: String,
    pub data_drive: String,
    pub access_token: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub features: Vec<String>,
}

impl Footprint {
    pub fn new(data_dir: &Path, app_dir: &Path) -> Self {
        let data_dir = data_dir.to_string_lossy().to_string();
        let drive = Path::new(&data_dir)
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_else(|| "C:".to_string());
        let now = Utc::now().to_rfc3339();
        Self {
            version: 1,
            aruaru_version: crate::cli::VERSION.to_string(),
            plugin: super::PLUGIN_ID.to_string(),
            data_dir: data_dir.clone(),
            app_dir: app_dir.to_string_lossy().to_string(),
            data_drive: drive,
            access_token: Uuid::new_v4().as_simple().to_string(),
            created_at: now.clone(),
            updated_at: now,
            features: vec![
                "drive-isolation".into(),
                "access-restore".into(),
                "backup".into(),
                "migration".into(),
                "sync".into(),
            ],
        }
    }

    pub fn write(&self, marker_path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = marker_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(marker_path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn read(path: &Path) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct FootprintScan {
    pub marker_path: String,
    pub data_dir: String,
    pub app_dir: String,
    pub data_drive: String,
    pub updated_at: String,
}

pub fn marker_path(data_dir: &Path, marker_file: &str) -> PathBuf {
    data_dir.join(marker_file)
}

/// 固定ドライブを走査して footprint を探す（Windows 向け）
#[cfg(windows)]
pub fn scan_drives(marker_file: &str) -> Vec<FootprintScan> {
    let mut out = Vec::new();
    for letter in b'A'..=b'Z' {
        let root = format!("{}:\\", letter as char);
        let root_path = Path::new(&root);
        if !root_path.exists() {
            continue;
        }
        scan_tree(root_path, marker_file, 4, &mut out);
    }
    out
}

#[cfg(not(windows))]
pub fn scan_drives(marker_file: &str) -> Vec<FootprintScan> {
    let mut out = Vec::new();
    if Path::new("/").exists() {
        scan_tree(Path::new("/"), marker_file, 3, &mut out);
    }
    out
}

fn scan_tree(dir: &Path, marker_file: &str, depth: u32, out: &mut Vec<FootprintScan>) {
    if depth == 0 {
        return;
    }
    let direct = dir.join(marker_file);
    if direct.is_file() {
        if let Ok(fp) = Footprint::read(&direct) {
            out.push(FootprintScan {
                marker_path: direct.to_string_lossy().to_string(),
                data_dir: fp.data_dir,
                app_dir: fp.app_dir,
                data_drive: fp.data_drive,
                updated_at: fp.updated_at,
            });
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('$')
                || name == "Windows"
                || name == "Program Files"
                || name == "Program Files (x86)"
                || name == "node_modules"
                || name == "target"
            {
                continue;
            }
            scan_tree(&path, marker_file, depth - 1, out);
        }
    }
}
