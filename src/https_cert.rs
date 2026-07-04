//! Let's Encrypt / certbot 連携 — ドメイン登録後の自動 HTTPS + 更新メンテナンス

use std::path::PathBuf;
use std::process::Stdio;

use tracing::{info, warn};

use crate::state::AppState;
use crate::vhost;

pub async fn configure_domain(state: &AppState, domain: &str) -> anyhow::Result<String> {
    let acme = acme_webroot(state);
    std::fs::create_dir_all(&acme)?;

    // HTTP + ACME challenge 用 vhost を先に配置
    vhost::regenerate_all(state).await?;
    prepare_web_server(state)?;

    #[cfg(target_os = "linux")]
    {
        let email = &state.config.https_monitor.certbot_email;
        if email.is_empty() || email == "admin@example.com" {
            anyhow::bail!("https_monitor.certbot_email を data/config.toml に設定してください");
        }

        let webroot = acme.to_string_lossy().to_string();

        // example.com なら www.example.com も同時に取得（サブドメインは除外）
        let www = vhost::www_variant(domain);
        let mut certbot_args = vec![
            "certonly",
            "--webroot",
            "-w",
            &webroot,
            "-d",
            domain,
        ];
        let www_str;
        if let Some(ref w) = www {
            www_str = w.clone();
            certbot_args.extend_from_slice(&["-d", &www_str]);
        }
        certbot_args.extend_from_slice(&[
            "--non-interactive",
            "--agree-tos",
            "-m",
            email,
            "--keep-until-expiring",
        ]);

        let output = tokio::process::Command::new("certbot")
            .args(&certbot_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(o) if o.status.success() => {
                let www_msg = www.as_deref().map(|w| format!(" + {w}")).unwrap_or_default();
                info!("certbot OK for {domain}{www_msg}");
                mark_https_enabled(state, domain).await?;
                state.on_https_enabled(domain).await.ok();
                vhost::regenerate_all(state).await?;
                let install = vhost::install_symlinks(state).unwrap_or_else(|e| e.to_string());
                let reload = vhost::reload_proxy(&state.config.https_monitor.web_server)?;
                remove_queue_job(state, domain)?;
                Ok(format!("HTTPS enabled for {domain}{www_msg}. {install} {reload}"))
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                warn!("certbot failed for {domain}: {err}");
                queue_job(state, domain, &err)?;
                anyhow::bail!("certbot failed: {err}")
            }
            Err(e) => {
                queue_job(state, domain, &e.to_string())?;
                anyhow::bail!("certbot not available: {e}")
            }
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        queue_job(state, domain, "certbot requires Linux VPS")?;
        Ok(format!(
            "Queued HTTPS job for {domain} (deploy on Linux VPS to apply certbot)"
        ))
    }
}

/// 証明書の自動更新（certbot renew）+ vhost 再生成 + nginx reload
pub async fn renew_certificates(state: &AppState) -> anyhow::Result<String> {
    #[cfg(target_os = "linux")]
    {
        info!("certbot renew — maintenance");
        let output = tokio::process::Command::new("certbot")
            .args(["renew", "--quiet", "--no-random-sleep-on-renew"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        let renew_msg = match &output {
            Ok(o) if o.status.success() => {
                let out = String::from_utf8_lossy(&o.stdout);
                if out.trim().is_empty() {
                    "renew: no certificates due".into()
                } else {
                    format!("renew: {out}")
                }
            }
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                format!("renew warning: {err}")
            }
            Err(e) => format!("renew skipped: {e}"),
        };

        vhost::regenerate_all(state).await?;
        let install = vhost::install_symlinks(state).unwrap_or_else(|e| e.to_string());
        let reload = vhost::reload_proxy(&state.config.https_monitor.web_server)?;
        Ok(format!("{renew_msg}. {install} {reload}"))
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok("renew skipped (not Linux)".into())
    }
}

/// https-queue/ に残ったジョブを再試行
pub async fn process_queue(state: &AppState) -> anyhow::Result<()> {
    let dir = state.data_dir.join("https-queue");
    if !dir.is_dir() {
        return Ok(());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                names.push(stem.to_string());
            }
        }
    }
    for domain in names {
        if is_domain_https_enabled(state, &domain).await.unwrap_or(false) {
            remove_queue_job(state, &domain)?;
            continue;
        }
        info!("https-queue retry: {domain}");
        let _ = configure_domain(state, &domain).await;
    }
    Ok(())
}

