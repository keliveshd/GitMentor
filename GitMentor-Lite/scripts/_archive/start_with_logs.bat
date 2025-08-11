@echo off
echo ========================================
echo    GitMentor-Lite 启动脚本 (带日志)
echo    Author: Evilek, Date: 2025-01-09
echo ========================================
echo.

:: 检查是否存在编译后的exe文件
set "EXE_PATH=src-tauri\target\release\gitmentor-lite.exe"
if not exist "%EXE_PATH%" (
    echo 错误：找不到编译后的可执行文件！
    echo 路径：%EXE_PATH%
    echo.
    echo 请先运行以下命令编译项目：
    echo   npm run tauri build
    echo.
    pause
    exit /b 1
)

:: 清理旧的日志文件
if exist "startup.log" (
    echo 清理旧的日志文件...
    del "startup.log"
)

echo 启动GitMentor-Lite...
echo 日志将记录到：startup.log
echo.
echo 如果程序闪退，请查看startup.log文件获取详细错误信息。
echo.

:: 启动应用程序
echo [%date% %time%] 启动GitMentor-Lite >> startup.log
start "" "%EXE_PATH%"

:: 等待一下让程序启动
timeout /t 2 /nobreak >nul

:: 实时显示日志内容
echo 正在监控启动日志...
echo ========================================
echo.

:monitor_loop
if exist "startup.log" (
    type "startup.log"
    echo.
    echo ========================================
    echo 日志监控中... 按Ctrl+C退出监控
    echo ========================================
    timeout /t 3 /nobreak >nul
    cls
    echo ========================================
    echo    GitMentor-Lite 日志监控
    echo    Author: Evilek, Date: 2025-01-09
    echo ========================================
    echo.
    goto monitor_loop
) else (
    echo 等待日志文件生成...
    timeout /t 1 /nobreak >nul
    goto monitor_loop
)
