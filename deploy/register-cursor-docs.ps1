#Requires -Version 5.1
param(
    [switch]$SkipMcpTest
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
Set-Location $Root

Write-Host "=== aruaru Cursor Docs setup ===" -ForegroundColor Cyan

$docsDir = Join-Path $Root "docs\@docs"
$mcpFile = Join-Path $Root ".cursor\mcp.json"
$registry = Join-Path $Root ".cursor\docs-registry.json"

if (-not (Test-Path $docsDir)) {
    Write-Error "Missing docs\@docs folder"
}

$entries = @(
    @{ name = "Tauri"; url = "https://tauri.app/develop/" },
    @{ name = "Rust"; url = "https://doc.rust-lang.org/book/" },
    @{ name = "Poem"; url = "https://docs.rs/poem/latest/poem/" },
    @{ name = "WunderGraph"; url = "https://docs.wundergraph.com/" },
    @{ name = "WunderGraph-Cosmo"; url = "https://cosmo-docs.wundergraph.com/" },
    @{ name = "Bootstrap"; url = "https://getbootstrap.com/docs/" },
    @{ name = "TypeScript"; url = "https://www.typescriptlang.org/docs/" },
    @{ name = "PostgreSQL"; url = "https://www.postgresql.org/docs/" },
    @{ name = "SQLx"; url = "https://docs.rs/sqlx/latest/sqlx/" },
    @{ name = "VersionLess-API"; url = "https://wundergraph.com/blog/versionless_apis_making_apis_backwards_compatible_forever_to_enable_businesses_to_collaborate" }
)

$registryObj = @{
    version = 1
    note = "Local Docs substitute for aruaru"
    entries = $entries
}
$registryObj | ConvertTo-Json -Depth 5 | Set-Content -Path $registry -Encoding UTF8
Write-Host "OK: .cursor\docs-registry.json" -ForegroundColor Green

$mdCount = (Get-ChildItem $docsDir -Filter "*.md").Count
Write-Host "OK: docs\@docs\ ($mdCount files)" -ForegroundColor Green

if (Test-Path $mcpFile) {
    Write-Host "OK: .cursor\mcp.json" -ForegroundColor Green
} else {
    Write-Warning "Missing .cursor\mcp.json"
}

if (-not $SkipMcpTest) {
    Write-Host "=== MCP fetch test ===" -ForegroundColor Cyan
    if (-not (Get-Command npm -ErrorAction SilentlyContinue)) {
        Write-Warning "npm not found. Install Node.js 18+"
    } else {
        $ver = cmd /c "npm view @kazuph/mcp-fetch version 2>nul"
        if ($LASTEXITCODE -eq 0 -and $ver) {
            Write-Host ("OK: @kazuph/mcp-fetch v{0} (npm registry)" -f $ver.Trim()) -ForegroundColor Green
            Write-Host "  (npm warn about whatwg/devdir is harmless)" -ForegroundColor DarkGray
        } else {
            Write-Warning "fetch MCP package not found. Check network / npm."
        }
    }
}

Write-Host ""
Write-Host "Next (about 2 min):" -ForegroundColor Yellow
Write-Host "1. Ctrl+Shift+P -> Developer: Reload Window"
Write-Host "2. Settings -> Tools and MCP -> fetch -> ON (green)"
Write-Host "3. Chat: use @docs/@docs/Poem.md in Cursor"
Write-Host ""
Write-Host "Note: MCP fetch test does NOT run npx (server would hang)." -ForegroundColor DarkGray
Write-Host "      npm warn about whatwg/devdir is harmless." -ForegroundColor DarkGray
