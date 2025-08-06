@echo off
setlocal enabledelayedexpansion

REM ============================================================================
REM GitMentor 开发模式快速启动脚本
REM 作者：Evilek
REM 功能：快速启动开发环境，支持热重载
REM ============================================================================

echo.
echo ========================================
echo   GitMentor 开发模式启动
echo ========================================
echo.

REM 检查是否在正确的目录
if not exist "GitMentor-Lite\package.json" (
    echo [错误] 请在GitMentor项目根目录运行此脚本
    echo 当前目录: %CD%
    echo 期望找到: GitMentor-Lite\package.json
    pause
    exit /b 1
)

REM 进入项目目录
cd GitMentor-Lite

echo [信息] 正在检查开发环境...

REM 检查Node.js
where node >nul 2>nul
if %errorlevel% neq 0 (
    echo [错误] Node.js 未安装，请先安装 Node.js 18+
    pause
    exit /b 1
)

REM 检查npm依赖
if not exist "node_modules" (
    echo [信息] 正在安装前端依赖...
    call npm install
    if %errorlevel% neq 0 (
        echo [错误] 依赖安装失败
        pause
        exit /b 1
    )
)

REM 检查Rust
where cargo >nul 2>nul
if %errorlevel% neq 0 (
    echo [错误] Rust 未安装，请先安装 Rust 1.70+
    pause
    exit /b 1
)

echo [信息] 环境检查完成
echo.
echo [启动] 正在启动开发服务器...
echo [提示] 应用将自动打开，支持热重载
echo [提示] 按 Ctrl+C 停止开发服务器
echo.

REM 设置环境变量禁用libpng警告
set LIBPNG_NO_WARN=1

REM 启动开发模式
npm run tauri:dev

echo.
echo [完成] 开发服务器已停止
pause
