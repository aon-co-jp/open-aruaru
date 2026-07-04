# aruaru API key import helper for Windows PowerShell.
# This script does not print the API key. Run from the project root or scripts/manual.

param(
  [ValidateSet("openai", "anthropic", "gemini", "deepseek", "custom")]
  [string]$Provider = "anthropic",

  [ValidateSet("process", "user")]
  [string]$Scope = "process",

  [string]$CustomEnvName = ""
)

$ErrorActionPreference = "Stop"

function Get-EnvName([string]$Provider, [string]$CustomEnvName) {
  if ($Provider -eq "openai") { return "OPENAI_API_KEY" }
  if ($Provider -eq "anthropic") { return "ANTHROPIC_API_KEY" }
  if ($Provider -eq "gemini") { return "GEMINI_API_KEY" }
  if ($Provider -eq "deepseek") { return "DEEPSEEK_API_KEY" }
  if ([string]::IsNullOrWhiteSpace($CustomEnvName)) { return "ARUARU_CUSTOM_AI_API_KEY" }
  return $CustomEnvName
}

function Convert-SecureStringToPlain([System.Security.SecureString]$Secure) {
  $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($Secure)
  try {
    return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
  } finally {
    [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
  }
}

function Mask-Secret([string]$Secret) {
  if ([string]::IsNullOrWhiteSpace($Secret)) { return "<empty>" }
  if ($Secret.Length -le 8) { return "********" }
  return $Secret.Substring(0,4) + "..." + $Secret.Substring($Secret.Length - 4)
}

$envName = Get-EnvName $Provider $CustomEnvName
Write-Host "== aruaru API key import helper =="
Write-Host "Provider: $Provider"
Write-Host "Target environment variable: $envName"
Write-Host "Scope: $Scope"
Write-Host ""
Write-Host "Do not paste someone else's key. Use only a key you own or are authorized to use."
$keySecure = Read-Host "Paste API key" -AsSecureString
$keyPlain = Convert-SecureStringToPlain $keySecure

if ([string]::IsNullOrWhiteSpace($keyPlain)) {
  throw "API key is empty."
}

if ($Scope -eq "process") {
  Set-Item -Path "Env:$envName" -Value $keyPlain
  Write-Host "Stored for this PowerShell process only: $envName=$(Mask-Secret $keyPlain)"
  Write-Host "Run aruaru bug check from this same PowerShell window."
} else {
  [Environment]::SetEnvironmentVariable($envName, $keyPlain, "User")
  Write-Host "Stored as user environment variable: $envName=$(Mask-Secret $keyPlain)"
  Write-Host "Restart PowerShell or aruaru-desktop before using it."
}

$keyPlain = $null
Write-Host "OK: API key import helper completed without printing the full key."
