#!/usr/bin/env bash
# install-deps.sh — open-web-server の依存関係を主要ディストリビューションへ導入する。
#
# 対応: Ubuntu/Debian系(apt) と RHEL/AlmaLinux/Rocky Linux系(dnf) の両方。
# /etc/os-release から自動判別するため、手動でパッケージマネージャを覚える必要はない。
#
# 導入するもの:
#   - rclone (aruaru-brain.json のクラウド同期用)
#   - firewalld/ufw のポート開放 (8080=open-web-server, 50051=open-runo gRPC, 4433/udp=QUIC)
#   - SELinux (RHEL系のみ) 実行コンテキストの調整

set -euo pipefail

if [[ ! -f /etc/os-release ]]; then
  echo "エラー: /etc/os-release が見つかりません。対応外のOSの可能性があります。"
  exit 1
fi
# shellcheck disable=SC1091
source /etc/os-release
FAMILY="${ID_LIKE:-$ID}"

echo "検出したディストリビューション: $PRETTY_NAME (ID=$ID, ID_LIKE=${ID_LIKE:-なし})"

install_debian_family() {
  echo "== Ubuntu/Debian系として導入します =="
  sudo apt-get update
  sudo apt-get install -y rclone ufw
  sudo ufw allow 8080/tcp comment 'open-web-server'
  sudo ufw allow 50051/tcp comment 'open-runo gRPC'
  sudo ufw allow 4433/udp comment 'open-runo QUIC'
}

install_rhel_family() {
  echo "== RHEL/AlmaLinux/Rocky Linux系として導入します =="
  # rclone は EPEL 経由、もしくは公式インストーラを利用 (dnf標準リポジトリに無いバージョンがあるため)
  if ! command -v rclone >/dev/null 2>&1; then
    if sudo dnf install -y epel-release && sudo dnf install -y rclone; then
      echo "rclone を EPEL 経由で導入しました。"
    else
      echo "EPEL経由で見つからないため公式インストーラを使用します。"
      curl -fsSL https://rclone.org/install.sh | sudo bash
    fi
  fi

  sudo dnf install -y firewalld
  sudo systemctl enable --now firewalld
  sudo firewall-cmd --permanent --add-port=8080/tcp   # open-web-server
  sudo firewall-cmd --permanent --add-port=50051/tcp  # open-runo gRPC
  sudo firewall-cmd --permanent --add-port=4433/udp   # open-runo QUIC
  sudo firewall-cmd --reload

  # SELinux: バイナリを /usr/local/bin に置く場合の一般的な実行コンテキスト付与
  if command -v getenforce >/dev/null 2>&1 && [[ "$(getenforce)" != "Disabled" ]]; then
    echo "SELinux有効を検出。bin_t コンテキストを付与します。"
    sudo semanage fcontext -a -t bin_t "/usr/local/bin/open-web-server" 2>/dev/null || \
      echo "policycoreutils-python-utils が無い場合は 'sudo dnf install -y policycoreutils-python-utils' を実行してください。"
    sudo restorecon -v /usr/local/bin/open-web-server 2>/dev/null || true
  fi
}

case "$FAMILY" in
  *debian*|*ubuntu*) install_debian_family ;;
  *rhel*|*fedora*)   install_rhel_family ;;
  *)
    case "$ID" in
      almalinux|rocky|rhel|centos) install_rhel_family ;;
      ubuntu|debian|linuxmint) install_debian_family ;;
      *)
        echo "未対応の ID='$ID' / ID_LIKE='${ID_LIKE:-}' です。"
        echo "RHEL系相当と判断できる場合は install_rhel_family を、Debian系相当なら install_debian_family を手動で呼んでください。"
        exit 1
        ;;
    esac
    ;;
esac

echo "完了: 依存関係の導入とポート開放が終わりました。"
