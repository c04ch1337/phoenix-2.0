@echo off
echo ===== Phoenix AGI: Sola Edition Windows Installer Builder =====

REM Set paths
set INNO_PATH="C:\Program Files (x86)\Inno Setup 6\ISCC.exe"

REM Check Inno Setup exists
if not exist %INNO_PATH% (
    echo ERROR: Inno Setup not found at %INNO_PATH%
    echo Please install Inno Setup 6 or update this script with the correct path.
    exit /b 1
)

REM First run the build script to create the staging directory
echo Running build script to prepare files...
call build_windows.cmd
if %ERRORLEVEL% neq 0 (
    echo ERROR: Build failed. See above for details.
    exit /b %ERRORLEVEL%
)

REM Now compile the installer
echo.
echo Building installer with Inno Setup...
%INNO_PATH% installer.iss
if %ERRORLEVEL% neq 0 (
    echo ERROR: Installer compilation failed.
    exit /b %ERRORLEVEL%
)

echo.
echo ===== Build Complete =====
echo Installer created: PAGI-SolaSetup.exe
echo.