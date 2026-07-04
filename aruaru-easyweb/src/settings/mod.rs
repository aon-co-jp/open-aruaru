//! アプリ設定を DB に保存・読み込み
//! .env / VI 操作不要。Web フォームから登録・変更できる。

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::db::Pool;

/// 全設定をまとめた構造体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub app_url:              String,
    pub admin_email:          String,
    pub token_expiry_minutes: String,
    pub smtp_host:            String,
    pub smtp_port:            String,
    pub smtp_username:        String,
    pub smtp_password:        String,
    pub smtp_from:            String,
    pub setup_completed:      bool,
}

/// DB から全設定を読み込む
pub async fn load(pool: &Pool) -> Result<AppSettings> {
    let rows = sqlx::query_as::<_, (String, String)>("SELECT key, value FROM app_settings")
        .fetch_all(pool)
        .await?;

    let mut s = AppSettings::default();
    for (key, value) in rows {
        match key.as_str() {
            "app_url"              => s.app_url              = value,
            "admin_email"          => s.admin_email          = value,
            "token_expiry_minutes" => s.token_expiry_minutes = value,
            "smtp_host"            => s.smtp_host            = value,
            "smtp_port"            => s.smtp_port            = value,
            "smtp_username"        => s.smtp_username        = value,
            "smtp_password"        => s.smtp_password        = value,
            "smtp_from"            => s.smtp_from            = value,
            "setup_completed"      => s.setup_completed      = value == "1",
            _ => {}
        }
    }
    Ok(s)
}

/// 設定を DB に保存（UPSERT）
pub async fn save(pool: &Pool, s: &AppSettings) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    let pairs = [
        ("app_url",              s.app_url.as_str()),
        ("admin_email",          s.admin_email.as_str()),
        ("token_expiry_minutes", s.token_expiry_minutes.as_str()),
        ("smtp_host",            s.smtp_host.as_str()),
        ("smtp_port",            s.smtp_port.as_str()),
        ("smtp_username",        s.smtp_username.as_str()),
        ("smtp_password",        s.smtp_password.as_str()),
        ("smtp_from",            s.smtp_from.as_str()),
        ("setup_completed",      if s.setup_completed { "1" } else { "0" }),
    ];
    for (k, v) in pairs {
        sqlx::query(
            "INSERT INTO app_settings (key, value, updated_at)
             VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=excluded.updated_at"
        ).bind(k).bind(v).bind(&now)
         .execute(pool)
         .await?;
    }
    Ok(())
}

/// セットアップが完了しているか確認
pub async fn is_setup_completed(pool: &Pool) -> bool {
    sqlx::query_as::<_, (String,)>("SELECT value FROM app_settings WHERE key='setup_completed'")
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
        .map(|(v,)| v == "1")
        .unwrap_or(false)
}

/// AppSettings を AppConfig に変換
impl AppSettings {
    pub fn to_app_config(&self) -> crate::config::AppConfig {
        crate::config::AppConfig {
            host:                 "0.0.0.0".to_string(),
            port:                 8080,
            database_url:         "sqlite:data/aruaru.db".to_string(),
            app_url:              self.app_url.clone(),
            token_expiry_minutes: self.token_expiry_minutes.parse().unwrap_or(15),
            admin_email:          self.admin_email.clone(),
            smtp: crate::config::SmtpConfig {
                host:     self.smtp_host.clone(),
                port:     self.smtp_port.parse().unwrap_or(587),
                username: self.smtp_username.clone(),
                password: self.smtp_password.clone(),
                from:     self.smtp_from.clone(),
            },
        }
    }
}
