#!/bin/bash
# aruaru — 接続拒否の診断（VPS 上で実行）
set -uo pipefail

echo "========== aruaru diagnose =========="
echo "date: $(date -Iseconds 2>/dev/null || date)"
echo ""

echo "--- [1] systemd aruaru ---"
if systemctl list-unit-files aruaru.service &>/dev/null; then
  systemctl is-active aruaru 2>/dev/null || true
  systemctl status aruaru --no-pager 2>/dev/null | head -20 || true
else
  echo "aruaru.service 未登録"
fi
echo ""

echo "--- [2] port 3000 listen ---"
if command -v ss &>/dev/null; then
  ss -tlnp | grep ':3000' || echo "3000 番で待ち受けなし"
else
  netstat -tlnp 2>/dev/null | grep ':3000' || echo "3000 番で待ち受けなし"
fi
echo ""

echo "--- [3] curl localhost ---"
curl -s -m 3 http://127.0.0.1:3000/api/health && echo "" || echo "127.0.0.1:3000 応答なし"
echo ""

echo "--- [4] aruaru binary ---"
for p in ~/aruaru/bin/aruaru /opt/aruaru/bin/aruaru; do
  [ -x "$p" ] && echo "found: $p" && ls -la "$p"
done
echo ""

echo "--- [5] config ---"
for c in ~/aruaru/data/config.toml /opt/aruaru/data/config.toml; do
  if [ -f "$c" ]; then
    echo "=== $c ==="
    grep -E '^(host|port|public_url|primary|postgres_url)' "$c" 2>/dev/null || cat "$c"
  fi
done
echo ""

echo "--- [6] PostgreSQL ---"
if command -v pg_isready &>/dev/null; then
  pg_isready -h 127.0.0.1 -p 5432 || echo "PostgreSQL 未起動"
else
  echo "pg_isready なし"
fi
echo ""

echo "--- [7] firewall ---"
if command -v ufw &>/dev/null; then
  ufw status 2>/dev/null || true
elif command -v firewall-cmd &>/dev/null; then
  firewall-cmd --list-all 2>/dev/null || true
else
  echo "ufw/firewalld なし"
fi
echo ""

echo "--- [8] recent logs ---"
journalctl -u aruaru -n 25 --no-pager 2>/dev/null || tail -25 /tmp/aruaru.log 2>/dev/null || echo "ログなし"
echo ""
echo "========== end =========="
