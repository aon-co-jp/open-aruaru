//! SSH 鍵ペアの自動生成・管理
//! Ed25519（推奨）/ RSA 4096 に対応

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::Pool;

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub id:          String,
    pub name:        String,
    pub key_type:    String,
    pub public_key:  String,
    pub fingerprint: String,
    pub created_at:  String,
}

/// Ed25519 鍵ペアを生成して DB に保存
pub async fn generate_ed25519(pool: &Pool, name: &str) -> Result<KeyPair> {
    use ssh_key::{Algorithm, PrivateKey};

    let private_key = PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519)?;
    let public_key  = private_key.public_key();

    let private_pem  = private_key.to_openssh(ssh_key::LineEnding::LF)?.to_string();
    let public_openssh = public_key.to_openssh()?;
    let fingerprint  = public_key.fingerprint(ssh_key::HashAlg::Sha256).to_string();

    save_keypair(pool, name, "ed25519", &public_openssh, &private_pem, &fingerprint).await
}

/// RSA 4096 鍵ペアを生成して DB に保存
pub async fn generate_rsa(pool: &Pool, name: &str) -> Result<KeyPair> {
    use ssh_key::{Algorithm, PrivateKey};

    let private_key = PrivateKey::random(
        &mut rand_core::OsRng,
        Algorithm::Rsa { hash: Some(ssh_key::HashAlg::Sha256) },
    )?;
    let public_key   = private_key.public_key();
    let private_pem  = private_key.to_openssh(ssh_key::LineEnding::LF)?.to_string();
    let public_openssh = public_key.to_openssh()?;
    let fingerprint  = public_key.fingerprint(ssh_key::HashAlg::Sha256).to_string();

    save_keypair(pool, name, "rsa", &public_openssh, &private_pem, &fingerprint).await
}

async fn save_keypair(
    pool: &Pool, name: &str, key_type: &str,
    public_key: &str, private_key: &str, fingerprint: &str,
) -> Result<KeyPair> {
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "INSERT INTO ssh_keys (id,name,key_type,public_key,private_key,fingerprint,created_at)
         VALUES (?,?,?,?,?,?,?)",
        id, name, key_type, public_key, private_key, fingerprint, now
    ).execute(pool).await?;

    Ok(KeyPair { id, name: name.to_string(), key_type: key_type.to_string(),
        public_key: public_key.to_string(), fingerprint: fingerprint.to_string(),
        created_at: now })
}

pub async fn list(pool: &Pool) -> Result<Vec<KeyPair>> {
    let rows = sqlx::query!(
        "SELECT id,name,key_type,public_key,fingerprint,created_at FROM ssh_keys ORDER BY created_at DESC"
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|r| KeyPair {
        id: r.id, name: r.name, key_type: r.key_type,
        public_key: r.public_key, fingerprint: r.fingerprint,
        created_at: r.created_at,
    }).collect())
}

/// 秘密鍵を取得（VPS への接続時に使用）
pub async fn get_private_key(pool: &Pool, key_id: &str) -> Result<Option<String>> {
    Ok(sqlx::query!("SELECT private_key FROM ssh_keys WHERE id=?", key_id)
        .fetch_optional(pool).await?.map(|r| r.private_key))
}

pub async fn delete(pool: &Pool, id: &str) -> Result<()> {
    sqlx::query!("DELETE FROM ssh_keys WHERE id=?", id).execute(pool).await?;
    Ok(())
}
