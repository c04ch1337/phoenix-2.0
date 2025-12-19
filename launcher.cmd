@echo off
echo Starting Sola...
start "" "%~dp0\pagi-sola-web.exe"
timeout /t 2 /nobreak > nul
start "" "http://127.0.0.1:8888"
echo Sola started. Browser window should open automatically.
