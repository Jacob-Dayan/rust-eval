@echo off
setlocal

cd /d "%~dp0\.."

echo Building rust-eval (release mode)...
cargo build --release
if errorlevel 1 (
    echo Build failed.
    exit /b %errorlevel%
)

set "TARGET_DIR=%ProgramFiles%\Rust-Eval\app"
if not exist "%TARGET_DIR%" (
    echo Creating directory "%TARGET_DIR%"...
    mkdir "%TARGET_DIR%"
)

echo Moving binary to "%TARGET_DIR%"...
move /y "target\release\rs-eval.exe" "%TARGET_DIR%\rs-eval.exe"
if errorlevel 1 (
    echo Failed to move binary. Make sure you run this script as Administrator.
    exit /b %errorlevel%
)

echo Done! Installed rs-eval to "%TARGET_DIR%\rs-eval.exe".
endlocal
