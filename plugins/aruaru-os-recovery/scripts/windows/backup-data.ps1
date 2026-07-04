# aruaru-OS-Recovery: DATA ディレクトリのバックアップ（robocopy）
param(
    [Parameter(Mandatory)][string]$DataDir,
    [Parameter(Mandatory)][string]$BackupDir
)

$ErrorActionPreference = "Stop"
$dataDir = [System.IO.Path]::GetFullPath($DataDir)
$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$dest = Join-Path ([System.IO.Path]::GetFullPath($BackupDir)) $stamp
New-Item -ItemType Directory -Force -Path $dest | Out-Null

$log = Join-Path $dest "robocopy.log"
robocopy $dataDir $dest /MIR /R:2 /W:3 /XD "recovery\backups" /LOG:$log /NFL /NDL /NJH /NJS
if ($LASTEXITCODE -ge 8) { throw "robocopy failed with exit $LASTEXITCODE" }
Write-Output $dest
