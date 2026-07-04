use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tokio::fs;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use walkdir::WalkDir;

use crate::config::AppConfig;
use crate::generator::{build_readme, write_outputs};

#[derive(Clone, Debug, Default)]
pub struct AppStatus {
    pub root: String,
    pub output_mode: String,
    pub last_scan_at: Option<String>,
    pub generated_count: u64,
    pub skipped_count: u64,
    pub error_count: u64,
    pub last_generated: Vec<String>,
    pub last_errors: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct SeenReadme {
    modified: Option<SystemTime>,
    len: u64,
}

pub type SharedStatus = Arc<Mutex<AppStatus>>;

pub async fn run_initial_scan(config: &AppConfig, status: &SharedStatus) -> Result<()> {
    let mut seen: BTreeMap<PathBuf, SeenReadme> = BTreeMap::new();
    scan_once(config, status, &mut seen).await
}

pub async fn run_scanner(config: AppConfig, status: SharedStatus) {
    let mut seen: BTreeMap<PathBuf, SeenReadme> = BTreeMap::new();
    loop {
        if let Err(err) = scan_once(&config, &status, &mut seen).await {
            let mut locked = status.lock().await;
            locked.error_count += 1;
            push_limited(&mut locked.last_errors, format!("scanner error: {err:#}"));
        }
        sleep(Duration::from_secs(config.scan_interval_secs)).await;
    }
}

async fn scan_once(
    config: &AppConfig,
    status: &SharedStatus,
    seen: &mut BTreeMap<PathBuf, SeenReadme>,
) -> Result<()> {
    let mut found = Vec::new();
    for entry in WalkDir::new(&config.root).follow_links(false).into_iter() {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                let mut locked = status.lock().await;
                locked.error_count += 1;
                push_limited(&mut locked.last_errors, format!("walk error: {err}"));
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name() == "README.md" {
            found.push(entry.path().to_path_buf());
        }
    }

    for path in found {
        match process_readme_if_changed(config, &path, seen).await {
            Ok(ProcessOutcome::Generated(title)) => {
                let mut locked = status.lock().await;
                locked.generated_count += 1;
                push_limited(
                    &mut locked.last_generated,
                    format!("{} ({})", path.display(), title),
                );
            }
            Ok(ProcessOutcome::Skipped) => {
                let mut locked = status.lock().await;
                locked.skipped_count += 1;
            }
            Err(err) => {
                let mut locked = status.lock().await;
                locked.error_count += 1;
                push_limited(
                    &mut locked.last_errors,
                    format!("{}: {err:#}", path.display()),
                );
            }
        }
    }

    let mut locked = status.lock().await;
    locked.last_scan_at = Some(DateTime::<Utc>::from(SystemTime::now()).to_rfc3339());
    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
enum ProcessOutcome {
    Generated(String),
    Skipped,
}

async fn process_readme_if_changed(
    config: &AppConfig,
    path: &Path,
    seen: &mut BTreeMap<PathBuf, SeenReadme>,
) -> Result<ProcessOutcome> {
    let meta1 = fs::metadata(path)
        .await
        .with_context(|| format!("failed to read metadata for {}", path.display()))?;

    if meta1.len() > config.max_readme_bytes {
        return Err(anyhow::anyhow!(
            "README.md is too large: {} bytes > {} bytes",
            meta1.len(),
            config.max_readme_bytes
        ));
    }

    sleep(Duration::from_millis(config.stable_wait_millis)).await;

    let meta2 = fs::metadata(path)
        .await
        .with_context(|| format!("failed to re-read metadata for {}", path.display()))?;

    if meta1.len() != meta2.len() || meta1.modified().ok() != meta2.modified().ok() {
        return Ok(ProcessOutcome::Skipped);
    }

    let key = path.to_path_buf();
    let current = SeenReadme {
        modified: meta2.modified().ok(),
        len: meta2.len(),
    };

    if seen
        .get(&key)
        .map(|old| old.len == current.len && old.modified == current.modified)
        == Some(true)
    {
        return Ok(ProcessOutcome::Skipped);
    }

    let markdown = fs::read_to_string(path)
        .await
        .with_context(|| format!("failed to read {}", path.display()))?;
    let generated = build_readme(&markdown, path, config);
    write_outputs(path, &generated, config).await?;
    seen.insert(key, current);

    Ok(ProcessOutcome::Generated(generated.title))
}

pub fn new_status(config: &AppConfig) -> SharedStatus {
    Arc::new(Mutex::new(AppStatus {
        root: config.root.display().to_string(),
        output_mode: config.output_mode.as_str().to_string(),
        ..Default::default()
    }))
}

fn push_limited(target: &mut Vec<String>, value: String) {
    target.insert(0, value);
    target.truncate(20);
}
