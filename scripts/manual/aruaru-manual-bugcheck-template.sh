#!/usr/bin/env bash
# aruaru manual bug check template for Linux/macOS
# Usage:
#   chmod +x ./scripts/manual/aruaru-manual-bugcheck-template.sh
#   ./scripts/manual/aruaru-manual-bugcheck-template.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_ROOT"

echo "== cargo quality gate =="
cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings

echo "== banned desktop/web policy marker check =="
if grep -RinE 'tauri|rest api' src Cargo.toml; then
  echo "Banned marker found in implementation files."
  exit 1
fi

echo "== README generation smoke test =="
rm -rf ./tmp-web
mkdir -p ./tmp-web
printf '# TEST\n\n- item 1\n- item 2\n' > ./tmp-web/README.md
cargo run -- --root ./tmp-web --listen 127.0.0.1:7878 --output both --scan-interval-secs 1 &
pid=$!
trap 'kill $pid 2>/dev/null || true' EXIT
sleep 8
test -f ./tmp-web/README.rs
test -f ./tmp-web/README.html
kill $pid 2>/dev/null || true
trap - EXIT

echo "OK: manual bug check passed"
