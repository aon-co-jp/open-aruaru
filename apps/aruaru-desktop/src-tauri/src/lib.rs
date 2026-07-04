use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

mod vps;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    #[serde(default = "default_data_dir")]
    pub data_dir: String,
    #[serde(default = "default_root_dir")]
    pub root_dir: String,
    #[serde(default = "default_server")]
    pub server_url: String,
    #[serde(default = "default_os")]
    pub os_drive: String,
    #[serde(default = "default_data_drive")]
    pub data_drive: String,
}

fn default_data_dir() -> String {
    if Path::new("F:\\aruaru-data").exists() || Path::new("F:\\").exists() {
        "F:\\aruaru-data".into()
    } else {
        "E:\\aruaru-data".into()
    }
}
fn default_root_dir() -> String {
    if Path::new("F:\\aruaru").exists() {
        "F:\\aruaru".into()
    } else {
        "E:\\aruaru".into()
    }
}
fn default_server() -> String {
    "http://127.0.0.1:3000".into()
}
fn default_os() -> String {
    "C:".into()
}
fn default_data_drive() -> String {
    if Path::new("F:\\").exists() {
        "F:".into()
    } else {
        "E:".into()
    }
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            root_dir: default_root_dir(),
            server_url: default_server(),
            os_drive: default_os(),
            data_drive: default_data_drive(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DriveInfo {
    pub letter: String,
    pub label: String,
    pub free_gb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FootprintFile {
    data_dir: String,
    app_dir: String,
    data_drive: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FootprintScan {
    pub marker_path: String,
    pub data_dir: String,
    pub app_dir: String,
    pub data_drive: String,
    pub updated_at: String,
}

fn config_dir() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("aruaru")
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".config")
            .join("aruaru")
    }
}

fn config_path() -> PathBuf {
    config_dir().join("desktop.toml")
}

#[tauri::command]
fn get_desktop_config() -> Result<DesktopConfig, String> {
    let path = config_path();
    if !path.exists() {
        return Ok(DesktopConfig::default());
    }
    let raw = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    toml::from_str(&raw).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_desktop_config(config: DesktopConfig) -> Result<(), String> {
    std::fs::create_dir_all(config_dir()).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), toml::to_string_pretty(&config).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_drives() -> Result<Vec<DriveInfo>, String> {
    let mut out = Vec::new();
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_LogicalDisk -Filter \"DriveType=3\" | ForEach-Object { \"$($_.DeviceID)|$($_.VolumeName)|$([math]::Round($_.FreeSpace/1GB,1))\" }",
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() < 3 {
                    continue;
                }
                let letter = parts[0].trim().to_string();
                if letter.is_empty() {
                    continue;
                }
                let label = parts[1].trim().to_string();
                let free_gb = parts[2].trim().parse::<f64>().unwrap_or(0.0) as u64;
                out.push(DriveInfo {
                    letter,
                    label,
                    free_gb,
                });
            }
        }
        if out.is_empty() {
            for letter in b'A'..=b'Z' {
                let root = format!("{}:\\", letter as char);
                if Path::new(&root).exists() {
                    out.push(DriveInfo {
                        letter: format!("{}:", letter as char),
                        label: String::new(),
                        free_gb: 0,
                    });
                }
            }
        }
    }
    #[cfg(not(windows))]
    {
        out.push(DriveInfo {
            letter: "/".into(),
            label: "root".into(),
            free_gb: 0,
        });
    }
    Ok(out)
}

#[tauri::command]
fn pick_folder(title: String) -> Option<String> {
    rfd::FileDialog::new()
        .set_title(&title)
        .pick_folder()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
async fn ping_aruaru(url: String) -> Result<bool, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .map_err(|e| e.to_string())?;
    Ok(client
        .get(&url)
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false))
}

#[tauri::command]
async fn install_plugin(server_url: String, id: String) -> Result<serde_json::Value, String> {
    api_post(&server_url, "/api/install", serde_json::json!({ "target": id })).await
}

