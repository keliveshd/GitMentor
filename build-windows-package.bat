@echo off
setlocal enabledelayedexpansion

REM ============================================================================
REM GitMentor MVP 统一构建脚本 v2.0
REM 功能：自动化构建和打包GitMentor MVP为Windows可执行程序
REM 支持：Windows 10/11, PowerShell 5.0+
REM 作者：Evilek
REM ============================================================================

echo.
echo ========================================
echo   GitMentor MVP 统一构建脚本 v2.0
echo ========================================
echo.

REM 设置颜色代码（如果支持）
set "GREEN=[92m"
set "RED=[91m"
set "YELLOW=[93m"
set "BLUE=[94m"
set "CYAN=[96m"
set "RESET=[0m"

REM 解析命令行参数
set "BUILD_MODE=release"
set "CLEAN_BUILD=false"
set "SKIP_DEPS=false"
set "VERBOSE=false"
set "DEV_MODE=false"
set "OPEN_OUTPUT=false"

:parse_args
if "%~1"=="" goto :args_done
if /i "%~1"=="--debug" set "BUILD_MODE=debug"
if /i "%~1"=="--dev" set "DEV_MODE=true"
if /i "%~1"=="--clean" set "CLEAN_BUILD=true"
if /i "%~1"=="--skip-deps" set "SKIP_DEPS=true"
if /i "%~1"=="--verbose" set "VERBOSE=true"
if /i "%~1"=="--open" set "OPEN_OUTPUT=true"
if /i "%~1"=="--help" goto :show_help
shift
goto :parse_args

:args_done

echo %BLUE%构建模式: %BUILD_MODE%%RESET%
echo %BLUE%开发模式: %DEV_MODE%%RESET%
echo %BLUE%清理构建: %CLEAN_BUILD%%RESET%
echo %BLUE%跳过依赖检查: %SKIP_DEPS%%RESET%
echo.

REM 检查是否在正确的目录
if not exist "GitMentor-Lite\package.json" (
    echo %RED%错误: 请在GitMentor项目根目录运行此脚本%RESET%
    echo 当前目录: %CD%
    echo 期望找到: GitMentor-Lite\package.json
    pause
    exit /b 1
)

cd GitMentor-Lite

REM 环境检查
echo %YELLOW%正在检查构建环境...%RESET%

REM 检查Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo %RED%错误: Node.js 未安装%RESET%
    echo 请从 https://nodejs.org/ 下载并安装 Node.js 18+ 
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('node --version') do set NODE_VERSION=%%i
echo %GREEN%✓ Node.js: %NODE_VERSION%%RESET%

