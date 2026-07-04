# aruaru-OS-Recovery: OS 再インストール後に footprint からアクセスを復元
param(
    [Parameter(Mandatory)][string]$FootprintPath,
    [string]$DesktopConfigDir = "$env:LOCALAPPDATA\aruaru"
)

$ErrorActionPreference = "Stop"
if (-not (Test-Path -LiteralPath $FootprintPath)) {
    throw "Footprint not found: $FootprintPath"
}

$fp = Get-Content -LiteralPath $FootprintPath -Raw -Encoding UTF8 | ConvertFrom-Json
$dataDir = $fp.data_dir
$appDir = $fp.app_dir

if (-not (Test-Path -LiteralPath $dataDir)) {
    throw "Data directory missing: $dataDir"
}

New-Item -ItemType Directory -Force -Path $DesktopConfigDir | Out-Null
$configPath = Join-Path $DesktopConfigDir "desktop.toml"

$lines = @(
    "# aruaru desktop — auto-restored by aruaru-OS-Recovery",
    "data_dir = `"$($dataDir -replace '\\','\\')`"",
    "root_dir = `"$($appDir -replace '\\','\\')`"",
    "footprint = `"$($FootprintPath -replace '\\','\\')`"",
    "restored_at = `"$((Get-Date).ToUniversalTime().ToString('o'))`""
)
$lines | Set-Content -Path $configPath -Encoding UTF8
Write-Output $configPath
