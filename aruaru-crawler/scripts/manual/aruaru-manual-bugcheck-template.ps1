# aruaru manual bug check template for Windows PowerShell
# Usage:
#   powershell -ExecutionPolicy Bypass -File .\scripts\manual\aruaru-manual-bugcheck-template.ps1
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Resolve-Path (Join-Path $ScriptDir "..\..")
Set-Location $ProjectRoot

Write-Host "== cargo quality gate =="
cargo fmt --all
cargo fmt --all -- --check
cargo check
cargo test
cargo clippy --all-targets -- -D warnings

Write-Host "== banned desktop/web policy marker check =="
$hits = Select-String -Path .\src\*.rs,.\Cargo.toml -Pattern "tauri","rest api" -CaseSensitive:$false
if ($hits) {
  $hits
  throw "Banned marker found in implementation files."
}

Write-Host "== README generation smoke test =="
Remove-Item -Recurse -Force .\tmp-web -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force .\tmp-web | Out-Null
Set-Content .\tmp-web\README.md "# TEST`n`n- item 1`n- item 2" -Encoding UTF8

$proc = Start-Process cargo -ArgumentList @("run","--","--root",".\tmp-web","--listen","127.0.0.1:7878","--output","both","--scan-interval-secs","1") -PassThru -WindowStyle Hidden
try {
  Start-Sleep -Seconds 8
  if (!(Test-Path .\tmp-web\README.rs)) { throw "README.rs was not generated" }
  if (!(Test-Path .\tmp-web\README.html)) { throw "README.html was not generated" }
}
finally {
  if ($proc -and -not $proc.HasExited) { Stop-Process -Id $proc.Id -Force }
}

Write-Host "OK: manual bug check passed"
