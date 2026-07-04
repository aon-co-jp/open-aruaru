#Requires -Version 5.1
# ワンショット: 自己修復 -> VPS アップロード -> Android ビルド
$ErrorActionPreference = 'Stop'
$deploy = Split-Path $PSScriptRoot -Leaf
$root = if ($deploy -eq 'deploy') { Split-Path $PSScriptRoot -Parent } else { 'F:\aruaru' }
& (Join-Path $PSScriptRoot 'self-heal.ps1') -Fix all
& (Join-Path $PSScriptRoot 'upload-vps.ps1') -Root $root
& (Join-Path $root 'apps\aruaru-smartphone\installer\build-installer.ps1')
