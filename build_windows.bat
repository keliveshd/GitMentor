@echo off
echo GitMentor Windows Build Script
echo ===============================

REM Check Python
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Python not installed
    echo Please install Python 3.8+ from https://python.org
    pause
    exit /b 1
)

REM Check Node.js
node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js not installed
    echo Please install Node.js 16+ from https://nodejs.org
    pause
    exit /b 1
)

REM Check Rust
rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust not installed
    echo Please install Rust from https://rustup.rs/
    echo.
    echo To install Rust, run this command in PowerShell:
    echo curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    pause
    exit /b 1
)

echo Environment check passed
echo.

REM Install frontend dependencies
echo Installing frontend dependencies...
call npm install
if %errorlevel% neq 0 (
    echo ERROR: Frontend dependencies installation failed
    pause
    exit /b 1
)

REM Install backend dependencies
echo Installing backend dependencies...
cd backend
call pip install -r requirements.txt
if %errorlevel% neq 0 (
    echo ERROR: Backend dependencies installation failed
    pause
    exit /b 1
)

call pip install pyinstaller
if %errorlevel% neq 0 (
    echo ERROR: PyInstaller installation failed
    pause
    exit /b 1
)

cd ..

REM Build frontend
echo Building frontend...
call npm run build
if %errorlevel% neq 0 (
    echo ERROR: Frontend build failed
    pause
    exit /b 1
)

REM Build backend executable
echo Building backend executable...
call python build_backend_simple.py
if %errorlevel% neq 0 (
    echo ERROR: Backend build failed
    pause
    exit /b 1
)

REM Build Tauri application
echo Building Tauri application...
call npm run tauri build
if %errorlevel% neq 0 (
    echo ERROR: Tauri build failed
    pause
    exit /b 1
)

echo.
echo ===============================
echo GitMentor build completed!
echo ===============================
echo.
echo Package location: src-tauri\target\release\bundle\msi
echo Executable location: src-tauri\target\release\GitMentor.exe
echo.
echo You can now distribute the MSI installer to customers!
echo.

pause
