#Requires -RunAsAdministrator
# ClipVault - One-shot dev environment setup for Windows
# Run this once in an elevated PowerShell. Takes ~15-25 min total.

$ErrorActionPreference = 'Stop'
$ProgressPreference    = 'SilentlyContinue'

function Write-Step($msg) { Write-Host "`n=== $msg ===" -ForegroundColor Cyan }
function Write-Ok($msg)   { Write-Host "  [OK] $msg" -ForegroundColor Green }
function Write-Skip($msg) { Write-Host "  [SKIP] $msg" -ForegroundColor Yellow }

# --- 1. Rust ---------------------------------------------------------------
Write-Step "1/3 - Rust + Cargo"
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Skip "Rust already installed: $((Get-Command cargo).Source)"
} else {
    Write-Host "  Installing Rust via winget (this may take a few minutes)..."
    winget install --id Rustlang.Rustup -e --source winget --accept-package-agreements --accept-source-agreements
    # winget runs in a separate PATH context; refresh env for this session
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Ok "Rust installed: $((Get-Command cargo).Source)"
    } else {
        Write-Host "  Rust installed but cargo is not on PATH yet. Close and reopen this terminal, then re-run the dev command." -ForegroundColor Yellow
    }
}

# --- 2. Visual Studio Build Tools (C++ workload) --------------------------
Write-Step "2/3 - Visual Studio Build Tools (Desktop development with C++)"
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$hasCpp  = $false
if (Test-Path $vsWhere) {
    $installs = & $vsWhere -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($installs) { $hasCpp = $true }
}

if ($hasCpp) {
    Write-Skip "MSVC C++ tools already present"
} else {
    Write-Host "  Downloading and installing VS Build Tools..."
    $installer = "$env:TEMP\vs_buildtools.exe"
    Invoke-WebRequest -Uri "https://aka.ms/vs/17/release/vs_buildtools.exe" -OutFile $installer -UseBasicParsing
    $args = @(
        "--quiet","--wait","--norestart","--nocache",
        "--installPath","$env:ProgramFiles\Microsoft Visual Studio\2022\BuildTools",
        "--add","Microsoft.VisualStudio.Workload.VCTools",
        "--includeRecommended"
    )
    $proc = Start-Process -FilePath $installer -ArgumentList $args -Wait -PassThru
    if ($proc.ExitCode -eq 0 -or $proc.ExitCode -eq 3010) {
        Write-Ok "VS Build Tools installed (reboot may be required)"
    } else {
        Write-Host "  VS Build Tools installer exited with code $($proc.ExitCode). Check logs at %TEMP%\dd_*.log" -ForegroundColor Red
    }
    Remove-Item $installer -ErrorAction SilentlyContinue
}

# --- 3. WebView2 Runtime ---------------------------------------------------
Write-Step "3/3 - WebView2 Runtime"
$wv2Key = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
$wv2Ver = (Get-ItemProperty -Path $wv2Key -Name pv -ErrorAction SilentlyContinue).pv
if ($wv2Ver) {
    Write-Skip "WebView2 already installed: $wv2Ver"
} else {
    Write-Host "  Installing WebView2 Runtime..."
    $wv2 = "$env:TEMP\MicrosoftEdgeWebview2Setup.exe"
    Invoke-WebRequest -Uri "https://go.microsoft.com/fwlink/p/?LinkId=2124703" -OutFile $wv2 -UseBasicParsing
    Start-Process -FilePath $wv2 -ArgumentList "/silent","/install" -Wait
    Remove-Item $wv2 -ErrorAction SilentlyContinue
    Write-Ok "WebView2 installed"
}

# --- 4. Node tooling -------------------------------------------------------
Write-Step "Bonus - pnpm (optional, npm works too)"
if (Get-Command pnpm -ErrorAction SilentlyContinue) {
    Write-Skip "pnpm already installed"
} else {
    npm install -g pnpm 2>$null
    if (Get-Command pnpm -ErrorAction SilentlyContinue) {
        Write-Ok "pnpm installed"
    } else {
        Write-Skip "pnpm install failed - you can use npm instead"
    }
}

Write-Host "`n=== DONE ===" -ForegroundColor Green
Write-Host "If a reboot was requested, do it now. Then:" -ForegroundColor Cyan
Write-Host "  cd 'c:\Users\gonib\Downloads\ClipVault'"
Write-Host "  pnpm tauri:dev          # run the app"
Write-Host "  pnpm tauri:build        # produce the installer"
