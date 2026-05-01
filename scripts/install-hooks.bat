@echo off
echo Installing git hooks...

if not exist ".git" (
    echo ERROR: Run this script from the root of your git repository.
    pause
    exit /b 1
)

copy /Y "scripts\pre-push" ".git\hooks\pre-push" >nul

echo Done! Direct pushes to main are now blocked.
pause
