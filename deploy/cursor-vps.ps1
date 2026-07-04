#Requires -Version 5.1
<#
.SYNOPSIS
  Cursor / エージェント向け ConoHa VPS 操作

.DESCRIPTION
  PowerShell 専用。SSH 鍵パス（C: コロン問題）と config を自動処理し、
  接続・状態確認結果を JSON でも出力する。

.EXAMPLE
  .\deploy\cursor-vps.ps1 check      # 接続 + aruaru 状態（エージェント用）
  .\deploy\cursor-vps.ps1 status     # 同上（人間向け表示）
  .\deploy\cursor-vps.ps1 connect    # 対話 SSH ログイン
  .\deploy\cursor-vps.ps1 fix-ssh    # ~/.ssh/config 修復
#>
param(
    [Parameter(Position = 0)]
    [ValidateSet("check", "status", "connect", "fix-ssh")]
    [string]$Action = "status"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-ProjectRoot {
    Split-Path $PSScriptRoot -Parent
}

function Read-TomlValue {
    param([string]$Path, [string]$Section, [string]$Key)
    if (-not (Test-Path $Path)) { return $null }
    $inSection = $false
    foreach ($line in Get-Content $Path) {
        $t = $line.Trim()
        if ($t -match '^\[(.+)\]$') {
            $inSection = ($Matches[1] -eq $Section)
            continue
        }
        if (-not $inSection) { continue }
        if ($t.StartsWith("$Key = ")) {
            $val = $t.Substring($Key.Length + 3).Trim()
            if ($val.StartsWith('"') -and $val.EndsWith('"')) {
                return $val.Substring(1, $val.Length - 2)
            }
            return $val
        }
    }
    return $null
}

function Get-VpsConfig {
    $root = Get-ProjectRoot
    $cfgPath = Join-Path $root "config\vps.toml"
    $localPath = Join-Path $root "config\vps.local.toml"

    $host_ = Read-TomlValue $cfgPath "vps" "host"
    $user = Read-TomlValue $cfgPath "vps" "user"
    $alias = Read-TomlValue $cfgPath "vps" "ssh_alias"
    $port = [int](Read-TomlValue $cfgPath "vps" "app_port")
    $remoteDir = Read-TomlValue $cfgPath "vps" "remote_dir"
    $keyName = Read-TomlValue $cfgPath "vps" "ssh_key_name"
    $keyPath = $null

    if (Test-Path $localPath) {
        $oh = Read-TomlValue $localPath "vps" "host"
        $ou = Read-TomlValue $localPath "vps" "user"
        $ok = Read-TomlValue $localPath "vps" "ssh_key"
        if ($oh) { $host_ = $oh }
        if ($ou) { $user = $ou }
        if ($ok) { $keyPath = $ok }
    }

    if (-not $keyPath) {
        $name = if ($keyName) { $keyName } else { "conoha-vps.pem" }
        $keyPath = Join-Path $env:USERPROFILE (Join-Path ".ssh" $name)
    }

    [PSCustomObject]@{
        Host       = $host_
        User       = $user
        Alias      = $alias
        AppPort    = if ($port) { $port } else { 3000 }
        RemoteDir  = if ($remoteDir) { $remoteDir } else { "/root/aruaru" }
        KeyPath    = $keyPath
        ConfigPath = Join-Path $env:USERPROFILE ".ssh\config"
    }
}

function Repair-SshConfig {
    param($Cfg)

    $sshDir = Split-Path $Cfg.KeyPath -Parent
    New-Item -ItemType Directory -Force -Path $sshDir | Out-Null

    $keyForConfig = ($Cfg.KeyPath -replace '\\', '/')

    $content = @"
Host $($Cfg.Alias)
    HostName $($Cfg.Host)
    User $($Cfg.User)
    IdentityFile $keyForConfig
    IdentitiesOnly yes

Host $($Cfg.Host)
    HostName $($Cfg.Host)
    User $($Cfg.User)
    IdentityFile $keyForConfig
    IdentitiesOnly yes
"@

    try {
        Set-Content -Path $Cfg.ConfigPath -Value $content -Encoding UTF8
        icacls $Cfg.ConfigPath /inheritance:r | Out-Null
        icacls $Cfg.ConfigPath /grant:r "${env:USERNAME}:(R)" | Out-Null
        if (Test-Path $Cfg.KeyPath) {
            icacls $Cfg.KeyPath /inheritance:r | Out-Null
            icacls $Cfg.KeyPath /grant:r "${env:USERNAME}:(R)" | Out-Null
        }
        Write-Host "OK: $($Cfg.ConfigPath) updated"
    }
    catch {
        Write-Warning "config 書き込みスキップ（権限）: $($Cfg.ConfigPath)"
    }
}

function Invoke-VpsSsh {
    param(
        $Cfg,
        [string]$RemoteCmd,
        [switch]$Interactive
    )

    if (-not (Test-Path $Cfg.KeyPath)) {
        throw "SSH key not found: $($Cfg.KeyPath)"
    }

    $keyPath = ($Cfg.KeyPath -replace '\\', '/')
    $target = "$($Cfg.User)@$($Cfg.Host)"

    try {
        if ($Interactive) {
            & ssh -i "$keyPath" $target
            return $null
        }
        & ssh -i "$keyPath" -o BatchMode=yes -o ConnectTimeout=15 -o StrictHostKeyChecking=accept-new `
            $target $RemoteCmd 2>&1
    }
    finally {
    }
}

function Get-RemoteStatusJson {
    param($Cfg)

    $probe = Invoke-VpsSsh -Cfg $Cfg -RemoteCmd "echo ok"
    if ($LASTEXITCODE -ne 0 -or ($probe | Out-String).Trim() -ne "ok") {
        return [PSCustomObject]@{
            ssh_ok     = $false
            error      = ($probe | Out-String).Trim()
            host       = $Cfg.Host
            checked_at = (Get-Date).ToString("o")
        }
    }

    $hostname = (Invoke-VpsSsh -Cfg $Cfg -RemoteCmd "hostname -s 2>/dev/null || hostname").ToString().Trim()
    $port3000 = (Invoke-VpsSsh -Cfg $Cfg -RemoteCmd "ss -tlnp 2>/dev/null | grep -q ':3000' && echo true || echo false").ToString().Trim() -eq "true"
    $healthRaw = (Invoke-VpsSsh -Cfg $Cfg -RemoteCmd "curl -sf -m 3 http://127.0.0.1:3000/api/health || true").ToString().Trim()
    $systemd = (Invoke-VpsSsh -Cfg $Cfg -RemoteCmd "systemctl is-active aruaru 2>/dev/null || echo unknown").ToString().Trim()
    $ip = (Invoke-VpsSsh -Cfg $Cfg -RemoteCmd "curl -s --max-time 3 ifconfig.me 2>/dev/null || echo UNKNOWN").ToString().Trim()

    $health = $null
    if ($healthRaw -and $healthRaw.StartsWith("{")) {
        try { $health = $healthRaw | ConvertFrom-Json } catch { }
    }

    [PSCustomObject]@{
        ssh_ok      = $true
        hostname    = $hostname
        port_3000   = $port3000
        health      = $health
        systemd     = $systemd
        public_url  = "http://${ip}:3000"
        host        = $Cfg.Host
        checked_at  = (Get-Date).ToString("o")
    }
}

function Show-HumanStatus {
    param($Result, $Cfg)

    Write-Host ""
    Write-Host "=== ConoHa VPS ($($Cfg.Host)) ===" -ForegroundColor Cyan
    if (-not $Result.ssh_ok) {
        Write-Host "SSH  .......... NG" -ForegroundColor Red
        Write-Host $Result.error
        Write-Host ""
        Write-Host "修復: .\deploy\cursor-vps.ps1 fix-ssh" -ForegroundColor Yellow
        return
    }

    Write-Host "SSH  .......... OK" -ForegroundColor Green
    Write-Host "Hostname ...... $($Result.hostname)"
    Write-Host "Port 3000 ..... $(if ($Result.port_3000) { 'LISTEN' } else { 'closed' })"
    Write-Host "systemd ....... $($Result.systemd)"
    if ($Result.health) {
        Write-Host "aruaru ........ OK (v$($Result.health.version))" -ForegroundColor Green
        Write-Host "DB backend .... $($Result.health.database.backend)"
        Write-Host "public_url .... $($Result.public_url)"
    }
    else {
        Write-Host "aruaru ........ not responding" -ForegroundColor Yellow
        Write-Host "修復 (VPS): cd $($Cfg.RemoteDir) && ./deploy/fix-start.sh"
    }
    Write-Host ""
}

# --- main ---
$cfg = Get-VpsConfig

switch ($Action) {
    "fix-ssh" {
        Repair-SshConfig $cfg
    }
    "connect" {
        if (-not (Test-Path $cfg.ConfigPath)) { Repair-SshConfig $cfg }
        Invoke-VpsSsh -Cfg $cfg -Interactive
    }
    "check" {
        if (-not (Test-Path $cfg.KeyPath)) {
            @{ ok = $false; error = "key_missing"; path = $cfg.KeyPath } | ConvertTo-Json -Compress
            exit 1
        }
        $result = Get-RemoteStatusJson $cfg
        $result | ConvertTo-Json -Depth 6 -Compress
        if (-not $result.ssh_ok) { exit 2 }
        if (-not $result.port_3000) { exit 3 }
        if (-not $result.health) { exit 4 }
    }
    "status" {
        if (-not (Test-Path $cfg.KeyPath)) {
            Write-Host "鍵がありません: $($cfg.KeyPath)" -ForegroundColor Red
            exit 1
        }
        $result = Get-RemoteStatusJson $cfg
        Show-HumanStatus $result $cfg
        if (-not $result.ssh_ok) { exit 2 }
    }
}
