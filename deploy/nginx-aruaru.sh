#!/bin/bash
# nginx リバースプロキシ（:80 → :3000）— Debian / RHEL 両対応
set -euo pipefail

ARUARU_DIR="${1:-${ARUARU_DIR:-$HOME/aruaru}}"
ACME="$ARUARU_DIR/data/acme-webroot"
mkdir -p "$ACME"

if ! command -v nginx &>/dev/null; then
  echo "nginx not installed — skip"
  exit 0
fi

NGINX_BODY="server {
    listen 80 default_server;
    listen [::]:80 default_server;
    server_name _;
    location /.well-known/acme-challenge/ { root $ACME; }
    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}"

if [ -d /etc/nginx/conf.d ]; then
  # AlmaLinux / RHEL / CentOS
  echo "$NGINX_BODY" | sudo tee /etc/nginx/conf.d/aruaru.conf >/dev/null
  sudo rm -f /etc/nginx/conf.d/default.conf 2>/dev/null || true
else
  # Debian / Ubuntu
  sudo mkdir -p /etc/nginx/sites-available /etc/nginx/sites-enabled
  echo "$NGINX_BODY" | sudo tee /etc/nginx/sites-available/aruaru.conf >/dev/null
  sudo ln -sf /etc/nginx/sites-available/aruaru.conf /etc/nginx/sites-enabled/aruaru.conf
  sudo rm -f /etc/nginx/sites-enabled/default 2>/dev/null || true
fi

sudo nginx -t
sudo systemctl enable nginx
sudo systemctl start nginx 2>/dev/null || true
sudo systemctl reload nginx
echo "OK: nginx → 127.0.0.1:3000"
