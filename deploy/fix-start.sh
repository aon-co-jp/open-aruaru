#!/bin/bash
# aruaru — 起動修復（PostgreSQL 失敗時は sqlite にフォールバックして起動）
set -euo pipefail

ARUARU_DIR="${ARUARU_DIR:-$HOME/aruaru}"
cd "$ARUARU_DIR"

BIN="$ARUARU_DIR/bin/aruaru"
DATA="$ARUARU_DIR/data"
CFG="$DATA/config.toml"

mkdir -p "$DATA"

if [ ! -x "$BIN" ]; then
  echo "ERROR: $BIN がありません。先に deploy/vps-full-setup.sh を実行してください"
  exit 1
fi

# PostgreSQL が無い/死んでいる場合は sqlite に切替
if ! command -v pg_isready &>/dev/null || ! pg_isready -h 127.0.0.1 -p 5432 -q 2>/dev/null; then
  echo "PostgreSQL 未起動 → sqlite モードで起動します"
  if [ -f "$CFG" ]; then
    sed -i 's|^primary = .*|primary = "sqlite"|' "$CFG"
  else
    cp config/default.toml "$CFG"
    sed -i 's|^primary = .*|primary = "sqlite"|' "$CFG"
  fi
fi

export ARUARU_ROOT="$ARUARU_DIR"
export ARUARU_DATA_DIR="$DATA"

# 手動テスト起動
echo "テスト: aruaru serve（5秒）"
timeout 5 env ARUARU_ROOT="$ARUARU_ROOT" ARUARU_DATA_DIR="$ARUARU_DATA_DIR" "$BIN" serve &
sleep 4
curl -s http://127.0.0.1:3000/api/health && echo "" || echo "まだ起動していません — ログを確認"

# systemd 再設定
if [ "$(id -u)" -eq 0 ] || sudo -n true 2>/dev/null; then
  PG_URL="${ARUARU_DATABASE_URL:-}"
  sudo tee /etc/systemd/system/aruaru.service >/dev/null <<UNIT
[Unit]
Description=aruaru Ver 0.1
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
WorkingDirectory=$ARUARU_DIR
Environment=ARUARU_ROOT=$ARUARU_DIR
Environment=ARUARU_DATA_DIR=$DATA
Environment=ARUARU_DATABASE_URL=${PG_URL}
ExecStart=$BIN serve
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
UNIT
  sudo systemctl daemon-reload
  sudo systemctl enable aruaru
  sudo systemctl restart aruaru
  sleep 2
  systemctl is-active aruaru
  curl -s http://127.0.0.1:3000/api/health && echo ""
fi

# ファイアウォール + nginx
chmod +x "$ARUARU_DIR/deploy/firewall-aruaru.sh" "$ARUARU_DIR/deploy/nginx-aruaru.sh" 2>/dev/null || true
[ -x "$ARUARU_DIR/deploy/firewall-aruaru.sh" ] && "$ARUARU_DIR/deploy/firewall-aruaru.sh" || true
[ -x "$ARUARU_DIR/deploy/nginx-aruaru.sh" ] && "$ARUARU_DIR/deploy/nginx-aruaru.sh" "$ARUARU_DIR" || true

IP=$(curl -s --max-time 5 ifconfig.me 2>/dev/null || hostname -I | awk '{print $1}')
echo ""
echo "完了 — ブラウザ:"
echo "  http://${IP}/#/register  (推奨: port 80)"
echo "  http://${IP}:3000/#/register"
echo "ログ: journalctl -u aruaru -f"
