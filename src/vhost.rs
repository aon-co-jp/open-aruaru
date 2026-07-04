//! Apache / Nginx vhost 生成と proxy リロード

use std::path::{Path, PathBuf};
use std::process::Command;

use tracing::info;

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct VhostEntry {
    pub name: String,
    pub https_enabled: bool,
}

pub async fn regenerate_all(state: &AppState) -> anyhow::Result<PathBuf> {
    let mut entries = Vec::new();
    for d in state.list_platform_domains().await? {
        entries.push(VhostEntry {
            name: d.name,
            https_enabled: d.https_enabled != 0,
        });
    }
    for d in state.list_domains().await? {
        entries.push(VhostEntry {
            name: d.name,
            https_enabled: d.https_enabled != 0,
        });
    }
    write_configs(state, &entries)
}

pub fn write_configs(state: &AppState, entries: &[VhostEntry]) -> anyhow::Result<PathBuf> {
    let base = state.data_dir.join("vhost");
    let nginx_dir = base.join("nginx");
    let apache_dir = base.join("apache");
    std::fs::create_dir_all(&nginx_dir)?;
    std::fs::create_dir_all(&apache_dir)?;

    let port = state.config.server.port;
    let acme = acme_webroot(state);

    let mut nginx = String::from("# aruaru — auto-generated nginx vhosts\n\n");
    let mut apache = String::from("# aruaru — auto-generated apache vhosts\n\n");

    for d in entries {
        let cert = cert_path(&d.name);
        let ssl = d.https_enabled && cert.exists();
        nginx.push_str(&nginx_server(&d.name, port, &acme, ssl, &cert));
        apache.push_str(&apache_vhost(&d.name, port, &acme, ssl, &cert));
    }

    if entries.is_empty() {
        nginx.push_str(&format!("# No domains. aruaru on 127.0.0.1:{port}\n"));
    }

    std::fs::write(nginx_dir.join("aruaru.conf"), &nginx)?;
    std::fs::write(apache_dir.join("aruaru.conf"), &apache)?;
    std::fs::write(
        base.join("README.txt"),
        format!(
            "nginx: {}\napache: {}\nRun feature vhost.reload after symlink to sites-enabled\n",
            nginx_dir.join("aruaru.conf").display(),
            apache_dir.join("aruaru.conf").display()
        ),
    )?;

    info!("vhost regenerated: {}", base.display());
    Ok(base)
}

fn acme_webroot(state: &AppState) -> PathBuf {
    let wr = &state.config.https_monitor.acme_webroot;
    let p = PathBuf::from(wr);
    if p.is_absolute() {
        p
    } else {
        state.data_dir.join(wr)
    }
}

fn cert_path(domain: &str) -> PathBuf {
    PathBuf::from("/etc/letsencrypt/live")
        .join(domain)
        .join("fullchain.pem")
}

/// example.com → www.example.com を返す（サブドメインはそのまま）
pub fn www_variant(domain: &str) -> Option<String> {
    // すでに www. 始まりなら不要
    if domain.starts_with("www.") {
        return None;
    }
    // ドット数が1つ（example.com）→ www 付与
    // ドットが2つ以上（sub.example.com）→ www 不要
    let dots = domain.chars().filter(|&c| c == '.').count();
    if dots == 1 {
        Some(format!("www.{domain}"))
    } else {
        None
    }
}

fn nginx_server(domain: &str, port: u16, acme: &Path, ssl: bool, cert: &Path) -> String {
    let www = www_variant(domain);
    // server_name に www も追加
    let server_name = match &www {
        Some(w) => format!("{domain} {w}"),
        None => domain.to_string(),
    };

    if ssl {
        let key = match cert.parent() {
            Some(dir) => dir.join("privkey.pem"),
            None => {
                tracing::warn!("vhost: cert path has no parent dir: {}", cert.display());
                return format!("# ERROR: invalid cert path for {domain}\n\n");
            }
        };
        format!(
            r#"server {{
    listen 80;
    server_name {server_name};
    location /.well-known/acme-challenge/ {{ root {acme}; }}
    return 301 https://$host$request_uri;
}}
server {{
    listen 443 ssl;
    server_name {server_name};
    ssl_certificate {cert};
    ssl_certificate_key {key};
    proxy_set_header X-Forwarded-Proto https;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_set_header Host $host;
    }}
}}

"#,
            acme = acme.display(),
            cert = cert.display(),
            key = key.display()
        )
    } else {
        format!(
            r#"server {{
    listen 80;
    server_name {server_name};
    location /.well-known/acme-challenge/ {{ root {acme}; }}
    location / {{
        proxy_pass http://127.0.0.1:{port};
        proxy_set_header Host $host;
    }}
}}

