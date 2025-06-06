@echo off
echo GitMentor Complete Build Script (China Mirrors)
echo ================================================

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
    pause
    exit /b 1
)

echo Environment check passed

REM Configure npm mirror
echo Configuring npm mirror...
call npm config set registry https://registry.npmmirror.com/

REM Configure Rust mirror
echo Configuring Rust mirror...
if not exist ".cargo" mkdir .cargo
echo [source.crates-io] > .cargo\config.toml
echo replace-with = "tuna" >> .cargo\config.toml
echo. >> .cargo\config.toml
echo [source.tuna] >> .cargo\config.toml
echo registry = "https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git" >> .cargo\config.toml

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

REM Build backend executable
echo Building backend executable...
call python build_backend_china.py
if %errorlevel% neq 0 (
    echo ERROR: Backend build failed
    pause
    exit /b 1
)

REM Verify backend executable exists
if not exist "backend\gitmentor-backend.exe" (
    echo ERROR: Backend executable not found
    echo Expected: backend\gitmentor-backend.exe
    pause
    exit /b 1
)

echo Backend executable verified: backend\gitmentor-backend.exe

REM Build Tauri application with backend
echo Building Tauri application with embedded backend...
call npm run tauri build
if %errorlevel% neq 0 (
    echo ERROR: Tauri build failed
    pause
    exit /b 1
)

echo.
echo ================================================
echo GitMentor complete build finished!
echo ================================================
echo.
echo Package location: src-tauri\target\release\bundle\msi
echo Executable location: src-tauri\target\release\app.exe
echo.
echo The application now includes:
echo - Complete Vue 3 frontend interface
echo - Embedded Python FastAPI backend
echo - SQLite database support
echo - AI Agent system framework
echo.
echo Ready for customer distribution!
echo.

pause
