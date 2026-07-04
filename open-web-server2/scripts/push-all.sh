#!/usr/bin/env bash
# push-all.sh — f:\open-aruaru (または e:\open-aruaru) 配下の各リポジトリを
# まとめて git init → commit → push する。
#
# 前提:
#   - 各サブディレクトリ名と同名のGitHubリポジトリを事前に作成しておくこと
#     (例: github.com/<YOUR_ACCOUNT>/open-web-server)
#   - GitHub CLI (gh) か、SSH鍵 / PAT でのpush認証が済んでいること
#
# 使い方 (f:\open-aruaru 直下、Git Bash/WSL等から):
#   GITHUB_USER=your-account bash push-all.sh
#
# 個別リポジトリだけ対象にしたい場合:
#   GITHUB_USER=your-account bash push-all.sh open-web-server open-runo

set -euo pipefail

USER="${GITHUB_USER:?GITHUB_USER を環境変数で指定してください (例: export GITHUB_USER=your-account)}"
DEFAULT_REPOS=(open-web-server open-runo aruaru-db open-client open-aruaru)
REPOS=("${@:-${DEFAULT_REPOS[@]}}")

for repo in "${REPOS[@]}"; do
  if [[ ! -d "$repo" ]]; then
    echo "スキップ: $repo ディレクトリが見つかりません"
    continue
  fi

  echo "== $repo =="
  pushd "$repo" > /dev/null

  if [[ ! -d .git ]]; then
    git init -b main
    git remote add origin "git@github.com:${USER}/${repo}.git" 2>/dev/null || true
  fi

  # ARU3セグメント等の実データを誤ってpushしないためのデフォルト除外
  if [[ ! -f .gitignore ]]; then
    cat > .gitignore << 'EOF'
/target
/data/
!/data/.gitkeep
Cargo.lock
node_modules/
EOF
  fi

  git add -A
  if git diff --cached --quiet; then
    echo "  変更なし。push をスキップします。"
  else
    git commit -m "sync: update from f:/open-aruaru ($(date '+%Y-%m-%d %H:%M:%S'))"
  fi

  git push -u origin main
  popd > /dev/null
  echo
done

echo "完了。対象: ${REPOS[*]}"
