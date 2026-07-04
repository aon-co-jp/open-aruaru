use anyhow::{bail, Result};
use aruaru_core::DriveFileEntry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, time::SystemTime};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveScanPolicy {
    pub allowlisted_roots: Vec<String>,
    pub max_depth: usize,
    pub include_hidden: bool,
    pub max_files: usize,
}

impl DriveScanPolicy {
    pub fn safe_default() -> Self {
        Self { allowlisted_roots: vec![], max_depth: 4, include_hidden: false, max_files: 2_000 }
    }
    pub fn with_root(root: String) -> Self {
        let mut p = Self::safe_default();
        p.allowlisted_roots.push(root);
        p
    }
    pub fn allows(&self, path: &str) -> bool {
        let path = normalize(path);
        self.allowlisted_roots.iter().any(|root| {
            let root = normalize(root);
            path == root || path.starts_with(&(root + "/"))
        })
    }
}

fn normalize(value: &str) -> String { value.replace('\\', "/").trim_end_matches('/').to_string() }

fn system_time_to_utc(value: SystemTime) -> DateTime<Utc> { DateTime::<Utc>::from(value) }

pub fn scan_allowlisted_path(path: &str, policy: &DriveScanPolicy) -> Result<Vec<DriveFileEntry>> {
    if !policy.allows(path) { bail!("path is not allowlisted: {path}"); }
    let root = Path::new(path);
    if !root.exists() { bail!("path does not exist: {path}"); }
    let mut out = Vec::new();
    for entry in WalkDir::new(root).max_depth(policy.max_depth).follow_links(false).into_iter().filter_map(Result::ok) {
        if out.len() >= policy.max_files { break; }
        let p = entry.path();
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or_default();
        if !policy.include_hidden && name.starts_with('.') { continue; }
        if p.is_file() {
            let meta = fs::metadata(p)?;
            out.push(DriveFileEntry {
                path: p.to_string_lossy().to_string(),
                size_bytes: meta.len(),
                modified_at: meta.modified().ok().map(system_time_to_utc),
            });
        }
    }
    Ok(out)
}
