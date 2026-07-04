#!/usr/bin/env bash
set -euo pipefail
cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
if grep -RniE 'tauri|REST API|rest api' src Cargo.toml README.md; then
  echo '禁止仕様候補が見つかりました。README内の説明か実装混入か確認してください。'
fi
echo 'BUGチェック完了'
