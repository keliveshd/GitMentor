@echo off
echo ========================================
echo    Move AI Icons to Public Directory
echo    Author: Evilek, Date: 2025-01-09
echo ========================================
echo.

set "SOURCE_DIR=src\assets\static-webp\light"
set "TARGET_DIR=public\icons"

:: Check if source directory exists
if not exist "%SOURCE_DIR%" (
    echo ERROR: Source directory does not exist!
    echo Path: %SOURCE_DIR%
    pause
    exit /b 1
)

:: Create target directory if it doesn't exist
if not exist "public" mkdir "public"
if not exist "%TARGET_DIR%" mkdir "%TARGET_DIR%"

echo Moving AI provider icons to public directory...
echo Source: %SOURCE_DIR%
echo Target: %TARGET_DIR%
echo.

:: List of 12 AI provider icons to move (updated with qwen.webp)
set "ICONS=openai.webp ollama.webp anthropic.webp zhipu.webp deepseek.webp gemini.webp qwen.webp doubao.webp siliconcloud.webp openrouter.webp together.webp groq.webp"

set "MOVED_COUNT=0"
set "MISSING_COUNT=0"

for %%f in (%ICONS%) do (
    if exist "%SOURCE_DIR%\%%f" (
        copy "%SOURCE_DIR%\%%f" "%TARGET_DIR%\" >nul
        echo [OK] Moved: %%f
        set /a MOVED_COUNT+=1
    ) else (
        echo [MISSING] %%f
        set /a MISSING_COUNT+=1
    )
)

echo.
echo ========================================
echo            MOVE COMPLETE!
echo ========================================
echo Icons moved: %MOVED_COUNT%
echo Icons missing: %MISSING_COUNT%
echo.

if %MISSING_COUNT% GTR 0 (
    echo WARNING: Some icons are missing from source directory!
    echo Please check if these files exist:
    for %%f in (%ICONS%) do (
        if not exist "%SOURCE_DIR%\%%f" (
            echo   - %%f
        )
    )
    echo.
)

echo Icons are now available at: %TARGET_DIR%
echo These will be accessible in the built app as: /icons/[filename]
echo.

echo Next steps:
echo 1. Run the cleanup script to remove unused icons from src/assets
echo 2. Build the project to test icon display
echo 3. Icons should now display correctly in production build
echo.
pause
