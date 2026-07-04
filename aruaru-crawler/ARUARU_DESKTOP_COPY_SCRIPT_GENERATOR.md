# aruaru-desktop / aruaru-ai manual COPY script generator

## 目的

aruaru-desktop と aruaru-web の自動BUGチェックを前提にしながら、ユーザーが手動でCOPYできるスクリプトを aruaru-ai が自動生成します。

これにより、以下の場面でも作業を継続できます。

- PowerShell実行ポリシーやGUI操作が不安定な場合
- GT730級PCなど低スペック環境で、手動確認しながらコピーしたい場合
- VPSへアップロードする前に、必要なファイルだけを安全にバックアップしたい場合
- aruaru-ai が生成した修正差分を、ユーザーが自分で確認してから移動したい場合

## 標準機能

```text
aruaru-ai
├─ 自動BUGチェック
├─ 自動ダウンロード検出
├─ 手動BUGチェックスクリプト生成
└─ 手動COPYスクリプト生成
```

## 安全設計

- dry-run を標準にする
- `.env`、秘密鍵、APIキー、SSHキーを標準除外する
- `target`、`node_modules`、`.git` など巨大・不要フォルダを標準除外する
- 削除操作をしない
- 実行前にユーザー承認を取る
- 実行ログを `.aruaru/logs` に保存する設計にする

## 生成対象

- Windows PowerShell: `aruaru-manual-copy.ps1`
- Linux/macOS Bash: `aruaru-manual-copy.sh`
- 説明書: `COPY_SCRIPT_USAGE.md`

## 実装ファイル

```text
src/copy_script_generator.rs
scripts/manual/aruaru-manual-copy-template.ps1
scripts/manual/aruaru-manual-copy-template.sh
scripts/manual/COPY_SCRIPT_USAGE.md
```
