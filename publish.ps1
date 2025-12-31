# Generic Cargo Project - Itch.io Publish Script
# Creates distributable packages for Windows and WebGL
# Automatically reads project name from Cargo.toml

param(
    [switch]$SkipBuild = $false,
    [switch]$WindowsOnly = $false,
    [switch]$WebGLOnly = $false
)

$ErrorActionPreference = "Stop"
$ProjectRoot = $PSScriptRoot
$DistDir = Join-Path $ProjectRoot "dist"
$CargoToml = Join-Path $ProjectRoot "Cargo.toml"

# Parse project name from Cargo.toml
if (-not (Test-Path $CargoToml)) {
    Write-Error "Cargo.toml not found at: $CargoToml"
    exit 1
}

$CargoContent = Get-Content $CargoToml -Raw
if ($CargoContent -match 'name\s*=\s*"([^"]+)"') {
    $ProjectName = $matches[1]
} else {
    Write-Error "Could not parse project name from Cargo.toml"
    exit 1
}

# Create display-friendly title (capitalize words, replace underscores with spaces)
$ProjectTitle = ($ProjectName -replace '_', ' ').ToUpper()

Write-Host "=== $ProjectTitle Publisher ===" -ForegroundColor Cyan
Write-Host "Project: $ProjectName"
Write-Host ""

# Calculate steps based on what we're building
$buildWindows = -not $WebGLOnly
$buildWebGL = -not $WindowsOnly
$totalSteps = 2  # Clean + Summary
if ($buildWindows) { $totalSteps += 2 }  # Build + Package
if ($buildWebGL) { $totalSteps += 2 }     # Build + Package
$currentStep = 0

# Step: Clean dist folder
$currentStep++
Write-Host "[$currentStep/$totalSteps] Preparing dist folder..." -ForegroundColor Yellow
if (Test-Path $DistDir) {
    Remove-Item $DistDir -Recurse -Force
}
New-Item -ItemType Directory -Path $DistDir -Force | Out-Null

# === WINDOWS BUILD ===
if ($buildWindows) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building Windows release..." -ForegroundColor Yellow
        cargo build --release
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Windows build failed!"
            exit 1
        }
        Write-Host "Windows build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping Windows build (using existing)" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging Windows build..." -ForegroundColor Yellow
    $WindowsPackageDir = Join-Path $DistDir "windows"
    New-Item -ItemType Directory -Path $WindowsPackageDir -Force | Out-Null

    # Copy executable
    $ExePath = Join-Path $ProjectRoot "target\release\$ProjectName.exe"
    if (-not (Test-Path $ExePath)) {
        Write-Error "Executable not found at: $ExePath"
        exit 1
    }
    Copy-Item $ExePath $WindowsPackageDir

    # Copy assets folder if it exists
    $AssetsPath = Join-Path $ProjectRoot "assets"
    if (Test-Path $AssetsPath) {
        Copy-Item $AssetsPath -Destination $WindowsPackageDir -Recurse
    }

    # Create Windows zip
    $WindowsZipPath = Join-Path $DistDir "${ProjectName}_windows.zip"
    Compress-Archive -Path "$WindowsPackageDir\*" -DestinationPath $WindowsZipPath -CompressionLevel Optimal
    Write-Host "Windows package created!" -ForegroundColor Green
}

