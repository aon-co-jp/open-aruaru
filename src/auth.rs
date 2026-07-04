use anyhow::Context;
use sha2::{Digest, Sha256};

use crate::config::AppConfig;
use crate::state::AppState;

pub fn hash_token(token: &str) -> String {
    let mut h = Sha256::new();
    h.update(token.as_bytes());
    hex::encode(h.finalize())
}

pub async fn register_email(state: &AppState, email: &str) -> anyhow::Result<String> {
    let email = email.trim().to_lowercase();
    if !email.contains('@') {
        anyhow::bail!("invalid email");
    }

    sqlx::query(
        "INSERT INTO admin_users (email, created_at) VALUES ($1, $2)
         ON CONFLICT (email) DO NOTHING",
    )
        .bind(&email)
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&state.pool)
        .await?;

    issue_magic_link(state, &email).await
}

pub async fn request_login_link(state: &AppState, email: &str) -> anyhow::Result<String> {
    let email = email.trim().to_lowercase();
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM admin_users WHERE email = $1")
        .bind(&email)
        .fetch_one(&state.pool)
        .await?;
    if exists == 0 {
        anyhow::bail!("email not registered");
    }
    issue_magic_link(state, &email).await
}

async fn issue_magic_link(state: &AppState, email: &str) -> anyhow::Result<String> {
    let token = uuid::Uuid::new_v4().to_string();
    let hash = hash_token(&token);
    let expires = chrono::Utc::now()
        + chrono::Duration::minutes(state.config.auth.magic_link_ttl_minutes);

    sqlx::query(
        "INSERT INTO magic_tokens (token_hash, email, expires_at, created_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(&hash)
    .bind(email)
    .bind(expires.to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&state.pool)
    .await?;

    let url = format!(
        "{}/auth/verify?token={token}",
        state.effective_public_url().trim_end_matches('/')
    );

    deliver_link(&state.config, email, &url).await?;
    tracing::info!("magic link for {email}: {url}");
    Ok(url)
}

async fn deliver_link(config: &AppConfig, email: &str, url: &str) -> anyhow::Result<()> {
    if !config.smtp.enabled {
        tracing::warn!(
            "SMTP が無効です。config/default.toml の [smtp] enabled = true に設定してください。\
             \nデバッグ用リンク → {url}"
        );
        return Ok(());
    }

    use lettre::message::header::ContentType;
    use lettre::transport::smtp::authentication::Credentials;
    use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

    let password = config.smtp.resolved_password();
    if password.is_empty() {
        anyhow::bail!(
            "SMTPパスワードが未設定です。\
             環境変数 ARUARU_SMTP_PASSWORD を設定するか、\
             config/default.toml の [smtp] password に記述してください。"
        );
    }

    let body_html = format!(
        r#"<!DOCTYPE html>
<html lang="ja"><body>
<p>aruaru にログインするには、以下のリンクをクリックしてください：</p>
<p><a href="{url}">{url}</a></p>
<p>このリンクは {ttl} 分間有効です。</p>
<p>このメールに心当たりがない場合は無視してください。</p>
</body></html>"#,
        url = url,
        ttl = config.auth.magic_link_ttl_minutes,
    );

    let msg = Message::builder()
        .from(
            config.smtp.from.parse()
                .with_context(|| format!("無効な送信者アドレス: {}", config.smtp.from))?,
        )
        .to(
            email.parse()
                .with_context(|| format!("無効な宛先アドレス: {email}"))?,
        )
        .subject("【aruaru】ログインリンク")
        .header(ContentType::TEXT_HTML)
        .body(body_html)?;

    let creds = Credentials::new(config.smtp.user.clone(), password);

    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp.host)
        .with_context(|| format!("SMTPホスト接続失敗: {}", config.smtp.host))?
        .port(config.smtp.port)
        .credentials(creds)
        .build();

    mailer
        .send(msg)
        .await
        .with_context(|| format!("メール送信失敗 → {email}"))?;

    tracing::info!("magic link sent to {email}");
    Ok(())
}

pub async fn verify_token(state: &AppState, token: &str) -> anyhow::Result<String> {
    let hash = hash_token(token);
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT email, expires_at FROM magic_tokens WHERE token_hash = $1 AND used = 0",
    )
    .bind(&hash)
    .fetch_optional(&state.pool)
    .await?;

    let Some((email, expires_at)) = row else {
        anyhow::bail!("invalid or expired link");
    };

    let exp: chrono::DateTime<chrono::Utc> = expires_at.parse()?;
    if chrono::Utc::now() > exp {
        anyhow::bail!("link expired");
    }

    sqlx::query("UPDATE magic_tokens SET used = 1 WHERE token_hash = $1")
        .bind(&hash)
        .execute(&state.pool)
        .await?;

    Ok(email)
}

pub async fn has_admin(state: &AppState) -> bool {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM admin_users")
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0)
        > 0
}

pub async fn send_reminder_links(state: &AppState) -> anyhow::Result<Vec<String>> {
    let emails: Vec<String> =
        sqlx::query_scalar("SELECT email FROM admin_users ORDER BY created_at")
            .fetch_all(&state.pool)
            .await?;

    let mut urls = Vec::new();
    for email in emails {
        if let Ok(url) = issue_magic_link(state, &email).await {
            urls.push(url);
        }
    }
    Ok(urls)
}
