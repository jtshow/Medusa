# Medusa Skill Framework - One-Line Installer (Windows Native)
# Usage: irm https://raw.githubusercontent.com/your-repo/medusa/main/install.ps1 | iex

Write-Host "Medusa Skill Framework (MSF) - Installer" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

# Check if Rust is installed
try {
    $rustVersion = & rustc --version 2>$null
    Write-Host "[1/3] Rust already installed ($rustVersion)" -ForegroundColor Green
} catch {
    Write-Host "[1/3] Installing Rust..." -ForegroundColor Yellow
    $rustUrl = "https://win.rustup.rs/x86_64"
    $rustInstaller = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest -Uri $rustUrl -OutFile $rustInstaller
    Start-Process $rustInstaller -ArgumentList "-y" -Wait
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
}

# Check if project exists
if (Test-Path "medusa") {
    Write-Host "[2/3] Project directory exists, building..." -ForegroundColor Green
    Set-Location "medusa"
} else {
    Write-Host "[2/3] Cloning repository..." -ForegroundColor Yellow
    git clone https://github.com/your-repo/medusa.git
    Set-Location "medusa"
}

# Build natively on Windows
Write-Host "[3/3] Building Medusa (native Windows)..." -ForegroundColor Yellow
$env:PATH += ";$env:USERPROFILE\.cargo\bin"
cargo build --release

Write-Host ""
Write-Host "✅ Medusa installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "Binary location: $(Get-Location)\target\release\medusa.exe" -ForegroundColor Cyan
Write-Host ""
Write-Host "Run: .\target\release\medusa.exe --help" -ForegroundColor Yellow
