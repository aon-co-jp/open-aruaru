//! VPS サーバー管理（IP入力・接続確認・情報自動登録）

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::Pool;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct VpsServer {
    pub id:           String,
    pub name:         String,
    pub ip_address:   String,
    pub port:         i64,
    pub username:     String,
    pub auth_method:  String,
    pub key_id:       Option<String>,
    pub status:       String,
    pub last_checked: Option<String>,
    pub notes:        Option<String>,
    pub created_at:   String,
}

/// VPS を Web フォームから登録（IP アドレス入力のみで OK）
pub async fn register(
    pool:       &Pool,
    name:       &str,
    ip_address: &str,
    port:       u16,
    username:   &str,
    auth_method: &str,
    key_id:     Option<&str>,
) -> Result<VpsServer> {
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "INSERT INTO vps_servers (id,name,ip_address,port,username,auth_method,key_id,created_at)
         VALUES (?,?,?,?,?,?,?,?)",
        id, name, ip_address, port as i64, username, auth_method, key_id, now
    ).execute(pool).await?;

    Ok(get(pool, &id).await?.unwrap())
}

pub async fn list(pool: &Pool) -> Result<Vec<VpsServer>> {
    Ok(sqlx::query_as::<_, VpsServer>(
        "SELECT * FROM vps_servers ORDER BY created_at DESC"
    ).fetch_all(pool).await?)
}

pub async fn get(pool: &Pool, id: &str) -> Result<Option<VpsServer>> {
    Ok(sqlx::query_as::<_, VpsServer>(
        "SELECT * FROM vps_servers WHERE id=?"
    ).bind(id).fetch_optional(pool).await?)
}

/// 接続確認（ping）
pub async fn check_connectivity(ip: &str, port: u16) -> bool {
    use std::net::SocketAddr;
    let addr: SocketAddr = format!("{}:{}", ip, port).parse().unwrap_or_else(|_| "0.0.0.0:22".parse().unwrap());
    tokio::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(5)).is_ok()
}

/// ステータスを更新
pub async fn update_status(pool: &Pool, id: &str, status: &str) -> Result<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE vps_servers SET status=?, last_checked=? WHERE id=?",
        status, now, id
    ).execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &Pool, id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM vps_servers WHERE id=?", id).execute(pool).await?;
    Ok(())
}
