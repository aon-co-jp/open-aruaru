# Windows PowerShell 実行コマンド注意

## 今回のエラー

`scripts` フォルダ内に移動した状態で、さらに `.
 scripts\check-full.ps1` を指定したため、次のような存在しないパスを探して失敗した。

```powershell
PS F:\aruaru\aruaru-rs4\scripts> powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
```

この場所から見ると、正しいファイルは以下。

```powershell
.\check-full.ps1
```

## 正しい実行方法 1: プロジェクトルートから実行

```powershell
cd F:\aruaru\aruaru-rs4
powershell -ExecutionPolicy Bypass -File .\scripts\check-full.ps1
```

## 正しい実行方法 2: scripts フォルダ内から実行

```powershell
cd F:\aruaru\aruaru-rs4\scripts
powershell -ExecutionPolicy Bypass -File .\check-full.ps1
```

## 署名エラーが出る場合

```powershell
Unblock-File .\check-full.ps1
powershell -ExecutionPolicy Bypass -File .\check-full.ps1
```

## 毎回の基本BUGチェック

```powershell
cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```
