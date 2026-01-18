@echo off
setlocal enabledelayedexpansion

:: Build script for pomodorust (Windows)
:: Usage: build.bat [release|fast|debug] [--clean] [--sign]

set "PROJECT_ROOT=%~dp0"
set "DIST_DIR=%PROJECT_ROOT%dist"
set "PROFILE=release"
set "CLEAN=0"
set "SIGN=0"

:: Parse arguments
:parse_args
if "%~1"=="" goto :find_vs
if /i "%~1"=="release" set "PROFILE=release" & shift & goto :parse_args
if /i "%~1"=="fast" set "PROFILE=release-fast" & shift & goto :parse_args
if /i "%~1"=="debug" set "PROFILE=debug" & shift & goto :parse_args
if /i "%~1"=="--clean" set "CLEAN=1" & shift & goto :parse_args
if /i "%~1"=="-c" set "CLEAN=1" & shift & goto :parse_args
if /i "%~1"=="--sign" set "SIGN=1" & shift & goto :parse_args
if /i "%~1"=="-s" set "SIGN=1" & shift & goto :parse_args
if /i "%~1"=="--help" goto :show_help
if /i "%~1"=="-h" goto :show_help
echo Unknown option: %~1
exit /b 1

:show_help
echo Usage: build.bat [profile] [options]
echo.
echo Profiles:
echo   release    Optimized for size (default)
echo   fast       Optimized for speed
echo   debug      Debug build
echo.
echo Options:
echo   --clean, -c    Clean before building
echo   --sign, -s     Sign executable after build
echo   --help, -h     Show this help
exit /b 0

:find_vs
:: Auto-detect Visual Studio
set "VCVARS="

:: Try VS 2022
for %%p in (
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
) do (
    if exist %%p (
        set "VCVARS=%%~p"
        goto :found_vs
    )
)

:: Try VS 2019
for %%p in (
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Community\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Professional\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\Enterprise\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
) do (
    if exist %%p (
        set "VCVARS=%%~p"
        goto :found_vs
    )
)

:: Try vswhere
where vswhere >nul 2>&1
if %errorlevel%==0 (
    for /f "usebackq tokens=*" %%i in (`vswhere -latest -property installationPath`) do (
        set "VCVARS=%%i\VC\Auxiliary\Build\vcvars64.bat"
        if exist "!VCVARS!" goto :found_vs
    )
)

echo Warning: Visual Studio not found, attempting build anyway...
goto :build

:found_vs
echo Setting up Visual Studio environment...
call "%VCVARS%" >nul 2>&1

:build
cd /d "%PROJECT_ROOT%"

:: Clean if requested
if "%CLEAN%"=="1" (
    echo Cleaning...
    cargo clean
    if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
)

:: Build
echo Building with profile: %PROFILE%
if "%PROFILE%"=="debug" (
    cargo build
    set "SOURCE_DIR=%PROJECT_ROOT%target\debug"
) else (
    cargo build --profile %PROFILE%
    set "SOURCE_DIR=%PROJECT_ROOT%target\%PROFILE%"
)

if %errorlevel% neq 0 (
    echo Build failed!
    exit /b 1
)

:: Create dist directory
if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"

set "BINARY=pomodorust.exe"
set "SOURCE_PATH=%SOURCE_DIR%\%BINARY%"
set "DEST_PATH=%DIST_DIR%\%BINARY%"

if not exist "%SOURCE_PATH%" (
    echo Binary not found: %SOURCE_PATH%
    exit /b 1
)

:: Try UPX compression for release builds
set "COMPRESSED=0"
if not "%PROFILE%"=="debug" (
    where upx >nul 2>&1
    if %errorlevel%==0 (
        echo Compressing with UPX...
        upx --best --lzma "%SOURCE_PATH%" -f -o "%DEST_PATH%" >nul 2>&1
        if %errorlevel%==0 set "COMPRESSED=1"
    )
)

if "%COMPRESSED%"=="0" (
    copy /y "%SOURCE_PATH%" "%DEST_PATH%" >nul
)

:: Sign if requested
if "%SIGN%"=="1" (
    echo.
    powershell -ExecutionPolicy Bypass -File "%PROJECT_ROOT%scripts\sign.ps1" -ExePath "%DEST_PATH%"
)

:: Create ZIP archive
set "VERSION="
for /f "tokens=3" %%v in ('findstr /r "^version" Cargo.toml') do (
    set "VERSION=%%~v"
    goto :got_version
)
:got_version
set "ZIP_NAME=pomodorust-v%VERSION%-windows-x64.zip"
set "ZIP_PATH=%DIST_DIR%\%ZIP_NAME%"

if exist "%ZIP_PATH%" del "%ZIP_PATH%"
echo Creating ZIP archive...
powershell -Command "Compress-Archive -Path '%DEST_PATH%' -DestinationPath '%ZIP_PATH%' -Force"

:: Show result
echo.
echo Output: %DEST_PATH%
for %%A in ("%DEST_PATH%") do echo Size: %%~zA bytes
if exist "%ZIP_PATH%" (
    echo Archive: %ZIP_PATH%
    for %%A in ("%ZIP_PATH%") do echo Archive size: %%~zA bytes
)
echo.
echo Done!
