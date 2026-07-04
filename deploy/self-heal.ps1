#Requires -Version 5.1
param([ValidateSet('all','vps','android')][string]$Fix='all')
$jbr='C:\Program Files\Android\Android Studio\jbr'
if(Test-Path $jbr){$env:JAVA_HOME=$jbr}
$sdk=Join-Path $env:LOCALAPPDATA 'Android\Sdk'
if(Test-Path $sdk){$env:ANDROID_HOME=$sdk;$env:ANDROID_SDK_ROOT=$sdk;$pt=Join-Path $sdk 'platform-tools';if(Test-Path $pt){$env:Path="$pt;$env:Path"}}
$pem=Join-Path $env:USERPROFILE '.ssh\conoha-vps.pem'
if(-not (Test-Path $pem)){$alt=Get-ChildItem (Join-Path $env:USERPROFILE '.ssh') -Filter '*.pem' -EA SilentlyContinue|Select -First 1;if($alt){Copy-Item $alt.FullName $pem -Force -EA SilentlyContinue}}
