#Requires -Version 5.1
# aruaru-android APK/AAB installer build (Windows auto-fix)
param(
    [switch]$SkipNpm,
    [switch]$SkipIcon,
    [ValidateSet('apk', 'aab', 'all')]
    [string]$Target = 'apk'
)

$ErrorActionPreference = 'Stop'
$AppRoot = Split-Path $PSScriptRoot -Parent
$aruaruRoot = (Resolve-Path (Join-Path $AppRoot '..\..')).Path
$heal = Join-Path $aruaruRoot 'deploy\self-heal.ps1'
if (Test-Path $heal) { & $heal -Fix android | Out-Null }
Set-Location $AppRoot
$env:CARGO_TARGET_DIR = Join-Path $AppRoot 'src-tauri\target'

Write-Host '=== aruaru-android installer build ===' -ForegroundColor Cyan
Write-Host 'Output: F:/aruaru/apps/aruaru-smartphone/dist/' -ForegroundColor DarkGray

# --- auto-detect Android SDK / Java ---
$jbr = 'C:\Program Files\Android\Android Studio\jbr'
if ((-not $env:JAVA_HOME -or -not (Test-Path $env:JAVA_HOME)) -and (Test-Path $jbr)) {
    $env:JAVA_HOME = $jbr
    Write-Host "AUTO: JAVA_HOME=$jbr" -ForegroundColor DarkGray
}
$sdk = Join-Path $env:LOCALAPPDATA 'Android\Sdk'
if ((-not $env:ANDROID_HOME -or -not (Test-Path $env:ANDROID_HOME)) -and (Test-Path $sdk)) {
    $env:ANDROID_HOME = $sdk
    $env:ANDROID_SDK_ROOT = $sdk
    Write-Host "AUTO: ANDROID_HOME=$sdk" -ForegroundColor DarkGray
}
$pt = Join-Path $sdk 'platform-tools'
if ((Test-Path $pt) -and ($env:Path -notlike "*$pt*")) {
    $env:Path = "$pt;$env:Path"
}

# --- symlink privilege (Windows 8.1 / no Dev Mode) ---
function Test-SymlinkOk {
    $d = Join-Path $env:TEMP "aruaru-sym-$([Guid]::NewGuid().ToString('N').Substring(0,6))"
    New-Item -ItemType Directory -Force -Path $d | Out-Null
    $src = Join-Path $d 'a.txt'
    'x' | Set-Content $src
    $lnk = Join-Path $d 'link.txt'
    try {
        New-Item -ItemType SymbolicLink -Path $lnk -Target $src -ErrorAction Stop | Out-Null
        Remove-Item $d -Recurse -Force
        return $true
    } catch {
        Remove-Item $d -Recurse -Force -ErrorAction SilentlyContinue
        return $false
    }
}
if (-not (Test-SymlinkOk)) {
    Write-Host 'WARN: symlink privilege missing - trying Developer Mode registry...' -ForegroundColor Yellow
    $regPath = 'HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock'
    try {
        if (-not (Test-Path $regPath)) { New-Item -Path $regPath -Force | Out-Null }
        Set-ItemProperty -Path $regPath -Name AllowDevelopmentWithoutDevLicense -Value 1 -Type DWord -Force -ErrorAction Stop
        Write-Host 'AUTO: Developer Mode registry enabled (re-login may be needed)' -ForegroundColor DarkGray
    } catch {
        Write-Host '  Enable: Settings -> Developer Mode, or run as Admin' -ForegroundColor DarkGray
        Write-Host '  https://docs.microsoft.com/en-us/windows/security/threat-protection/security-policy-settings/create-symbolic-links' -ForegroundColor DarkGray
    }
    # Cargo: copy instead of hardlink where possible
    $env:CARGO_TARGET_DIR = Join-Path $AppRoot 'src-tauri\target'
}

# --- Android project ---
$genAndroid = Join-Path $AppRoot 'src-tauri\gen\android'
if (-not (Test-Path $genAndroid)) {
    Write-Host 'tauri android init...' -ForegroundColor Yellow
    if (-not (Test-Path node_modules)) { npm install }
    npx tauri android init
    if ($LASTEXITCODE -ne 0) { throw 'tauri android init failed' }
}

# --- icons ---
$iconIco = Join-Path $AppRoot 'src-tauri\icons\icon.ico'
if (-not $SkipIcon -and -not (Test-Path $iconIco)) {
    Write-Host 'Generating icons...' -ForegroundColor Yellow
    & (Join-Path $AppRoot 'scripts\generate-app-icon.ps1')
    npx tauri icon app-icon.png
    if ($LASTEXITCODE -ne 0) { throw 'tauri icon failed' }
}

