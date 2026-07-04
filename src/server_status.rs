//! サーバー状態（CPU / メモリ / ディスク / SSL / バックアップ）— Linux VPS ダッシュボード用

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::json;

use crate::state::SharedState;

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub ok: bool,
    pub hostname: String,
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub disk_percent: Option<f64>,
    pub ssl_ok: bool,
    pub ssl_label: String,
    pub domain_ok: bool,
    pub primary_domain: Option<String>,
    pub domain_count: usize,
    pub backup_ok: bool,
    pub backup_label: String,
    pub aruaru_systemd: Option<String>,
    pub nginx_active: Option<bool>,
    pub apache_active: Option<bool>,
    pub port_3000: bool,
    pub uptime_secs: Option<u64>,
    pub checked_at: String,
}

pub async fn collect(state: &SharedState) -> ServerStatus {
    let checked_at = Utc::now().to_rfc3339();
    let hostname = hostname_linux().unwrap_or_else(|| "localhost".into());

    let domains = state.list_platform_domains().await.unwrap_or_default();
    let domain_count = domains.len();
    let primary = state.primary_platform_domain().await.ok().flatten();
    let primary_name = primary.as_ref().map(|d| d.name.clone());
    let domain_ok = primary.is_some() || domain_count > 0;

    let https_count = domains.iter().filter(|d| d.https_enabled != 0).count();
    let (ssl_ok, ssl_label) = if domain_count == 0 {
        (false, "ドメイン未登録".into())
    } else if https_count == domain_count {
        (true, format!("正常（{https_count} 件 HTTPS）"))
    } else if https_count > 0 {
        (
            false,
            format!("一部のみ HTTPS（{https_count}/{domain_count}）"),
        )
    } else {
        (false, "HTTPS 未設定".into())
    };

    let (backup_ok, backup_label) = backup_status(&state.data_dir);

    let aruaru_systemd = systemd_status("aruaru");
    let nginx_active = systemd_status("nginx").map(|s| s == "active");
    let apache_active = systemd_status("apache2")
        .or_else(|| systemd_status("httpd"))
        .map(|s| s == "active");
    let port_3000 = port_listening(3000);

    ServerStatus {
        ok: true,
        hostname,
        cpu_percent: cpu_percent_linux(),
        memory_percent: memory_percent_linux(),
        disk_percent: disk_percent_linux(&state.data_dir),
        ssl_ok,
        ssl_label,
        domain_ok,
        primary_domain: primary_name,
        domain_count,
        backup_ok,
        backup_label,
        aruaru_systemd,
        nginx_active,
        apache_active,
        port_3000,
        uptime_secs: uptime_linux(),
        checked_at,
    }
}

fn backup_status(data_dir: &std::path::Path) -> (bool, String) {
    let dir = data_dir.join("recovery").join("backups");
    if !dir.is_dir() {
        return (false, "バックアップ未設定".into());
    }
    let mut latest: Option<(DateTime<Utc>, String)> = None;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let path = e.path();
            let name = e.file_name().to_string_lossy().to_string();
            if let Ok(meta) = e.metadata() {
                if let Ok(modified) = meta.modified() {
                    let ts: DateTime<Utc> = modified.into();
                    if latest.as_ref().map(|(t, _)| ts > *t).unwrap_or(true) {
                        latest = Some((ts, name));
                    }
                }
            } else if path.is_dir() {
                latest = Some((Utc::now(), name));
            }
        }
    }
    match latest {
        Some((ts, _)) => {
            let days = (Utc::now() - ts).num_days();
            let label = if days == 0 {
                "今日完了".into()
            } else if days == 1 {
                "昨日完了".into()
            } else {
                format!("{days} 日前")
            };
            (true, label)
        }
        None => (false, "バックアップなし".into()),
    }
}

#[cfg(target_os = "linux")]
fn hostname_linux() -> Option<String> {
    std::fs::read_to_string("/etc/hostname")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
}

