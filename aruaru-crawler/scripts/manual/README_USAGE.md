# 手動BUGチェックスクリプトの使い方

このフォルダには、aruaru-desktop / aruaru-web が自動生成する手動BUGチェックスクリプトのテンプレートが入っています。

## Windows PowerShell

プロジェクトルートから実行:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-bugcheck-template.ps1
```

このフォルダ内から実行:

```powershell
powershell -ExecutionPolicy Bypass -File .\aruaru-manual-bugcheck-template.ps1
```

## Linux / macOS Bash

```bash
chmod +x ./scripts/manual/aruaru-manual-bugcheck-template.sh
./scripts/manual/aruaru-manual-bugcheck-template.sh
```

## 実行される内容

```text
cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
README生成スモークテスト
禁止仕様混入チェック
```

## 成功表示

最後に以下が出ればOKです。

```text
OK: manual bug check passed
```
