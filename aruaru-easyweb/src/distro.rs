//! Linux ディストリビューション自動検出

use std::collections::HashMap;
use tracing::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum Distro {
    Ubuntu,
    Debian,
    CentOS,
    Rocky,
    AlmaLinux,
    Fedora,
    AmazonLinux,
    OpenSUSE,
    Arch,
    Alpine,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageManager {
    Apt,   // Ubuntu, Debian
    Dnf,   // Fedora, CentOS 8+, Rocky, Alma, Amazon Linux 2023
    Yum,   // CentOS 7, Amazon Linux 2
    Zypper,// OpenSUSE
    Pacman,// Arch
    Apk,   // Alpine
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FirewallType {
    Firewalld, // CentOS/RHEL/Fedora/Rocky/Alma
    Ufw,       // Ubuntu/Debian
    Nftables,  // Modern fallback
    Iptables,  // Legacy fallback
    None,
}

#[derive(Debug, Clone)]
pub struct SysInfo {
    pub distro:          Distro,
    pub version:         String,
    pub package_manager: PackageManager,
    pub firewall:        FirewallType,
}

impl SysInfo {
    pub fn distro_name(&self) -> &str {
        match &self.distro {
            Distro::Ubuntu      => "Ubuntu",
            Distro::Debian      => "Debian",
            Distro::CentOS      => "CentOS",
            Distro::Rocky       => "Rocky Linux",
            Distro::AlmaLinux   => "AlmaLinux",
            Distro::Fedora      => "Fedora",
            Distro::AmazonLinux => "Amazon Linux",
            Distro::OpenSUSE    => "OpenSUSE",
            Distro::Arch        => "Arch Linux",
            Distro::Alpine      => "Alpine Linux",
            Distro::Unknown(s)  => s.as_str(),
        }
    }

    pub fn pkg_cmd(&self) -> &str {
        match self.package_manager {
            PackageManager::Apt    => "apt-get",
            PackageManager::Dnf    => "dnf",
            PackageManager::Yum    => "yum",
            PackageManager::Zypper => "zypper",
            PackageManager::Pacman => "pacman",
            PackageManager::Apk    => "apk",
            PackageManager::Unknown => "unknown",
        }
    }
}

/// /etc/os-release を読んでディストリビューション情報を取得
pub async fn detect() -> SysInfo {
    let fields = read_os_release().await;

    let id = fields.get("ID").map(|s| s.to_lowercase()).unwrap_or_default();
    let id_like = fields.get("ID_LIKE").map(|s| s.to_lowercase()).unwrap_or_default();
    let version = fields.get("VERSION_ID").cloned().unwrap_or_default();

    let distro = match id.as_str() {
        "ubuntu"      => Distro::Ubuntu,
        "debian"      => Distro::Debian,
        "centos"      => Distro::CentOS,
        "rocky"       => Distro::Rocky,
        "almalinux"   => Distro::AlmaLinux,
        "fedora"      => Distro::Fedora,
        "amzn"        => Distro::AmazonLinux,
        "opensuse" | "opensuse-leap" | "opensuse-tumbleweed" => Distro::OpenSUSE,
        "arch"        => Distro::Arch,
        "alpine"      => Distro::Alpine,
        _ => {
            // ID_LIKE で推定
            if id_like.contains("debian") || id_like.contains("ubuntu") {
                Distro::Debian
            } else if id_like.contains("rhel") || id_like.contains("centos") || id_like.contains("fedora") {
                Distro::CentOS
            } else {
                Distro::Unknown(id.clone())
            }
        }
    };

    let package_manager = detect_package_manager(&distro, &version).await;
    let firewall = detect_firewall().await;

    let info = SysInfo { distro, version, package_manager, firewall };
    debug!("🐧 検出OS: {} {} / pkg: {} / fw: {:?}",
        info.distro_name(), info.version, info.pkg_cmd(), info.firewall);
    info
}

async fn read_os_release() -> HashMap<String, String> {
    let content = match tokio::fs::read_to_string("/etc/os-release").await {
        Ok(s) => s,
        Err(_) => tokio::fs::read_to_string("/usr/lib/os-release").await.unwrap_or_default(),
    };

    content.lines()
        .filter_map(|line| {
            let (k, v) = line.split_once('=')?;
            Some((k.trim().to_string(), v.trim().trim_matches('"').to_string()))
        })
        .collect()
}

async fn detect_package_manager(distro: &Distro, version: &str) -> PackageManager {
    // コマンドの存在で確認（which の代わりに実際に存在チェック）
    for (cmd, pm) in [
        ("dnf",    PackageManager::Dnf),
        ("apt-get",PackageManager::Apt),
        ("yum",    PackageManager::Yum),
        ("zypper", PackageManager::Zypper),
        ("pacman", PackageManager::Pacman),
        ("apk",    PackageManager::Apk),
    ] {
        if tokio::process::Command::new("which").arg(cmd).output().await
            .map(|o| o.status.success()).unwrap_or(false) {
            return pm;
        }
    }

    // フォールバック: ディストリビューションから推定
    match distro {
        Distro::Ubuntu | Distro::Debian => PackageManager::Apt,
        Distro::Fedora | Distro::Rocky | Distro::AlmaLinux => PackageManager::Dnf,
        Distro::CentOS => {
            // CentOS 7 → yum, 8+ → dnf
            let major: u32 = version.split('.').next()
                .and_then(|v| v.parse().ok()).unwrap_or(8);
            if major >= 8 { PackageManager::Dnf } else { PackageManager::Yum }
        }
        Distro::AmazonLinux => {
            // Amazon Linux 2023 → dnf, Amazon Linux 2 → yum
            let major: u32 = version.parse().unwrap_or(2);
            if major >= 2023 { PackageManager::Dnf } else { PackageManager::Yum }
        }
        Distro::OpenSUSE => PackageManager::Zypper,
        Distro::Arch     => PackageManager::Pacman,
        Distro::Alpine   => PackageManager::Apk,
        _                => PackageManager::Unknown,
    }
}

async fn detect_firewall() -> FirewallType {
    // firewalld が動いているか
    let firewalld_active = tokio::process::Command::new("systemctl")
        .args(["is-active", "firewalld"])
        .output().await
        .map(|o| o.status.success())
        .unwrap_or(false);
    if firewalld_active { return FirewallType::Firewalld; }

    // ufw が有効か
    let ufw_active = tokio::process::Command::new("ufw")
        .arg("status")
        .output().await
        .map(|o| String::from_utf8_lossy(&o.stdout).contains("active"))
        .unwrap_or(false);
    if ufw_active { return FirewallType::Ufw; }

    // nftables
    if tokio::process::Command::new("which").arg("nft").output().await
        .map(|o| o.status.success()).unwrap_or(false) {
        return FirewallType::Nftables;
    }

    // iptables
    if tokio::process::Command::new("which").arg("iptables").output().await
        .map(|o| o.status.success()).unwrap_or(false) {
        return FirewallType::Iptables;
    }

    FirewallType::None
}
