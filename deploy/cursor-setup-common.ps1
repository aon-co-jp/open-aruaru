#Requires -Version 5.1
# Shared helpers for aruaru Cursor setup scripts.
# Dot-source: . (Join-Path $PSScriptRoot 'cursor-setup-common.ps1')

function Get-AruaruRoot {
    param([string]$Preferred)

    if ($Preferred -and (Test-Path $Preferred)) {
        return (Resolve-Path $Preferred).Path
    }
    foreach ($p in @('F:\aruaru', 'E:\aruaru')) {
        if (Test-Path $p) { return (Resolve-Path $p).Path }
    }
    $fromScript = Split-Path $PSScriptRoot -Parent
    if (Test-Path (Join-Path $fromScript 'docs\@docs')) {
        return (Resolve-Path $fromScript).Path
    }
    throw 'aruaru root not found. Use -Root F:\aruaru'
}

function Get-CursorExecutable {
    $found = @()
    if (Get-Command cursor -ErrorAction SilentlyContinue) {
        $found += (Get-Command cursor).Source
    }
    foreach ($p in @(
        "$env:LOCALAPPDATA\Programs\cursor\resources\app\bin\cursor.cmd",
        "$env:LOCALAPPDATA\Programs\Cursor\resources\app\bin\cursor.cmd",
        'C:\Program Files\cursor\resources\app\bin\cursor.cmd',
        'C:\Program Files\Cursor\resources\app\bin\cursor.cmd'
    )) {
        if (Test-Path $p) { $found += $p }
    }
    $found | Select-Object -Unique | Select-Object -First 1
}

function Install-GlobalCursorMcp {
    param([string]$Root)

    $globalDir = Join-Path $env:USERPROFILE '.cursor'
    $globalMcp = Join-Path $globalDir 'mcp.json'
    $projMcp = Join-Path $Root '.cursor\mcp.json'
    if (-not (Test-Path $projMcp)) {
        Write-Warning "Missing project mcp.json: $projMcp"
        return $false
    }
    New-Item -ItemType Directory -Force -Path $globalDir | Out-Null
    Copy-Item -LiteralPath $projMcp -Destination $globalMcp -Force
    Write-Host "OK: global MCP -> $globalMcp" -ForegroundColor Green
    return $true
}

function Invoke-CursorAddMcpFetch {
    param(
        [string]$Root,
        [string]$CursorCmd = (Get-CursorExecutable)
    )

    if (-not $CursorCmd) {
        Write-Warning 'Cursor not found. Install Cursor first, then re-run.'
        return $false
    }

    $batch = Join-Path $PSScriptRoot 'add-mcp-fetch.cmd'
    if (-not (Test-Path $batch)) {
        Write-Warning "Missing $batch"
        return $false
    }

    $prevEap = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $out = cmd /c "`"$batch`" `"$Root`"" 2>&1
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $prevEap
    }
    if ($out) {
        $text = ($out | ForEach-Object { "$_" }) -join "`n"
        if ($text -match 'Added MCP servers') {
            Write-Host $text
        } elseif ($text -notmatch 'DeprecationWarning|punycode') {
            Write-Host $text
        }
    }
    if ($code -eq 0) {
        Write-Host 'OK: cursor --add-mcp fetch' -ForegroundColor Green
        return $true
    }
    Write-Warning 'cursor --add-mcp failed (open Cursor once, then re-run restore-cursor-setup.cmd)'
    return $false
}

function Test-CursorSetupPrerequisites {
    $r = @{
        Node = [bool](Get-Command node -ErrorAction SilentlyContinue)
        Npm  = [bool](Get-Command npm -ErrorAction SilentlyContinue)
        Cursor = [bool](Get-CursorExecutable)
    }
    foreach ($k in $r.Keys) {
        $ok = $r[$k]
        $color = if ($ok) { 'Green' } else { 'Yellow' }
        $mark = if ($ok) { 'YES' } else { 'NO' }
        Write-Host ("  {0}: {1}" -f $k, $mark) -ForegroundColor $color
    }
    return $r
}

function Write-CursorSetupManifest {
    param(
        [string]$Root,
        [hashtable]$Status
    )

    $manifestDir = Join-Path $env:USERPROFILE '.cursor'
    $manifest = Join-Path $manifestDir 'aruaru-setup.json'
    New-Item -ItemType Directory -Force -Path $manifestDir | Out-Null
    $obj = @{
        version     = 1
        root        = $Root
        lastRunUtc  = (Get-Date).ToUniversalTime().ToString('o')
        cursorPath  = (Get-CursorExecutable)
        status      = $Status
        restoreCmd  = "powershell -NoProfile -ExecutionPolicy Bypass -File `"$Root\deploy\enable-cursor-setup.ps1`""
    }
    $obj | ConvertTo-Json -Depth 4 | Set-Content -Path $manifest -Encoding UTF8
    Write-Host "OK: manifest -> $manifest" -ForegroundColor Green
}

function Register-CursorRestoreShortcut {
    param([string]$Root)

    $target = Join-Path $Root 'deploy\restore-cursor-setup.cmd'
    if (-not (Test-Path $target)) { return $false }

    $desktop = [Environment]::GetFolderPath('Desktop')
    $lnkPath = Join-Path $desktop 'aruaru Cursor Setup.lnk'
    try {
        $wsh = New-Object -ComObject WScript.Shell
        $sc = $wsh.CreateShortcut($lnkPath)
        $sc.TargetPath = $target
        $sc.WorkingDirectory = Join-Path $Root 'deploy'
        $sc.Description = 'Restore aruaru Cursor Docs + MCP after reinstall'
        $sc.Save()
        Write-Host "OK: desktop shortcut -> $lnkPath" -ForegroundColor Green
        return $true
    } catch {
        Write-Warning "Could not create shortcut: $_"
        return $false
    }
}

function Open-CursorWorkspace {
    param(
        [string]$Root,
        [string]$CursorCmd = (Get-CursorExecutable)
    )

    if (-not $CursorCmd) { return $false }
    Start-Process -FilePath $CursorCmd -ArgumentList @('-r', $Root)
    Write-Host "Opening Cursor: $Root" -ForegroundColor Cyan
    return $true
}
