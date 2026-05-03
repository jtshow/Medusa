@echo off
echo Medusa Skill Framework (MSF) - Installer
echo ========================================

REM Check if Rust is installed
where rustc >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo [1/3] Installing Rust...
    powershell -Command "Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile '$env:TEMP\rustup-init.exe'; Start-Process '$env:TEMP\rustup-init.exe' -ArgumentList '-y' -Wait"
)

echo [2/3] Building Medusa...
cd medusa 2>nul || git clone https://github.com/your-repo/medusa.git && cd medusa

echo [3/3] Building Medusa (native Windows)...
cargo build --release

echo.
echo ✅ Medusa installed successfully!
echo.
echo Binary location: %CD%\target\release\medusa.exe
echo.
echo Run: .\target\release\medusa.exe --help
pause
