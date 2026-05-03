@echo off
REM Medusa Build Script - Windows
REM Usage: build.bat [release|debug]

if "%1"=="debug" (
    echo Building Medusa in debug mode...
    cargo build
    echo ✅ Built: target\debug\medusa.exe
) else (
    echo Building Medusa in release mode...
    cargo build --release
    echo ✅ Built: target\release\medusa.exe
)
pause
