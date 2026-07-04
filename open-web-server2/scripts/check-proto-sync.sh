#!/usr/bin/env bash
# check-proto-sync.sh
# open-web-server と open-runo が同一の proto/runo.proto を持っているか検証する。
# 相性事故 (片方だけ更新して契約がズレる) を CI で機械的に防ぐためのスクリプト。
#
# 使い方 (f:\open-aruaru 直下から):
#   bash open-web-server/scripts/check-proto-sync.sh
#
# 終了コード: 0=一致 / 1=不一致 (どちらを更新すべきか表示)

set -euo pipefail

A="open-web-server/proto/runo.proto"
B="open-runo/proto/runo.proto"

if [[ ! -f "$A" || ! -f "$B" ]]; then
  echo "エラー: $A または $B が見つかりません。カレントディレクトリを確認してください。"
  exit 1
fi

HASH_A=$(sha256sum "$A" | cut -d' ' -f1)
HASH_B=$(sha256sum "$B" | cut -d' ' -f1)

if [[ "$HASH_A" == "$HASH_B" ]]; then
  echo "OK: proto/runo.proto は open-web-server と open-runo で一致しています ($HASH_A)"
  exit 0
else
  echo "NG: proto/runo.proto が2リポジトリ間でズレています。"
  echo "  $A : $HASH_A"
  echo "  $B : $HASH_B"
  echo "→ 新しい方を確認し、古い方へ手動コピーしてください (将来 runo-proto 共有crateへ統合予定)。"
  exit 1
fi
