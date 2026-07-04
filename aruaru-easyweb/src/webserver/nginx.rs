//! Nginx 設定の自動生成・書き込み・有効化・リロード（VI操作不要）

use anyhow::Result;
use std::path::Path;
use tracing::info;

pub async fn apply_domain(domain: &str, https: bool, proxy_port: Option<u16>) -> Result<()> {
    let config = generate_vhost(domain, https, proxy_port);
    write_vhost(domain, &config).await?;
    enable_site(domain).await?;
    test_config().await?;
    reload().await?;
    info!("Nginx 自動適用完了: {} (https={}, www自動対応)", domain, https);
    Ok(())
}

/// バーチャルホスト設定生成（www/非www 両対応 + HTTP→HTTPS 自動リダイレクト）
pub fn generate_vhost(domain: &str, https: bool, proxy_port: Option<u16>) -> String {
    let bare = domain.trim_start_matches("www.");
    let www  = format!("www.{}", bare);

    let upstream = match proxy_port {
        Some(port) => format!(
            "    location / {{\n\
             \t\tproxy_pass         http://127.0.0.1:{port};\n\
             \t\tproxy_http_version 1.1;\n\
             \t\tproxy_set_header   Upgrade $http_upgrade;\n\
             \t\tproxy_set_header   Connection 'upgrade';\n\
             \t\tproxy_set_header   Host $host;\n\
             \t\tproxy_set_header   X-Real-IP $remote_addr;\n\
             \t\tproxy_set_header   X-Forwarded-For $proxy_add_x_forwarded_for;\n\
             \t\tproxy_set_header   X-Forwarded-Proto $scheme;\n\
             \t}}"
        ),
        None => format!(
            "    root  /var/www/{bare}/public;\n\
             \tindex index.html index.php;\n\n\
             \tlocation / {{\n\
             \t\ttry_files $uri $uri/ =404;\n\
             \t}}"
        ),
    };

    if https {
        format!(
r#"# ============================================================
# {bare}  —  自動生成 by aruaru-easyweb（編集不要）
# ============================================================

# [1] HTTP → HTTPS リダイレクト（bare + www 両方）
server {{
    listen      80;
    listen      [::]:80;
    server_name {bare} {www};
    return 301  https://{bare}$request_uri;
}}

# [2] www → bare リダイレクト（HTTPS）
server {{
    listen      443 ssl http2;
    listen      [::]:443 ssl http2;
    server_name {www};

    ssl_certificate     /etc/letsencrypt/live/{bare}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/{bare}/privkey.pem;
    include             /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam         /etc/letsencrypt/ssl-dhparams.pem;

    return 301  https://{bare}$request_uri;
}}

# [3] メインサーバー
server {{
    listen      443 ssl http2;
    listen      [::]:443 ssl http2;
    server_name {bare};

    ssl_certificate     /etc/letsencrypt/live/{bare}/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/{bare}/privkey.pem;
    include             /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam         /etc/letsencrypt/ssl-dhparams.pem;

    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options    nosniff;
    add_header X-Frame-Options           SAMEORIGIN;
    add_header X-XSS-Protection          "1; mode=block";

    gzip            on;
    gzip_vary       on;
    gzip_min_length 1024;
    gzip_types      text/plain text/css application/json
                    application/javascript text/xml image/svg+xml;

    access_log /var/log/nginx/{bare}_access.log;
    error_log  /var/log/nginx/{bare}_error.log warn;

{upstream}
}}
"#)
    } else {
        format!(
r#"# ============================================================
# {bare}  —  自動生成 by aruaru-easyweb（編集不要）
# ============================================================

# [1] www → bare リダイレクト（HTTP）
server {{
    listen      80;
    listen      [::]:80;
    server_name {www};
    return 301  http://{bare}$request_uri;
}}

# [2] メインサーバー（HTTP）
server {{
    listen      80;
    listen      [::]:80;
    server_name {bare};

    gzip            on;
    gzip_types      text/plain text/css application/json application/javascript;

    access_log /var/log/nginx/{bare}_access.log;
    error_log  /var/log/nginx/{bare}_error.log warn;

{upstream}
}}
"#)
    }
}

pub async fn write_vhost(domain: &str, config: &str) -> Result<()> {
    let bare = domain.trim_start_matches("www.");
    let path = format!("/etc/nginx/sites-available/{}.conf", bare);
    tokio::fs::write(&path, config).await?;
    info!("Nginx 設定書き込み: {}", path);
    Ok(())
}

pub async fn enable_site(domain: &str) -> Result<()> {
    let bare    = domain.trim_start_matches("www.");
    let avail   = format!("/etc/nginx/sites-available/{}.conf", bare);
    let enabled = format!("/etc/nginx/sites-enabled/{}.conf",   bare);
    if Path::new(&enabled).exists() { tokio::fs::remove_file(&enabled).await?; }
    tokio::fs::symlink(&avail, &enabled).await?;
    info!("Nginx 有効化: {}", bare);
    Ok(())
}

pub async fn disable_site(domain: &str) -> Result<()> {
    let enabled = format!("/etc/nginx/sites-enabled/{}.conf", domain.trim_start_matches("www."));
    if Path::new(&enabled).exists() { tokio::fs::remove_file(&enabled).await?; }
    Ok(())
}

pub async fn test_config() -> Result<()> {
    let out = tokio::process::Command::new("nginx").arg("-t").output().await?;
    if !out.status.success() {
        anyhow::bail!("Nginx 設定テスト失敗:\n{}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(())
}

pub async fn reload() -> Result<()> {
    let out = tokio::process::Command::new("nginx").args(["-s","reload"]).output().await?;
    if !out.status.success() {
        anyhow::bail!("Nginx リロード失敗: {}", String::from_utf8_lossy(&out.stderr));
    }
    info!("Nginx リロード完了");
    Ok(())
}

pub async fn remove_vhost(domain: &str) -> Result<()> {
    let bare = domain.trim_start_matches("www.");
    disable_site(bare).await?;
    let path = format!("/etc/nginx/sites-available/{}.conf", bare);
    if Path::new(&path).exists() { tokio::fs::remove_file(&path).await?; }
    reload().await?;
    Ok(())
}