#[tauri::command]
async fn uninstall_plugin(
    server_url: String,
    id: String,
    purge: bool,
) -> Result<serde_json::Value, String> {
    api_post(
        &server_url,
        "/api/uninstall",
        serde_json::json!({ "id": id, "purge": purge }),
    )
    .await
}

#[tauri::command]
async fn bind_drive(
    data_dir: String,
    app_dir: String,
    server_url: String,
) -> Result<serde_json::Value, String> {
    api_post(
        &server_url,
        "/api/recovery/bind",
        serde_json::json!({ "data_dir": data_dir, "app_dir": app_dir }),
    )
    .await
}

#[tauri::command]
async fn restore_access(
    marker_path: String,
    server_url: String,
) -> Result<String, String> {
    let res = api_post(
        &server_url,
        "/api/recovery/restore",
        serde_json::json!({ "marker_path": marker_path }),
    )
    .await?;
    if res.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return local_restore_access(&marker_path);
    }
    Ok(res
        .get("desktop_config")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

fn local_restore_access(marker_path: &str) -> Result<String, String> {
    let script = recovery_script("restore-access.ps1");
    let out = run_powershell(&script, &["-FootprintPath", marker_path])?;
    Ok(out.trim().to_string())
}

#[tauri::command]
fn scan_footprints() -> Result<Vec<FootprintScan>, String> {
    let marker = ".aruaru-footprint.json";
    let mut out = Vec::new();
    #[cfg(windows)]
    {
        for letter in b'A'..=b'Z' {
            let root = format!("{}:\\", letter as char);
            if Path::new(&root).exists() {
                scan_tree(Path::new(&root), marker, 4, &mut out);
            }
        }
    }
    Ok(out)
}

fn scan_tree(dir: &Path, marker: &str, depth: u32, out: &mut Vec<FootprintScan>) {
    if depth == 0 {
        return;
    }
    let direct = dir.join(marker);
    if direct.is_file() {
        if let Ok(raw) = std::fs::read_to_string(&direct) {
            if let Ok(fp) = serde_json::from_str::<FootprintFile>(&raw) {
                out.push(FootprintScan {
                    marker_path: direct.to_string_lossy().to_string(),
                    data_dir: fp.data_dir,
                    app_dir: fp.app_dir,
                    data_drive: fp.data_drive,
                    updated_at: fp.updated_at,
                });
            }
        }
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if ["Windows", "Program Files", "Program Files (x86)", "node_modules", "target"]
                .contains(&name.as_str())
                || name.starts_with('$')
            {
                continue;
            }
            scan_tree(&path, marker, depth - 1, out);
        }
    }
}

#[tauri::command]
fn run_upload_vps(root_dir: String, skip_setup: bool) -> Result<String, String> {
    let script = deploy_script("upload-vps.ps1");
    run_powershell(
        &script,
        &["-Root", &root_dir, if skip_setup { "-SkipSetup" } else { "" }]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>(),
    )
    .map(|s| s.trim().to_string())
}

