pub mod ssl;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::Pool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Domain {
    pub id:              String,
    pub name:            String,
    pub www_redirect:    bool,
    pub is_app_domain:   bool,
    pub webserver:       String,
    pub proxy_port:      Option<i64>,
    pub https_enabled:   bool,
    pub cert_expires_at: Option<String>,
    pub gzip_enabled:    bool,
    pub created_at:      String,
    pub updated_at:      String,
}

pub async fn list(pool: &Pool) -> Result<Vec<Domain>> {
    Ok(sqlx::query_as::<_, Domain>("SELECT * FROM domains ORDER BY created_at DESC")
        .fetch_all(pool).await?)
}

pub async fn add(
    pool: &Pool, name: &str, is_app: bool,
    webserver: &str, proxy_port: Option<u16>,
) -> Result<Domain> {
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let pp  = proxy_port.map(|p| p as i64);
    // bare domain に正規化（www. を除去）
    let bare = name.trim_start_matches("www.");

    sqlx::query(
        "INSERT INTO domains (id,name,is_app_domain,webserver,proxy_port,created_at,updated_at)
         VALUES (?,?,?,?,?,?,?)"
    ).bind(&id).bind(bare).bind(is_app).bind(webserver).bind(pp).bind(&now).bind(&now)
     .execute(pool).await?;

    Ok(sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id=?")
        .bind(&id).fetch_one(pool).await?)
}

pub async fn set_https(pool: &Pool, id: &str, enabled: bool) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE domains SET https_enabled=?, updated_at=? WHERE id=?")
        .bind(enabled).bind(&now).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn set_webserver(pool: &Pool, id: &str, ws: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE domains SET webserver=?, updated_at=? WHERE id=?")
        .bind(ws).bind(&now).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &Pool, id: &str) -> Result<Option<Domain>> {
    let d = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id=?")
        .bind(id).fetch_optional(pool).await?;
    if d.is_some() {
        sqlx::query("DELETE FROM domains WHERE id=?").bind(id).execute(pool).await?;
    }
    Ok(d)
}
