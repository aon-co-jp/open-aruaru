$ErrorActionPreference = "Stop"

# Always run from project root even when launched from the scripts directory.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path (Join-Path $ScriptDir "..")
Set-Location $ProjectRoot


cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings

$matches = Select-String -Path .\src\*.rs,.\Cargo.toml,.\README.md -Pattern "tauri","REST API","rest api" -CaseSensitive:$false
if ($matches) {
  Write-Host "禁止仕様候補が見つかりました。README内の説明か実装混入か確認してください。"
  $matches | Format-Table -AutoSize
}

Write-Host "BUGチェック完了"