fn deploy_script(name: &str) -> PathBuf {
    let root = desktop_config_root();
    let candidates = [
        root.join("deploy").join(name),
        PathBuf::from("deploy").join(name),
        PathBuf::from("../../deploy").join(name),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    candidates[0].clone()
}

#[tauri::command]
fn run_backup(data_dir: String, backup_dir: String) -> Result<String, String> {
    let script = recovery_script("backup-data.ps1");
    run_powershell(
        &script,
        &["-DataDir", &data_dir, "-BackupDir", &backup_dir],
    )
    .map(|s| s.trim().to_string())
}

#[tauri::command]
fn run_sync(data_dir: String, target_dir: String) -> Result<String, String> {
    let script = recovery_script("sync-to-target.ps1");
    run_powershell(
        &script,
        &["-DataDir", &data_dir, "-TargetDir", &target_dir],
    )
    .map(|s| s.trim().to_string())
}

fn recovery_script(name: &str) -> PathBuf {
    // 開発時: リポジトリ内 plugins。配布時: インストール先 plugins
    let candidates = [
        PathBuf::from("plugins/aruaru-os-recovery/scripts/windows").join(name),
        PathBuf::from("../../plugins/aruaru-os-recovery/scripts/windows").join(name),
        desktop_config_root()
            .join("plugins")
            .join("aruaru-os-recovery")
            .join("scripts/windows")
            .join(name),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    candidates[0].clone()
}

pub(crate) fn desktop_config_root() -> PathBuf {
    get_desktop_config()
        .map(|c| PathBuf::from(c.root_dir))
        .unwrap_or_else(|_| PathBuf::from("E:\\aruaru"))
}

pub(crate) fn read_toml_value(path: &Path, section: &str, key: &str) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut in_section = false;
    for line in raw.lines() {
        let t = line.trim();
        if let Some(s) = t.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_section = s == section;
            continue;
        }
        if !in_section {
            continue;
        }
        let prefix = format!("{key} = ");
        if let Some(rest) = t.strip_prefix(&prefix) {
            let val = rest.trim().trim_matches('"');
            return Some(val.to_string());
        }
    }
    None
}

#[derive(Debug, Clone)]
struct VpsTomlConfig {
    host: String,
    user: String,
    remote_dir: String,
    key_path: PathBuf,
}

pub(crate) fn load_vps_config(root: &Path) -> Result<VpsTomlConfig, String> {
    let cfg_path = root.join("config").join("vps.toml");
    let local_path = root.join("config").join("vps.local.toml");

    let mut host = read_toml_value(&local_path, "vps", "host")
        .or_else(|| read_toml_value(&cfg_path, "vps", "host"))
        .ok_or_else(|| "config/vps.toml: missing [vps] host".to_string())?;

    let user = read_toml_value(&cfg_path, "vps", "user").unwrap_or_else(|| "root".into());

    let remote_dir = read_toml_value(&cfg_path, "vps", "remote_dir")
        .unwrap_or_else(|| "/root/aruaru".into());

    let key_path = read_toml_value(&local_path, "vps", "ssh_key_path")
        .or_else(|| read_toml_value(&local_path, "vps", "ssh_key"))
        .map(PathBuf::from)
        .or_else(|| {
            read_toml_value(&cfg_path, "vps", "ssh_key_name").map(|name| {
                ssh_home().join(name)
            })
        })
        .unwrap_or_else(|| ssh_home().join("conoha-vps.pem"));

    if let Some(h) = read_toml_value(&local_path, "vps", "host") {
        host = h;
    }

    Ok(VpsTomlConfig {
        host,
        user,
        remote_dir,
        key_path,
    })
}

pub(crate) fn ssh_home() -> PathBuf {
    #[cfg(windows)]
    {
        std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join(".ssh"))
            .unwrap_or_else(|_| PathBuf::from(".ssh"))
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME")
            .map(|p| PathBuf::from(p).join(".ssh"))
            .unwrap_or_else(|_| PathBuf::from(".ssh"))
    }
}

