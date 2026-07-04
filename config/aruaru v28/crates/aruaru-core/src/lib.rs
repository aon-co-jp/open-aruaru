use anyhow::{bail, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fs, path::{Path, PathBuf}};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientKind { Web, Desktop, Mobile, Worker }

impl ClientKind {
    pub fn parse(input: &str) -> Self {
        match input.to_ascii_lowercase().as_str() {
            "web" => Self::Web,
            "mobile" | "android" => Self::Mobile,
            "worker" => Self::Worker,
            _ => Self::Desktop,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub id: Uuid,
    pub name: String,
    pub kind: ClientKind,
    pub trusted: bool,
    pub last_seen_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainProfile {
    pub id: Uuid,
    pub domain: String,
    pub primary: bool,
    pub public_root: String,
    pub vps_ip: String,
    pub verified: bool,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProfile {
    pub id: Uuid,
    pub name: String,
    pub vps_ip: String,
    pub upload_directory: String,
    pub public_root: String,
    pub ssh_or_ftp_hint: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSource {
    pub id: Uuid,
    pub name: String,
    pub github_url: Option<String>,
    pub local_drive_path: Option<String>,
    pub vps_directory: Option<String>,
    pub enabled: bool,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineSource {
    pub id: Uuid,
    pub label: String,
    pub source_url: String,
    pub category: String,
    pub enabled: bool,
    pub last_cached_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineCacheEntry {
    pub id: Uuid,
    pub source_id: Uuid,
    pub label: String,
    pub source_url: String,
    pub source_host: String,
    pub local_path: String,
    pub content_type: String,
    pub cached_at: DateTime<Utc>,
    pub keep_forever: bool,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySnapshot {
    pub id: Uuid,
    pub project_name: String,
    pub source: String,
    pub summary: String,
    pub collected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveFileEntry {
    pub path: String,
    pub size_bytes: u64,
    pub modified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadmeReport {
    pub id: Uuid,
    pub title: String,
    pub markdown: String,
    pub target_folder: String,
    pub file_path: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlatformState {
    pub devices: Vec<DeviceProfile>,
    pub domains: Vec<DomainProfile>,
    pub servers: Vec<ServerProfile>,
    pub projects: Vec<ProjectSource>,
    pub offline_sources: Vec<OfflineSource>,
    pub offline_cache: Vec<OfflineCacheEntry>,
    pub snapshots: Vec<ActivitySnapshot>,
    pub drive_files: Vec<DriveFileEntry>,
    pub reports: Vec<ReadmeReport>,
    pub updated_at: DateTime<Utc>,
}

impl Default for PlatformState {
    fn default() -> Self {
        Self {
            devices: vec![], domains: vec![], servers: vec![], projects: vec![],
            offline_sources: vec![], offline_cache: vec![], snapshots: vec![], drive_files: vec![], reports: vec![],
            updated_at: Utc::now(),
        }
    }
}

impl PlatformState {
    pub fn add_or_update_project(&mut self, mut project: ProjectSource) -> ProjectSource {
        if let Some(existing) = self.projects.iter_mut().find(|p| p.name == project.name) {
            project.id = existing.id;
            *existing = project.clone();
            project
        } else {
            self.projects.push(project.clone());
            project
        }
    }
}

pub fn workspace_root() -> PathBuf {
    std::env::var("ARUARU_WORKSPACE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("workspace"))
}

pub fn default_project_bundle_folder() -> PathBuf { workspace_root().join("aruaru") }
pub fn default_readme_folder() -> PathBuf { workspace_root().join("aruaru（README）") }
pub fn default_state_file() -> PathBuf { workspace_root().join("state").join("platform-state.json") }
pub fn default_offline_cache_folder() -> PathBuf { workspace_root().join("offline-cache") }

pub fn ensure_workspace_dirs() -> Result<()> {
    for dir in [workspace_root(), default_project_bundle_folder(), default_readme_folder(), workspace_root().join("state"), default_offline_cache_folder()] {
        fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub fn load_state(path: &Path) -> Result<PlatformState> {
    if !path.exists() { return Ok(PlatformState::default()); }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() { return Ok(PlatformState::default()); }
    Ok(serde_json::from_str(&text)?)
}

pub fn save_state(path: &Path, state: &PlatformState) -> Result<()> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
    let text = serde_json::to_string_pretty(state)?;
    fs::write(path, text)?;
    Ok(())
}

pub fn validate_domain(domain: &str) -> Result<()> {
    let domain = domain.trim();
    if domain.is_empty() || domain.len() > 253 { bail!("domain length is invalid"); }
    if domain.contains("..") || domain.starts_with('.') || domain.ends_with('.') { bail!("domain dot placement is invalid"); }
    for label in domain.split('.') {
        if label.is_empty() || label.len() > 63 { bail!("domain label length is invalid"); }
        if label.starts_with('-') || label.ends_with('-') { bail!("domain label hyphen placement is invalid"); }
        if !label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') { bail!("domain contains unsupported characters"); }
    }
    Ok(())
}

pub fn validate_ip_or_host(value: &str) -> Result<()> {
    let v = value.trim();
    if v.is_empty() { bail!("server address is empty"); }
    if v.parse::<std::net::IpAddr>().is_ok() { return Ok(()); }
    validate_domain(v)
}

pub fn validate_github_url(value: &str) -> Result<()> {
    let url = Url::parse(value)?;
    if url.scheme() != "https" { bail!("GitHub URL must use https"); }
    let host = url.host_str().unwrap_or_default();
    if host != "github.com" { bail!("only github.com URL is accepted in this starter implementation"); }
    let segments: Vec<_> = url.path_segments().map(|s| s.collect()).unwrap_or_else(Vec::<&str>::new);
    if segments.len() < 2 { bail!("GitHub URL must contain owner and repository"); }
    Ok(())
}

pub fn sanitize_folder_name(name: &str) -> String {
    let cleaned: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect();
    let trimmed = cleaned.trim_matches('-');
    if trimmed.is_empty() { "aruaru-project".to_string() } else { trimmed.to_string() }
}


pub fn is_audiocafe_source_url(value: &str) -> bool {
    match Url::parse(value) {
        Ok(url) => url.host_str().map(|h| h.eq_ignore_ascii_case("audiocafe.tokyo") || h.ends_with(".audiocafe.tokyo")).unwrap_or(false),
        Err(_) => false,
    }
}

pub fn validate_http_or_https_url(value: &str) -> Result<Url> {
    let url = Url::parse(value)?;
    match url.scheme() {
        "http" | "https" => Ok(url),
        _ => bail!("URL must use http or https"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audiocafe_hosts_are_protected() {
        assert!(is_audiocafe_source_url("https://audiocafe.tokyo/recommend"));
        assert!(is_audiocafe_source_url("https://www.audiocafe.tokyo/item"));
        assert!(!is_audiocafe_source_url("https://example.com/audiocafe.tokyo"));
    }

    #[test]
    fn invalid_domain_is_rejected() {
        assert!(validate_domain("aruaru.tokyo").is_ok());
        assert!(validate_domain("-bad.example").is_err());
        assert!(validate_domain("bad..example").is_err());
    }
}