#[cfg(not(target_os = "linux"))]
fn hostname_linux() -> Option<String> {
    Some("localhost".into())
}

#[cfg(target_os = "linux")]
fn cpu_percent_linux() -> Option<f64> {
    let load = std::fs::read_to_string("/proc/loadavg").ok()?;
    let one_min: f64 = load.split_whitespace().next()?.parse().ok()?;
    let ncpus = std::thread::available_parallelism().ok()?.get() as f64;
    Some((one_min / ncpus * 100.0).min(100.0).round())
}

#[cfg(not(target_os = "linux"))]
fn cpu_percent_linux() -> Option<f64> {
    None
}

#[cfg(target_os = "linux")]
fn memory_percent_linux() -> Option<f64> {
    let raw = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0u64;
    let mut avail = 0u64;
    for line in raw.lines() {
        if let Some(v) = line.strip_prefix("MemTotal:") {
            total = parse_kb(v)?;
        } else if let Some(v) = line.strip_prefix("MemAvailable:") {
            avail = parse_kb(v)?;
        }
    }
    if total == 0 {
        return None;
    }
    Some(((total - avail) as f64 / total as f64 * 100.0).round())
}

#[cfg(not(target_os = "linux"))]
fn memory_percent_linux() -> Option<f64> {
    None
}

fn parse_kb(s: &str) -> Option<u64> {
    s.trim().split_whitespace().next()?.parse().ok()
}

#[cfg(target_os = "linux")]
fn disk_percent_linux(path: &std::path::Path) -> Option<f64> {
    let out = std::process::Command::new("df")
        .args(["-P", path.to_str()?])
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().nth(1)?;
    let pct = line.split_whitespace().nth(4)?.trim_end_matches('%');
    pct.parse().ok()
}

#[cfg(not(target_os = "linux"))]
fn disk_percent_linux(_path: &std::path::Path) -> Option<f64> {
    None
}

#[cfg(target_os = "linux")]
fn uptime_linux() -> Option<u64> {
    let raw = std::fs::read_to_string("/proc/uptime").ok()?;
    raw.split_whitespace()
        .next()?
        .parse::<f64>()
        .ok()
        .map(|s| s as u64)
}

#[cfg(not(target_os = "linux"))]
fn uptime_linux() -> Option<u64> {
    None
}

#[cfg(target_os = "linux")]
fn systemd_status(unit: &str) -> Option<String> {
    let out = std::process::Command::new("systemctl")
        .args(["is-active", unit])
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

#[cfg(not(target_os = "linux"))]
fn systemd_status(_unit: &str) -> Option<String> {
    None
}

#[cfg(target_os = "linux")]
fn port_listening(port: u16) -> bool {
    let out = std::process::Command::new("sh")
        .args([
            "-c",
            &format!("ss -tlnp 2>/dev/null | grep -q ':{port} ' && echo y"),
        ])
        .output();
    matches!(out, Ok(o) if String::from_utf8_lossy(&o.stdout).trim() == "y")
}

#[cfg(not(target_os = "linux"))]
fn port_listening(port: u16) -> bool {
    let _ = port;
    false
}

pub fn status_json(state: &SharedState, status: &ServerStatus) -> serde_json::Value {
    json!({
        "ok": status.ok,
        "hostname": status.hostname,
        "cpu_percent": status.cpu_percent,
        "memory_percent": status.memory_percent,
        "disk_percent": status.disk_percent,
        "ssl": { "ok": status.ssl_ok, "label": status.ssl_label },
        "domain": {
            "ok": status.domain_ok,
            "primary": status.primary_domain,
            "count": status.domain_count
        },
        "backup": { "ok": status.backup_ok, "label": status.backup_label },
        "services": {
            "aruaru": status.aruaru_systemd,
            "nginx": status.nginx_active,
            "apache": status.apache_active,
            "port_3000": status.port_3000
        },
        "uptime_secs": status.uptime_secs,
        "public_url": state.effective_public_url(),
        "checked_at": status.checked_at
    })
}
