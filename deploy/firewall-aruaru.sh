#!/bin/bash
# ファイアウォール — HTTP/HTTPS/3000 を開放
set -euo pipefail

if command -v firewall-cmd &>/dev/null; then
  sudo firewall-cmd --permanent --add-service=http 2>/dev/null || true
  sudo firewall-cmd --permanent --add-service=https 2>/dev/null || true
  sudo firewall-cmd --permanent --add-port=3000/tcp 2>/dev/null || true
  sudo firewall-cmd --reload 2>/dev/null || true
  echo "OK: firewalld http/https/3000"
fi

if command -v ufw &>/dev/null && ufw status 2>/dev/null | grep -qi active; then
  sudo ufw allow 80/tcp 2>/dev/null || true
  sudo ufw allow 443/tcp 2>/dev/null || true
  sudo ufw allow 3000/tcp 2>/dev/null || true
  echo "OK: ufw 80/443/3000"
fi

echo "NOTE: ConoHa パネルでも TCP 80/443 を許可してください"
