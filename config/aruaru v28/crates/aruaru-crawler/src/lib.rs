use anyhow::Result;
use aruaru_core::{validate_github_url, ActivitySnapshot, ProjectSource};
use chrono::Utc;
use std::{path::Path, process::Command};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CrawlerSchedule { pub every_minutes: u64, pub all_day: bool }
impl CrawlerSchedule { pub fn every_five_minutes() -> Self { Self { every_minutes: 5, all_day: true } } }

#[derive(Debug, Clone)]
pub struct CrawlPlanner { pub schedule: CrawlerSchedule }
impl Default for CrawlPlanner { fn default() -> Self { Self::new() } }

impl CrawlPlanner {
    pub fn new() -> Self { Self { schedule: CrawlerSchedule::every_five_minutes() } }

    pub fn collect_snapshot(&self, source: &ProjectSource) -> ActivitySnapshot {
        let mut parts = Vec::new();
        if let Some(github) = &source.github_url {
            parts.push(match crawl_github_summary(github) {
                Ok(s) => format!("GitHub: {s}"),
                Err(e) => format!("GitHub registered but not crawled yet: {e}"),
            });
        }
        if let Some(local) = &source.local_drive_path {
            parts.push(match local_summary(local) {
                Ok(s) => format!("Local: {s}"),
                Err(e) => format!("Local registered but not scanned yet: {e}"),
            });
        }
        if let Some(vps) = &source.vps_directory { parts.push(format!("VPS directory registered: {vps}")); }
        if parts.is_empty() { parts.push("source is not configured".to_string()); }
        ActivitySnapshot {
            id: Uuid::new_v4(),
            project_name: source.name.clone(),
            source: source.github_url.clone().or_else(|| source.local_drive_path.clone()).or_else(|| source.vps_directory.clone()).unwrap_or_else(|| "unconfigured".to_string()),
            summary: parts.join(" / "),
            collected_at: Utc::now(),
        }
    }
}

pub fn crawl_github_summary(github_url: &str) -> Result<String> {
    validate_github_url(github_url)?;
    let output = Command::new("git").args(["ls-remote", "--heads", github_url]).output();
    match output {
        Ok(out) if out.status.success() => {
            let text = String::from_utf8_lossy(&out.stdout);
            let count = text.lines().count();
            Ok(format!("{count} remote branches detected by git ls-remote"))
        }
        Ok(out) => Ok(format!("git command returned status {}; URL kept for next crawl", out.status)),
        Err(_) => Ok("git command is unavailable; URL kept for worker/VPS crawl".to_string()),
    }
}

fn local_summary(path: &str) -> Result<String> {
    let p = Path::new(path);
    if !p.exists() { return Ok("path does not exist on this device; waiting for linked desktop agent".to_string()); }
    let mut files = 0usize;
    for entry in std::fs::read_dir(p)? { if entry?.path().is_file() { files += 1; } }
    Ok(format!("top-level files: {files}"))
}
