<#
.SYNOPSIS
  One-command installer for IBM Bob v2.1.0+ tuning (Windows PowerShell).
.DESCRIPTION
  Builds the native Rust `bob-tuner` CLI when Cargo is available; otherwise falls back
  automatically to the zero-dependency Node.js engine (`scripts/bob-sync.mjs`) with full parity.
#>
[CmdletBinding()]
param(
  [string[]]$Workspace = @(),
  [switch]$AllModes,
  [string]$Model = "",
  [switch]$NoTurbo,
  [switch]$DryRun,
  [switch]$SkipRust
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host " IBM BOB v2.1.0+ TUNING INSTALLER (PE-AGENTS-IBM)" -ForegroundColor Cyan
Write-Host " Repo: $RepoRoot" -ForegroundColor DarkGray
Write-Host "================================================================================" -ForegroundColor Cyan

$argsList = @("sync")
foreach ($ws in $Workspace) {
  if ($ws) { $argsList += @("--workspace", $ws) }
}
if ($AllModes) { $argsList += "--all-modes" }
if ($Model -ne "") { $argsList += @("--model", $Model) }
if ($NoTurbo) { $argsList += "--no-turbo" }
if ($DryRun) { $argsList += "--dry-run" }

$cargo = if ($SkipRust) { $null } else { Get-Command cargo -ErrorAction SilentlyContinue }
$usedRust = $false

if ($cargo) {
  Write-Host "[1/2] Rust/Cargo detected -> compiling native bob-tuner (release)..." -ForegroundColor Green
  Push-Location $RepoRoot
  try {
    & cargo build --release
    if ($LASTEXITCODE -eq 0) {
      $bin = Join-Path $RepoRoot "target\release\bob-tuner.exe"
      Write-Host "[2/2] Running native bob-tuner sync..." -ForegroundColor Green
      & $bin --repo $RepoRoot @argsList
      if ($LASTEXITCODE -ne 0) { throw "bob-tuner sync failed with exit code $LASTEXITCODE" }
      & $bin --repo $RepoRoot validate
      $usedRust = $true
    } else {
      Write-Warning "Cargo build failed (exit $LASTEXITCODE) — falling back to zero-deps Node engine."
    }
  } finally {
    Pop-Location
  }
}

if (-not $usedRust) {
  $node = Get-Command node -ErrorAction SilentlyContinue
  if (-not $node) {
    throw "Neither Cargo nor Node.js was found in PATH. Install Node.js >= 18 or Rust/Cargo and retry."
  }
  Write-Host "[1/2] Running zero-dependency Node engine (scripts/bob-sync.mjs)..." -ForegroundColor Yellow
  $syncScript = Join-Path $RepoRoot "scripts\bob-sync.mjs"
  & $node.Source $syncScript @argsList
  if ($LASTEXITCODE -ne 0) { throw "bob-sync.mjs failed with exit code $LASTEXITCODE" }
  Write-Host "[2/2] Running deterministic validation..." -ForegroundColor Green
  & $node.Source $syncScript validate
}

Write-Host ""
Write-Host "[DONE] Next step inside IBM Bob IDE:" -ForegroundColor Green
Write-Host "  1. Press Ctrl+Shift+P -> 'Developer: Reload Window' (loads all MCP servers)"
Write-Host "  2. Open a new chat (+) — 'PE-AGENTS-IBM Orquestrador' is now default"
Write-Host "  3. For any additional project folder, run:"
Write-Host "     .\scripts\install.ps1 -Workspace C:\path\to\your\project" -ForegroundColor DarkGray
