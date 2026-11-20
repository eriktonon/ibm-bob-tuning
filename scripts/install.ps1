<#
.SYNOPSIS
  One-command installer for IBM Bob v2.1.0+ tuning (Windows PowerShell).
.DESCRIPTION
  Auto-bootstraps self-contained Rust GNU (`x86_64-pc-windows-gnu` + `llvm-tools`) when Cargo is missing,
  compiles the native `bob-tuner.exe` CLI, installs it globally in `%USERPROFILE%\.cargo\bin\bob-tuner.exe`,
  and falls back to the zero-dependency Node.js engine (`scripts/bob-sync.mjs`) only if offline/SkipRust.
#>
[CmdletBinding(PositionalBinding = $false)]
param(
  [string[]]$Workspace = @(),
  [switch]$AllModes,
  [string]$Model = "",
  [switch]$NoTurbo,
  [switch]$DryRun,
  [switch]$SkipRust,
  [Parameter(ValueFromRemainingArguments = $true)]
  [string[]]$RemainingArgs = @()
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

# Guard against unquoted paths with spaces passed after -Workspace (e.g. -Workspace C:\Users\Name\My Project)
if ($RemainingArgs.Count -gt 0) {
  if ($Workspace.Count -gt 0) {
    $joinedCandidate = "$($Workspace[-1]) $($RemainingArgs -join ' ')"
    if (Test-Path $joinedCandidate) {
      $Workspace[$Workspace.Count - 1] = (Resolve-Path $joinedCandidate).Path
      $RemainingArgs = @()
    }
  }
  if ($RemainingArgs.Count -gt 0) {
    throw "Unexpected unquoted arguments: '$($RemainingArgs -join ' ')'. Always wrap workspace paths containing spaces in double quotes, e.g.: -Workspace `"C:\path\to\your project`""
  }
}

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host " IBM BOB v2.1.0+ TUNING INSTALLER (PE-AGENTS-IBM)" -ForegroundColor Cyan
Write-Host " Repo: $RepoRoot" -ForegroundColor DarkGray
Write-Host "================================================================================" -ForegroundColor Cyan

# Ensure ~/.cargo/bin and self-contained GNU/LLVM toolchain dirs are in current session PATH
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
$toolchainBin = Join-Path $env:USERPROFILE ".rustup\toolchains\stable-x86_64-pc-windows-gnu\bin"
$rustlibBin = Join-Path $env:USERPROFILE ".rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\x86_64-pc-windows-gnu\bin"
$selfContained = Join-Path $rustlibBin "self-contained"

foreach ($dir in @($selfContained, $rustlibBin, $toolchainBin, $cargoBin)) {
  if ((Test-Path $dir) -and ($env:Path -notlike "*$dir*")) {
    $env:Path = "$dir;$env:Path"
  }
}

function Ensure-SelfContainedDlltool {
  if (Test-Path "$rustlibBin\llvm-ar.exe") {
    if ((Test-Path "$selfContained\dlltool.exe") -and -not (Test-Path "$selfContained\dlltool.gnu.bak")) {
      Copy-Item "$selfContained\dlltool.exe" "$selfContained\dlltool.gnu.bak" -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $selfContained) {
      Copy-Item "$rustlibBin\llvm-ar.exe" "$selfContained\dlltool.exe" -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $cargoBin) {
      Copy-Item "$rustlibBin\llvm-ar.exe" "$cargoBin\dlltool.exe" -Force -ErrorAction SilentlyContinue
    }
  }
}

if (-not $SkipRust -and -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
  Write-Host "[0/2] Rust/Cargo not found -> bootstrapping self-contained Rust (x86_64-pc-windows-gnu)..." -ForegroundColor Yellow
  try {
    $rustupInit = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit -UseBasicParsing
    & $rustupInit -y --default-host x86_64-pc-windows-gnu --default-toolchain stable
    foreach ($dir in @($selfContained, $rustlibBin, $toolchainBin, $cargoBin)) {
      if ((Test-Path $dir) -and ($env:Path -notlike "*$dir*")) {
        $env:Path = "$dir;$env:Path"
      }
    }
    if (Test-Path "$cargoBin\rustup.exe") {
      & "$cargoBin\rustup.exe" component add llvm-tools
    }
  } catch {
    Write-Warning "Automatic Rust bootstrap failed ($($_.Exception.Message)) — falling back to Node.js engine."
  }
}

Ensure-SelfContainedDlltool

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
    $bin = Join-Path $RepoRoot "target\release\bob-tuner.exe"
    # Avoid os error 5 (Access Denied) on Windows if bob-tuner.exe is currently running as an MCP process
    if (Test-Path $bin) {
      $oldBin = "$bin.old"
      Move-Item -Path $bin -Destination $oldBin -Force -ErrorAction SilentlyContinue
      Remove-Item -Path $oldBin -Force -ErrorAction SilentlyContinue
    }

    & cargo build --release
    if ($LASTEXITCODE -eq 0 -and (Test-Path $bin)) {
      if (-not $DryRun) {
        if (-not (Test-Path $cargoBin)) {
          New-Item -ItemType Directory -Path $cargoBin -Force | Out-Null
        }
        $globalBin = Join-Path $cargoBin "bob-tuner.exe"
        if (Test-Path $globalBin) {
          $oldGlobal = "$globalBin.old"
          Move-Item -Path $globalBin -Destination $oldGlobal -Force -ErrorAction SilentlyContinue
          Remove-Item -Path $oldGlobal -Force -ErrorAction SilentlyContinue
        }
        Copy-Item -Path $bin -Destination $globalBin -Force -ErrorAction SilentlyContinue

        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        foreach ($persistDir in @($toolchainBin, $cargoBin)) {
          if ((Test-Path $persistDir) -and ($userPath -notlike "*$persistDir*")) {
            $userPath = "$persistDir;$userPath"
          }
        }
        [Environment]::SetEnvironmentVariable("Path", $userPath, "User")
        [Environment]::SetEnvironmentVariable("BOB_TUNING_REPO", $RepoRoot, "User")
        $env:BOB_TUNING_REPO = $RepoRoot
      }

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
Write-Host "  3. Global CLI installed: run 'bob-tuner status' or 'bob-tuner set-model claude-opus-4-6' from any folder"
Write-Host "  4. For any additional project folder, run:"
Write-Host "     .\scripts\install.ps1 -Workspace `"C:\path\to\your\project`"" -ForegroundColor DarkGray
