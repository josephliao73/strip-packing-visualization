$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
$AppDir = Join-Path $RootDir "packing_interface"
$DistDir = Join-Path $RootDir "dist\windows"
$BundleDir = Join-Path $DistDir "packing-app"
$BinDir = Join-Path $BundleDir "bin"
$SrcDir = Join-Path $BundleDir "src"

function Require-Command {
    param([string]$Name)

    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Missing required command: $Name"
    }
}

Require-Command cargo
Require-Command python

Write-Host "Building Windows release binary..."
& cargo build --manifest-path (Join-Path $AppDir "Cargo.toml") --release

Write-Host "Assembling Windows bundle at $BundleDir"
if (Test-Path $BundleDir) {
    Remove-Item $BundleDir -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $SrcDir | Out-Null

Copy-Item (Join-Path $AppDir "target\release\packing_interface.exe") (Join-Path $BinDir "packing_interface.exe")
Copy-Item (Join-Path $AppDir "src\algorithm_templates") (Join-Path $SrcDir "algorithm_templates") -Recurse
Copy-Item (Join-Path $AppDir "src\runner_utils") (Join-Path $SrcDir "runner_utils") -Recurse
Copy-Item (Join-Path $AppDir "src\runner_lib") (Join-Path $SrcDir "runner_lib") -Recurse
Copy-Item (Join-Path $AppDir "requirements.txt") (Join-Path $BundleDir "requirements.txt")

@'
$ErrorActionPreference = "Stop"

$BundleDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$VenvDir = Join-Path $BundleDir ".venv"
$CppMinMajor = 8

function Get-PythonRuntime {
    $candidates = @(
        (Join-Path $VenvDir "Scripts\python.exe"),
        (Join-Path $VenvDir "Scripts\python"),
        "python"
    )

    foreach ($candidate in $candidates) {
        if ($candidate -like "*\*" -or $candidate -like "*/*") {
            if (Test-Path $candidate) {
                return $candidate
            }
        } elseif (Get-Command $candidate -ErrorAction SilentlyContinue) {
            return $candidate
        }
    }

    return $null
}

function Test-PythonRuntime {
    $py = Get-PythonRuntime
    if (-not $py) {
        Write-Host "Python runtime unavailable: python not found." -ForegroundColor Yellow
        return $false
    }

    try {
        & $py -c "import numpy, scipy" *> $null
        return $true
    } catch {
        Write-Host "Python runtime unavailable: failed to import numpy/scipy with $py." -ForegroundColor Yellow
        Write-Host "Run .\setup-python.bat inside the bundle to install the bundled Python dependencies." -ForegroundColor Yellow
        return $false
    }
}

function Test-CppRuntime {
    $gxx = Get-Command g++ -ErrorAction SilentlyContinue
    if (-not $gxx) {
        Write-Host "C++ runtime unavailable: g++ not found." -ForegroundColor Yellow
        return $false
    }

    $version = (& $gxx.Source -dumpfullversion -dumpversion 2>$null | Select-Object -First 1)
    $major = 0
    if ($version -match '^(\d+)') {
        $major = [int]$matches[1]
    }

    if ($major -lt $CppMinMajor) {
        Write-Host "C++ runtime unavailable: g++ $version is too old. Need g++ $CppMinMajor+ with C++17 support." -ForegroundColor Yellow
        return $false
    }

    $tmpBase = Join-Path $env:TEMP ("packing-cpp-check-" + [guid]::NewGuid().ToString("N"))
    $tmpSrc = "$tmpBase.cpp"
    $tmpExe = "$tmpBase.exe"
    try {
        Set-Content -Path $tmpSrc -Value 'int main() { return 0; }' -Encoding ASCII
        & $gxx.Source -std=c++17 $tmpSrc -o $tmpExe *> $null
        return $LASTEXITCODE -eq 0
    } catch {
        Write-Host "C++ runtime unavailable: g++ failed a C++17 compile check." -ForegroundColor Yellow
        return $false
    } finally {
        Remove-Item $tmpSrc, $tmpExe -ErrorAction SilentlyContinue
    }
}

$runtimeArgs = @()

if (Test-PythonRuntime) {
    $runtimeArgs += "python"
}

if (Test-CppRuntime) {
    $runtimeArgs += "cpp"
}

if ($runtimeArgs.Count -eq 0) {
    Write-Host "No supported Python or C++ runtime detected. The app will start with both language options hidden." -ForegroundColor Yellow
}

& (Join-Path $BundleDir "bin\packing_interface.exe") @runtimeArgs
exit $LASTEXITCODE
'@ | Set-Content -Path (Join-Path $BundleDir "packing-app.ps1") -Encoding ASCII

@'
@echo off
setlocal
powershell -ExecutionPolicy Bypass -File "%~dp0packing-app.ps1"
exit /b %errorlevel%
'@ | Set-Content -Path (Join-Path $BundleDir "packing-app.bat") -Encoding ASCII

@'
@echo off
setlocal

set "BUNDLE_DIR=%~dp0"
set "VENV_DIR=%BUNDLE_DIR%.venv"

where python >nul 2>nul
if errorlevel 1 (
  echo Missing required command: python
  exit /b 1
)

if not exist "%VENV_DIR%" (
  python -m venv "%VENV_DIR%"
)

"%VENV_DIR%\Scripts\python.exe" -m pip install --upgrade pip
if errorlevel 1 exit /b %errorlevel%

"%VENV_DIR%\Scripts\python.exe" -m pip install -r "%BUNDLE_DIR%requirements.txt"
exit /b %errorlevel%
'@ | Set-Content -Path (Join-Path $BundleDir "setup-python.bat") -Encoding ASCII

@'
Packing App Windows Bundle
==========================

Contents:
- packing-app.bat: launcher that auto-detects working Python/C++ runtimes
- packing-app.ps1: PowerShell launcher used by the batch file
- setup-python.bat: creates a bundle-local .venv and installs numpy/scipy
- bin\packing_interface.exe: compiled Rust GUI binary
- src\: bundled templates and runner helper files

Host requirements:
- Windows desktop session
- Python if you want Python algorithms enabled in the app
- g++ if you want C++ algorithms enabled in the app

Recommended first run:
1. setup-python.bat
2. packing-app.bat

If g++ is installed and supports C++17, the app will also enable C++ templates.
'@ | Set-Content -Path (Join-Path $BundleDir "README-windows.txt") -Encoding ASCII

Write-Host "Windows bundle created:"
Write-Host "  $BundleDir"
Write-Host ""
Write-Host "Run it with:"
Write-Host "  cd `"$BundleDir`""
Write-Host "  .\setup-python.bat"
Write-Host "  .\packing-app.bat"
