@echo off
REM aruaru Cursor setup restore (after Cursor reinstall / OS reinstall)
setlocal
set "SCRIPT=%~dp0enable-cursor-setup.ps1"
powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT%" -RegisterShortcut
echo.
echo If fetch MCP is grey: Ctrl+Shift+J - Tools and MCP - fetch ON
echo Then: Ctrl+Shift+P - Developer: Reload Window
pause
