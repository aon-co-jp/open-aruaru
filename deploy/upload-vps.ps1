#Requires -Version 5.1
<#
.SYNOPSIS
  aruaru を VPS にアップロードし vps-full-setup.sh を実行
  Windows: tar|ssh パイプは OutOfMemoryException になるため tar ファイル + scp を使用
#>
param(
    [string]$Root = '',
    [switch]$DryRun,
    [switch]$SkipSetup
)

$ErrorActionPreference = 'Stop'
$heal = Join-Path $PSScriptRoot 'self-heal.ps1'
if (Test-Path $heal) { & $heal -Fix vps | Out-Null }
if (-not $Root) {
    foreach ($p in @('F:\aruaru', 'E:\aruaru')) {
        if (Test-Path $p) { $Root = $p; break }
    }
}
if (-not $Root) { throw 'aruaru root not found' }
$Root = (Resolve-Path $Root).Path

function Read-TomlValue {
    param([string]$Path, [string]$Section, [string]$Key)
    if (-not (Test-Path $Path)) { return $null }
    $inSection = $false
    foreach ($line in Get-Content $Path) {
        $t = $line.Trim()
        if ($t -match '^\[(.+)\]$') { $inSection = ($Matches[1] -eq $Section); continue }
        if (-not $inSection) { continue }
        if ($t.StartsWith("$Key = ")) {
            return $t.Substring($Key.Length + 3).Trim().Trim('"')
        }
    }
    return $null
}

$cfgPath = Join-Path $Root 'config\vps.toml'
$localPath = Join-Path $Root 'config\vps.local.toml'
$host_ = Read-TomlValue $localPath 'vps' 'host'
if (-not $host_) { $host_ = Read-TomlValue $cfgPath 'vps' 'host' }
$user = Read-TomlValue $cfgPath 'vps' 'user'
if (-not $user) { $user = 'root' }
$remoteDir = Read-TomlValue $cfgPath 'vps' 'remote_dir'
if (-not $remoteDir) { $remoteDir = '/root/aruaru' }
$keyName = Read-TomlValue $cfgPath 'vps' 'ssh_key_name'
$keyPath = Read-TomlValue $localPath 'vps' 'ssh_key_path'
if (-not $keyPath -and $keyName) {
    $keyPath = Join-Path $env:USERPROFILE ".ssh\$keyName"
}

if (-not $host_) { throw 'config/vps.toml: missing [vps] host' }
if (-not (Test-Path $keyPath)) {
    throw "SSH key not found: $keyPath`nPlace ConoHa .pem at $env:USERPROFILE\.ssh\conoha-vps.pem"
}

function Test-CargoCheck {
    param([string]$ProjectRoot)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    Push-Location $ProjectRoot
    try {
        $out = & cargo check 2>&1
        $code = $LASTEXITCODE
        if ($code -ne 0) {
            $out | ForEach-Object { Write-Host $_ }
            throw "cargo check failed (exit $code)"
        }
        Write-Host '  cargo check OK' -ForegroundColor Green
    } finally {
        Pop-Location
        $ErrorActionPreference = $prev
    }
}

