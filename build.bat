@echo off
echo.
echo ===============================================
echo   Desktop Icon Toggler - Quick Build
echo ===============================================
echo.

:: Build both applications and create distribution
powershell -ExecutionPolicy Bypass -File "%~dp0build.ps1" -Release -Distribute

if %errorlevel% equ 0 (
    echo.
    echo Build successful! Press any key to exit...
    pause >nul
) else (
    echo.
    echo Build failed! Press any key to exit...
    pause >nul
    exit /b %errorlevel%
)
