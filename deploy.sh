#!/bin/bash
# =============================================================
# deploy.sh — aruaru を VPS へアップロードして cargo build
# 使い方: bash deploy.sh <VPS_IP> [SSHユーザー] [SSHポート]
# 例:     bash deploy.sh 123.456.789.0
#         bash deploy.sh 123.456.789.0 root 22
# =============================================================
set -e

VPS_IP="${1:?使い方: bash deploy.sh <VPS_IP> [user] [port]}"
VPS_USER="${2:-root}"
VPS_PORT="${3:-22}"
SSH="ssh -p $VPS_PORT $VPS_USER@$VPS_IP"
SCP="rsync -avz --progress -e \"ssh -p $VPS_PORT\""

echo "======================================"
echo " 🚀 aruaru デプロイ開始"
echo "    VPS: $VPS_USER@$VPS_IP:$VPS_PORT"
echo "======================================"

# -------------------------------------------------------
# 1. ソースコードを VPS へ転送（target/ は除外）
# -------------------------------------------------------
echo ""
echo "📦 [1/4] ソースを転送中…"
rsync -avz --progress \
  -e "ssh -p $VPS_PORT" \
  --exclude='target/' \
  --exclude='.git/' \
  --exclude='*.db' \
  "$(dirname "$0")/" \
  "$VPS_USER@$VPS_IP:/open-aruaru/"

# -------------------------------------------------------
# 2. VPS 上でビルド
# -------------------------------------------------------
echo ""
echo "🔨 [2/4] VPS 上で cargo build --release…"
$SSH bash << 'REMOTE'
set -e
cd /open-aruaru

# Rust が未インストールなら setup-vps.sh を先に実行してください
export PATH="$HOME/.cargo/bin:$PATH"

echo "→ aruaru-easyweb をビルド中…"
cargo build --release -p aruaru-easyweb

echo "→ aruaru-core をビルド中…"
cargo build --release -p aruaru-core

echo "✅ ビルド完了"
ls -lh target/release/aruaru-easyweb target/release/aruaru-core
REMOTE

# -------------------------------------------------------
# 3. ディレクトリ構成を整備
# -------------------------------------------------------
echo ""
echo "📁 [3/4] ディレクトリ整備中…"
$SSH bash << 'REMOTE'
set -e

# aruaru-easyweb
mkdir -p /open-aruaru-easyweb/{data,logs,static}
cp /open-aruaru/target/release/aruaru-easyweb /open-aruaru-easyweb/
cp -r /open-aruaru/aruaru-easyweb/static/     /open-aruaru-easyweb/
cp -r /open-aruaru/aruaru-easyweb/config/     /open-aruaru-easyweb/
echo "✅ aruaru-easyweb セット完了"

# aruaru-core
mkdir -p /open-aruaru-core/{data,logs,static}
cp /open-aruaru/target/release/aruaru-core /open-aruaru-core/
cp -r /open-aruaru/aruaru-core/static/     /open-aruaru-core/
cp -r /open-aruaru/aruaru-core/config/     /open-aruaru-core/
echo "✅ aruaru-core セット完了"
REMOTE

# -------------------------------------------------------
# 4. systemd サービス登録 & 起動
# -------------------------------------------------------
echo ""
echo "⚙️  [4/4] systemd サービスを登録・起動中…"
$SSH bash << 'REMOTE'
set -e

# --- aruaru-easyweb.service ---
cat > /etc/systemd/system/aruaru-easyweb.service << 'SVC'
[Unit]
Description=aruaru-easyweb (第二のKUSANAGI / Web Server Manager)
After=network.target

[Service]
User=root
WorkingDirectory=/open-aruaru-easyweb
ExecStart=/open-aruaru-easyweb/aruaru-easyweb
Restart=always
RestartSec=3
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
SVC

# --- aruaru-core.service ---
cat > /etc/systemd/system/aruaru-core.service << 'SVC'
[Unit]
Description=aruaru-core (SSH / SFTP / VPS 管理)
After=network.target

[Service]
User=root
WorkingDirectory=/open-aruaru-core
ExecStart=/open-aruaru-core/aruaru-core
Restart=always
RestartSec=3
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
SVC

systemctl daemon-reload
systemctl enable  aruaru-easyweb aruaru-core
systemctl restart aruaru-easyweb aruaru-core

sleep 2
systemctl is-active aruaru-easyweb && echo "✅ aruaru-easyweb: 起動中" || echo "❌ aruaru-easyweb: 起動失敗"
systemctl is-active aruaru-core    && echo "✅ aruaru-core:    起動中" || echo "❌ aruaru-core:    起動失敗"
REMOTE

echo ""
echo "======================================"
echo " 🎉 デプロイ完了！"
echo ""
echo " aruaru-easyweb → http://$VPS_IP:8080"
echo "   初回: http://$VPS_IP:8080/setup"
echo "   AI連携: http://$VPS_IP:8080/anthropic"
echo ""
echo " aruaru-core → http://$VPS_IP:8081"
echo "======================================"
