@echo off
echo ========================================
echo    AI Icon Cleanup - Keep Only 12 Icons
echo    Author: Evilek, Date: 2025-01-09
echo ========================================
echo.

set "ICON_DIR=src\assets\static-webp\light"

if not exist "%ICON_DIR%" (
    echo ERROR: Icon directory does not exist!
    echo Path: %ICON_DIR%
    pause
    exit /b 1
)

echo Analyzing icon directory: %ICON_DIR%
echo.

:: Count files before cleanup
for /f %%i in ('dir /b "%ICON_DIR%\*.webp" 2^>nul ^| find /c /v ""') do set "BEFORE_COUNT=%%i"
echo Files before cleanup: %BEFORE_COUNT%

:: Create temp directory to save needed files
set "TEMP_DIR=%ICON_DIR%\temp_keep"
if exist "%TEMP_DIR%" rmdir /s /q "%TEMP_DIR%"
mkdir "%TEMP_DIR%"

echo.
echo Backing up 12 required AI provider icons...

:: 12 AI provider icons to keep (updated with qwen.webp)
set "KEEP_FILES=openai.webp ollama.webp anthropic.webp zhipu.webp deepseek.webp gemini.webp qwen.webp doubao.webp siliconcloud.webp openrouter.webp together.webp groq.webp"

set "KEPT_COUNT=0"
for %%f in (%KEEP_FILES%) do (
    if exist "%ICON_DIR%\%%f" (
        copy "%ICON_DIR%\%%f" "%TEMP_DIR%\" >nul
        echo [OK] Backed up: %%f
        set /a KEPT_COUNT+=1
    ) else (
        echo [MISSING] %%f
    )
)

echo.
echo Successfully backed up: %KEPT_COUNT%/12 icon files

if %KEPT_COUNT% NEQ 12 (
    echo.
    echo WARNING: Some icon files are missing!
    echo Continue cleanup anyway? [Y/N]
    set /p "CONTINUE="
    if /i not "%CONTINUE%"=="Y" (
        rmdir /s /q "%TEMP_DIR%"
        echo Operation cancelled.
        pause
        exit /b 0
    )
)

echo.
echo Deleting all webp files...
del "%ICON_DIR%\*.webp" /q >nul 2>&1

echo Restoring required icon files...
move "%TEMP_DIR%\*.webp" "%ICON_DIR%\" >nul 2>&1

:: Remove temp directory
rmdir /s /q "%TEMP_DIR%"

:: Count files after cleanup
for /f %%i in ('dir /b "%ICON_DIR%\*.webp" 2^>nul ^| find /c /v ""') do set "AFTER_COUNT=%%i"

echo.
echo ========================================
echo            CLEANUP COMPLETE!
echo ========================================
echo Files before: %BEFORE_COUNT%
echo Files after: %AFTER_COUNT%
set /a "DELETED_COUNT=%BEFORE_COUNT%-%AFTER_COUNT%"
echo Files deleted: %DELETED_COUNT%
echo.

echo Kept AI provider icons:
for %%f in (%KEEP_FILES%) do (
    if exist "%ICON_DIR%\%%f" (
        echo   [OK] %%f
    ) else (
        echo   [MISSING] %%f
    )
)

echo.
echo Cleanup complete! Project size optimized.
pause
