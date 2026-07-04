use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use sqlx::AnyPool;

use super::install::list_installed;
use super::manifest::{load, path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub plugin_id: String,
    pub message: String,
}

pub async fn scan_conflicts(pool: &AnyPool, root: &Path) -> anyhow::Result<Vec<ConflictReport>> {
    let plugins_dir = root.join("plugins");
    let mut out = Vec::new();
    for p in list_installed(pool).await? {
        let mp = path(&plugins_dir, &p.id);
        if !mp.exists() {
            continue;
        }
        let m = load(&mp)?;
        for key in &m.detect_conflicts {
            if let Some(msg) = detect(key) {
                out.push(ConflictReport {
                    plugin_id: p.id.clone(),
                    message: msg,
                });
            }
        }
    }
    Ok(out)
}

fn detect(key: &str) -> Option<String> {
    match key {
        "kusanagi-ssl" | "kusanagi-vhost" | "kusanagi-domain" => {
            if Path::new("/opt/kusanagi").exists() {
                Some("KUSANAGI 検出 — aruaru 設定と重複する可能性".into())
            } else {
                None
            }
        }
        "certbot-standalone" => {
            if Command::new("which").arg("certbot").output().ok()?.status.success() {
                Some("certbot 検出 — aruaru-https と重複する可能性".into())
            } else {
                None
            }
        }
        _ => None,
    }
}