REM 检查npm
where npm >nul 2>nul
if %errorlevel% neq 0 (
    echo %RED%错误: npm 未安装%RESET%
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('npm --version') do set NPM_VERSION=%%i
echo %GREEN%✓ npm: %NPM_VERSION%%RESET%

REM 检查Rust
where rustc >nul 2>nul
if %errorlevel% neq 0 (
    echo %RED%错误: Rust 未安装%RESET%
    echo 请运行: winget install Rustlang.Rustup
    echo 或访问: https://rustup.rs/
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('rustc --version') do set RUST_VERSION=%%i
echo %GREEN%✓ Rust: %RUST_VERSION%%RESET%

REM 检查Cargo
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo %RED%错误: Cargo 未安装%RESET%
    pause
    exit /b 1
)

for /f "tokens=*" %%i in ('cargo --version') do set CARGO_VERSION=%%i
echo %GREEN%✓ Cargo: %CARGO_VERSION%%RESET%

REM 检查Tauri CLI
cargo tauri --version >nul 2>nul
if %errorlevel% neq 0 (
    echo %YELLOW%Tauri CLI 未安装，正在安装...%RESET%
    cargo install tauri-cli --version "^2.0"
    if %errorlevel% neq 0 (
        echo %RED%错误: Tauri CLI 安装失败%RESET%
        pause
        exit /b 1
    )
)

for /f "tokens=*" %%i in ('cargo tauri --version') do set TAURI_VERSION=%%i
echo %GREEN%✓ Tauri CLI: %TAURI_VERSION%%RESET%

echo.
echo %GREEN%✓ 环境检查完成%RESET%
echo.

REM 清理构建缓存（如果需要）
if "%CLEAN_BUILD%"=="true" (
    echo %YELLOW%正在清理构建缓存...%RESET%
    
    if exist "node_modules" (
        echo   删除 node_modules...
        rmdir /s /q "node_modules"
    )
    
    if exist "src-tauri\target" (
        echo   删除 Rust target 目录...
        rmdir /s /q "src-tauri\target"
    )
    
    if exist "dist" (
        echo   删除前端构建目录...
        rmdir /s /q "dist"
    )
    
    if exist "src-tauri\gen" (
        echo   删除 Tauri 生成文件...
        rmdir /s /q "src-tauri\gen"
    )
    
    echo %GREEN%✓ 清理完成%RESET%
    echo.
)

REM 安装依赖
if "%SKIP_DEPS%"=="false" (
    echo %YELLOW%正在检查和安装依赖...%RESET%
    
    if not exist "node_modules" (
        echo   安装前端依赖...
        call npm install
        if %errorlevel% neq 0 (
            echo %RED%错误: 前端依赖安装失败%RESET%
            pause
            exit /b 1
        )
    ) else (
        echo   前端依赖已存在，跳过安装
    )
    
    echo %GREEN%✓ 依赖检查完成%RESET%
    echo.
)

REM 构建应用
if "%DEV_MODE%"=="true" (
    echo %YELLOW%正在启动开发模式...%RESET%
    echo   开发服务器将在浏览器中打开
    echo   按 Ctrl+C 停止开发服务器
    echo.
    npm run tauri:dev
    goto :eof
) else (
    echo %YELLOW%正在构建GitMentor MVP应用...%RESET%

    if "%BUILD_MODE%"=="debug" (
        echo   构建模式: 调试版本
        if "%VERBOSE%"=="true" (
            cargo tauri build --debug --verbose
        ) else (
            cargo tauri build --debug
        )
    ) else (
        echo   构建模式: 发布版本
        if "%VERBOSE%"=="true" (
            cargo tauri build --verbose
        ) else (
            cargo tauri build
        )
    )

    if %errorlevel% neq 0 (
        echo %RED%错误: 应用构建失败%RESET%
        echo.
        echo %YELLOW%常见解决方案:%RESET%
        echo 1. 运行 --clean 参数重新构建
        echo 2. 检查 src-tauri\src 目录下的Rust代码是否有编译错误
        echo 3. 检查 src 目录下的前端代码是否有TypeScript错误
        echo 4. 运行 --verbose 参数查看详细错误信息
        echo 5. 确保所有依赖都已正确安装
        pause
        exit /b 1
    )
)

echo.
echo %GREEN%✓ 构建完成！%RESET%
echo.

REM 查找构建产物
echo %BLUE%正在查找构建产物...%RESET%

if "%BUILD_MODE%"=="debug" (
    set "TARGET_DIR=src-tauri\target\debug"
    set "BUNDLE_DIR=src-tauri\target\debug\bundle"
) else (
    set "TARGET_DIR=src-tauri\target\release"
    set "BUNDLE_DIR=src-tauri\target\release\bundle"
)

echo.
echo ========================================
echo           构建产物位置
echo ========================================

REM 查找可执行文件
if exist "%TARGET_DIR%\gitmentor-lite.exe" (
    echo %GREEN%✓ 可执行文件:%RESET% %CD%\%TARGET_DIR%\gitmentor-lite.exe
    for %%A in ("%TARGET_DIR%\gitmentor-lite.exe") do echo   文件大小: %%~zA 字节
)

REM 查找MSI安装包
if exist "%BUNDLE_DIR%\msi" (
    for %%f in ("%BUNDLE_DIR%\msi\*.msi") do (
        echo %GREEN%✓ MSI安装包:%RESET% %%f
        for %%A in ("%%f") do echo   文件大小: %%~zA 字节
    )
)

REM 查找NSIS安装包
if exist "%BUNDLE_DIR%\nsis" (
    for %%f in ("%BUNDLE_DIR%\nsis\*.exe") do (
        echo %GREEN%✓ NSIS安装包:%RESET% %%f
        for %%A in ("%%f") do echo   文件大小: %%~zA 字节
    )
)

echo.
echo %CYAN%========================================%RESET%
echo %CYAN%           打包完成！%RESET%
echo %CYAN%========================================%RESET%
echo.
echo %GREEN%🎉 GitMentor MVP Windows版本打包成功！%RESET%
echo.
echo %YELLOW%使用说明:%RESET%
echo 1. 可执行文件可直接运行，无需安装
echo 2. MSI/NSIS安装包提供完整的安装体验
echo 3. 建议将可执行文件复制到独立目录使用
echo 4. 首次运行需要配置LLM服务（Ollama或OpenAI API）
echo.

REM 打开输出目录（如果需要）
if "%OPEN_OUTPUT%"=="true" (
    if exist "%TARGET_DIR%\gitmentor-lite.exe" (
        echo %BLUE%📂 正在打开输出目录...%RESET%
        start "" "%CD%\%TARGET_DIR%"
    )
)

echo %GRAY%构建完成时间: %date% %time%%RESET%
pause
goto :eof

:show_help
echo.
echo %CYAN%GitMentor MVP 统一构建脚本使用说明%RESET%
echo.
echo %YELLOW%用法:%RESET% build-windows-package.bat [选项]
echo.
echo %YELLOW%选项:%RESET%
echo   --debug      构建调试版本（默认：发布版本）
echo   --dev        启动开发模式（热重载）
echo   --clean      清理所有缓存后重新构建
echo   --skip-deps  跳过依赖检查和安装
echo   --verbose    显示详细构建信息
echo   --open       构建完成后打开输出目录
echo   --help       显示此帮助信息
echo.
echo %YELLOW%示例:%RESET%
echo   %GREEN%# 标准发布构建%RESET%
echo   build-windows-package.bat
echo.
echo   %GREEN%# 清理重新构建%RESET%
echo   build-windows-package.bat --clean
echo.
echo   %GREEN%# 开发模式（热重载）%RESET%
echo   build-windows-package.bat --dev
echo.
echo   %GREEN%# 调试版本 + 详细输出%RESET%
echo   build-windows-package.bat --debug --verbose
echo.
echo   %GREEN%# 构建并打开输出目录%RESET%
echo   build-windows-package.bat --clean --open
echo.
echo %YELLOW%注意:%RESET%
echo - 首次运行建议使用 --clean 参数
echo - 开发模式会启动热重载服务器
echo - 构建产物位于 GitMentor-Lite\src-tauri\target\ 目录
echo.
pause
goto :eof
