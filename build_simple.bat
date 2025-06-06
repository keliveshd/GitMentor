@echo off
chcp 65001 >nul

echo GitMentor Build Tool
echo ============================

REM Check environment
echo Checking build environment...

python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Python not installed
    pause
    exit /b 1
)

node --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Node.js not installed
    pause
    exit /b 1
)

rustc --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Rust not installed, please visit https://rustup.rs/
    pause
    exit /b 1
)

echo Environment check passed

REM Install dependencies
echo Installing dependencies...
npm install
if %errorlevel% neq 0 (
    echo ERROR: Frontend dependencies installation failed
    pause
    exit /b 1
)

cd backend
pip install -r requirements.txt
if %errorlevel% neq 0 (
    echo ERROR: Backend dependencies installation failed
    pause
    exit /b 1
)

pip install pyinstaller
if %errorlevel% neq 0 (
    echo ERROR: PyInstaller installation failed
    pause
    exit /b 1
)

cd ..

REM Build frontend
echo Building frontend...
npm run build
if %errorlevel% neq 0 (
    echo ERROR: Frontend build failed
    pause
    exit /b 1
)

REM Build backend
echo Building backend...
python build_backend.py
if %errorlevel% neq 0 (
    echo ERROR: Backend build failed
    pause
    exit /b 1
)

REM Build Tauri application
echo Building Tauri application...
npm run tauri build
if %errorlevel% neq 0 (
    echo ERROR: Tauri build failed
    pause
    exit /b 1
)

echo.
echo GitMentor packaging completed!
echo ============================
echo Package location: src-tauri\target\release\bundle\msi
echo.
echo Executable file generated, ready for distribution!
echo.

pause
