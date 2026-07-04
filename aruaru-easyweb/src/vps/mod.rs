//! VPS 登録・管理・自動セットアップ

pub mod autosetup;

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
    pub ssh_port:     i64,
    pub ssh_user:     String,
    pub auth_method:  String,
    pub keypair_id:   Option<String>,
    pub status:       String,
    pub last_checked: Option<String>,
    pub created_at:   String,
    pub updated_at:   String,
}

pub async fn list(pool: &Pool) -> Result<Vec<VpsServer>> {
    Ok(sqlx::query_as::<_, VpsServer>(
        "SELECT id,name,ip_address,ssh_port,ssh_user,auth_method,keypair_id,
                status,last_checked,created_at,updated_at
         FROM vps_servers ORDER BY created_at DESC"
    ).fetch_all(pool).await?)
}

pub async fn add(
    pool:       &Pool,
    name:       &str,
    ip:         &str,
    port:       u16,
    user:       &str,
    auth_method: &str,
    password:   Option<&str>,
    keypair_id: Option<&str>,
) -> Result<VpsServer> {
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "INSERT INTO vps_servers
         (id,name,ip_address,ssh_port,ssh_user,auth_method,ssh_password,keypair_id,created_at,updated_at)
         VALUES (?,?,?,?,?,?,?,?,?,?)",
        id, name, ip, port as i64, user, auth_method, password, keypair_id, now, now
    ).execute(pool).await?;

    Ok(sqlx::query_as::<_, VpsServer>(
        "SELECT id,name,ip_address,ssh_port,ssh_user,auth_method,keypair_id,
                status,last_checked,created_at,updated_at
         FROM vps_servers WHERE id=?"
    ).bind(&id).fetch_one(pool).await?)
}

/// SSH 接続テストとステータス更新
pub async fn check_connection(pool: &Pool, vps: &VpsServer, auth: &crate::ssh::SshAuth) -> bool {
    let ok = crate::ssh::connect(&vps.ip_address, vps.ssh_port as u16, &vps.ssh_user, auth)
        .await.is_ok();
    let status = if ok { "online" } else { "offline" };
    let now = Utc::now().to_rfc3339();
    let _ = sqlx::query!(
        "UPDATE vps_servers SET status=?, last_checked=?, updated_at=? WHERE id=?",
        status, now, now, vps.id
    ).execute(pool).await;
    ok
}
