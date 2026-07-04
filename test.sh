#!/bin/bash
# =============================================================
# test.sh — aruaru デプロイ後の動作確認
# 使い方: bash test.sh <VPS_IP>
# =============================================================

VPS_IP="${1:?使い方: bash test.sh <VPS_IP>}"
EASYWEB="http://$VPS_IP:8080"
CORE="http://$VPS_IP:8081"

PASS=0
FAIL=0

check() {
  local label="$1"
  local url="$2"
  local expect_code="${3:-200}"
  local expect_body="$4"

  local result
  result=$(curl -s -o /tmp/aruaru_test_body -w "%{http_code}" --max-time 5 "$url" 2>/dev/null)

  if [ "$result" = "$expect_code" ]; then
    if [ -n "$expect_body" ]; then
      if grep -q "$expect_body" /tmp/aruaru_test_body 2>/dev/null; then
        echo "  ✅  $label ($result + body OK)"
        PASS=$((PASS+1))
      else
        echo "  ❌  $label ($result, body に '$expect_body' が見つからない)"
        FAIL=$((FAIL+1))
      fi
    else
      echo "  ✅  $label ($result)"
      PASS=$((PASS+1))
    fi
  else
    echo "  ❌  $label (期待: $expect_code, 実際: $result)"
    FAIL=$((FAIL+1))
  fi
}

echo ""
echo "======================================"
echo " 🧪 aruaru テスト開始"
echo "    VPS: $VPS_IP"
echo "======================================"

# -------------------------------------------------------
# aruaru-easyweb (:8080)
# -------------------------------------------------------
echo ""
echo "── aruaru-easyweb (:8080) ──────────────"

check "TOP (未セットアップ → /setup へ 302)" \
  "$EASYWEB/" "302"

check "GET /setup (セットアップ画面)" \
  "$EASYWEB/setup" "200" "aruaru"

check "GET /login (ログイン画面)" \
  "$EASYWEB/login" "200" "メールアドレス"

check "GET /anthropic (Anthropic ログイン画面)" \
  "$EASYWEB/anthropic" "200" "ANTHROPIC LOGIN"

check "GET /static/css/app.css (静的ファイル)" \
  "$EASYWEB/static/css/app.css" "200"

# -------------------------------------------------------
# aruaru-core (:8081)
# -------------------------------------------------------
echo ""
echo "── aruaru-core (:8081) ─────────────────"

check "TOP (VPS ダッシュボード)" \
  "$CORE/" "200" "aruaru-core"

check "GET /api/vps (VPS 一覧 JSON)" \
  "$CORE/api/vps" "200"

check "GET /api/vps/detect-ip (IP 自動検出)" \
  "$CORE/api/vps/detect-ip" "200" "ip"

check "GET /api/keys (SSH 鍵一覧 JSON)" \
  "$CORE/api/keys" "200"

check "GET /terminal (Web SSH ターミナル)" \
  "$CORE/terminal" "200" "xterm"

check "GET /files (Web SFTP ファイル管理)" \
  "$CORE/files" "200"

check "GET /static/css/core.css (静的ファイル)" \
  "$CORE/static/css/core.css" "200"

# -------------------------------------------------------
# サービス状態確認（SSH が使える場合）
# -------------------------------------------------------
echo ""
echo "── systemd サービス状態 ────────────────"
if command -v ssh &> /dev/null; then
  ssh -o ConnectTimeout=5 -o BatchMode=yes "root@$VPS_IP" \
    "systemctl is-active aruaru-easyweb && echo '✅  aruaru-easyweb: 稼働中' || echo '❌  aruaru-easyweb: 停止'; \
     systemctl is-active aruaru-core    && echo '✅  aruaru-core:    稼働中' || echo '❌  aruaru-core:    停止'" 2>/dev/null \
  || echo "  ⚠️  SSH 接続できません（HTTP テストのみ有効）"
else
  echo "  ⚠️  ssh コマンドが見つかりません"
fi

# -------------------------------------------------------
# 結果サマリー
# -------------------------------------------------------
echo ""
echo "======================================"
TOTAL=$((PASS+FAIL))
echo " 結果: $PASS / $TOTAL 合格"
if [ $FAIL -eq 0 ]; then
  echo " 🎉 全テスト通過！"
  echo ""
  echo " 次のステップ:"
  echo "   1. http://$VPS_IP:8080/setup でセットアップ"
  echo "   2. http://$VPS_IP:8080/anthropic で API Key 連携"
  echo "   3. http://$VPS_IP:8081/ で VPS 追加・SSH 接続"
else
  echo " ⚠️  $FAIL 件失敗。ログを確認してください:"
  echo "   ssh root@$VPS_IP 'journalctl -u aruaru-easyweb -n 50'"
  echo "   ssh root@$VPS_IP 'journalctl -u aruaru-core -n 50'"
fi
echo "======================================"
