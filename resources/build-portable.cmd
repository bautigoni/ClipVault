@echo off
REM Build the portable distribution: a self-contained folder that can be copied
REM to a USB drive. The data dir is the executable's sibling "data" folder.

setlocal
set CONFIG=release
set PORTABLE_DIR=target\portable\clipvault-portable
if exist "%PORTABLE_DIR%" rmdir /s /q "%PORTABLE_DIR%"
mkdir "%PORTABLE_DIR%\data"

cd src-tauri
cargo build --release --features portable
if errorlevel 1 exit /b %errorlevel%
cd ..

copy "src-tauri\target\release\clipvault.exe" "%PORTABLE_DIR%\clipvault.exe"
xcopy /s /e /y "src-tauri\target\release\resources\*" "%PORTABLE_DIR%\resources\"
xcopy /s /e /y "src-tauri\target\release\locales\*" "%PORTABLE_DIR%\locales\"

REM Create a README in the portable folder
(
  echo ClipVault Portable.
  echo.
  echo Run clipvault.exe; data is stored in the .\data\ folder next to the executable.
  echo Copy this whole folder to a USB drive to take your clipboard history with you.
) > "%PORTABLE_DIR%\README.txt"

echo Portable build ready at %PORTABLE_DIR%
