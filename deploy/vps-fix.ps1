#Requires -Version 5.1
# VPS 接続修復 — diagnose + fix-start + nginx + firewall（SSH 経由）
param([string]$Root = '')

$ErrorActionPreference = 'Stop'
if (-not $Root) {
    foreach ($p in @('F:\aruaru', 'E:\aruaru')) {
        if (Test-Path $p) { $Root = $p; break }
    }
}
$deploy = Join-Path $Root 'deploy'
& (Join-Path $deploy 'upload-vps.ps1') -Root $Root -SkipSetup
Write-Host ''
Write-Host '=== VPS fix (remote) ===' -ForegroundColor Cyan
$cfg = & (Join-Path $deploy 'cursor-vps.ps1') check 2>$null
& powershell -ExecutionPolicy Bypass -File (Join-Path $deploy 'cursor-vps.ps1') status

$keyPath = Join-Path $env:USERPROFILE '.ssh\conoha-vps.pem'
if (-not (Test-Path $keyPath)) { throw "key missing: $keyPath" }
$key = ($keyPath -replace '\\', '/')
$target = 'root@160.251.237.162'
$remote = '/root/aruaru'
$ssh = @('-i', $key, '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=accept-new')

$prev = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$cmds = @(
    "cd '$remote' && chmod +x deploy/*.sh && ./deploy/diagnose.sh",
    "cd '$remote' && ./deploy/fix-start.sh",
    "cd '$remote' && ./deploy/nginx-aruaru.sh '$remote' && ./deploy/firewall-aruaru.sh",
    "curl -sf -m 5 http://127.0.0.1:3000/api/health; echo; curl -sf -m 5 http://127.0.0.1/api/health; echo"
)
foreach ($c in $cmds) {
    Write-Host ">> $c" -ForegroundColor DarkGray
    & ssh @ssh $target $c
}
$ErrorActionPreference = $prev

Write-Host ''
Write-Host 'ブラウザで開く:' -ForegroundColor Green
Write-Host '  http://160.251.237.162/#/register  (port 80 推奨)'
Write-Host '  http://160.251.237.162:3000/#/register'