pub(crate) fn run_ssh(key: &Path, target: &str, remote_cmd: &str) -> Result<String, String> {
    if !key.exists() {
        return Err(format!("SSH key not found: {}", key.display()));
    }
    let key_arg = key.to_string_lossy().replace('\\', "/");
    let out = Command::new("ssh")
        .args([
            "-i",
            &key_arg,
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=15",
            "-o",
            "StrictHostKeyChecking=accept-new",
            target,
            remote_cmd,
        ])
        .output()
        .map_err(|e| format!("ssh failed to run: {e}"))?;

    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    if out.status.success() {
        Ok(stdout)
    } else {
        let msg = if stderr.is_empty() { stdout } else { stderr };
        Err(if msg.is_empty() {
            "ssh authentication failed".into()
        } else {
            msg
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VpsConnectionStatus {
    pub ssh_ok: bool,
    pub host: String,
    pub user: String,
    pub key_path: String,
    pub key_exists: bool,
    pub remote_dir: String,
    pub hostname: Option<String>,
    pub port_3000: Option<bool>,
    pub systemd: Option<String>,
    pub aruaru_health: Option<serde_json::Value>,
    pub public_url: Option<String>,
    pub error: Option<String>,
    pub checked_at: String,
}

pub(crate) fn probe_vps_connection(root: &Path) -> VpsConnectionStatus {
    let checked_at = chrono::Utc::now().to_rfc3339();
    let cfg = match load_vps_config(root) {
        Ok(c) => c,
        Err(e) => {
            return VpsConnectionStatus {
                ssh_ok: false,
                host: String::new(),
                user: String::new(),
                key_path: String::new(),
                key_exists: false,
                remote_dir: String::new(),
                hostname: None,
                port_3000: None,
                systemd: None,
                aruaru_health: None,
                public_url: None,
                error: Some(e),
                checked_at,
            };
        }
    };
    probe_vps_direct(
        &cfg.host,
        &cfg.user,
        &cfg.key_path,
        &cfg.remote_dir,
        checked_at,
    )
}

pub(crate) fn probe_vps_direct(
    host: &str,
    user: &str,
    key_path: &Path,
    remote_dir: &str,
    checked_at: String,
) -> VpsConnectionStatus {
    let key_exists = key_path.exists();
    let target = format!("{user}@{host}");
    let mut status = VpsConnectionStatus {
        ssh_ok: false,
        host: host.to_string(),
        user: user.to_string(),
        key_path: key_path.to_string_lossy().to_string(),
        key_exists,
        remote_dir: remote_dir.to_string(),
        hostname: None,
        port_3000: None,
        systemd: None,
        aruaru_health: None,
        public_url: None,
        error: None,
        checked_at,
    };

    if !key_exists {
        status.error = Some(format!(
            "SSH key not found: {} — place ConoHa .pem as conoha-vps.pem",
            status.key_path
        ));
        return status;
    }

    match run_ssh(key_path, &target, "echo ok") {
        Ok(out) if out == "ok" => status.ssh_ok = true,
        Ok(_) => {
            status.error = Some("unexpected ssh response".into());
            return status;
        }
        Err(e) => {
            status.error = Some(e);
            return status;
        }
    }

    status.hostname = run_ssh(
        key_path,
        &target,
        "hostname -s 2>/dev/null || hostname",
    )
    .ok();

    status.port_3000 = run_ssh(
        key_path,
        &target,
        "ss -tlnp 2>/dev/null | grep -q ':3000' && echo true || echo false",
    )
    .ok()
    .map(|s| s == "true");

    status.systemd = run_ssh(
        key_path,
        &target,
        "systemctl is-active aruaru 2>/dev/null || echo unknown",
    )
    .ok();

    if let Ok(raw) = run_ssh(
        key_path,
        &target,
        "curl -sf -m 3 http://127.0.0.1:3000/api/health || true",
    ) {
        if raw.starts_with('{') {
            status.aruaru_health = serde_json::from_str(&raw).ok();
        }
    }

    status.public_url = run_ssh(
        key_path,
        &target,
        "curl -s --max-time 3 ifconfig.me 2>/dev/null || echo",
    )
    .ok()
    .filter(|ip| !ip.is_empty() && ip != "echo")
    .map(|ip| format!("http://{ip}:3000"));

    status
}

#[tauri::command]
fn pick_pem_file() -> Option<String> {
    rfd::FileDialog::new()
        .set_title("秘密鍵 (.pem) を選択")
        .add_filter("PEM", &["pem"])
        .add_filter("All", &["*"])
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
fn list_ssh_keys() -> Vec<vps::SshKeyInfo> {
    vps::list_ssh_keys()
}

#[tauri::command]
fn generate_ssh_key(name: String) -> Result<vps::SshKeyInfo, String> {
    vps::generate_ssh_key(name)
}

#[tauri::command]
fn test_vps_wizard(input: vps::VpsWizardInput) -> vps::WizardTestResult {
    vps::test_wizard_connection(&input)
}

#[tauri::command]
fn save_vps_wizard(input: vps::VpsWizardInput, root_dir: String) -> Result<(), String> {
    vps::save_wizard_config(&input, &root_dir)
}

#[tauri::command]
fn fetch_server_dashboard(root_dir: Option<String>) -> vps::ServerDashboard {
    vps::fetch_server_dashboard(root_dir)
}

#[tauri::command]
fn check_vps_connection(root_dir: Option<String>) -> VpsConnectionStatus {
    let root = root_dir
        .map(PathBuf::from)
        .unwrap_or_else(desktop_config_root);
    probe_vps_connection(&root)
}

async fn api_post(
    server_url: &str,
    path: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let url = format!("{}{}", server_url.trim_end_matches('/'), path);
    let client = reqwest::Client::new();
    let res = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;
    if res.get("ok").and_then(|v| v.as_bool()) == Some(false) {
        return Err(
            res.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("api error")
                .to_string(),
        );
    }
    Ok(res)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopFeatureItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub platform: String,
    pub runnable: bool,
    pub source: String,
}

#[tauri::command]
fn list_desktop_features() -> Vec<DesktopFeatureItem> {
    vec![
        DesktopFeatureItem {
            id: "drive.bind".into(),
            name: "ドライブ独立（footprint 作成）".into(),
            description: "DATA / アプリを別ドライブに固定し footprint を書き込む".into(),
            category: "drive".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "recovery.scan".into(),
            name: "footprint スキャン".into(),
            description: "全ドライブから .aruaru-footprint.json を検索".into(),
            category: "recovery".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "backup.run".into(),
            name: "DATA バックアップ".into(),
            description: "params: backup_dir（省略時 data/recovery/backups）".into(),
            category: "backup".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "sync.run".into(),
            name: "DATA 同期 / 引越し".into(),
            description: "params: target_dir（NAS 等）".into(),
            category: "backup".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "vps.check".into(),
            name: "VPS 接続状態確認".into(),
            description: "ConoHa VPS への SSH 鍵接続と aruaru 稼働状態を確認".into(),
            category: "vps".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "vps.upload".into(),
            name: "VPS アップロード + セットアップ".into(),
            description: "upload-vps.ps1 を実行（nginx / certbot 含む）".into(),
            category: "vps".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "vps.upload_only".into(),
            name: "VPS アップロードのみ".into(),
            description: "ファイル転送のみ（セットアップはスキップ）".into(),
            category: "vps".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "plugin.install_recovery".into(),
            name: "OS-Recovery インストール".into(),
            description: "aruaru-os-recovery をサーバにインストール".into(),
            category: "plugin".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "plugin.uninstall_recovery".into(),
            name: "OS-Recovery アンインストール".into(),
            description: "params: purge=true でデータも削除".into(),
            category: "plugin".into(),
            platform: "desktop".into(),
            runnable: true,
            source: "local".into(),
        },
        DesktopFeatureItem {
            id: "server.ping".into(),
            name: "サーバ接続確認".into(),
            description: "設定中の aruaru サーバ /api/health を ping".into(),
            category: "server".into(),
            platform: "both".into(),
            runnable: true,
            source: "local".into(),
        },
    ]
}

#[tauri::command]
async fn fetch_server_features(server_url: String) -> Result<Vec<DesktopFeatureItem>, String> {
    let url = format!("{}/api/features", server_url.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;
    if res.get("ok").and_then(|v| v.as_bool()) != Some(true) {
        return Err(
            res.get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("features fetch failed")
                .to_string(),
        );
    }
    let mut out = Vec::new();
    if let Some(arr) = res.get("features").and_then(|v| v.as_array()) {
        for f in arr {
            out.push(DesktopFeatureItem {
                id: f.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                name: f.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                description: f
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                category: f
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("server")
                    .to_string(),
                platform: f
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .unwrap_or("vps")
                    .to_string(),
                runnable: f.get("runnable").and_then(|v| v.as_bool()).unwrap_or(true),
                source: "server".into(),
            });
        }
    }
    Ok(out)
}

#[tauri::command]
async fn run_desktop_feature(
    id: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let cfg = get_desktop_config()?;
    match id.as_str() {
        "drive.bind" => {
            let res = bind_drive(cfg.data_dir.clone(), cfg.root_dir.clone(), cfg.server_url.clone())
                .await?;
            Ok(res)
        }
        "recovery.scan" => {
            let items = scan_footprints()?;
            Ok(serde_json::json!({ "footprints": items }))
        }
        "backup.run" => {
            let backup_dir = params
                .get("backup_dir")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| format!("{}\\recovery\\backups", cfg.data_dir));
            let dest = run_backup(cfg.data_dir, backup_dir)?;
            Ok(serde_json::json!({ "message": dest }))
        }
        "sync.run" => {
            let target = params
                .get("target_dir")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "params.target_dir required".to_string())?;
            let dest = run_sync(cfg.data_dir, target.to_string())?;
            Ok(serde_json::json!({ "message": dest }))
        }
        "vps.upload" => {
            let out = run_upload_vps(cfg.root_dir, false)?;
            Ok(serde_json::json!({ "message": out }))
        }
        "vps.check" => {
            let st = check_vps_connection(Some(cfg.root_dir));
            Ok(serde_json::to_value(st).map_err(|e| e.to_string())?)
        }
        "vps.upload_only" => {
            let out = run_upload_vps(cfg.root_dir, true)?;
            Ok(serde_json::json!({ "message": out }))
        }
        "plugin.install_recovery" => {
            let res = install_plugin(cfg.server_url, "aruaru-os-recovery".into()).await?;
            Ok(res)
        }
        "plugin.uninstall_recovery" => {
            let purge = params.get("purge").and_then(|v| v.as_bool()).unwrap_or(false);
            let res = uninstall_plugin(cfg.server_url, "aruaru-os-recovery".into(), purge).await?;
            Ok(res)
        }
        "server.ping" => {
            let ok = ping_aruaru(format!("{}/api/health", cfg.server_url.trim_end_matches('/')))
                .await?;
            Ok(serde_json::json!({ "ok": ok, "url": cfg.server_url }))
        }
        _ => Err(format!("unknown desktop feature: {id}")),
    }
}

#[tauri::command]
async fn run_server_feature(
    server_url: String,
    id: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    api_post(
        &server_url,
        "/api/features",
        serde_json::json!({ "id": id, "params": params }),
    )
    .await
}

fn run_powershell(script: &Path, args: &[&str]) -> Result<String, String> {
    #[cfg(windows)]
    {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script.to_string_lossy(),
        ]);
        cmd.args(args);
        let out = cmd.output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("script failed: {err}"));
        }
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    }
    #[cfg(not(windows))]
    {
        let _ = (script, args);
        Err("PowerShell scripts are Windows-only".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_desktop_config,
            save_desktop_config,
            list_drives,
            pick_folder,
            ping_aruaru,
            install_plugin,
            uninstall_plugin,
            bind_drive,
            restore_access,
            scan_footprints,
            run_backup,
            run_sync,
            run_upload_vps,
            check_vps_connection,
            fetch_server_dashboard,
            list_ssh_keys,
            generate_ssh_key,
            test_vps_wizard,
            save_vps_wizard,
            pick_pem_file,
            list_desktop_features,
            fetch_server_features,
            run_desktop_feature,
            run_server_feature,
        ])
        .run(tauri::generate_context!())
        .expect("error while running aruaru desktop");
}
