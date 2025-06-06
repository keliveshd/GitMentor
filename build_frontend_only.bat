@echo off
echo GitMentor Frontend-Only Build Script
echo =====================================

REM Check Node.js
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js not installed
    pause
    exit /b 1
)

REM Check Rust
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust not installed
    echo Please install Rust from https://rustup.rs/
    pause
    exit /b 1
)

echo Environment check passed

REM Install frontend dependencies
echo Installing frontend dependencies...
call npm install
if %errorlevel% neq 0 (
    echo ERROR: Frontend dependencies installation failed
    pause
    exit /b 1
)

REM Build frontend
echo Building frontend...
call npm run build
if %errorlevel% neq 0 (
    echo ERROR: Frontend build failed
    pause
    exit /b 1
)

REM Build Tauri application (frontend only)
echo Building Tauri application...
call npm run tauri build
if %errorlevel% neq 0 (
    echo ERROR: Tauri build failed
    pause
    exit /b 1
)

echo.
echo =====================================
echo GitMentor frontend build completed!
echo =====================================
echo.
echo Package location: src-tauri\target\release\bundle\msi
echo Executable location: src-tauri\target\release\GitMentor.exe
echo.
echo Note: This version contains only the frontend.
echo Backend integration will be added in the next step.
echo.

pause
