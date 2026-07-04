use anyhow::{Context, Result};
use aruaru_core::{
    default_offline_cache_folder, is_audiocafe_source_url, sanitize_folder_name,
    validate_http_or_https_url, OfflineCacheEntry, OfflineSource,
};
use chrono::{Duration, Utc};
use std::{fs, path::{Path, PathBuf}};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OfflineCachePolicy {
    pub retention_days: i64,
    pub keep_audiocafe_forever: bool,
    pub max_bytes_per_item: usize,
}

impl Default for OfflineCachePolicy {
    fn default() -> Self {
        Self {
            retention_days: 3,
            keep_audiocafe_forever: true,
            max_bytes_per_item: 4 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OfflineCacheManager {
    pub root: PathBuf,
    pub policy: OfflineCachePolicy,
}

impl Default for OfflineCacheManager {
    fn default() -> Self {
        Self { root: default_offline_cache_folder(), policy: OfflineCachePolicy::default() }
    }
}

impl OfflineCacheManager {
    pub fn new(root: PathBuf, policy: OfflineCachePolicy) -> Self { Self { root, policy } }

    pub fn prefetch_enabled_sources(&self, sources: &[OfflineSource]) -> Vec<OfflineCacheEntry> {
        sources
            .iter()
            .filter(|source| source.enabled)
            .filter_map(|source| match self.prefetch_one(source) {
                Ok(entry) => Some(entry),
                Err(err) => {
                    eprintln!("offline prefetch skipped for {}: {err}", source.source_url);
                    None
                }
            })
            .collect()
    }

    pub fn prefetch_one(&self, source: &OfflineSource) -> Result<OfflineCacheEntry> {
        let url = validate_http_or_https_url(&source.source_url)?;
        let host = url.host_str().unwrap_or("unknown-host").to_string();
        let keep_forever = self.policy.keep_audiocafe_forever && is_audiocafe_source_url(&source.source_url);
        let safe_host = sanitize_folder_name(&host);
        let safe_label = sanitize_folder_name(&source.label);
        let cache_id = Uuid::new_v4();
        let folder = self.root.join(safe_host);
        fs::create_dir_all(&folder)?;
        let file_path = folder.join(format!("{}-{}.html", safe_label, cache_id));

        let html = match fetch_limited_text(source, self.policy.max_bytes_per_item) {
            Ok(text) => text,
            Err(err) => fallback_snapshot(source, &format!("download failed: {err}")),
        };
        fs::write(&file_path, html.as_bytes()).with_context(|| format!("write offline cache {:?}", file_path))?;

        Ok(OfflineCacheEntry {
            id: cache_id,
            source_id: source.id,
            label: source.label.clone(),
            source_url: source.source_url.clone(),
            source_host: host,
            local_path: file_path.to_string_lossy().to_string(),
            content_type: "text/html; charset=utf-8".to_string(),
            cached_at: Utc::now(),
            keep_forever,
            bytes: html.len() as u64,
        })
    }

    pub fn purge_expired_entries(&self, entries: &mut Vec<OfflineCacheEntry>) -> Result<usize> {
        let cutoff = Utc::now() - Duration::days(self.policy.retention_days);
        let mut removed = 0usize;
        let mut kept = Vec::with_capacity(entries.len());

        for entry in entries.drain(..) {
            let protected = entry.keep_forever || is_audiocafe_source_url(&entry.source_url);
            let expired = entry.cached_at < cutoff;
            if expired && !protected {
                remove_file_if_inside_root(&self.root, Path::new(&entry.local_path))?;
                removed += 1;
            } else {
                kept.push(entry);
            }
        }

        *entries = kept;
        Ok(removed)
    }
}

fn fetch_limited_text(source: &OfflineSource, max_bytes: usize) -> Result<String> {
    let response = reqwest::blocking::Client::builder()
        .user_agent("aruaru-offline-cache/0.28")
        .timeout(std::time::Duration::from_secs(15))
        .build()?
        .get(&source.source_url)
        .send()?;

    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("http status {status}");
    }

    let bytes = response.bytes()?;
    let slice = if bytes.len() > max_bytes { &bytes[..max_bytes] } else { &bytes[..] };
    let text = String::from_utf8_lossy(slice);
    Ok(wrap_cached_document(source, &text, bytes.len() > max_bytes))
}

fn fallback_snapshot(source: &OfflineSource, reason: &str) -> String {
    wrap_cached_document(
        source,
        &format!("<p>Online download was not completed. {reason}</p><p>Source: {}</p>", source.source_url),
        false,
    )
}

fn wrap_cached_document(source: &OfflineSource, body: &str, truncated: bool) -> String {
    let escaped_title = html_escape(&source.label);
    let escaped_url = html_escape(&source.source_url);
    format!(
        "<!doctype html><html lang=\"ja\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{escaped_title}</title></head><body><main><h1>{escaped_title}</h1><p>Source: <code>{escaped_url}</code></p><p>Cached by aruaru offline cache at {}</p><p>Truncated: {truncated}</p><hr>{body}</main></body></html>",
        Utc::now().to_rfc3339()
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn remove_file_if_inside_root(root: &Path, candidate: &Path) -> Result<()> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let candidate_abs = fs::canonicalize(candidate).unwrap_or_else(|_| candidate.to_path_buf());
    if candidate_abs.starts_with(&root) && candidate_abs.is_file() {
        fs::remove_file(candidate_abs)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::fs;

    fn source(url: &str) -> OfflineSource {
        OfflineSource {
            id: Uuid::new_v4(),
            label: "test".to_string(),
            source_url: url.to_string(),
            category: "test".to_string(),
            enabled: true,
            last_cached_at: None,
        }
    }

    #[test]
    fn purge_removes_old_non_audiocafe_but_keeps_audiocafe() {
        let root = std::env::temp_dir().join(format!("aruaru-cache-test-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let old_normal = root.join("old-normal.html");
        let old_audio = root.join("old-audio.html");
        fs::write(&old_normal, "normal").unwrap();
        fs::write(&old_audio, "audio").unwrap();

        let normal_source = source("https://news.yahoo.co.jp/");
        let audio_source = source("https://audiocafe.tokyo/recommend");
        let mut entries = vec![
            OfflineCacheEntry {
                id: Uuid::new_v4(),
                source_id: normal_source.id,
                label: normal_source.label,
                source_url: normal_source.source_url,
                source_host: "news.yahoo.co.jp".to_string(),
                local_path: old_normal.to_string_lossy().to_string(),
                content_type: "text/html".to_string(),
                cached_at: Utc::now() - Duration::days(4),
                keep_forever: false,
                bytes: 6,
            },
            OfflineCacheEntry {
                id: Uuid::new_v4(),
                source_id: audio_source.id,
                label: audio_source.label,
                source_url: audio_source.source_url,
                source_host: "audiocafe.tokyo".to_string(),
                local_path: old_audio.to_string_lossy().to_string(),
                content_type: "text/html".to_string(),
                cached_at: Utc::now() - Duration::days(30),
                keep_forever: true,
                bytes: 5,
            },
        ];

        let manager = OfflineCacheManager::new(root.clone(), OfflineCachePolicy::default());
        let removed = manager.purge_expired_entries(&mut entries).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(entries.len(), 1);
        assert!(entries[0].source_url.contains("audiocafe.tokyo"));
        assert!(!old_normal.exists());
        assert!(old_audio.exists());
        let _ = fs::remove_dir_all(root);
    }
}
