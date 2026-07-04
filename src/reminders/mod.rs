use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::AnyPool;

use crate::config::ReminderConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderItem {
    pub id: String,
    pub question: String,
}

pub fn items() -> Vec<ReminderItem> {
    vec![
        ReminderItem { id: "easyweb".into(), question: "aruaru-easyweb の設定を続けますか？".into() },
        ReminderItem { id: "wp-rust".into(), question: "WordPress Rust版の開発を再開しますか？".into() },
        ReminderItem { id: "redmine-rust".into(), question: "Redmine Rust版の開発を再開しますか？".into() },
        ReminderItem { id: "egov-global".into(), question: "e-ガバメント（デジタルガバメント）の開発を再開しますか？".into() },
        ReminderItem { id: "trade-global".into(), question: "国際貿易版の開発を再開しますか？".into() },
        ReminderItem { id: "realestate".into(), question: "不動産電子契約の開発を再開しますか？".into() },
        ReminderItem { id: "ai-construction".into(), question: "AI建設DXの開発を再開しますか？".into() },
        ReminderItem { id: "ai-builder".into(), question: "AI工務店DXの開発を再開しますか？".into() },
        ReminderItem { id: "sap-marketplace".into(), question: "産業別SAP・業者マーケットプレイス・LINE・瑕疵保険付き電子契約の開発を再開しますか？".into() },
    ]
}

pub async fn due(pool: &AnyPool, cfg: &ReminderConfig) -> anyhow::Result<bool> {
    if !cfg.enabled {
        return Ok(false);
    }
    let last: Option<String> = sqlx::query_scalar("SELECT value FROM reminder_state WHERE key='last_reminder'")
        .fetch_optional(pool)
        .await?;
    Ok(match last.and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok()) {
        Some(ts) => Utc::now() >= ts + Duration::days(cfg.interval_days),
        None => true,
    })
}

pub async fn mark_shown(pool: &AnyPool) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO reminder_state (key, value) VALUES ('last_reminder', $1)
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
    )
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}
