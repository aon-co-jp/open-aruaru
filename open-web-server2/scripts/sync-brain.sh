#!/usr/bin/env bash
# sync-brain.sh — open-web-server の「学習済みの賢さ」を Googleドライブへ同期する。
#
# 設計方針:
#   - Googleドライブ専用アプリを自作するのではなく、実績十分な rclone に同期そのものを任せる。
#   - open-web-server 側は aruaru-brain.json に BLAKE3 チェックサムを同梱して書き出し済みなので、
#     途中で切れた・壊れた同期を起動時に自動検出し、安全に「新規学習」へフォールバックする
#     (誤った学習状態を引き継がない = ミッションクリティアル原則と同じ考え方)。
#
# 前提: rclone がインストール済みで、`rclone config` で "gdrive" という名前の
#       リモートを一度だけ作成済みであること (Google認証はここで一度だけ必要)。
#       参考: Ubuntu/Linux Mint なら「オンラインアカウント」でGoogleを追加しておくと
#       Nautilus/Nemo からも同じアカウントで参照できて分かりやすい。
#
# 使い方:
#   bash sync-brain.sh push   # ローカル → Googleドライブ (学習結果をアップロード)
#   bash sync-brain.sh pull   # Googleドライブ → ローカル (新PC/OS再インストール後の復元)
#   bash sync-brain.sh watch  # 5分毎に自動push (crontab登録を推奨)
#
# crontab 例 (5分毎に自動バックアップ):
#   */5 * * * * bash /path/to/sync-brain.sh push >> /var/log/aruaru-brain-sync.log 2>&1

set -euo pipefail

REMOTE="gdrive:open-aruaru-brain"
LOCAL_DIR="${ARUARU_DATA_DIR:-./data}"
BRAIN_FILE="aruaru-brain.json"

require_rclone() {
  if ! command -v rclone >/dev/null 2>&1; then
    echo "rclone が見つかりません。"
    echo "Ubuntu/Debian系・RHEL/AlmaLinux/Rocky Linux系のどちらでも次で導入できます:"
    echo "  bash \"$(dirname "$0")/install-deps.sh\""
    exit 1
  fi
}

cmd_push() {
  require_rclone
  if [[ ! -f "$LOCAL_DIR/$BRAIN_FILE" ]]; then
    echo "警告: $LOCAL_DIR/$BRAIN_FILE がまだ存在しません (100リクエスト到達前は未生成)。"
    exit 0
  fi
  rclone copy "$LOCAL_DIR/$BRAIN_FILE" "$REMOTE/" --checksum
  echo "OK: $BRAIN_FILE を $REMOTE へ同期しました。"
}

cmd_pull() {
  require_rclone
  mkdir -p "$LOCAL_DIR"
  rclone copy "$REMOTE/$BRAIN_FILE" "$LOCAL_DIR/" --checksum
  echo "OK: $REMOTE から $BRAIN_FILE を復元しました。次回 open-web-server 起動時に自動で読み込まれます。"
}

cmd_watch() {
  echo "5分毎に push します。Ctrl+C で停止 (常駐させたい場合は crontab 登録を推奨)。"
  while true; do
    cmd_push || true
    sleep 300
  done
}

case "${1:-}" in
  push) cmd_push ;;
  pull) cmd_pull ;;
  watch) cmd_watch ;;
  *) echo "使い方: $0 {push|pull|watch}"; exit 1 ;;
esac
