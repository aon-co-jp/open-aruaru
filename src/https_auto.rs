//! ドメイン登録 → 自動 HTTPS 化 + キュー再試行

use std::sync::Arc;

use tracing::{info, warn};

use crate::https_cert;
use crate::state::AppState;

pub fn schedule_configure(state: Arc<AppState>, domain: &str) {
    let domain = domain.to_string();
    tokio::spawn(async move {
        info!("auto-https: scheduling {domain}");
        if let Err(e) = https_cert::configure_domain(&state, &domain).await {
            warn!("auto-https {domain}: {e}");
        } else {
            info!("auto-https: completed {domain}");
        }
    });
}

/// auto_https 有効かつ未 HTTPS のドメインを処理
pub async fn process_pending(state: &AppState) -> anyhow::Result<()> {
    let targets = state.list_all_https_targets().await?;
    for t in targets {
        if t.auto_https == 0 {
            continue;
        }
        if is_https_enabled(state, &t.name, &t.table).await? {
            continue;
        }
        if !state.config.https_monitor.auto_configure {
            continue;
        }
        info!("auto-https: pending {}", t.name);
        if let Err(e) = https_cert::configure_domain(state, &t.name).await {
            warn!("auto-https pending {}: {e}", t.name);
        }
    }
    Ok(())
}

async fn is_https_enabled(state: &AppState, name: &str, table: &str) -> anyhow::Result<bool> {
    let v: i64 = if table == "platform" {
        sqlx::query_scalar("SELECT https_enabled FROM platform_domains WHERE name = $1")
            .bind(name)
            .fetch_one(&state.pool)
            .await?
    } else {
        sqlx::query_scalar("SELECT https_enabled FROM domains WHERE name = $1")
            .bind(name)
            .fetch_one(&state.pool)
            .await?
    };
    Ok(v != 0)
}
