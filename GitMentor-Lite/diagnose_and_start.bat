@echo off
echo ========================================
echo    GitMentor-Lite 诊断和启动工具
echo    Author: Evilek, Date: 2025-01-09
echo ========================================
echo.

:: 检查编译后的exe文件
set "EXE_PATH=src-tauri\target\release\gitmentor-lite.exe"
echo [1/6] 检查可执行文件...
if exist "%EXE_PATH%" (
    echo ✓ 找到可执行文件: %EXE_PATH%
    
    :: 获取文件信息
    for %%i in ("%EXE_PATH%") do (
        echo   文件大小: %%~zi 字节
        echo   修改时间: %%~ti
    )
) else (
    echo ✗ 可执行文件不存在: %EXE_PATH%
    echo.
    echo 请先编译项目：
    echo   npm run tauri build
    echo.
    pause
    exit /b 1
)

echo.
echo [2/6] 检查端口占用情况...
netstat -ano | findstr :1420 >nul
if %errorlevel% equ 0 (
    echo ⚠️  端口1420被占用，正在清理...
    call kill-port-1420.bat
) else (
    echo ✓ 端口1420空闲
)

echo.
echo [3/6] 检查配置目录...
if exist ".config" (
    echo ✓ 配置目录存在: .config
    dir ".config" /b
) else (
    echo ℹ️  配置目录不存在，将在首次启动时创建
)

echo.
echo [4/6] 清理旧日志...
if exist "startup.log" (
    del "startup.log"
    echo ✓ 已清理旧日志文件
) else (
    echo ℹ️  无旧日志文件需要清理
)

echo.
echo [5/6] 检查系统环境...
echo 当前工作目录: %cd%
echo 系统时间: %date% %time%
echo 用户名: %username%

echo.
echo [6/6] 启动应用程序...
echo ========================================
echo.

echo 启动GitMentor-Lite...
echo 日志将记录到: startup.log
echo.

:: 记录启动时间
echo [%date% %time%] === 诊断启动开始 === >> startup.log
echo [%date% %time%] 工作目录: %cd% >> startup.log
echo [%date% %time%] 可执行文件: %EXE_PATH% >> startup.log

:: 启动应用
start "" "%EXE_PATH%"

echo 应用已启动，等待3秒检查状态...
timeout /t 3 /nobreak >nul

:: 检查进程是否还在运行
tasklist | findstr "gitmentor-lite.exe" >nul
if %errorlevel% equ 0 (
    echo ✓ 应用正在运行
    echo.
    echo 选择操作：
    echo [1] 查看实时日志
    echo [2] 查看当前日志
    echo [3] 退出
    echo.
    set /p "choice=请选择 (1-3): "
    
    if "%choice%"=="1" (
        echo.
        echo 开始实时监控日志...
        call view_logs.bat
    ) else if "%choice%"=="2" (
        echo.
        echo 当前日志内容：
        echo ========================================
        if exist "startup.log" type "startup.log"
        echo ========================================
        pause
    )
) else (
    echo ✗ 应用可能已闪退！
    echo.
    echo 检查日志文件获取错误信息...
    if exist "startup.log" (
        echo ========================================
        echo 启动日志内容：
        echo ========================================
        type "startup.log"
        echo ========================================
    ) else (
        echo 未生成日志文件，可能是启动失败
    )
    echo.
    pause
)

echo.
echo 诊断完成。
