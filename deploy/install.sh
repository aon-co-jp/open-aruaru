#!/bin/bash
# aruaru-easyweb v0.1 — VPS インストール（自動起動）
set -euo pipefail

ARUARU_DIR="${ARUARU_DIR:-$HOME/aruaru}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "========================================"
echo " aruaru-easyweb v0.1"
echo "========================================"

mkdir -p "$ARUARU_DIR"/{bin,data,config,plugins,static}
if [ ! -f "$ARUARU_DIR/data/config.toml" ]; then
  cp "$PROJECT_DIR/config/default.toml" "$ARUARU_DIR/data/config.toml"
fi

if command -v cargo &>/dev/null; then
  echo "[1/4] cargo build --release"
  cd "$PROJECT_DIR"
  cargo build --release
  cp target/release/aruaru "$ARUARU_DIR/bin/"
else
  cp "$PROJECT_DIR/target/release/aruaru" "$ARUARU_DIR/bin/" 2>/dev/null || { echo "cargo またはビルド済み bin が必要です"; exit 1; }
fi

echo "[2/4] ファイル配置"
cp -r "$PROJECT_DIR/plugins" "$ARUARU_DIR/"
cp -r "$PROJECT_DIR/static" "$ARUARU_DIR/"
cp -r "$PROJECT_DIR/migrations" "$ARUARU_DIR/" 2>/dev/null || true
cp "$PROJECT_DIR/config/default.toml" "$ARUARU_DIR/config/" 2>/dev/null || true
cp "$PROJECT_DIR/deploy/install-cmd" "$ARUARU_DIR/bin/install" 2>/dev/null || true
chmod +x "$ARUARU_DIR/bin/install" 2>/dev/null || true
chmod +x "$PROJECT_DIR/deploy/postgres-init.sh" 2>/dev/null || true

# PostgreSQL 主 DB（config の primary=postgres 時）
if grep -q 'primary = "postgres"' "$ARUARU_DIR/data/config.toml" 2>/dev/null \
   || grep -q 'primary = "postgres"' "$PROJECT_DIR/config/default.toml" 2>/dev/null; then
  if command -v psql &>/dev/null || command -v apt-get &>/dev/null; then
    echo "[2b] PostgreSQL セットアップ"
    if ! command -v psql &>/dev/null && command -v apt-get &>/dev/null; then
      sudo apt-get update -qq
      sudo DEBIAN_FRONTEND=noninteractive apt-get install -y postgresql postgresql-contrib
      sudo systemctl enable --now postgresql 2>/dev/null || true
    fi
    if [ -x "$PROJECT_DIR/deploy/postgres-init.sh" ]; then
      "$PROJECT_DIR/deploy/postgres-init.sh"
      PG_URL="${ARUARU_DATABASE_URL:-postgres://aruaru:aruaru@127.0.0.1:5432/aruaru}"
      export ARUARU_DATABASE_URL="$PG_URL"
      mkdir -p "$ARUARU_DIR/data"
      if [ -f "$ARUARU_DIR/data/config.toml" ]; then
        sed -i "s|^postgres_url = .*|postgres_url = \"$PG_URL\"|" "$ARUARU_DIR/data/config.toml" 2>/dev/null || true
        sed -i 's|^primary = .*|primary = "postgres"|' "$ARUARU_DIR/data/config.toml" 2>/dev/null || true
      fi
    fi
  fi
fi

echo "[3/4] aruaru install aruaru-easyweb"
export ARUARU_ROOT="$ARUARU_DIR"
export ARUARU_DATA_DIR="$ARUARU_DIR/data"
export ARUARU_DATABASE_URL="${ARUARU_DATABASE_URL:-}"
"$ARUARU_DIR/bin/aruaru" install aruaru-easyweb

# public_url を VPS IP に更新（任意）
IP=$(curl -s ifconfig.me 2>/dev/null || hostname -I 2>/dev/null | awk '{print $1}')
if [ -n "$IP" ]; then
  sed -i "s|public_url = .*|public_url = \"http://${IP}:3000\"|" "$ARUARU_DIR/data/config.toml" 2>/dev/null || true
fi

if [ "$(id -u)" -eq 0 ] && [ -d /etc/systemd/system ]; then
  echo "[4/4] systemd 自動起動"
  PG_URL="${ARUARU_DATABASE_URL:-postgres://aruaru:aruaru@127.0.0.1:5432/aruaru}"
  sed "s|/opt/aruaru|$ARUARU_DIR|g" "$PROJECT_DIR/deploy/systemd/aruaru.service" \
    | sed "s|postgres://aruaru:aruaru@127.0.0.1:5432/aruaru|$PG_URL|" \
    > /etc/systemd/system/aruaru.service
  systemctl daemon-reload
  systemctl enable aruaru
  systemctl restart aruaru
else
  echo "[4/4] 手動起動:"
  echo "  ARUARU_ROOT=$ARUARU_DIR ARUARU_DATA_DIR=$ARUARU_DIR/data ARUARU_DATABASE_URL=${ARUARU_DATABASE_URL:-} $ARUARU_DIR/bin/aruaru serve"
  nohup env ARUARU_ROOT="$ARUARU_DIR" ARUARU_DATA_DIR="$ARUARU_DIR/data" ARUARU_DATABASE_URL="${ARUARU_DATABASE_URL:-}" "$ARUARU_DIR/bin/aruaru" serve >/tmp/aruaru.log 2>&1 &
fi

echo "========================================"
echo " 完了: http://${IP:-YOUR_IP}:3000/register"
echo " または: install aruaru-easyweb （bin/install）"
echo "========================================"
