#Requires -Version 5.1
param(
    [string]$PathE = 'E:\aruaru',
    [string]$PathF = 'F:\aruaru',
    [switch]$DryRun,
    [switch]$IncludeBuildArtifacts
)

$ErrorActionPreference = 'Stop'

$MinFreeBytes = 512MB
$script:StoppedForSpace = $false
$MsgDiskFullPath = Join-Path $PSScriptRoot 'msg-disk-full.txt'
$MsgDiskFull = if (Test-Path $MsgDiskFullPath) {
    (Get-Content -LiteralPath $MsgDiskFullPath -Raw -Encoding UTF8).Trim()
} else {
    'Disk full: sync stopped.'
}

$ExcludeDirs = @(
    'node_modules',
    'target',
    '.git',
    '$RECYCLE.BIN',
    'System Volume Information',
    '.tmp.driveupload'
)

function Get-DriveFreeBytes([string]$fullPath) {
    $root = [System.IO.Path]::GetPathRoot($fullPath)
    if (-not $root) { return [long]::MaxValue }
    $letter = $root.TrimEnd('\', ':')
    $ps = Get-PSDrive -Name $letter -ErrorAction SilentlyContinue
    if ($ps) { return [long]$ps.Free }
    return (New-Object System.IO.DriveInfo($root)).AvailableFreeSpace
}

function Test-HasSpace([string]$destPath, [long]$bytesNeeded) {
    $free = Get-DriveFreeBytes $destPath
    return ($free -ge ($bytesNeeded + $MinFreeBytes))
}

function Stop-ForLowSpace([string]$destPath) {
    $freeGb = [math]::Round((Get-DriveFreeBytes $destPath) / 1GB, 2)
    $bufMb = [math]::Round($MinFreeBytes / 1MB)
    Write-Host ''
    Write-Host $MsgDiskFull -ForegroundColor Red
    Write-Host ('  dest: {0}' -f $destPath) -ForegroundColor Yellow
    Write-Host ('  free: {0} GB (buffer: {1} MB)' -f $freeGb, $bufMb) -ForegroundColor Yellow
    $script:StoppedForSpace = $true
}

function Test-ExcludedRelative([string]$rel) {
    foreach ($p in ($rel -split '[\\/]')) {
        if ($ExcludeDirs -contains $p) { return $true }
    }
    return $false
}

function Get-RelativeFiles([string]$root) {
    $map = @{}
    if (-not (Test-Path $root)) { return $map }
    Get-ChildItem -Path $root -Recurse -File -Force -ErrorAction SilentlyContinue | ForEach-Object {
        $rel = $_.FullName.Substring($root.Length).TrimStart('\', '/')
        if (-not (Test-ExcludedRelative $rel)) { $map[$rel] = $_ }
    }
    return $map
}

function Copy-NewerFile {
    param([System.IO.FileInfo]$Source, [string]$DestFull)

    if (-not (Test-HasSpace $DestFull $Source.Length)) {
        Stop-ForLowSpace $DestFull
        return $false
    }
    if (-not $DryRun) {
        if (-not (Test-Path -LiteralPath $Source.FullName)) {
            return $true
        }
        $destDir = Split-Path $DestFull -Parent
        if (-not (Test-Path $destDir)) {
            New-Item -ItemType Directory -Force -Path $destDir | Out-Null
        }
        Copy-Item -LiteralPath $Source.FullName -Destination $DestFull -Force
    }
    return $true
}

function Sync-Between {
    param(
        [string]$FromRoot, [string]$ToRoot,
        [hashtable]$FromMap, [hashtable]$ToMap,
        [ref]$Copied, [ref]$Skipped
    )

    foreach ($rel in $FromMap.Keys) {
        if ($script:StoppedForSpace) { return }
        $src = $FromMap[$rel]
        $destFull = Join-Path $ToRoot $rel
        if ($ToMap.ContainsKey($rel)) {
            if ($src.LastWriteTimeUtc -le $ToMap[$rel].LastWriteTimeUtc) {
                $Skipped.Value++
                continue
            }
        }
        if (-not (Copy-NewerFile -Source $src -DestFull $destFull)) { return }
        $Copied.Value++
        $verb = if ($DryRun) { 'would copy' } else { 'copied' }
        Write-Host ('  {0} : {1}' -f $verb, $rel) -ForegroundColor DarkGray
    }

    foreach ($rel in $ToMap.Keys) {
        if ($script:StoppedForSpace) { return }
        if ($FromMap.ContainsKey($rel)) { continue }
        $src = $ToMap[$rel]
        $destFull = Join-Path $FromRoot $rel
        if (-not (Copy-NewerFile -Source $src -DestFull $destFull)) { return }
        $Copied.Value++
        $verb = if ($DryRun) { 'would copy' } else { 'copied' }
        Write-Host ('  {0} : {1}' -f $verb, $rel) -ForegroundColor DarkGray
    }
}

Write-Host '=== aruaru drive sync (E and F) ===' -ForegroundColor Cyan
Write-Host ('  primary: {0}' -f $PathF) -ForegroundColor Green
Write-Host ('  E path: {0}' -f $PathE) -ForegroundColor Gray
Write-Host ('  F path: {0}' -f $PathF) -ForegroundColor Gray
if ($DryRun) { Write-Host '  [DryRun]' -ForegroundColor Yellow }

foreach ($p in @($PathE, $PathF)) {
    if (-not (Test-Path $p)) {
        if (-not $DryRun) { New-Item -ItemType Directory -Force -Path $p | Out-Null }
    }
}

$freeE = [math]::Round((Get-DriveFreeBytes $PathE) / 1GB, 2)
$freeF = [math]::Round((Get-DriveFreeBytes $PathF) / 1GB, 2)
Write-Host ('  free: E={0} GB F={1} GB' -f $freeE, $freeF) -ForegroundColor Gray

Write-Host 'Collecting file lists...' -ForegroundColor Cyan
$mapE = Get-RelativeFiles $PathE
$mapF = Get-RelativeFiles $PathF
Write-Host ('  E={0} files F={1} files (excl node_modules,target,.git)' -f $mapE.Count, $mapF.Count) -ForegroundColor Gray

$copied = 0
$skipped = 0
$refC = [ref]$copied
$refS = [ref]$skipped

Write-Host 'Syncing (newer timestamp wins)...' -ForegroundColor Cyan
Sync-Between -FromRoot $PathE -ToRoot $PathF -FromMap $mapE -ToMap $mapF -Copied $refC -Skipped $refS
if (-not $script:StoppedForSpace) {
    $mapE2 = Get-RelativeFiles $PathE
    $mapF2 = Get-RelativeFiles $PathF
    Sync-Between -FromRoot $PathF -ToRoot $PathE -FromMap $mapF2 -ToMap $mapE2 -Copied $refC -Skipped $refS
}

Write-Host ''
if ($script:StoppedForSpace) {
    Write-Host ('Stopped early. copied={0}' -f $copied) -ForegroundColor Red
    exit 2
}

if (-not $DryRun) {
    $marker = Join-Path $PathF 'docs\DRIVE.md'
    $markerDir = Split-Path $marker -Parent
    if (-not (Test-Path $markerDir)) { New-Item -ItemType Directory -Force -Path $markerDir | Out-Null }
    $lines = @(
        '# aruaru workspace drive'
        ''
        '- Primary: F:\aruaru'
        '- Mirror: E:\aruaru'
        '- Sync: .\deploy\sync-drives.ps1'
    )
    Set-Content -Path $marker -Value ($lines -join "`n") -Encoding UTF8
}

Write-Host ('Done. copied={0} skipped={1}' -f $copied, $skipped) -ForegroundColor Green
Write-Host ('Primary workspace: {0}' -f $PathF) -ForegroundColor Green
exit 0