async fn is_domain_https_enabled(state: &AppState, domain: &str) -> anyhow::Result<bool> {
    let pf: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM platform_domains WHERE name = $1 AND https_enabled = 1")
        .bind(domain)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    if pf > 0 {
        return Ok(true);
    }
    let site: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM domains WHERE name = $1 AND https_enabled = 1")
        .bind(domain)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    Ok(site > 0)
}

fn prepare_web_server(state: &AppState) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        let _ = vhost::install_symlinks(state);
        let _ = vhost::reload_proxy(&state.config.https_monitor.web_server);
    }
    let _ = state;
    Ok(())
}

async fn mark_https_enabled(state: &AppState, domain: &str) -> anyhow::Result<()> {
    let now = chrono::Utc::now().to_rfc3339();
    let pf: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM platform_domains WHERE name = $1")
        .bind(domain)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    if pf > 0 {
        sqlx::query(
            "UPDATE platform_domains SET https_enabled = 1, last_checked = $1 WHERE name = $2",
        )
        .bind(&now)
        .bind(domain)
        .execute(&state.pool)
        .await?;
    } else {
        sqlx::query("UPDATE domains SET https_enabled = 1, last_checked = $1 WHERE name = $2")
            .bind(&now)
            .bind(domain)
            .execute(&state.pool)
            .await?;
    }
    Ok(())
}

fn queue_job(state: &AppState, domain: &str, note: &str) -> anyhow::Result<()> {
    let dir = state.data_dir.join("https-queue");
    std::fs::create_dir_all(&dir)?;
    let job = serde_json::json!({
        "domain": domain,
        "action": "letsencrypt",
        "queued_at": chrono::Utc::now().to_rfc3339(),
        "note": note
    });
    std::fs::write(
        dir.join(format!("{domain}.json")),
        serde_json::to_string_pretty(&job)?,
    )?;
    Ok(())
}

fn remove_queue_job(state: &AppState, domain: &str) -> anyhow::Result<()> {
    let path = state
        .data_dir
        .join("https-queue")
        .join(format!("{domain}.json"));
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// HTTPS 監視・キュー状態（Web UI / ダッシュボード用）
pub fn monitor_status(state: &AppState) -> serde_json::Value {
    use serde_json::json;

    let cfg = &state.config.https_monitor;
    let dir = state.data_dir.join("https-queue");
    let mut pending = Vec::new();
    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(raw) = std::fs::read_to_string(&path) {
                    if let Ok(job) = serde_json::from_str::<serde_json::Value>(&raw) {
                        pending.push(job);
                    }
                }
            }
        }
    }
    pending.sort_by(|a, b| {
        a.get("domain")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .cmp(b.get("domain").and_then(|v| v.as_str()).unwrap_or(""))
    });

    json!({
        "enabled": cfg.enabled,
        "auto_configure": cfg.auto_configure,
        "certbot_email_set": !cfg.certbot_email.is_empty(),
        "web_server": cfg.web_server,
        "acme_webroot": cfg.acme_webroot,
        "renewal_interval_secs": cfg.renewal_interval_secs,
        "scan_interval_secs": cfg.scan_interval_secs,
        "pending_queue": pending,
        "queue_count": pending.len(),
    })
}

fn acme_webroot(state: &AppState) -> PathBuf {
    let wr = &state.config.https_monitor.acme_webroot;
    let p = PathBuf::from(wr);
    if p.is_absolute() {
        p
    } else {
        state.data_dir.join(wr)
    }
}

pub async fn configure_all_pending(state: &AppState) -> anyhow::Result<Vec<String>> {
    let targets = state.list_all_https_targets().await?;
    let mut results = Vec::new();
    for t in targets {
        if t.auto_https == 0 {
            continue;
        }
        match configure_domain(state, &t.name).await {
            Ok(m) => results.push(m),
            Err(e) => results.push(format!("{}: {e}", t.name)),
        }
    }
    Ok(results)
}
