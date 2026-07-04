#Requires -Version 5.1
param(
    [switch]$SkipInstaller,
    [switch]$ExeOnly,
    [switch]$SafeTarget
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
Set-Location $Root

$localTarget = Join-Path $env:LOCALAPPDATA "aruaru-github-target"
$projectTarget = Join-Path $Root "target"

function Invoke-CargoBuild {
    param([string]$TargetDir)

    $env:CARGO_TARGET_DIR = $TargetDir
    Write-Host "CARGO_TARGET_DIR=$TargetDir"

    $rel = Join-Path $TargetDir "release"
    if (Test-Path $rel) {
        Remove-Item -Recurse -Force $rel -ErrorAction SilentlyContinue
    }

    $out = cmd /c "cargo build --release 2>&1"
    $code = $LASTEXITCODE
    if ($out) { Write-Host $out }
    return @{ ExitCode = $code; Output = ($out -join "`n") }
}

Write-Host "=== [1/4] cargo build --release ===" -ForegroundColor Cyan

$targetDir = if ($SafeTarget) { $localTarget } else { $projectTarget }
$result = Invoke-CargoBuild -TargetDir $targetDir

if ($result.ExitCode -ne 0 -and -not $SafeTarget) {
    $crc = $result.Output -match "CRC|巡回冗長|os error 23|error 23"
    if ($crc) {
        Write-Host "CRC error on project drive. Retrying with C: target..." -ForegroundColor Yellow
        Write-Host "Tip: run chkdsk E: /F as Administrator to repair exFAT." -ForegroundColor Yellow
        $targetDir = $localTarget
        $result = Invoke-CargoBuild -TargetDir $targetDir
    }
}

if ($result.ExitCode -ne 0) {
    Write-Host "Build failed. Try: .\installer\build-installer.ps1 -SafeTarget -ExeOnly" -ForegroundColor Red
    exit 1
}

$exe = Join-Path $targetDir "release\aruaru-github.exe"
if (-not (Test-Path $exe)) {
    Write-Error "Build output not found: $exe"
}

$len = (Get-Item $exe).Length
if ($len -lt 100000) {
    Write-Error "exe too small: $len bytes"
}

try {
    $null = [System.IO.File]::ReadAllBytes($exe)[0..15]
}
catch {
    Write-Error "exe read error (CRC/IO). Run: chkdsk E: /F"
}

Write-Host "=== [2/4] copy to dist ===" -ForegroundColor Cyan
$dist = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null
$destExe = Join-Path $dist "aruaru-github.exe"

Copy-Item $exe $destExe -Force
Write-Host "OK: $destExe size=$len" -ForegroundColor Green

if ($ExeOnly -or $SkipInstaller) {
    Write-Host "Done (exe only)."
    exit 0
}

Write-Host "=== [3/4] portable ZIP via tar.exe ===" -ForegroundColor Cyan
$zip = Join-Path $dist "aruaru-github-portable.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }

if (Get-Command tar.exe -ErrorAction SilentlyContinue) {
    Push-Location $dist
    tar.exe -a -c -f "aruaru-github-portable.zip" "aruaru-github.exe"
    Pop-Location
    if ($LASTEXITCODE -ne 0) { Write-Host "tar failed - use dist exe directly" -ForegroundColor Yellow }
    else { Write-Host "OK: $zip" -ForegroundColor Green }
}

Write-Host "=== [4/4] Inno Setup ===" -ForegroundColor Cyan
$iscc = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
) | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $iscc) {
    Write-Host "Inno Setup not installed. Use dist\aruaru-github.exe" -ForegroundColor Yellow
    exit 0
}

& $iscc (Join-Path $Root "installer\aruaru-github.iss")
if ($LASTEXITCODE -ne 0) {
    Write-Host "Inno failed. dist\aruaru-github.exe is still usable." -ForegroundColor Yellow
    exit 1
}

Write-Host "Done:" -ForegroundColor Green
Get-ChildItem $dist | ForEach-Object { Write-Host $_.FullName }