"#,
            acme = acme.display()
        )
    }
}

fn apache_vhost(domain: &str, port: u16, acme: &Path, ssl: bool, cert: &Path) -> String {
    let www = www_variant(domain);
    let server_alias = www
        .as_deref()
        .map(|w| format!("    ServerAlias {w}\n"))
        .unwrap_or_default();

    if ssl {
        let key = match cert.parent() {
            Some(dir) => dir.join("privkey.pem"),
            None => {
                tracing::warn!("vhost: cert path has no parent dir: {}", cert.display());
                return format!("# ERROR: invalid cert path for {domain}\n\n");
            }
        };
        format!(
            r#"<VirtualHost *:443>
    ServerName {domain}
{server_alias}    SSLEngine on
    SSLCertificateFile {cert}
    SSLCertificateKeyFile {key}
    ProxyPass / http://127.0.0.1:{port}/
    ProxyPassReverse / http://127.0.0.1:{port}/
</VirtualHost>

"#,
            cert = cert.display(),
            key = key.display()
        )
    } else {
        format!(
            r#"<VirtualHost *:80>
    ServerName {domain}
{server_alias}    Alias /.well-known/acme-challenge/ {acme}
    ProxyPass / http://127.0.0.1:{port}/
    ProxyPassReverse / http://127.0.0.1:{port}/
</VirtualHost>

"#,
            acme = acme.display()
        )
    }
}

/// nginx/apache 設定を sites-enabled 等へ symlink（Linux VPS）
pub fn install_symlinks(state: &AppState) -> anyhow::Result<String> {
    let base = state.data_dir.join("vhost");
    let nginx_conf = base.join("nginx").join("aruaru.conf");
    let apache_conf = base.join("apache").join("aruaru.conf");
    let ws = state.config.https_monitor.web_server.to_lowercase();

    #[cfg(target_os = "linux")]
    {
        if ws == "nginx" && nginx_conf.exists() {
            let msg = install_with_sudo("nginx", &nginx_conf)?;
            return Ok(msg);
        }
        if ws == "apache" && apache_conf.exists() {
            let msg = install_with_sudo("apache", &apache_conf)?;
            return Ok(msg);
        }
    }

    let _ = (nginx_conf, apache_conf);
    Ok(format!(
        "web_server={ws} — vhost files at {} (run on Linux VPS as root/sudo)",
        base.display()
    ))
}

#[cfg(target_os = "linux")]
fn install_with_sudo(kind: &str, src: &Path) -> anyhow::Result<String> {
    let (target, enabled) = if kind == "nginx" {
        (
            "/etc/nginx/sites-available/aruaru.conf",
            "/etc/nginx/sites-enabled/aruaru.conf",
        )
    } else {
        (
            "/etc/apache2/sites-available/aruaru.conf",
            "/etc/apache2/sites-enabled/aruaru.conf",
        )
    };
    let src_s = src.to_string_lossy();
    let cmds = [
        format!("cp '{src_s}' '{target}'"),
        format!("ln -sf '{target}' '{enabled}'"),
    ];
    for script in cmds {
        if !run_sudo_sh(&script)? {
            anyhow::bail!("sudo failed: {script}");
        }
    }
    Ok(format!("{kind} installed: {enabled}"))
}

#[cfg(not(target_os = "linux"))]
fn install_with_sudo(_kind: &str, _src: &Path) -> anyhow::Result<String> {
    anyhow::bail!("vhost install requires Linux VPS")
}

#[cfg(target_os = "linux")]
fn run_sudo_sh(script: &str) -> anyhow::Result<bool> {
    let o = Command::new("sudo")
        .args(["sh", "-c", script])
        .output()?;
    Ok(o.status.success())
}

pub fn reload_proxy(web_server: &str) -> anyhow::Result<String> {
    let ws = web_server.to_lowercase();
    let msg = if ws == "nginx" {
        try_commands(&[
            &["nginx", "-t"],
            &["systemctl", "reload", "nginx"],
        ])
    } else if ws == "apache" {
        try_commands(&[
            &["apachectl", "configtest"],
            &["systemctl", "reload", "apache2"],
            &["systemctl", "reload", "httpd"],
        ])
    } else {
        return Ok("web_server=none — vhost files only".into());
    };
    Ok(msg)
}

fn try_commands(candidates: &[&[&str]]) -> String {
    for args in candidates {
        if let Ok(o) = Command::new(args[0]).args(&args[1..]).output() {
            if o.status.success() {
                return format!("{} OK", args.join(" "));
            }
        }
    }
    "reload skipped (need root + nginx/apache)".into()
}
