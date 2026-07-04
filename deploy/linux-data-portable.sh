#!/bin/bash
# aruaru VPS — DATA ディレクトリを OS から分離（Linux footprint）
# 使い方: sudo ARUARU_DATA=/var/lib/aruaru ./deploy/linux-data-portable.sh
set -euo pipefail

ARUARU_DIR="${ARUARU_DIR:-$(cd "$(dirname "$0")/.." && pwd)}"
DATA_DIR="${ARUARU_DATA:-/var/lib/aruaru}"
MARKER="${DATA_DIR}/.aruaru-footprint.json"

echo "=== aruaru Linux DATA portable ==="
echo "  app:  $ARUARU_DIR"
echo "  data: $DATA_DIR"

sudo mkdir -p "$DATA_DIR"
sudo chown "$(whoami):$(whoami)" "$DATA_DIR" 2>/dev/null || true

cat > "$MARKER" <<EOF
{
  "data_dir": "$DATA_DIR",
  "app_dir": "$ARUARU_DIR",
  "data_drive": "/",
  "updated_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platform": "linux"
}
EOF

echo "OK: $MARKER"
echo "OS 再インストール後: upload-vps.ps1 または vps-full-setup.sh で app を再配置し、DATA_DIR を復元"
