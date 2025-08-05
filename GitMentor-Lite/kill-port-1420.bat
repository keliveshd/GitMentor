@echo off
echo ========================================
echo Finding process using port 1420...
echo ========================================

:: Find process using port 1420
for /f "tokens=5" %%a in ('netstat -ano ^| findstr :1420') do (
    set PID=%%a
    goto :found
)

echo No process found using port 1420
goto :end

:found
echo Found process using port 1420, PID: %PID%
echo.
echo Killing process...
taskkill /PID %PID% /F

if %errorlevel% equ 0 (
    echo Process killed successfully
) else (
    echo Failed to kill process, may need administrator privileges
)

:end
echo.
echo Press any key to exit...
pause >nul
