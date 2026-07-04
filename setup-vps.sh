#!/bin/bash
# =============================================================
# setup-vps.sh — VPS 初期環境セットアップ（Rust + 依存パッケージ）
# VPS 上で直接実行: bash setup-vps.sh
# または: ssh root@<VPS_IP> 'bash -s' < setup-vps.sh
# =============================================================
set -e

echo "======================================"
echo " 🔧 aruaru VPS 初期セットアップ"
echo "======================================"

# -------------------------------------------------------
# 1. システムパッケージ
# -------------------------------------------------------
echo ""
echo "📦 [1/4] システムパッケージをインストール中…"
apt-get update -qq
apt-get install -y -qq \
  build-essential pkg-config curl git \
  libssl-dev libsqlite3-dev \
  certbot \
  nginx \
  apache2 \
  openssl \
  rsync

echo "✅ パッケージインストール完了"

# -------------------------------------------------------
# 2. Rust インストール（未インストールの場合のみ）
# -------------------------------------------------------
echo ""
echo "🦀 [2/4] Rust をインストール中…"
if command -v cargo &> /dev/null; then
  echo "✅ Rust はすでにインストール済み: $(rustc --version)"
else
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
  echo "✅ Rust インストール完了: $(rustc --version)"
fi

export PATH="$HOME/.cargo/bin:$PATH"

# -------------------------------------------------------
# 3. ディレクトリ準備
# -------------------------------------------------------
echo ""
echo "📁 [3/4] ディレクトリを準備中…"
mkdir -p /open-aruaru
mkdir -p /open-aruaru-easyweb/{data,logs,static,config}
mkdir -p /open-aruaru-core/{data,logs,static,config}

# nginx の設定ディレクトリ
mkdir -p /etc/nginx/sites-available
mkdir -p /etc/nginx/sites-enabled

echo "✅ ディレクトリ準備完了"

# -------------------------------------------------------
# 4. firewall（ufw が入っていれば）
# -------------------------------------------------------
echo ""
echo "🔒 [4/4] ファイアウォール設定…"
if command -v ufw &> /dev/null; then
  ufw allow 22   comment 'SSH'    2>/dev/null || true
  ufw allow 8080 comment 'aruaru-easyweb' 2>/dev/null || true
  ufw allow 8081 comment 'aruaru-core'    2>/dev/null || true
  ufw allow 80   comment 'HTTP'   2>/dev/null || true
  ufw allow 443  comment 'HTTPS'  2>/dev/null || true
  echo "✅ ufw ポート開放完了"
else
  echo "⚠️  ufw が見つかりません。手動でポート 8080, 8081, 80, 443 を開放してください。"
fi

echo ""
echo "======================================"
echo " ✅ VPS セットアップ完了！"
echo ""
echo " 次のステップ:"
echo "   ローカルから: bash deploy.sh <この VPS の IP>"
echo "======================================"
