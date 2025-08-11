@echo off
echo ========================================
echo    GitMentor-Lite 日志查看器
echo    Author: Evilek, Date: 2025-01-09
echo ========================================
echo.

if not exist "startup.log" (
    echo 日志文件不存在：startup.log
    echo.
    echo 请先运行应用程序生成日志文件。
    pause
    exit /b 1
)

echo 显示启动日志内容：
echo ========================================
type "startup.log"
echo ========================================
echo.

echo 日志文件位置：%cd%\startup.log
echo.

echo 选择操作：
echo [1] 实时监控日志
echo [2] 清理日志文件
echo [3] 退出
echo.
set /p "choice=请选择 (1-3): "

if "%choice%"=="1" goto monitor
if "%choice%"=="2" goto cleanup
if "%choice%"=="3" goto exit

:monitor
echo.
echo 开始实时监控日志... 按Ctrl+C退出
echo ========================================
:monitor_loop
cls
echo ========================================
echo    GitMentor-Lite 实时日志监控
echo    Author: Evilek, Date: 2025-01-09
echo ========================================
echo.
if exist "startup.log" (
    type "startup.log"
) else (
    echo 日志文件不存在
)
echo.
echo ========================================
echo 实时监控中... 按Ctrl+C退出
echo ========================================
timeout /t 2 /nobreak >nul
goto monitor_loop

:cleanup
echo.
echo 确认要清理日志文件吗？ [Y/N]
set /p "confirm="
if /i "%confirm%"=="Y" (
    del "startup.log"
    echo 日志文件已清理。
) else (
    echo 操作已取消。
)
pause
goto exit

:exit
echo.
echo 退出日志查看器。
pause
