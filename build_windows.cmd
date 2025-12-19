@echo off
echo ===== Phoenix AGI: Sola Edition Windows Build Script =====

REM Set paths
set SCRIPT_DIR=%~dp0
set STAGING_DIR=%SCRIPT_DIR%staging
set RELEASE_DIR=%SCRIPT_DIR%target\release

REM Clean staging directory if it exists
if exist "%STAGING_DIR%" (
    echo Cleaning existing staging directory...
    rd /s /q "%STAGING_DIR%"
)

REM Create staging directory
echo Creating staging directory...
mkdir "%STAGING_DIR%"

REM Build frontend
echo Building frontend...
call scripts\build_frontend.cmd
if %ERRORLEVEL% neq 0 (
    echo Error building frontend.
    exit /b %ERRORLEVEL%
)

REM Build backend
echo Building backend...
cargo build --release --bin phoenix-web
if %ERRORLEVEL% neq 0 (
    echo Error building backend.
    exit /b %ERRORLEVEL%
)

REM Copy phoenix-web.exe to staging
echo Copying phoenix-web.exe to staging...
copy "%RELEASE_DIR%\phoenix-web.exe" "%STAGING_DIR%\"
if %ERRORLEVEL% neq 0 (
    echo Error copying phoenix-web.exe.
    exit /b %ERRORLEVEL%
)

REM Create data directory in staging
echo Creating data directory...
mkdir "%STAGING_DIR%\data"

REM Copy frontend assets to staging
echo Copying frontend assets to staging...
mkdir "%STAGING_DIR%\frontend"
xcopy /E /I /Y "frontend\dist" "%STAGING_DIR%\frontend\dist"
if %ERRORLEVEL% neq 0 (
    echo Error copying frontend assets.
    exit /b %ERRORLEVEL%
)

REM Copy launcher script to staging
echo Copying launcher...
copy "launcher.cmd" "%STAGING_DIR%\"
if %ERRORLEVEL% neq 0 (
    echo Error copying launcher.
    exit /b %ERRORLEVEL%
)

REM Create default .env file in staging
echo Creating default .env file...
(
echo # Phoenix AGI Configuration
echo # Set your OpenRouter API Key here
echo OPENROUTER_API_KEY=
echo # User name (for primary address in relational context)
echo USER_NAME=
echo # How Sola refers to you
echo USER_PREFERRED_ALIAS=
) > "%STAGING_DIR%\.env"

echo Build completed successfully!
echo Staging directory: %STAGING_DIR%