# aruaru-OS-Recovery: データドライブに footprint を書き込む
param(
    [Parameter(Mandatory)][string]$DataDir,
    [Parameter(Mandatory)][string]$AppDir,
    [string]$MarkerFile = ".aruaru-footprint.json"
)

$ErrorActionPreference = "Stop"
$dataDir = [System.IO.Path]::GetFullPath($DataDir)
$appDir = [System.IO.Path]::GetFullPath($AppDir)
New-Item -ItemType Directory -Force -Path $dataDir | Out-Null

$drive = [System.IO.Path]::GetPathRoot($dataDir).TrimEnd('\')
$now = (Get-Date).ToUniversalTime().ToString("o")
$token = [guid]::NewGuid().ToString("N")

$fp = @{
    version             = 1
    aruaru_version      = "0.1.0"
    plugin              = "aruaru-os-recovery"
    data_dir            = $dataDir
    app_dir             = $appDir
    data_drive          = $drive
    access_token        = $token
    created_at          = $now
    updated_at          = $now
    features            = @("drive-isolation", "access-restore", "backup", "migration", "sync")
}

$markerPath = Join-Path $dataDir $MarkerFile
$fp | ConvertTo-Json -Depth 5 | Set-Content -Path $markerPath -Encoding UTF8
Write-Output $markerPath
