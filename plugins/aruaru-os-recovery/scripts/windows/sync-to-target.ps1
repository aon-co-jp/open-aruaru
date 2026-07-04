# aruaru-OS-Recovery: DATA を別ドライブ / NAS へ同期（初版: robocopy ミラー）
param(
    [Parameter(Mandatory)][string]$DataDir,
    [Parameter(Mandatory)][string]$TargetDir
)

$ErrorActionPreference = "Stop"
$dataDir = [System.IO.Path]::GetFullPath($DataDir)
$target = [System.IO.Path]::GetFullPath($TargetDir)
New-Item -ItemType Directory -Force -Path $target | Out-Null

$log = Join-Path $target "sync-$(Get-Date -Format 'yyyyMMdd-HHmmss').log"
robocopy $dataDir $target /MIR /R:2 /W:3 /XD "recovery\backups" /LOG:$log /NFL /NDL /NJH /NJS
if ($LASTEXITCODE -ge 8) { throw "robocopy failed with exit $LASTEXITCODE" }
Write-Output $target
