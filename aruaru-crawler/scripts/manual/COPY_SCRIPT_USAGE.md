# aruaru manual COPY script usage

aruaru-desktop / aruaru-ai は、自動BUGチェックを標準にしながら、必要な時に手動で実行できるCOPYスクリプトも自動生成します。

## PowerShell版

プロジェクトを安全に別フォルダへコピーする dry-run です。

```powershell
cd F:\aruaru\aruaru-rs4
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-copy-template.ps1 -SourceRoot "F:\aruaru\aruaru-rs4" -DestinationRoot "F:\aruaru\backup\aruaru-rs4" -DryRun $true
```

実際にコピーする場合です。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-copy-template.ps1 -SourceRoot "F:\aruaru\aruaru-rs4" -DestinationRoot "F:\aruaru\backup\aruaru-rs4" -DryRun $false
```

## Bash版

```bash
cd /opt/aruaru/aruaru-rs4
SOURCE_ROOT=/opt/aruaru/aruaru-rs4 DESTINATION_ROOT=/opt/aruaru/backup/aruaru-rs4 DRY_RUN=1 ./scripts/manual/aruaru-manual-copy-template.sh
```

実際にコピーする場合です。

```bash
SOURCE_ROOT=/opt/aruaru/aruaru-rs4 DESTINATION_ROOT=/opt/aruaru/backup/aruaru-rs4 DRY_RUN=0 ./scripts/manual/aruaru-manual-copy-template.sh
```

## 除外されるもの

標準では以下をコピーしません。

```text
.git
target
node_modules
dist
build
tmp-web
.env
*.key
*.pem
*.pfx
id_rsa
id_ed25519
*.log
.aruaru
README.rs
README.html
```

`README.rs` / `README.html` もコピーしたい場合は、PowerShellでは `-IncludeGeneratedReadme $true` を指定します。

## 重要ルール

- 最初は必ず dry-run で確認します。
- 秘密鍵やAPIキーはコピー対象から外します。
- このCOPYスクリプトは削除操作をしません。
- aruaru-ai が自動生成したスクリプトは、実行前に内容を表示してユーザー承認を取ります。


## v12 修正メモ

PowerShellでは `param(...)` はスクリプト内の最初の実行文である必要があります。v11では `$ErrorActionPreference` が `param(...)` より前にあり、環境によって `代入式が無効です` で停止するBUGがありました。v12では `param(...)` を先頭へ移動しました。

実行例:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-copy-template.ps1 -SourceRoot "F:\aruaru\aruaru-rs4" -DestinationRoot "F:\aruaru\backup\aruaru-rs4" -DryRun $true
```

## v13 note: DryRun argument on Windows PowerShell

When calling a script through `powershell -File`, boolean arguments may be passed as strings.
The v13 script accepts all of these:

```powershell
-DryRun true
-DryRun false
-DryRun 1
-DryRun 0
-DryRun $true
-DryRun $false
```

Recommended dry-run command:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-copy-template.ps1 -SourceRoot "F:\aruaru\aruaru-rs4" -DestinationRoot "F:\aruaru\backup\aruaru-rs4" -DryRun true
```
