#!/bin/bash
# aruaru Ver 0.1c — ConoHa VPS フルセットアップ（PostgreSQL + ビルド + systemd）
# 使い方: aruaru を ~/aruaru にアップロード後、VPS コンソールまたは SSH で:
#   cd ~/aruaru && chmod +x deploy/vps-full-setup.sh && ./deploy/vps-full-setup.sh
set -euo pipefail

ARUARU_DIR="${ARUARU_DIR:-$(cd "$(dirname "$0")/.." && pwd)}"
DB_USER="${ARUARU_DB_USER:-aruaru}"
DB_PASS="${ARUARU_DB_PASS:-aruaru}"
DB_NAME="${ARUARU_DB_NAME:-aruaru}"
export ARUARU_DB_USER ARUARU_DB_PASS ARUARU_DB_NAME

cd "$ARUARU_DIR"

echo "========================================"
echo " aruaru Ver 0.1c — PostgreSQL セットアップ"
echo " dir: $ARUARU_DIR"
echo "========================================"

echo "[1/7] パッケージ（PostgreSQL / ビルドツール / nginx / certbot）"
if command -v apt-get &>/dev/null; then
  sudo apt-get update -qq
  sudo DEBIAN_FRONTEND=noninteractive apt-get install -y \
    postgresql postgresql-contrib \
    build-essential pkg-config curl libssl-dev \
    nginx certbot python3-certbot-nginx
elif command -v dnf &>/dev/null; then
  sudo dnf install -y postgresql-server postgresql-devel gcc openssl-devel \
    nginx certbot python3-certbot-nginx
  sudo postgresql-setup --initdb 2>/dev/null || true
fi

echo "[2/7] PostgreSQL 起動"
for svc in postgresql postgresql-16 postgresql-15 postgresql-14; do
  if systemctl list-unit-files "$svc.service" &>/dev/null; then
    sudo systemctl enable --now "$svc" 2>/dev/null && break
  fi
done

chmod +x deploy/postgres-init.sh
./deploy/postgres-init.sh

PG_URL="postgres://${DB_USER}:${DB_PASS}@127.0.0.1:5432/${DB_NAME}"
export ARUARU_DATABASE_URL="$PG_URL"

echo "[3/7] Rust"
if ! command -v cargo &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi
# shellcheck disable=SC1091
source "${HOME}/.cargo/env"

echo "[4/7] aruaru ビルド"
cargo build --release
mkdir -p "$ARUARU_DIR"/{bin,data}
cp target/release/aruaru "$ARUARU_DIR/bin/"
cp -r plugins static migrations "$ARUARU_DIR/" 2>/dev/null || true

echo "[5/7] 設定 + プラグイン + nginx / certbot"
mkdir -p "$ARUARU_DIR/data"
if [ ! -f "$ARUARU_DIR/data/config.toml" ]; then
  cp config/default.toml "$ARUARU_DIR/data/config.toml"
fi

# database セクションを PostgreSQL に合わせる
if grep -q '^\[database\]' "$ARUARU_DIR/data/config.toml"; then
  sed -i "s|^primary = .*|primary = \"postgres\"|" "$ARUARU_DIR/data/config.toml"
  sed -i "s|^postgres_url = .*|postgres_url = \"$PG_URL\"|" "$ARUARU_DIR/data/config.toml"
else
  cat >> "$ARUARU_DIR/data/config.toml" <<EOF

[database]
primary = "postgres"
postgres_url = "$PG_URL"
cockroach_url = ""
cockroach_sync_enabled = false
cockroach_sync_interval_secs = 300
EOF
fi

IP=$(curl -s --max-time 8 ifconfig.me 2>/dev/null || hostname -I 2>/dev/null | awk '{print $1}')
if [ -n "$IP" ]; then
  sed -i "s|^public_url = .*|public_url = \"http://${IP}:3000\"|" "$ARUARU_DIR/data/config.toml" || true
fi

export ARUARU_ROOT="$ARUARU_DIR"
export ARUARU_DATA_DIR="$ARUARU_DIR/data"
"$ARUARU_DIR/bin/aruaru" install aruaru-easyweb || true

# ACME webroot + nginx リバースプロキシ（ポート 80 → 3000）
chmod +x deploy/nginx-aruaru.sh deploy/firewall-aruaru.sh
./deploy/nginx-aruaru.sh "$ARUARU_DIR"
./deploy/firewall-aruaru.sh

# certbot メール（環境変数 CERTBOT_EMAIL で上書き可）
CERTBOT_EMAIL="${CERTBOT_EMAIL:-admin@example.com}"
if grep -q 'certbot_email' "$ARUARU_DIR/data/config.toml"; then
  sed -i "s|^certbot_email = .*|certbot_email = \"$CERTBOT_EMAIL\"|" "$ARUARU_DIR/data/config.toml"
fi
if grep -q 'web_server' "$ARUARU_DIR/data/config.toml"; then
  sed -i 's|^web_server = .*|web_server = "nginx"|' "$ARUARU_DIR/data/config.toml"
fi

echo "[6/7] systemd"
SVC="/etc/systemd/system/aruaru.service"
if [ "$(id -u)" -eq 0 ] || sudo -n true 2>/dev/null; then
  sudo tee "$SVC" >/dev/null <<UNIT
[Unit]
Description=aruaru Ver 0.1 (PostgreSQL)
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
WorkingDirectory=$ARUARU_DIR
Environment=ARUARU_ROOT=$ARUARU_DIR
Environment=ARUARU_DATA_DIR=$ARUARU_DIR/data
Environment=ARUARU_DATABASE_URL=$PG_URL
ExecStart=$ARUARU_DIR/bin/aruaru serve
Restart=on-failure
RestartSec=3

[Install]
WantedBy=multi-user.target
UNIT
  sudo systemctl daemon-reload
  sudo systemctl enable aruaru
  sudo systemctl restart aruaru
else
  echo "root 権限なし — 手動起動:"
  echo "  ARUARU_ROOT=$ARUARU_DIR ARUARU_DATA_DIR=$ARUARU_DIR/data ARUARU_DATABASE_URL=$PG_URL $ARUARU_DIR/bin/aruaru serve"
fi

sleep 3
echo ""
echo "[7/7] health check"
curl -s "http://127.0.0.1:3000/api/health" || echo "(起動待ち — journalctl -u aruaru -f)"
echo ""
echo "========================================"
echo " 完了"
echo " URL: http://${IP:-YOUR_IP}/#/register  (port 80, nginx → :3000)"
echo " 直接: http://${IP:-YOUR_IP}:3000/#/register (要ファイアウォール開放)"
echo " HTTPS: 管理画面 → 機能一覧 → https.certbot"
echo " DB:  $PG_URL"
echo " ログ: sudo journalctl -u aruaru -f"
echo "========================================"