# --- cleartext HTTP for VPS ---
$gradle = Join-Path $genAndroid 'app\build.gradle.kts'
if (Test-Path $gradle) {
    $g = Get-Content $gradle -Raw
    if ($g -match 'manifestPlaceholders\["usesCleartextTraffic"\] = "false"') {
        $g = $g.Replace(
            'manifestPlaceholders["usesCleartextTraffic"] = "false"',
            'manifestPlaceholders["usesCleartextTraffic"] = "true"'
        )
        Set-Content -Path $gradle -Value $g -NoNewline
    }
}
$manifest = Join-Path $genAndroid 'app\src\main\AndroidManifest.xml'
if (Test-Path $manifest) {
    $m = Get-Content $manifest -Raw
    if ($m -notmatch 'RECORD_AUDIO') {
        $m = $m.Replace(
            '<uses-permission android:name="android.permission.INTERNET" />',
            "<uses-permission android:name=`"android.permission.INTERNET`" />`n    <uses-permission android:name=`"android.permission.RECORD_AUDIO`" />"
        )
        Set-Content -Path $manifest -Value $m -NoNewline
    }
}

# --- build ---
$logFile = Join-Path $AppRoot 'dist\android-build.log'
New-Item -ItemType Directory -Force -Path (Split-Path $logFile) | Out-Null
if (-not $SkipNpm) {
    if (-not (Test-Path node_modules)) { npm install }
    Write-Host 'npm run android:build...' -ForegroundColor Cyan
    npm run android:build 2>&1 | Tee-Object -FilePath $logFile
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build log: F:/aruaru/apps/aruaru-smartphone/dist/android-build.log" -ForegroundColor Yellow
        throw 'android build failed'
    }
}

# --- find APK/AAB (multiple Tauri/Gradle output paths) ---
$dist = Join-Path $AppRoot 'dist'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
$version = '0.1.0'
$searchRoots = @(
    (Join-Path $env:CARGO_TARGET_DIR 'android'),
    (Join-Path $genAndroid 'app\build\outputs'),
    $AppRoot
)

function Find-BuildArtifacts {
    param([string]$Filter)
    foreach ($root in $searchRoots) {
        if (-not (Test-Path $root)) { continue }
        Get-ChildItem -Path $root -Recurse -Filter $Filter -ErrorAction SilentlyContinue |
            Where-Object { $_.Name -notmatch 'unsigned|debug' }
    }
}

$apks = @(Find-BuildArtifacts '*universal*release*.apk')
if (-not $apks.Count) { $apks = @(Find-BuildArtifacts '*release*.apk') }
$aabs = @(Find-BuildArtifacts '*release*.aab')

$copied = $false
if ($Target -in @('apk', 'all') -and $apks.Count) {
    $apk = $apks | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $dest = Join-Path $dist "aruaru-android_${version}_universal.apk"
    Copy-Item $apk.FullName $dest -Force
    Write-Host "OK APK: $dest" -ForegroundColor Green
    $copied = $true
}
if ($Target -in @('aab', 'all') -and $aabs.Count) {
    $aab = $aabs | Sort-Object LastWriteTime -Descending | Select-Object -First 1
    $dest = Join-Path $dist "aruaru-android_${version}.aab"
    Copy-Item $aab.FullName $dest -Force
    Write-Host "OK AAB: $dest" -ForegroundColor Green
    $copied = $true
}

if (-not $copied) {
    Write-Warning 'APK/AAB not found'
    Write-Host "Check log: F:/aruaru/apps/aruaru-smartphone/dist/android-build.log" -ForegroundColor Yellow
    Write-Host 'Ensure: Android Studio + SDK + NDK installed' -ForegroundColor DarkGray
    exit 1
}

Copy-Item (Join-Path $PSScriptRoot 'install-android.ps1') (Join-Path $dist 'install-android.ps1') -Force -ErrorAction SilentlyContinue
@'
# aruaru-android install
APK: aruaru-android_0.1.0_universal.apk
adb: .\install-android.ps1
Docs: F:/aruaru/apps/aruaru-smartphone/README.md
'@ | Set-Content (Join-Path $dist 'INSTALL.md') -Encoding UTF8

Write-Host 'Done: F:/aruaru/apps/aruaru-smartphone/dist/' -ForegroundColor Green
