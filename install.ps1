# Ldapex installer for Windows (PowerShell 5+).
#
# Run from an elevated PowerShell:
#   iwr -useb https://raw.githubusercontent.com/cgdev2019/ldapex/main/install.ps1 | iex
#
# What it does:
#   - queries the GitHub Releases API for the latest Ldapex tag
#   - downloads the NSIS setup.exe
#   - runs it silently (/S) — the installer handles the rest
#
# Environment overrides:
#   $env:LDAPEX_VERSION = 'v1.2.3'   # pin a release

[CmdletBinding()]
param(
  [string]$Version = $env:LDAPEX_VERSION
)

$ErrorActionPreference = 'Stop'
$Repo = 'cgdev2019/ldapex'

function Write-Step { param([string]$msg) Write-Host "→ $msg" -ForegroundColor Cyan }
function Write-Warn { param([string]$msg) Write-Host "! $msg" -ForegroundColor Yellow }
function Die {
  param([string]$msg)
  Write-Host "✗ $msg" -ForegroundColor Red
  exit 1
}

# --- resolve release tag --------------------------------------------

if ([string]::IsNullOrWhiteSpace($Version)) {
  Write-Step 'Resolving the latest Ldapex release…'
  try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
  } catch {
    Die "could not reach GitHub: $($_.Exception.Message)"
  }
} else {
  try {
    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/tags/$Version"
  } catch {
    Die "release $Version not found"
  }
}

$tag = $release.tag_name
if (-not $tag) { Die 'no release found yet — run `cargo tauri build` locally or wait for v0.1.0' }
Write-Step "Installing Ldapex $tag"

# --- pick the right asset -------------------------------------------

$asset = $release.assets |
  Where-Object { $_.name -match '_setup\.exe$|-setup\.exe$|\.msi$' } |
  Select-Object -First 1

if (-not $asset) { Die 'no setup.exe / .msi asset in the release' }

$installer = Join-Path $env:TEMP $asset.name
Write-Step "Downloading $($asset.name)"
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $installer -UseBasicParsing

# --- run it ---------------------------------------------------------

Write-Step 'Launching the installer…'
if ($asset.name -match '\.msi$') {
  Start-Process -Wait msiexec.exe -ArgumentList '/i',"""$installer""",'/qb'
} else {
  # NSIS silent flag
  Start-Process -Wait -FilePath $installer -ArgumentList '/S'
}

Remove-Item $installer -Force -ErrorAction SilentlyContinue
Write-Step 'Done. Find Ldapex in the Start menu.'
