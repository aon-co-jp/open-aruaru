use std::sync::Arc;
use std::time::Duration;

use tracing::info;

use crate::https_auto;
use crate::https_cert;
use crate::state::AppState;

pub fn spawn(state: Arc<AppState>) {
    if !state.config.https_monitor.enabled {
        return;
    }
    let secs = state.config.https_monitor.scan_interval_secs.max(60);
    let renewal_secs = state.config.https_monitor.renewal_interval_secs.max(3600);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(secs));
        let mut ticks: u64 = 0;
        let renewal_every = (renewal_secs / secs).max(1);

        loop {
            interval.tick().await;
            ticks += 1;

            if let Err(e) = scan_once(state.clone()).await {
                tracing::warn!("https monitor: {e}");
            }
            if let Err(e) = https_auto::process_pending(&state).await {
                tracing::warn!("https auto pending: {e}");
            }

            if ticks >= renewal_every {
                ticks = 0;
                match https_cert::renew_certificates(&state).await {
                    Ok(m) => info!("https maintenance: {m}"),
                    Err(e) => tracing::warn!("https renew: {e}"),
                }
            }
        }
    });
}

pub async fn scan_once(state: Arc<AppState>) -> anyhow::Result<()> {
    let targets = state.list_all_https_targets().await?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()?;

    for t in targets {
        if t.auto_https == 0 {
            continue;
        }
        let http_url = format!("http://{}", t.name);
        let https_url = format!("https://{}", t.name);

        let http_ok = client
            .get(&http_url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);
        let https_ok = client
            .get(&https_url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        let now = chrono::Utc::now().to_rfc3339();
        let table = t.table.as_str();
        if table == "platform" {
            sqlx::query("UPDATE platform_domains SET last_checked = $1 WHERE name = $2")
                .bind(&now)
                .bind(&t.name)
                .execute(&state.pool)
                .await?;
        } else {
            sqlx::query("UPDATE domains SET last_checked = $1 WHERE name = $2")
                .bind(&now)
                .bind(&t.name)
                .execute(&state.pool)
                .await?;
        }

        if https_ok {
            if table == "platform" {
                sqlx::query("UPDATE platform_domains SET https_enabled = 1 WHERE name = $1")
                    .bind(&t.name)
                    .execute(&state.pool)
                    .await?;
                let primary: Option<String> = sqlx::query_scalar(
                    "SELECT name FROM platform_domains WHERE is_primary = 1 AND name = $1",
                )
                .bind(&t.name)
                .fetch_optional(&state.pool)
                .await?;
                if primary.is_some() {
                    state.on_https_enabled(&t.name).await.ok();
                }
            } else {
                sqlx::query("UPDATE domains SET https_enabled = 1 WHERE name = $1")
                    .bind(&t.name)
                    .execute(&state.pool)
                    .await?;
            }
        } else if state.config.https_monitor.auto_configure {
            // DNS 伝播後に自動取得（HTTP が応答していなくても試行 — キューでリトライ）
            if http_ok {
                info!("aruaru-https: {} — HTTP OK, running certbot", t.name);
            }
            if let Err(e) = https_cert::configure_domain(&state, &t.name).await {
                tracing::debug!("certbot {name}: {e}", name = t.name);
            }
        }
    }
    Ok(())
}
