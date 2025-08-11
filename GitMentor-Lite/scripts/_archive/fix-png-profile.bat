@echo off
echo.
echo ========================================
echo   PNG sRGB Profile Fix Tool
echo   Author: Evilek
echo   Date: 2025-08-08
echo ========================================
echo.

:: Set ImageMagick path
set "MAGICK_PATH=D:\Soft\ImageMagick-7.1.2-Q16-HDRI"
set "MAGICK_EXE=%MAGICK_PATH%\magick.exe"

:: Check if ImageMagick exists
if not exist "%MAGICK_EXE%" (
    echo ERROR: ImageMagick not found!
    echo Path: %MAGICK_PATH%
    echo Please check the MAGICK_PATH variable in this script
    pause
    exit /b 1
)

:: Set icons directory
set "ICONS_DIR=%~dp0src-tauri\icons"

:: Check if icons directory exists
if not exist "%ICONS_DIR%" (
    echo ERROR: Icons directory not found!
    echo Path: %ICONS_DIR%
    pause
    exit /b 1
)

echo ImageMagick: %MAGICK_EXE%
echo Icons Dir: %ICONS_DIR%
echo.

:: Create backup directory
set "BACKUP_DIR=%ICONS_DIR%\backup_%date:~0,4%%date:~5,2%%date:~8,2%_%time:~0,2%%time:~3,2%%time:~6,2%"
set "BACKUP_DIR=%BACKUP_DIR: =0%"
mkdir "%BACKUP_DIR%" 2>nul

echo Creating backup directory: %BACKUP_DIR%
echo.

:: Backup original files
echo Backing up original PNG files...
for %%f in ("%ICONS_DIR%\*.png") do (
    echo    Backing up: %%~nxf
    copy "%%f" "%BACKUP_DIR%\" >nul
)
echo.

:: Fix PNG files
echo Starting PNG sRGB profile fix...
echo.

set "FIXED_COUNT=0"
set "ERROR_COUNT=0"

for %%f in ("%ICONS_DIR%\*.png") do (
    echo Processing: %%~nxf

    :: Use ImageMagick to strip color profile and set sRGB
    "%MAGICK_EXE%" "%%f" -strip -colorspace sRGB "%%f.tmp" 2>nul

    if exist "%%f.tmp" (
        move "%%f.tmp" "%%f" >nul
        echo    SUCCESS: Fixed
        set /a FIXED_COUNT+=1
    ) else (
        echo    ERROR: Failed to fix
        set /a ERROR_COUNT+=1
    )
    echo.
)

echo ========================================
echo   Fix Complete Summary
echo ========================================
echo SUCCESS: %FIXED_COUNT% files fixed
echo ERROR: %ERROR_COUNT% files failed
echo BACKUP: %BACKUP_DIR%
echo.

if %ERROR_COUNT% gtr 0 (
    echo WARNING: Some files failed to fix. Check ImageMagick installation and file permissions.
) else (
    echo SUCCESS: All PNG files fixed! libpng warnings should be gone!
)

echo.
echo TIP: If there are issues, restore from backup directory
echo.
pause
