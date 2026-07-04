//! 起動時にファイアウォールのポートを自動開放
//! Ubuntu/Debian/CentOS/Rocky/Alma/Fedora/Amazon/Alpine/Arch 対応

use tracing::{info, warn};
use crate::distro::{FirewallType, SysInfo};

/// 指定ポートを TCP で開放する（失敗しても起動は継続）
pub async fn open_port(port: u16, sys: &SysInfo) {
    let port_str = port.to_string();
    let ok = match sys.firewall {
        FirewallType::Firewalld => open_firewalld(&port_str).await,
        FirewallType::Ufw       => open_ufw(&port_str).await,
        FirewallType::Nftables  => open_nftables(&port_str).await,
        FirewallType::Iptables  => open_iptables(&port_str).await,
        FirewallType::None      => {
            info!("ℹ️  ファイアウォールなし — ポート {} はそのまま開いています", port);
            return;
        }
    };

    if ok {
        info!("🔓 {:?}: ポート {}/tcp を開放しました", sys.firewall, port);
    } else {
        // フォールバック: iptables で試みる
        warn!("⚠️  {:?} での開放に失敗 → iptables でリトライ", sys.firewall);
        if open_iptables(&port_str).await {
            info!("🔓 iptables (fallback): ポート {}/tcp を開放しました", port);
        } else {
            warn!("❌ ポート {} の自動開放に失敗しました（手動で開放してください）", port);
        }
    }
}

// --- firewall-cmd (CentOS/RHEL/Fedora/Rocky/AlmaLinux/Amazon Linux) ---
async fn open_firewalld(port: &str) -> bool {
    let add = tokio::process::Command::new("firewall-cmd")
        .args(["--permanent", "--add-port", &format!("{}/tcp", port)])
        .output().await;
    if add.map(|o| o.status.success()).unwrap_or(false) {
        let _ = tokio::process::Command::new("firewall-cmd")
            .arg("--reload").output().await;
        return true;
    }
    false
}

// --- ufw (Ubuntu/Debian) ---
async fn open_ufw(port: &str) -> bool {
    tokio::process::Command::new("ufw")
        .args(["allow", &format!("{}/tcp", port)])
        .output().await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// --- nftables (Arch/Alpine/modern Debian) ---
async fn open_nftables(port: &str) -> bool {
    // nftables でルールを追加（テーブルが存在しない場合は作成）
    let rule = format!(
        "add rule inet filter input tcp dport {} accept",
        port
    );
    let ok = tokio::process::Command::new("nft")
        .args(["-f", "-"])
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(rule.as_bytes());
            }
            child.wait().map(|s| s.success()).unwrap_or(false)
        })
        .unwrap_or(false);

    if ok { return true; }

    // フォールバック: nft コマンドで直接追加
    tokio::process::Command::new("nft")
        .args(["add", "rule", "inet", "filter", "input",
               "tcp", "dport", port, "accept"])
        .output().await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// --- iptables (汎用フォールバック) ---
async fn open_iptables(port: &str) -> bool {
    // 既にルールがあれば追加しない
    let exists = tokio::process::Command::new("iptables")
        .args(["-C", "INPUT", "-p", "tcp", "--dport", port, "-j", "ACCEPT"])
        .output().await
        .map(|o| o.status.success())
        .unwrap_or(false);
    if exists { return true; }

    tokio::process::Command::new("iptables")
        .args(["-I", "INPUT", "-p", "tcp", "--dport", port, "-j", "ACCEPT"])
        .output().await
        .map(|o| o.status.success())
        .unwrap_or(false)
}
