//! Let's Encrypt 証明書の自動取得・更新・監視（手動操作不要）

use anyhow::Result;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use tracing::{info, warn};
use crate::db::Pool;

/// ドメイン登録時に certbot で証明書を自動取得
pub async fn setup_https(
    pool: &Pool, domain_id: &str, domain: &str,
    admin_email: &str, webserver: &str,
) -> Result<()> {
    let bare   = domain.trim_start_matches("www.");
    let plugin = if webserver == "apache" { "--apache" } else { "--nginx" };

    info!("Let's Encrypt 証明書取得: {} + www.{}", bare, bare);

    let out = tokio::process::Command::new("certbot")
        .args([
            plugin, "--non-interactive", "--agree-tos", "--redirect",
            "-m", admin_email,
            "-d", bare,
            "-d", &format!("www.{}", bare),
        ])
        .output().await?;

    if !out.status.success() {
        anyhow::bail!("certbot 失敗:\n{}", String::from_utf8_lossy(&out.stderr));
    }

    let expiry = get_expiry(bare).await?;
    let exp_str = expiry.to_rfc3339();
    sqlx::query(
        "UPDATE domains SET https_enabled=1, cert_expires_at=? WHERE id=?"
    ).bind(&exp_str).bind(domain_id).execute(pool).await?;

    info!("証明書取得完了: {} (期限: {})", bare, exp_str);
    Ok(())
}

/// 自動更新チェック（毎日 03:00 に呼び出し）— 残り30日以内を自動更新
pub async fn auto_renew_all(pool: &Pool) -> Result<()> {
    info!("証明書自動更新チェック開始");
    let threshold = (Utc::now() + Duration::days(30)).to_rfc3339();
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM domains WHERE https_enabled=1 AND cert_expires_at < ?"
    ).bind(&threshold).fetch_all(pool).await?;

    for (id, name) in rows {
        let bare = name.trim_start_matches("www.").to_string();
        match renew(pool, &id, &bare).await {
            Ok(_)  => info!("更新成功: {}", bare),
            Err(e) => warn!("更新失敗: {} — {}", bare, e),
        }
    }
    Ok(())
}

pub async fn renew(pool: &Pool, domain_id: &str, bare: &str) -> Result<()> {
    let out = tokio::process::Command::new("certbot")
        .args(["renew", "--cert-name", bare, "--non-interactive", "--quiet"])
        .output().await?;
    if !out.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&out.stderr));
    }
    let exp = get_expiry(bare).await?.to_rfc3339();
    sqlx::query("UPDATE domains SET cert_expires_at=? WHERE id=?")
        .bind(&exp).bind(domain_id).execute(pool).await?;
    Ok(())
}

pub async fn get_expiry(bare: &str) -> Result<DateTime<Utc>> {
    let path = format!("/etc/letsencrypt/live/{}/cert.pem", bare);
    let out  = tokio::process::Command::new("openssl")
        .args(["x509", "-enddate", "-noout", "-in", &path])
        .output().await?;
    let text = String::from_utf8_lossy(&out.stdout);
    let date_str = text.trim().strip_prefix("notAfter=").unwrap_or("").trim();
    let naive = NaiveDateTime::parse_from_str(date_str, "%b %e %H:%M:%S %Y GMT")
        .or_else(|_| NaiveDateTime::parse_from_str(date_str, "%b  %e %H:%M:%S %Y GMT"))?;
    Ok(DateTime::from_naive_utc_and_offset(naive, Utc))
}
