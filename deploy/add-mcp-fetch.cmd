@echo off
REM Wrapper: pass aruaru root as %%1 (default F:\aruaru)
set "ROOT=%~1"
if "%ROOT%"=="" set "ROOT=F:\aruaru"
for %%I in ("%ROOT%") do set "ROOT=%%~fI"
set "CURSOR=%LOCALAPPDATA%\Programs\cursor\resources\app\bin\cursor.cmd"
if not exist "%CURSOR%" set "CURSOR=C:\Program Files\cursor\resources\app\bin\cursor.cmd"
if not exist "%CURSOR%" set "CURSOR=C:\Program Files\Cursor\resources\app\bin\cursor.cmd"
"%CURSOR%" --add-mcp "{\"name\":\"fetch\",\"command\":\"npx\",\"args\":[\"-y\",\"@kazuph/mcp-fetch\"]}" "%ROOT%"
exit /b %ERRORLEVEL%
