#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo build --release -p aruaru-gql-gateway
sudo install -Dm755 target/release/aruaru-gql-gateway /usr/local/bin/aruaru-gql-gateway
sudo tee /etc/systemd/system/aruaru-gql-gateway.service >/dev/null <<SERVICE
[Unit]
Description=aruaru GraphQL Gateway
After=network.target

[Service]
Environment=ARUARU_BIND=127.0.0.1:8787
Environment=ARUARU_WORKSPACE=/var/lib/aruaru/workspace
ExecStart=/usr/local/bin/aruaru-gql-gateway
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
SERVICE
sudo systemctl daemon-reload
sudo systemctl enable --now aruaru-gql-gateway
