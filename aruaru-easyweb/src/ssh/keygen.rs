//! SSH 鍵ペアの自動生成（Ed25519 / RSA 4096）
//! ブラウザの「鍵を生成」ボタン一発で生成→DB 保存→VPS に自動登録

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use crate::db::Pool;

#[derive(Debug, serde::Serialize)]
pub struct KeyPair {
    pub id:          String,
    pub name:        String,
    pub public_key:  String,
    pub private_key: String,
    pub key_type:    String,
    pub fingerprint: String,
    pub created_at:  String,
}

/// Ed25519 鍵ペアを生成して DB に保存
pub async fn generate_ed25519(pool: &Pool, name: &str) -> Result<KeyPair> {
    // openssh コマンドで生成（最も互換性が高い）
    let tmp_dir  = std::env::temp_dir();
    let key_path = tmp_dir.join(format!("aruaru_keygen_{}", Uuid::new_v4()));

    let out = tokio::process::Command::new("ssh-keygen")
        .args([
            "-t", "ed25519",
            "-C", &format!("aruaru-easyweb/{}", name),
            "-f", key_path.to_str().unwrap(),
            "-N", "",   // パスフレーズなし
            "-q",
        ])
        .output()
        .await?;

    if !out.status.success() {
        anyhow::bail!("鍵生成失敗: {}", String::from_utf8_lossy(&out.stderr));
    }

    let private_key = tokio::fs::read_to_string(&key_path).await?;
    let public_key  = tokio::fs::read_to_string(format!("{}.pub", key_path.display())).await?;

    // フィンガープリント取得
    let fp_out = tokio::process::Command::new("ssh-keygen")
        .args(["-l", "-f", key_path.to_str().unwrap()])
        .output()
        .await?;
    let fingerprint = String::from_utf8_lossy(&fp_out.stdout).trim().to_string();

    // 一時ファイル削除
    let _ = tokio::fs::remove_file(&key_path).await;
    let _ = tokio::fs::remove_file(format!("{}.pub", key_path.display())).await;

    save_to_db(pool, name, &public_key, &private_key, "ed25519", &fingerprint).await
}

/// RSA 4096 鍵ペアを生成して DB に保存
pub async fn generate_rsa4096(pool: &Pool, name: &str) -> Result<KeyPair> {
    let tmp_dir  = std::env::temp_dir();
    let key_path = tmp_dir.join(format!("aruaru_keygen_{}", Uuid::new_v4()));

    let out = tokio::process::Command::new("ssh-keygen")
        .args([
            "-t", "rsa",
            "-b", "4096",
            "-C", &format!("aruaru-easyweb/{}", name),
            "-f", key_path.to_str().unwrap(),
            "-N", "",
            "-q",
        ])
        .output()
        .await?;

    if !out.status.success() {
        anyhow::bail!("鍵生成失敗: {}", String::from_utf8_lossy(&out.stderr));
    }

    let private_key = tokio::fs::read_to_string(&key_path).await?;
    let public_key  = tokio::fs::read_to_string(format!("{}.pub", key_path.display())).await?;
    let fp_out = tokio::process::Command::new("ssh-keygen")
        .args(["-l", "-f", key_path.to_str().unwrap()])
        .output()
        .await?;
    let fingerprint = String::from_utf8_lossy(&fp_out.stdout).trim().to_string();

    let _ = tokio::fs::remove_file(&key_path).await;
    let _ = tokio::fs::remove_file(format!("{}.pub", key_path.display())).await;

    save_to_db(pool, name, &public_key, &private_key, "rsa4096", &fingerprint).await
}

async fn save_to_db(
    pool: &Pool, name: &str,
    public_key: &str, private_key: &str,
    key_type: &str, fingerprint: &str,
) -> Result<KeyPair> {
    let id  = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query!(
        "INSERT INTO ssh_keypairs (id,name,public_key,private_key,key_type,fingerprint,created_at)
         VALUES (?,?,?,?,?,?,?)",
        id, name, public_key, private_key, key_type, fingerprint, now
    )
    .execute(pool)
    .await?;

    Ok(KeyPair {
        id, name: name.to_string(),
        public_key: public_key.to_string(),
        private_key: private_key.to_string(),
        key_type: key_type.to_string(),
        fingerprint: fingerprint.to_string(),
        created_at: now,
    })
}

/// VPS の authorized_keys に公開鍵を自動登録
pub async fn install_pubkey_to_vps(
    ip: &str, port: u16, user: &str,
    auth: &super::SshAuth,
    public_key: &str,
) -> Result<()> {
    let sess = super::connect(ip, port, user, auth).await?;

    // ~/.ssh ディレクトリと authorized_keys を作成・追記
    let cmd = format!(
        r#"mkdir -p ~/.ssh && chmod 700 ~/.ssh && \
           echo '{}' >> ~/.ssh/authorized_keys && \
           chmod 600 ~/.ssh/authorized_keys && \
           echo 'OK'"#,
        public_key.trim()
    );

    let (stdout, stderr, code) = super::exec(&sess, &cmd)?;
    if code != 0 {
        anyhow::bail!("公開鍵登録失敗: {}", stderr);
    }
    tracing::info!("公開鍵を VPS に登録: {} → {}", ip, stdout.trim());
    Ok(())
}

pub async fn list_keypairs(pool: &Pool) -> Result<Vec<KeyPair>> {
    let rows = sqlx::query!(
        "SELECT id,name,public_key,private_key,key_type,fingerprint,created_at FROM ssh_keypairs ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|r| KeyPair {
        id:          r.id,
        name:        r.name,
        public_key:  r.public_key,
        private_key: r.private_key,
        key_type:    r.key_type,
        fingerprint: r.fingerprint,
        created_at:  r.created_at,
    }).collect())
}
