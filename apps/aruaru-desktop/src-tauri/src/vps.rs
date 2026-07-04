//! VPS 接続ウィザード・ダッシュボード（SSH）

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::{
    desktop_config_root, load_vps_config, probe_vps_connection, probe_vps_direct, read_toml_value,
    run_ssh, ssh_home, VpsConnectionStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpsWizardInput {
    pub provider: String,
    pub host: String,
    pub user: String,
    pub key_path: String,
    pub remote_dir: String,
    pub save_key_copy: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WizardCheck {
    pub id: String,
    pub label: String,
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WizardTestResult {
    pub ok: bool,
    pub checks: Vec<WizardCheck>,
    pub connection: Option<VpsConnectionStatus>,
    pub public_key: Option<String>,
    pub fix_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SshKeyInfo {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServerDashboard {
    pub ssh_ok: bool,
    pub host: String,
    pub status: Option<serde_json::Value>,
    pub error: Option<String>,
    pub checked_at: String,
}

pub fn list_ssh_keys() -> Vec<SshKeyInfo> {
    let dir = ssh_home();
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for e in entries.flatten() {
            let path = e.path();
            if !path.is_file() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_string();
            if name.ends_with(".pem") || name == "id_ed25519" || name == "id_rsa" {
                out.push(SshKeyInfo {
                    name: name.clone(),
                    path: path.to_string_lossy().to_string(),
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

pub fn generate_ssh_key(name: String) -> Result<SshKeyInfo, String> {
    let safe = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let file_name = if safe.ends_with(".pem") {
        safe
    } else {
        format!("{safe}.pem")
    };
    let path = ssh_home().join(&file_name);
    if path.exists() {
        return Err(format!("already exists: {}", path.display()));
    }
    std::fs::create_dir_all(ssh_home()).map_err(|e| e.to_string())?;

    let out = Command::new("ssh-keygen")
        .args([
            "-t",
            "ed25519",
            "-f",
            &path.to_string_lossy(),
            "-N",
            "",
            "-C",
            "aruaru-desktop",
        ])
        .output()
        .map_err(|e| format!("ssh-keygen not found: {e}"))?;

    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }

    Ok(SshKeyInfo {
        name: file_name,
        path: path.to_string_lossy().to_string(),
    })
}

pub fn public_key_from_private(key_path: &str) -> Result<String, String> {
    let out = Command::new("ssh-keygen")
        .args(["-y", "-f", key_path])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn diagnose_ssh_error(msg: &str) -> Option<String> {
    let m = msg.to_lowercase();
    if m.contains("permission denied") {
        Some(
            "秘密鍵が VPS に登録されていません。ConoHa 管理画面で公開鍵を登録するか、正しい .pem に差し替えてください。"
                .into(),
        )
    } else if m.contains("bad permissions") || m.contains("unprotected private key") {
        Some(
            "鍵ファイルの権限が緩すぎます。PowerShell: icacls \"鍵パス\" /inheritance:r /grant:r \"%USERNAME%:(R)\""
                .into(),
        )
    } else if m.contains("connection timed out") || m.contains("timed out") {
        Some("IP アドレスまたはファイアウォール（ポート22）を確認してください。".into())
    } else if m.contains("could not resolve") {
        Some("ホスト名を解決できません。IP アドレスを直接入力してください。".into())
    } else {
        None
    }
}

pub fn test_wizard_connection(input: &VpsWizardInput) -> WizardTestResult {
    let mut checks = Vec::new();
    let key = PathBuf::from(&input.key_path);

    let host_ok = !input.host.trim().is_empty();
    checks.push(WizardCheck {
        id: "host".into(),
        label: "IP / ホスト".into(),
        ok: host_ok,
        message: if host_ok {
            input.host.clone()
        } else {
            "ホストを入力してください".into()
        },
    });

    let key_exists = key.exists();
    checks.push(WizardCheck {
        id: "key".into(),
        label: "秘密鍵".into(),
        ok: key_exists,
        message: if key_exists {
            key.to_string_lossy().to_string()
        } else {
            "秘密鍵ファイルが見つかりません".into()
        },
    });

    let user_ok = !input.user.trim().is_empty();
    checks.push(WizardCheck {
        id: "user".into(),
        label: "ユーザー名".into(),
        ok: user_ok,
        message: input.user.clone(),
    });

    let public_key = if key_exists {
        public_key_from_private(&input.key_path).ok()
    } else {
        None
    };

    let mut ssh_ok = false;
    let mut ssh_msg = String::new();
    if host_ok && key_exists && user_ok {
        let target = format!("{}@{}", input.user.trim(), input.host.trim());
        match run_ssh(&key, &target, "echo ok") {
            Ok(s) if s == "ok" => {
                ssh_ok = true;
                ssh_msg = "接続成功".into();
            }
            Ok(s) => ssh_msg = format!("unexpected: {s}"),
            Err(e) => ssh_msg = e,
        }
    } else {
        ssh_msg = "上記の項目を修正してください".into();
    }

    checks.push(WizardCheck {
        id: "ssh".into(),
        label: "SSH 接続（ポート22）".into(),
        ok: ssh_ok,
        message: ssh_msg.clone(),
    });

    let fix_hint = if ssh_ok {
        None
    } else {
        diagnose_ssh_error(&ssh_msg)
    };

    WizardTestResult {
        ok: ssh_ok,
        checks,
        connection: if ssh_ok {
            Some(probe_vps_direct(
                input.host.trim(),
                input.user.trim(),
                &key,
                input.remote_dir.trim(),
                chrono::Utc::now().to_rfc3339(),
            ))
        } else {
            None
        },
        public_key,
        fix_hint,
    }
}

pub fn save_wizard_config(input: &VpsWizardInput, root_dir: &str) -> Result<(), String> {
    let root = PathBuf::from(root_dir);
    save_wizard_config_to_files(input, &root)?;

    if input.save_key_copy {
        let dest_name = read_toml_value(
            &root.join("config").join("vps.toml"),
            "vps",
            "ssh_key_name",
        )
        .unwrap_or_else(|| "conoha-vps.pem".into());
        let dest = ssh_home().join(&dest_name);
        if Path::new(&input.key_path) != dest.as_path() {
            std::fs::copy(&input.key_path, &dest).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn save_wizard_config_to_files(input: &VpsWizardInput, root: &Path) -> Result<(), String> {
    let config_dir = root.join("config");
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let cfg_path = config_dir.join("vps.toml");
    if !cfg_path.exists() {
        let default = format!(
            r#"# aruaru VPS — 接続ウィザードで生成
[vps]
host = "{}"
user = "{}"
ssh_alias = "conoha"
app_port = 3000
remote_dir = "{}"
ssh_key_name = "conoha-vps.pem"
"#,
            input.host.trim(),
            input.user.trim(),
            input.remote_dir.trim()
        );
        std::fs::write(&cfg_path, default).map_err(|e| e.to_string())?;
    }

    let local = format!(
        r#"# 接続ウィザード — local override
[vps]
host = "{}"
user = "{}"
ssh_key_path = "{}"
"#,
        input.host.trim(),
        input.user.trim(),
        input.key_path.replace('\\', "/")
    );
    std::fs::write(config_dir.join("vps.local.toml"), local).map_err(|e| e.to_string())?;

    let _ = input.provider;
    Ok(())
}

pub fn fetch_server_dashboard(root_dir: Option<String>) -> ServerDashboard {
    let checked_at = chrono::Utc::now().to_rfc3339();
    let root = root_dir
        .map(PathBuf::from)
        .unwrap_or_else(desktop_config_root);

    let conn = probe_vps_connection(&root);
    if !conn.ssh_ok {
        return ServerDashboard {
            ssh_ok: false,
            host: conn.host,
            status: None,
            error: conn.error,
            checked_at,
        };
    }

    let cfg = match load_vps_config(&root) {
        Ok(c) => c,
        Err(e) => {
            return ServerDashboard {
                ssh_ok: false,
                host: conn.host,
                status: None,
                error: Some(e),
                checked_at,
            };
        }
    };

    let target = format!("{}@{}", cfg.user, cfg.host);
    match run_ssh(
        &cfg.key_path,
        &target,
        "curl -sf -m 5 http://127.0.0.1:3000/api/server/status || echo '{}'",
    ) {
        Ok(raw) if raw.starts_with('{') => ServerDashboard {
            ssh_ok: true,
            host: cfg.host,
            status: serde_json::from_str(&raw).ok(),
            error: None,
            checked_at,
        },
        Ok(raw) => ServerDashboard {
            ssh_ok: true,
            host: cfg.host,
            status: None,
            error: Some(if raw.is_empty() {
                "aruaru /api/server/status に応答がありません".into()
            } else {
                raw
            }),
            checked_at,
        },
        Err(e) => ServerDashboard {
            ssh_ok: true,
            host: cfg.host,
            status: None,
            error: Some(e),
            checked_at,
        },
    }
}