# === WEBGL BUILD ===
if ($buildWebGL) {
    $currentStep++
    if (-not $SkipBuild) {
        Write-Host "[$currentStep/$totalSteps] Building WebGL release..." -ForegroundColor Yellow
        
        # Check if wasm32 target is installed
        $targets = rustup target list --installed
        if ($targets -notcontains "wasm32-unknown-unknown") {
            Write-Host "Installing wasm32-unknown-unknown target..." -ForegroundColor Yellow
            rustup target add wasm32-unknown-unknown
        }

        cargo build --release --target wasm32-unknown-unknown
        if ($LASTEXITCODE -ne 0) {
            Write-Error "WebGL build failed!"
            exit 1
        }
        Write-Host "WebGL build complete!" -ForegroundColor Green
    } else {
        Write-Host "[$currentStep/$totalSteps] Skipping WebGL build (using existing)" -ForegroundColor Gray
    }

    $currentStep++
    Write-Host "[$currentStep/$totalSteps] Packaging WebGL build..." -ForegroundColor Yellow
    $WebGLPackageDir = Join-Path $DistDir "webgl"
    New-Item -ItemType Directory -Path $WebGLPackageDir -Force | Out-Null

    # Copy WASM file
    $WasmPath = Join-Path $ProjectRoot "target\wasm32-unknown-unknown\release\$ProjectName.wasm"
    if (-not (Test-Path $WasmPath)) {
        Write-Error "WASM file not found at: $WasmPath"
        exit 1
    }
    Copy-Item $WasmPath $WebGLPackageDir

    # Copy assets folder if it exists
    $AssetsPath = Join-Path $ProjectRoot "assets"
    if (Test-Path $AssetsPath) {
        Copy-Item $AssetsPath -Destination $WebGLPackageDir -Recurse
    }

    # Download mq_js_bundle.js locally (so we don't rely on CDN)
    $JsBundlePath = Join-Path $WebGLPackageDir "mq_js_bundle.js"
    Write-Host "Downloading mq_js_bundle.js..." -ForegroundColor Gray
    try {
        Invoke-WebRequest -Uri "https://not-fl3.github.io/miniquad-samples/mq_js_bundle.js" -OutFile $JsBundlePath
    } catch {
        Write-Warning "Could not download mq_js_bundle.js - will use CDN reference"
    }
    $UseLocalJs = Test-Path $JsBundlePath

    # Create HTML wrapper
    $HtmlContent = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=1280, height=720, initial-scale=1.0">
    <title>$ProjectTitle</title>
    <style>
        html, body {
            margin: 0;
            padding: 0;
            width: 100%;
            height: 100%;
            overflow: hidden;
            background: #1a1a2e;
        }
        canvas {
            width: 100%;
            height: 100%;
            display: block;
        }
        #loading {
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: white;
            font-family: sans-serif;
            font-size: 24px;
        }
    </style>
</head>
<body>
    <div id="loading">Loading...</div>
    <canvas id="glcanvas" tabindex="1"></canvas>
    <script src="mq_js_bundle.js"></script>
    <script>
        document.getElementById('loading').style.display = 'none';
        load("$ProjectName.wasm");
    </script>
</body>
</html>
"@
    $HtmlPath = Join-Path $WebGLPackageDir "index.html"
    Set-Content -Path $HtmlPath -Value $HtmlContent

    # Create WebGL zip
    $WebGLZipPath = Join-Path $DistDir "${ProjectName}_webgl.zip"
    Compress-Archive -Path "$WebGLPackageDir\*" -DestinationPath $WebGLZipPath -CompressionLevel Optimal
    Write-Host "WebGL package created!" -ForegroundColor Green
}

# Summary
$currentStep++
Write-Host ""
Write-Host "=== Package Complete ===" -ForegroundColor Cyan

if ($buildWindows) {
    $WindowsZipPath = Join-Path $DistDir "${ProjectName}_windows.zip"
    $WinSize = [math]::Round((Get-Item $WindowsZipPath).Length / 1MB, 2)
    Write-Host "Windows: $WindowsZipPath (${WinSize} MB)" -ForegroundColor Green
}

if ($buildWebGL) {
    $WebGLZipPath = Join-Path $DistDir "${ProjectName}_webgl.zip"
    $WebSize = [math]::Round((Get-Item $WebGLZipPath).Length / 1MB, 2)
    Write-Host "WebGL:   $WebGLZipPath (${WebSize} MB)" -ForegroundColor Green
}

Write-Host ""
Write-Host "Next steps for itch.io:" -ForegroundColor Yellow
Write-Host "  1. Go to https://itch.io/dashboard" -ForegroundColor White
Write-Host "  2. Create/edit your project" -ForegroundColor White
if ($buildWindows) {
    Write-Host "  3. Upload Windows zip, mark as 'Windows'" -ForegroundColor White
}
if ($buildWebGL) {
    Write-Host "  4. Upload WebGL zip, mark as 'Play in browser'" -ForegroundColor White
}
Write-Host ""

# Open dist folder
explorer $DistDir
