//! Apache 設定の自動生成・書き込み・有効化・リロード（VI操作不要）

use anyhow::Result;
use std::path::Path;
use tracing::info;

pub async fn apply_domain(domain: &str, https: bool) -> Result<()> {
    let config = generate_vhost(domain, https);
    write_vhost(domain, &config).await?;
    enable_modules().await?;
    enable_site(domain).await?;
    test_config().await?;
    reload().await?;
    info!("Apache 自動適用完了: {} (https={}, www自動対応)", domain, https);
    Ok(())
}

/// バーチャルホスト設定生成（www/非www 両対応）
pub fn generate_vhost(domain: &str, https: bool) -> String {
    let bare = domain.trim_start_matches("www.");
    let www  = format!("www.{}", bare);

    if https {
        format!(
r#"# ============================================================
# {bare}  —  自動生成 by aruaru-easyweb（編集不要）
# ============================================================

# [1] HTTP → HTTPS リダイレクト
<VirtualHost *:80>
    ServerName  {bare}
    ServerAlias {www}
    RewriteEngine On
    RewriteRule ^ https://{bare}%{{REQUEST_URI}} [R=301,L]
</VirtualHost>

# [2] www → bare リダイレクト（HTTPS）
<VirtualHost *:443>
    ServerName {www}
    SSLEngine on
    SSLCertificateFile    /etc/letsencrypt/live/{bare}/fullchain.pem
    SSLCertificateKeyFile /etc/letsencrypt/live/{bare}/privkey.pem
    RewriteEngine On
    RewriteRule ^ https://{bare}%{{REQUEST_URI}} [R=301,L]
</VirtualHost>

# [3] メインサーバー
<VirtualHost *:443>
    ServerName  {bare}
    DocumentRoot /var/www/{bare}/public

    SSLEngine on
    SSLCertificateFile    /etc/letsencrypt/live/{bare}/fullchain.pem
    SSLCertificateKeyFile /etc/letsencrypt/live/{bare}/privkey.pem

    Header always set Strict-Transport-Security "max-age=31536000; includeSubDomains"
    Header always set X-Content-Type-Options    nosniff
    Header always set X-Frame-Options           SAMEORIGIN

    <IfModule mod_deflate.c>
        AddOutputFilterByType DEFLATE text/html text/plain text/css
        AddOutputFilterByType DEFLATE application/javascript application/json
    </IfModule>

    <Directory /var/www/{bare}/public>
        Options -Indexes +FollowSymLinks
        AllowOverride All
        Require all granted
    </Directory>

    ErrorLog  ${{APACHE_LOG_DIR}}/{bare}_error.log
    CustomLog ${{APACHE_LOG_DIR}}/{bare}_access.log combined
</VirtualHost>
"#)
    } else {
        format!(
r#"# ============================================================
# {bare}  —  自動生成 by aruaru-easyweb（編集不要）
# ============================================================

# [1] www → bare リダイレクト（HTTP）
<VirtualHost *:80>
    ServerName {www}
    RewriteEngine On
    RewriteRule ^ http://{bare}%{{REQUEST_URI}} [R=301,L]
</VirtualHost>

# [2] メインサーバー（HTTP）
<VirtualHost *:80>
    ServerName  {bare}
    DocumentRoot /var/www/{bare}/public

    <IfModule mod_deflate.c>
        AddOutputFilterByType DEFLATE text/html text/plain text/css application/javascript
    </IfModule>

    <Directory /var/www/{bare}/public>
        Options -Indexes +FollowSymLinks
        AllowOverride All
        Require all granted
    </Directory>

    ErrorLog  ${{APACHE_LOG_DIR}}/{bare}_error.log
    CustomLog ${{APACHE_LOG_DIR}}/{bare}_access.log combined
</VirtualHost>
"#)
    }
}

pub async fn write_vhost(domain: &str, config: &str) -> Result<()> {
    let bare = domain.trim_start_matches("www.");
    let path = format!("/etc/apache2/sites-available/{}.conf", bare);
    tokio::fs::write(&path, config).await?;
    info!("Apache 設定書き込み: {}", path);
    Ok(())
}

/// 必要モジュールを自動有効化（rewrite, ssl, headers, deflate）
pub async fn enable_modules() -> Result<()> {
    for m in &["rewrite", "ssl", "headers", "deflate"] {
        let _ = tokio::process::Command::new("a2enmod").arg(m).output().await;
    }
    Ok(())
}

pub async fn enable_site(domain: &str) -> Result<()> {
    let bare = domain.trim_start_matches("www.");
    let out = tokio::process::Command::new("a2ensite")
        .arg(format!("{}.conf", bare))
        .output().await?;
    if !out.status.success() {
        anyhow::bail!("a2ensite 失敗: {}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(())
}

pub async fn test_config() -> Result<()> {
    let out = tokio::process::Command::new("apachectl").arg("configtest").output().await?;
    if !out.status.success() {
        anyhow::bail!("Apache 設定テスト失敗:\n{}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(())
}

pub async fn reload() -> Result<()> {
    let out = tokio::process::Command::new("systemctl").args(["reload","apache2"]).output().await?;
    if !out.status.success() {
        anyhow::bail!("Apache リロード失敗: {}", String::from_utf8_lossy(&out.stderr));
    }
    info!("Apache リロード完了");
    Ok(())
}

pub async fn remove_vhost(domain: &str) -> Result<()> {
    let bare = domain.trim_start_matches("www.");
    let _ = tokio::process::Command::new("a2dissite").arg(format!("{}.conf", bare)).output().await;
    reload().await?;
    let path = format!("/etc/apache2/sites-available/{}.conf", bare);
    if Path::new(&path).exists() { tokio::fs::remove_file(&path).await?; }
    Ok(())
}
