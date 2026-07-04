param(
  [string]$SourceRoot = "F:\aruaru\aruaru-rs4",
  [string]$DestinationRoot = "F:\aruaru\aruaru-rs4-copy",
  [object]$DryRun = $true,
  [object]$IncludeGeneratedReadme = $false,
  [object]$IncludeLogs = $false
)

# aruaru manual COPY script template
# Run dry-run first. This template does not delete files.
# PowerShell rule: param(...) must be the first executable statement in the script.
# Boolean arguments are [object] intentionally, because `powershell -File ... -DryRun $true`
# can arrive as a string on Windows PowerShell. We normalize it below.

$ErrorActionPreference = "Stop"

function Convert-AruaruBool($Value, [bool]$DefaultValue, [string]$Name) {
  if ($null -eq $Value) { return $DefaultValue }
  if ($Value -is [bool]) { return [bool]$Value }
  if ($Value -is [int]) { return ([int]$Value -ne 0) }

  $text = ([string]$Value).Trim()
  if ($text.StartsWith('$')) { $text = $text.Substring(1) }
  $lower = $text.ToLowerInvariant()

  switch ($lower) {
    "true"  { return $true }
    "1"     { return $true }
    "yes"   { return $true }
    "y"     { return $true }
    "on"    { return $true }
    "false" { return $false }
    "0"     { return $false }
    "no"    { return $false }
    "n"     { return $false }
    "off"   { return $false }
    default {
      throw "${Name}: '$Value'. Use true/false, `$true/`$false, 1/0, yes/no, or on/off."
    }
  }
}

$DryRunValue = Convert-AruaruBool $DryRun $true "DryRun"
$IncludeGeneratedReadmeValue = Convert-AruaruBool $IncludeGeneratedReadme $false "IncludeGeneratedReadme"
$IncludeLogsValue = Convert-AruaruBool $IncludeLogs $false "IncludeLogs"

function Step($name) {
  Write-Host ""
  Write-Host "== $name =="
}

function Ensure-Dir($path) {
  if (!(Test-Path -LiteralPath $path)) {
    New-Item -ItemType Directory -Force -Path $path | Out-Null
  }
}

Step "copy plan"
Write-Host "Source:      $SourceRoot"
Write-Host "Destination: $DestinationRoot"
Write-Host "DryRun:      $DryRunValue"

if (!(Test-Path -LiteralPath $SourceRoot)) {
  throw "SourceRoot does not exist: $SourceRoot"
}
Ensure-Dir $DestinationRoot

$excludeDirs = @(".git", "target", "node_modules", "dist", "build", "tmp-web")
$excludeFiles = @(".env", "*.key", "*.pem", "*.pfx", "id_rsa", "id_ed25519")
if (-not $IncludeGeneratedReadmeValue) {
  $excludeFiles += @("README.rs", "README.html")
}
if (-not $IncludeLogsValue) {
  $excludeDirs += @(".aruaru")
  $excludeFiles += @("*.log")
}

Step "file scan"
$srcRootFull = (Resolve-Path -LiteralPath $SourceRoot).Path.TrimEnd('\','/')
$files = Get-ChildItem -LiteralPath $SourceRoot -Recurse -File | Where-Object {
  $relative = $_.FullName.Substring($srcRootFull.Length).TrimStart('\','/')
  $parts = $relative -split '[\\/]'
  foreach ($d in $excludeDirs) {
    if ($parts -contains $d) { return $false }
  }
  foreach ($f in $excludeFiles) {
    if ($_.Name -like $f) { return $false }
  }
  return $true
}
Write-Host ("Files to copy: " + $files.Count)

Step "copy"
foreach ($file in $files) {
  $relative = $file.FullName.Substring($srcRootFull.Length).TrimStart('\','/')
  $dest = Join-Path $DestinationRoot $relative
  $destDir = Split-Path -Parent $dest
  if ($DryRunValue) {
    Write-Host ("DRY-RUN copy " + $relative)
    continue
  }
  Ensure-Dir $destDir
  Copy-Item -LiteralPath $file.FullName -Destination $dest -Force
}

Step "done"
if ($DryRunValue) {
  Write-Host "OK: manual copy dry-run passed"
} else {
  Write-Host "OK: manual copy completed"
}
