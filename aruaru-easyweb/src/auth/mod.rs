pub mod email;

use anyhow::Result;
use chrono::{Duration, Utc};
use uuid::Uuid;
use crate::db::Pool;

pub async fn send_magic_link(
    pool: &Pool, email: &str, app_url: &str,
    expiry_min: i64, smtp: &crate::config::SmtpConfig,
) -> Result<()> {
    let uid = get_or_create_user(pool, email).await?;
    let token = Uuid::new_v4().to_string();
    let tid   = Uuid::new_v4().to_string();
    let exp   = (Utc::now() + Duration::minutes(expiry_min)).to_rfc3339();

    sqlx::query(
        "INSERT INTO login_tokens (id,user_id,token,expires_at) VALUES (?,?,?,?)"
    ).bind(&tid).bind(&uid).bind(&token).bind(&exp)
     .execute(pool).await?;

    let url = format!("{}/auth/verify?token={}", app_url, token);
    email::send_login_email(smtp, email, &url).await?;
    Ok(())
}

pub async fn verify_token(pool: &Pool, token: &str) -> Result<Option<String>> {
    let now = Utc::now().to_rfc3339();
    let row = sqlx::query_as::<_, (String, String, i64)>(
        "SELECT user_id, expires_at, used FROM login_tokens WHERE token=?"
    ).bind(token).fetch_optional(pool).await?;

    if let Some((user_id, expires_at, used)) = row {
        if used != 0 { return Ok(None); }
        if expires_at < now { return Ok(None); }
        sqlx::query("UPDATE login_tokens SET used=1 WHERE token=?")
            .bind(token).execute(pool).await?;
        sqlx::query("UPDATE users SET last_login=? WHERE id=?")
            .bind(&now).bind(&user_id).execute(pool).await?;
        return Ok(Some(user_id));
    }
    Ok(None)
}

async fn get_or_create_user(pool: &Pool, email: &str) -> Result<String> {
    if let Some((id,)) = sqlx::query_as::<_, (String,)>(
        "SELECT id FROM users WHERE email=?"
    ).bind(email).fetch_optional(pool).await? {
        return Ok(id);
    }
    let id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO users (id,email) VALUES (?,?)")
        .bind(&id).bind(email).execute(pool).await?;
    Ok(id)
}
