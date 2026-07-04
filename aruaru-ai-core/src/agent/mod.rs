//! Anthropic API 接続・検証・コマンド生成

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlanInfo {
    pub plan:    String,
    pub billing: String,
    pub account: String,
    pub verified: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CommandResult {
    pub commands:    Vec<String>,
    pub description: String,
    pub safe:        bool,
}

pub async fn get_api_key(pool: &SqlitePool) -> Result<Option<String>> {
    let row = sqlx::query_as::<_, (String, i64)>(
        "SELECT api_key, verified FROM ai_credentials WHERE id='anthropic'"
    ).fetch_optional(pool).await?;
    Ok(row.and_then(|(key, verified)| {
        if verified != 0 && !key.is_empty() { Some(key) } else { None }
    }))
}

pub async fn get_plan_info(pool: &SqlitePool) -> Result<Option<PlanInfo>> {
    let row = sqlx::query_as::<_, (String, String, String, i64)>(
        "SELECT plan, billing, account, verified FROM ai_credentials WHERE id='anthropic'"
    ).fetch_optional(pool).await?;
    Ok(row.and_then(|(plan, billing, account, verified)| {
        if verified != 0 {
            Some(PlanInfo { plan, billing, account, verified: true })
        } else { None }
    }))
}

/// API Key を検証して DB 保存
pub async fn connect_anthropic(
    pool:    &SqlitePool,
    api_key: &str,
    plan:    &str,
    billing: &str,
    account: &str,
) -> Result<PlanInfo> {
    let client = reqwest::Client::new();
    let res = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 5,
            "messages": [{"role": "user", "content": "ok"}]
        }))
        .send().await?;

    if !res.status().is_success() {
        let err: serde_json::Value = res.json().await.unwrap_or_default();
        anyhow::bail!("{}", err["error"]["message"].as_str().unwrap_or("API Key が無効です"));
    }

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE ai_credentials SET api_key=?,plan=?,billing=?,account=?,verified=1,connected_at=? WHERE id='anthropic'"
    )
    .bind(api_key).bind(plan).bind(billing).bind(account).bind(now)
    .execute(pool).await?;

    Ok(PlanInfo {
        plan: plan.to_string(),
        billing: billing.to_string(),
        account: account.to_string(),
        verified: true,
    })
}

pub async fn generate_commands(
    pool: &SqlitePool, instruction: &str, os_info: &str,
) -> Result<CommandResult> {
    let key = get_api_key(pool).await?
        .ok_or_else(|| anyhow::anyhow!("Anthropic が未接続です"))?;

    let prompt = format!(
        "Linux VPS 操作エージェント。OS:{os_info} 指示:{instruction}\n\
         JSONのみ返答: {{\"commands\":[\"cmd\"],\"description\":\"説明\",\"safe\":true}}"
    );
    let client = reqwest::Client::new();
    let res = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-haiku-4-5-20251001",
            "max_tokens": 512,
            "messages": [{"role":"user","content": prompt}]
        }))
        .send().await?;

    let data: serde_json::Value = res.json().await?;
    let text = data["content"][0]["text"].as_str().unwrap_or("{}");
    let json = if let (Some(s), Some(e)) = (text.find('{'), text.rfind('}'))
        { text[s..=e].to_string() } else { "{}".to_string() };
    Ok(serde_json::from_str(&json).unwrap_or(CommandResult {
        commands: vec![], description: text.to_string(), safe: false,
    }))
}
