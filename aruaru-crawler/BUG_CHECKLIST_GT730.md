# GT730環境向け 毎回BUGチェック手順

このファイルは、GT730付きPCへ移動した後に毎回確認するための手順です。

## 1. Rust基本チェック

```powershell
cd F:\aruaru\aruaru-readme-auto-rs

cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
```

## 2. 禁止仕様チェック

aruaru方針として、Tauri / REST API は混入させません。

```powershell
Select-String -Path .\src\*.rs,.\Cargo.toml,.\README.md -Pattern "tauri","REST API","rest api" -CaseSensitive:$false
```

READMEの説明文で検出される場合は問題ありません。Rust実装や依存関係に入っていたらNGです。

## 3. README.md → README.rs 生成チェック

```powershell
Remove-Item -Recurse -Force .\tmp-web -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force .\tmp-web
Set-Content .\tmp-web\README.md "# TEST`n`n- item 1`n- item 2" -Encoding UTF8

cargo run -- --root .\tmp-web --listen 127.0.0.1:7878 --output rs
```

別PowerShellで確認します。

```powershell
Test-Path .\tmp-web\README.rs
Get-Content .\tmp-web\README.rs -TotalCount 20
```

`True` と `pub const README_HTML` が見えればOKです。

## 4. README.html同時生成チェック

```powershell
cargo run -- --root .\tmp-web --listen 127.0.0.1:7878 --output both
```

別PowerShellで確認します。

```powershell
Test-Path .\tmp-web\README.rs
Test-Path .\tmp-web\README.html
```

両方 `True` ならOKです。

## 5. 日本語文字化けチェック

```powershell
Set-Content .\tmp-web\README.md "# 日本語テスト`n`nこんにちは。英会話とプログラミング学習。`n`n## 表`n`n|項目|内容|`n|---|---|`n|日本語|OK|" -Encoding UTF8
```

常駐起動中なら数秒待ちます。

```powershell
Start-Sleep -Seconds 6
Get-Content .\tmp-web\README.html -TotalCount 80
```

日本語が文字化けしていなければOKです。

## 6. FTPアップロード想定チェック

FTPは書き込み途中のREADME.mdを一瞬だけ見せることがあります。アプリ側は安定待ち後に生成します。

```powershell
Set-Content .\tmp-web\README.md "# FTP UPDATE TEST`n`nアップロード後の再生成テスト。" -Encoding UTF8
Start-Sleep -Seconds 6
Get-Item .\tmp-web\README.rs
Get-Item .\tmp-web\README.html
```

更新日時が変わっていればOKです。

## 7. ブラウザ表示チェック

```powershell
Start-Process .\tmp-web\README.html
```

ブラウザの開発者ツールで以下を確認します。

- スマホ縦
- スマホ横
- タブレット
- 1280×720
- 1280×800
- 1280×960
- 1366×768
- 1920×1080
- WQHD 2560×1440
- 4K
- 8K相当
- 16K相当

16K実機がなくても、横幅固定で崩れていないこと、本文幅が読みやすく制限されることを確認すればOKです。

## 8. 異常系チェック

```powershell
# 空README
Set-Content .\tmp-web\README.md "" -Encoding UTF8
Start-Sleep -Seconds 6
Test-Path .\tmp-web\README.rs

# script混入チェック
Set-Content .\tmp-web\README.md "# XSS TEST`n`n<script>alert(1)</script>" -Encoding UTF8
Start-Sleep -Seconds 6
Select-String -Path .\tmp-web\README.html -Pattern "<script>alert" -CaseSensitive:$false
```

`<script>alert` が検出されなければOKです。

## 最低合格ライン

以下を満たせば、その日の作業を進めてよいです。

- cargo fmt OK
- cargo check OK
- cargo test OK
- cargo clippy OK
- README.rs生成 OK
- README.html生成 OK
- 日本語文字化けなし
- script除去 OK
- Tauri / REST API混入なし
