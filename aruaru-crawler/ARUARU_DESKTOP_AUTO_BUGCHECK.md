# aruaru-desktop 追加機能: PowerShell自動操作・自動BUGチェック

## 目的

aruaru-desktop から Windows PowerShell を安全に操作し、Rust/Poem 系プロジェクトのBUGチェックを自動実行する。

## 固定搭載する機能

- プロジェクトフォルダ選択
- `Cargo.toml` 自動検出
- `scripts/check-full.ps1` 自動検出
- PowerShell 実行ポリシーエラーの自動判定
- 必要時のみ `-ExecutionPolicy Bypass -File` で1回実行
- `cargo fmt --all -- --check`
- `cargo check`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- Tauri / REST API 混入チェック
- README.md 変換アプリの場合は README.rs / README.html 生成チェック
- 実行ログ保存
- エラー箇所のAI要約
- 修正候補の提示
- ユーザー承認後のみ修正適用

## PowerShell実行ルール

通常実行で署名エラーが出た場合、以下へ自動切替する。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
```

この切替はPC全体の実行ポリシーを変更しない。

## 禁止事項

- ユーザー承認なしに `Set-ExecutionPolicy` を実行しない
- ユーザー承認なしにファイルを自動修正しない
- REST API / Tauri を復活させない
- 管理者権限を勝手に要求しない
- 秘密鍵、トークン、`.env` をログにそのまま出さない

## 自動BUGチェック結果の分類

- PASS: 問題なし
- WARN: 注意。生成物や表記の確認が必要
- FAIL: 修正必須
- BLOCKED: 実行ポリシー、cargo未導入、パス不一致などで未実行

## aruaru-ai 連携

1. ログを収集
2. 失敗原因を分類
3. 最小修正案を作成
4. 差分を表示
5. ユーザー承認
6. 適用
7. 再度BUGチェック

## 今回発見した実例

- PowerShell実行ポリシーで `.ps1` がブロックされた
- `main.rs` で `dashboard` 名が重複した
- Poem の `.data()` に必要な `EndpointExt` import が不足した
- `scripts` フォルダから実行すると `src` パスが見つからなかった
