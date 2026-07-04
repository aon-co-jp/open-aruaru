#Requires -Version 5.1
<#
.SYNOPSIS
  aruaru Cursor 環境を一括復元（Docs registry / MCP / グローバル mcp.json）

.DESCRIPTION
  同じ PC で Cursor を入れ直した後、または OS 再インストール後に実行。
  前提: Node.js 18+、Cursor、F:\aruaru（または E:\aruaru）が存在すること。

.EXAMPLE
  cd F:\aruaru
  .\deploy\enable-cursor-setup.ps1

.EXAMPLE
  .\deploy\enable-cursor-setup.ps1 -RegisterShortcut
  # デスクトップに「aruaru Cursor Setup」ショートカットを作成（再インストール後ワンクリック）
#>
param(
    [string]$Root = '',
    [switch]$SkipOpen,
    [switch]$SkipVerify,
    [switch]$RegisterShortcut,
    [switch]$Quiet
)

$ErrorActionPreference = 'Stop'
$DeployDir = $PSScriptRoot
. (Join-Path $DeployDir 'cursor-setup-common.ps1')

if (-not $Root) { $Root = Get-AruaruRoot } else { $Root = Get-AruaruRoot -Preferred $Root }

if (-not $Quiet) {
    Write-Host '=== aruaru enable-cursor-setup ===' -ForegroundColor Cyan
    Write-Host "Root: $Root"
    Write-Host ''
    Write-Host 'Prerequisites:' -ForegroundColor Yellow
}
$prereq = Test-CursorSetupPrerequisites

Push-Location $Root
try {
    & (Join-Path $Root 'deploy\register-cursor-docs.ps1')

    $mcpOk = Install-GlobalCursorMcp -Root $Root
    $addMcpOk = Invoke-CursorAddMcpFetch -Root $Root

    if (-not $SkipVerify) {
        $verifyScript = Join-Path $Root 'deploy\verify-cursor-docs.ps1'
        if (Test-Path $verifyScript) {
            Write-Host ''
            Write-Host '=== verify-cursor-docs ===' -ForegroundColor Cyan
            & $verifyScript -Root $Root
        }
    }

    if ($RegisterShortcut) {
        Register-CursorRestoreShortcut -Root $Root | Out-Null
    }

    $status = @{
        docsRegistry = Test-Path (Join-Path $Root '.cursor\docs-registry.json')
        globalMcp    = $mcpOk
        cursorAddMcp = $addMcpOk
        node         = $prereq.Node
        cursor       = $prereq.Cursor
    }
    Write-CursorSetupManifest -Root $Root -Status $status

    if (-not $SkipOpen) {
        Open-CursorWorkspace -Root $Root | Out-Null
    }

    if (-not $Quiet) {
        Write-Host ''
        Write-Host '=== Done ===' -ForegroundColor Green
        Write-Host '1. Ctrl+Shift+P -> Developer: Reload Window'
        Write-Host '2. Ctrl+Shift+J -> Tools and MCP -> fetch ON (if grey)'
        Write-Host '3. Rules load automatically (Poem, Cosmo, stack, etc.)'
        Write-Host ''
        Write-Host 'Reinstall shortcut: deploy\restore-cursor-setup.cmd' -ForegroundColor DarkGray
        Write-Host 'Or: .\deploy\enable-cursor-setup.ps1 -RegisterShortcut' -ForegroundColor DarkGray
    }
} finally {
    Pop-Location
}
