#Requires -Version 5.1
# adb で aruaru-android APK を端末にインストール
param(
    [string]$ApkPath,
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'
$DistDir = if ($PSScriptRoot -match 'dist$') { $PSScriptRoot } else { Join-Path (Split-Path $PSScriptRoot -Parent) 'dist' }

if ($Uninstall) {
    adb uninstall com.aruaru.android
    Write-Host 'Uninstalled com.aruaru.android' -ForegroundColor Green
    exit 0
}

if (-not $ApkPath) {
    $ApkPath = Get-ChildItem -Path $DistDir -Filter 'aruaru-android_*_universal.apk' -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1 -ExpandProperty FullName
}
if (-not $ApkPath -or -not (Test-Path $ApkPath)) {
    throw "APK not found. Build first: F:/aruaru/apps/aruaru-smartphone/installer/build-installer.ps1"
}

$adb = Get-Command adb -ErrorAction SilentlyContinue
if (-not $adb) {
    $sdk = $env:ANDROID_HOME
    if (-not $sdk) { $sdk = Join-Path $env:LOCALAPPDATA 'Android\Sdk' }
    $adbExe = Join-Path $sdk 'platform-tools\adb.exe'
    if (-not (Test-Path $adbExe)) {
        throw 'adb not found. Install Android SDK platform-tools or add to PATH.'
    }
    $adb = $adbExe
} else {
    $adb = $adb.Source
}

Write-Host "Installing: $ApkPath" -ForegroundColor Cyan
& $adb devices
& $adb install -r $ApkPath
if ($LASTEXITCODE -eq 0) {
    Write-Host 'OK: aruaru-android installed' -ForegroundColor Green
    Write-Host 'Open app → 設定 → VPS URL を入力' -ForegroundColor DarkGray
} else {
    throw 'adb install failed'
}
