#Requires -Version 5.1
# Verify docs registry, local @docs, rules, MCP config, and URL reachability.
param(
    [string]$Root = 'F:\aruaru',
    [switch]$SkipUrlTest
)

$ErrorActionPreference = 'Continue'
if (-not (Test-Path $Root)) { $Root = 'E:\aruaru' }

Write-Host '=== aruaru Cursor Docs verification ===' -ForegroundColor Cyan
Write-Host "Root: $Root`n"

$registryPath = Join-Path $Root '.cursor\docs-registry.json'
$docsDir = Join-Path $Root 'docs\@docs'
$rulesDir = Join-Path $Root '.cursor\rules'
$projMcp = Join-Path $Root '.cursor\mcp.json'
$globalMcp = Join-Path $env:USERPROFILE '.cursor\mcp.json'

# --- files ---
$checks = @()
function Add-Check([string]$Name, [bool]$Ok, [string]$Detail) {
    $script:checks += [pscustomobject]@{ Item = $Name; OK = $Ok; Detail = $Detail }
}

Add-Check 'docs-registry.json' (Test-Path $registryPath) $registryPath
Add-Check 'docs/@docs folder' (Test-Path $docsDir) $docsDir
Add-Check 'project .cursor/mcp.json' (Test-Path $projMcp) $projMcp
Add-Check 'global ~/.cursor/mcp.json' (Test-Path $globalMcp) $globalMcp
Add-Check 'rule aruaru-stack.mdc' (Test-Path (Join-Path $rulesDir 'aruaru-stack.mdc')) ''
Add-Check 'rule aruaru-docs-fallback.mdc' (Test-Path (Join-Path $rulesDir 'aruaru-docs-fallback.mdc')) ''
Add-Check 'rule aruaru-poem-always.mdc' (Test-Path (Join-Path $rulesDir 'aruaru-poem-always.mdc')) ''
Add-Check 'rule aruaru-tauri.mdc' (Test-Path (Join-Path $rulesDir 'aruaru-tauri.mdc')) ''

$mdFiles = @(Get-ChildItem $docsDir -Filter '*.md' -ErrorAction SilentlyContinue | Where-Object { $_.Name -ne 'README.md' })
Add-Check ('local docs count (excl README)') ($mdFiles.Count -ge 9) "$($mdFiles.Count) files"

# --- registry vs @docs ---
$registry = Get-Content $registryPath -Raw -Encoding UTF8 | ConvertFrom-Json
$urlRows = @()
$allMatch = $true
foreach ($e in $registry.entries) {
    $mdName = switch ($e.name) {
        'VersionLess-API' { 'VersionLess-API.md' }
        default { "$($e.name).md" }
    }
    $mdPath = Join-Path $docsDir $mdName
    $exists = Test-Path $mdPath
    $inMd = $false
    if ($exists) {
        $body = Get-Content $mdPath -Raw -Encoding UTF8
        $inMd = $body -like "*$($e.url)*"
    }
    if (-not $exists -or -not $inMd) { $allMatch = $false }
    $urlRows += [pscustomobject]@{
        Tech = $e.name
        RegistryURL = $e.url
        LocalMD = if ($exists) { $mdName } else { 'MISSING' }
        URLinMD = $inMd
    }
}
Add-Check 'registry <-> @docs URL match' $allMatch ''

# --- npm / node ---
$nodeOk = [bool](Get-Command node -ErrorAction SilentlyContinue)
$npmOk = [bool](Get-Command npm -ErrorAction SilentlyContinue)
Add-Check 'Node.js installed' $nodeOk ''
Add-Check 'npm installed' $npmOk ''
$mcpVer = $null
if ($npmOk) {
    $mcpVer = cmd /c "npm view @kazuph/mcp-fetch version 2>nul"
    Add-Check 'MCP fetch package (npm)' ($LASTEXITCODE -eq 0 -and $mcpVer) "v$($mcpVer.Trim())"
}

# --- cursor CLI ---
$cursor = 'C:\Program Files\cursor\resources\app\bin\cursor.cmd'
Add-Check 'Cursor CLI' (Test-Path $cursor) $cursor

# --- URL reachability ---
if (-not $SkipUrlTest) {
    Write-Host 'Testing official URLs (HTTP)...' -ForegroundColor Cyan
    foreach ($row in $urlRows) {
        $ok = $false
        $code = 'n/a'
        try {
            $r = Invoke-WebRequest -Uri $row.RegistryURL -Method Head -TimeoutSec 15 -UseBasicParsing -MaximumRedirection 5
            $code = [int]$r.StatusCode
            $ok = ($code -ge 200 -and $code -lt 400)
        } catch {
            try {
                $r2 = Invoke-WebRequest -Uri $row.RegistryURL -Method Get -TimeoutSec 15 -UseBasicParsing -MaximumRedirection 5
                $code = [int]$r2.StatusCode
                $ok = ($code -ge 200 -and $code -lt 400)
            } catch {
                $code = $_.Exception.Message
            }
        }
        $row | Add-Member -NotePropertyName HTTP -NotePropertyValue $(if ($ok) { "OK $code" } else { "FAIL $code" }) -Force
    }
}

# --- print ---
Write-Host ''
Write-Host '--- File / config ---' -ForegroundColor Yellow
$checks | Format-Table Item, @{L='OK';E={ if ($_.OK) { 'YES' } else { 'NO' } }}, Detail -AutoSize

Write-Host '--- Registry vs local @docs ---' -ForegroundColor Yellow
if ($SkipUrlTest) {
    $urlRows | Format-Table Tech, LocalMD, URLinMD, RegistryURL -AutoSize
} else {
    $urlRows | Format-Table Tech, LocalMD, URLinMD, HTTP, RegistryURL -AutoSize
}

Write-Host '--- Auto-reference layers ---' -ForegroundColor Yellow
Write-Host @'
Layer 1: .cursor/rules (alwaysApply)     -> aruaru-stack, aruaru-docs-fallback, aruaru-poem-always
Layer 2: .cursor/rules (globs)           -> aruaru-rust-poem (src/**), aruaru-tauri (apps/**/src-tauri/**)
Layer 3: docs/@docs/*.md                 -> manual @docs/@docs/<Name>.md in chat
Layer 4: MCP fetch                       -> live URL fetch (needs Tools & MCP ON)
Layer 5: Cursor @Docs manual             -> NOT verifiable from disk; optional
'@

Write-Host '--- What you should verify in Cursor UI ---' -ForegroundColor Yellow
Write-Host '1. Ctrl+Shift+J -> Tools and MCP -> fetch = green/enabled'
Write-Host '2. Optional: Settings -> Docs -> Tauri/Rust/Poem listed (manual @Docs)'
Write-Host '3. Ctrl+Shift+P -> Developer: Reload Window if fetch was just added'
Write-Host ''

$fail = @($checks | Where-Object { -not $_.OK }).Count
if (-not $SkipUrlTest) {
    $fail += @($urlRows | Where-Object { $_.HTTP -notlike 'OK*' }).Count
}
if ($fail -eq 0) {
    Write-Host 'RESULT: disk/config/URLs look OK. Confirm MCP fetch toggle in Cursor UI.' -ForegroundColor Green
    exit 0
}
Write-Host "RESULT: $fail issue(s) found. See table above." -ForegroundColor Red
exit 1
