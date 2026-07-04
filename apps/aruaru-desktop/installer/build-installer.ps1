#Requires -Version 5.1
# aruaru-desktop Windows installer build (Tauri NSIS + dist copy)
param(
    [switch]$SkipNpm,
    [switch]$DevOnly
)

$ErrorActionPreference = 'Stop'
$AppRoot = Split-Path $PSScriptRoot -Parent
Set-Location $AppRoot

# sandbox 外の target に出力（icon.ico 参照問題を回避）
$env:CARGO_TARGET_DIR = Join-Path $AppRoot 'src-tauri\target'

Write-Host '=== aruaru-desktop build ===' -ForegroundColor Cyan

$iconIco = Join-Path $AppRoot 'src-tauri\icons\icon.ico'
if (-not (Test-Path $iconIco)) {
    Write-Host 'Generating icons (app-icon.png -> tauri icon)...' -ForegroundColor Yellow
    if (-not (Test-Path (Join-Path $AppRoot 'app-icon.png'))) {
        throw 'Missing app-icon.png — run: npx tauri icon app-icon.png'
    }
    npx tauri icon app-icon.png
}

if (-not $SkipNpm) {
    if (-not (Test-Path 'node_modules')) {
        Write-Host 'npm install...' -ForegroundColor Cyan
        npm install
    }
    Write-Host 'npm run tauri build...' -ForegroundColor Cyan
    npm run tauri build
    if ($LASTEXITCODE -ne 0) { throw 'tauri build failed' }
}

$bundle = Join-Path $env:CARGO_TARGET_DIR 'release\bundle'
$dist = Join-Path $AppRoot 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null

$installers = Get-ChildItem -Path $bundle -Recurse -Include '*.exe','*.msi' -ErrorAction SilentlyContinue |
    Where-Object { $_.Name -match 'aruaru.*setup|aruaru.*\.msi' -or $_.DirectoryName -match 'nsis|msi' }

if ($installers) {
    foreach ($i in $installers) {
        Copy-Item $i.FullName $dist -Force
        Write-Host "OK: $($i.FullName) -> dist\" -ForegroundColor Green
    }
} else {
    Write-Warning "Installer not found under $bundle — run npm run tauri build"
}

Write-Host ''
Write-Host 'Output: apps\aruaru-desktop\dist\' -ForegroundColor Green
Write-Host 'After install: run aruaru-desktop, set DATA on E:/F:, use VPS tab to upload' -ForegroundColor DarkGray

if ($DevOnly) { exit 0 }

Write-Host ''
Write-Host 'Optional: copy deploy/ to DATA drive for upload-vps.ps1 from desktop' -ForegroundColor DarkGray
