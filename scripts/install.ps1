#Requires -Version 5.1
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

$Repo = "edu526/vori"
$Api  = "https://api.github.com/repos/$Repo/releases/latest"

function Write-Info  { param($msg) Write-Host "==> $msg" -ForegroundColor Blue }
function Write-Ok    { param($msg) Write-Host "  v $msg" -ForegroundColor Green }
function Write-Fail  { param($msg) Write-Error $msg }

# ── fetch latest release ──────────────────────────────────────────────────────

Write-Info "Fetching latest release from $Repo..."

try {
    $Release = Invoke-RestMethod -Uri $Api -UseBasicParsing
} catch {
    Write-Fail "Could not fetch release info: $_"
}

$Version = $Release.tag_name
Write-Info "Latest version: $Version"

# Prefer .msi, fall back to NSIS .exe
$Asset = $Release.assets | Where-Object { $_.name -like "*.msi" } | Select-Object -First 1
if (-not $Asset) {
    $Asset = $Release.assets | Where-Object { $_.name -like "*-setup.exe" } | Select-Object -First 1
}
if (-not $Asset) {
    Write-Fail "No Windows installer found in the latest release."
}

$Url  = $Asset.browser_download_url
$Ext  = if ($Asset.name -like "*.msi") { ".msi" } else { ".exe" }
$Tmp  = Join-Path $env:TEMP "vori-install$Ext"

# ── download ──────────────────────────────────────────────────────────────────

Write-Info "Downloading $Url..."
Invoke-WebRequest -Uri $Url -OutFile $Tmp -UseBasicParsing

# ── install ───────────────────────────────────────────────────────────────────

Write-Info "Installing Vori $Version..."

if ($Ext -eq ".msi") {
    Start-Process msiexec.exe -ArgumentList "/i `"$Tmp`" /quiet /norestart" -Wait
} else {
    Start-Process $Tmp -ArgumentList "/S" -Wait
}

Remove-Item $Tmp -Force

Write-Ok "Vori $Version installed successfully."
