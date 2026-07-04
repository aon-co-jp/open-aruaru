pub mod apache;
pub mod nginx;
pub mod recommender;

use anyhow::Result;
use crate::db::Pool;

/// ドメインを Web サーバーに自動登録（VI操作不要）
/// - 設定ファイル生成・書き込み・有効化・テスト・リロードを一括実行
pub async fn apply_domain_config(
    pool:        &Pool,
    domain_id:   &str,
    domain:      &str,
    webserver:   &str,
    https:       bool,
    proxy_port:  Option<u16>,
    admin_email: &str,
) -> Result<()> {
    match webserver {
        "apache" => apache::apply_domain(domain, https).await?,
        _        => nginx::apply_domain(domain, https, proxy_port).await?,
    }

    // HTTPS が有効なら証明書も自動取得
    if https {
        crate::domain::ssl::setup_https(pool, domain_id, domain, admin_email, webserver).await?;
    }

    Ok(())
}

/// Web サーバー切り替え（Apache ↔ Nginx）
pub async fn switch_webserver(
    pool:      &Pool,
    domain_id: &str,
    domain:    &str,
    from:      &str,
    to:        &str,
    https:     bool,
    proxy_port: Option<u16>,
    admin_email: &str,
) -> Result<()> {
    // 旧設定を削除
    match from {
        "apache" => apache::remove_vhost(domain).await?,
        _        => nginx::remove_vhost(domain).await?,
    }
    // 新設定を適用
    apply_domain_config(pool, domain_id, domain, to, https, proxy_port, admin_email).await
}

/// ドメイン削除時に設定ファイルも自動削除
pub async fn remove_domain_config(domain: &str, webserver: &str) -> Result<()> {
    match webserver {
        "apache" => apache::remove_vhost(domain).await?,
        _        => nginx::remove_vhost(domain).await?,
    }
    Ok(())
}