function Test-VpsSsh {
    param([string]$Key, [string]$Target)
    $keyUnix = ($Key -replace '\\', '/')
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & ssh -i $keyUnix -o BatchMode=yes -o ConnectTimeout=15 -o StrictHostKeyChecking=accept-new `
            $Target "echo aruaru-ssh-ok" *> $null
        return ($LASTEXITCODE -eq 0)
    } finally {
        $ErrorActionPreference = $prev
    }
}

function Show-SshHelp {
    param([string]$KeyPath, [string]$Target)
    Write-Host 'SSH failed: Permission denied (publickey)' -ForegroundColor Red
    Write-Host "  key: $KeyPath  target: $Target" -ForegroundColor Yellow
    Write-Host 'Fix: ConoHa panel -> register public key, or .\deploy\cursor-vps.ps1 check' -ForegroundColor Cyan
    throw 'ssh authentication failed'
}

function Invoke-Ssh {
    param([string[]]$SshArgs)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & ssh @SshArgs
        if ($LASTEXITCODE -ne 0) { throw "ssh failed (exit $LASTEXITCODE)" }
    } finally {
        $ErrorActionPreference = $prev
    }
}

function Invoke-Scp {
    param([string[]]$ScpArgs)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & scp @ScpArgs
        if ($LASTEXITCODE -ne 0) { throw "scp failed (exit $LASTEXITCODE)" }
    } finally {
        $ErrorActionPreference = $prev
    }
}

$keyPathUnix = ($keyPath -replace '\\', '/')
$target = "${user}@${host_}"
$sshBase = @('-i', $keyPathUnix, '-o', 'BatchMode=yes', '-o', 'StrictHostKeyChecking=accept-new')

Write-Host '=== aruaru VPS upload ===' -ForegroundColor Cyan
Write-Host "  local:  $Root"
Write-Host "  remote: ${target}:$remoteDir"
if ($DryRun) { Write-Host '  [DryRun]' -ForegroundColor Yellow }

$excludeDirs = @(
    'node_modules', 'target', '.git', '$RECYCLE.BIN',
    'System Volume Information', '.tmp.driveupload', 'dist'
)
$staging = Join-Path $env:TEMP "aruaru-vps-upload-$([Guid]::NewGuid().ToString('N').Substring(0,8))"
$tarFile = Join-Path $env:TEMP "aruaru-upload-$([Guid]::NewGuid().ToString('N').Substring(0,8)).tar.gz"
New-Item -ItemType Directory -Force -Path $staging | Out-Null

try {
    Write-Host 'Staging files (robocopy)...' -ForegroundColor Cyan
    if (-not $DryRun) {
        $xdArgs = $excludeDirs | ForEach-Object { '/XD', $_ }
        $prev = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        try {
            & robocopy $Root $staging /E /COPY:DAT /NFL /NDL /NJH /NJS /NP /R:1 /W:1 @xdArgs *> $null
            $rc = $LASTEXITCODE
        } finally {
            $ErrorActionPreference = $prev
        }
        if ($rc -ge 8) { throw "robocopy failed: $rc" }
    }

    if ($DryRun) {
        Write-Host "Would upload to $remoteDir and run vps-full-setup.sh"
        return
    }

    if (-not (Test-VpsSsh -Key $keyPath -Target $target)) {
        Show-SshHelp -KeyPath $keyPath -Target $target
    }

    Write-Host 'Pre-flight: cargo check (local)...' -ForegroundColor Cyan
    Test-CargoCheck -ProjectRoot $Root

    Write-Host 'Creating remote directory...' -ForegroundColor Cyan
    Invoke-Ssh ($sshBase + @($target, "mkdir -p '$remoteDir'"))

    # tar ファイル経由（パイプは Windows で OutOfMemoryException になる）
    Write-Host 'Creating archive (tar.gz)...' -ForegroundColor Cyan
    Push-Location $staging
    try {
        $prev = $ErrorActionPreference
        $ErrorActionPreference = 'Continue'
        try {
            & tar.exe --format=gnu -czf $tarFile . *> $null
            if ($LASTEXITCODE -ne 0) { throw 'tar create failed' }
        } finally {
            $ErrorActionPreference = $prev
        }
    } finally {
        Pop-Location
    }
    $sizeMb = [math]::Round((Get-Item $tarFile).Length / 1MB, 1)
    Write-Host "  archive: $tarFile ($sizeMb MB)" -ForegroundColor DarkGray

    $remoteTar = "$remoteDir/.aruaru-upload.tar.gz"
    Write-Host 'Uploading (scp)...' -ForegroundColor Cyan
    Invoke-Scp ($sshBase + @($tarFile, "${target}:$remoteTar"))

    Write-Host 'Extracting on VPS...' -ForegroundColor Cyan
    Invoke-Ssh ($sshBase + @($target, "cd '$remoteDir' && tar -xzf .aruaru-upload.tar.gz 2>&1 | grep -v 'SCHILY.fflags' | grep -v '^$' || true; rm -f .aruaru-upload.tar.gz"))

    if (-not $SkipSetup) {
        Write-Host 'Running vps-full-setup.sh on VPS...' -ForegroundColor Cyan
        Invoke-Ssh ($sshBase + @($target, "cd '$remoteDir' && chmod +x deploy/vps-full-setup.sh deploy/*.sh 2>/dev/null; ./deploy/vps-full-setup.sh"))
        Write-Host 'Running fix-start + nginx + firewall...' -ForegroundColor Cyan
        Invoke-Ssh ($sshBase + @($target, "cd '$remoteDir' && ./deploy/fix-start.sh && ./deploy/nginx-aruaru.sh '$remoteDir' && ./deploy/firewall-aruaru.sh"))
    }

    Write-Host ''
    Write-Host "Done. Open: http://${host_}/#/register  (port 80)" -ForegroundColor Green
    Write-Host "Also:    http://${host_}:3000/#/register"
    Write-Host "Domain:  http://${host_}/#/settings/domain"
} finally {
    if (Test-Path $staging) { Remove-Item -Recurse -Force $staging -ErrorAction SilentlyContinue }
    if (Test-Path $tarFile) { Remove-Item -Force $tarFile -ErrorAction SilentlyContinue }
}
