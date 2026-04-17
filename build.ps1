# Build script for Desktop Icon Toggler
# This script builds both the main app and settings UI automatically

param(
    [switch]$Release = $true,
    [switch]$Clean = $false,
    [switch]$Distribute = $false
)

$ErrorActionPreference = "Stop"

# Colors for output
function Write-Step($message) {
    Write-Host "`n> " -ForegroundColor Cyan -NoNewline
    Write-Host $message -ForegroundColor White
}

function Write-Success($message) {
    Write-Host "[OK] " -ForegroundColor Green -NoNewline
    Write-Host $message -ForegroundColor Green
}

function Write-Error-Custom($message) {
    Write-Host "[ERROR] " -ForegroundColor Red -NoNewline
    Write-Host $message -ForegroundColor Red
}

# Get script directory
$ProjectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$BuildType = if ($Release) { "release" } else { "debug" }

Write-Host "" -NoNewline
Write-Host "===========================================" -ForegroundColor Cyan
Write-Host "  Windows Multitool - Build Script" -ForegroundColor Cyan
Write-Host "===========================================" -ForegroundColor Cyan

# Step 1: Clean if requested
if ($Clean) {
    Write-Step "Cleaning previous builds..."
    
    if (Test-Path "$ProjectRoot\target") {
        Remove-Item "$ProjectRoot\target" -Recurse -Force
        Write-Success "Cleaned main app target directory"
    }
    
    if (Test-Path "$ProjectRoot\settings-ui\src-tauri\target") {
        Remove-Item "$ProjectRoot\settings-ui\src-tauri\target" -Recurse -Force
        Write-Success "Cleaned settings UI target directory"
    }
    
    if (Test-Path "$ProjectRoot\settings-ui\dist") {
        Remove-Item "$ProjectRoot\settings-ui\dist" -Recurse -Force
        Write-Success "Cleaned settings UI dist directory"
    }
}

# Step 2: Check prerequisites
Write-Step "Checking prerequisites..."

# Check Rust
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "Rust/Cargo not found. Please install Rust from https://rustup.rs/"
    exit 1
}
Write-Success "Rust/Cargo found: $(cargo --version)"

# Check Bun
if (-not (Get-Command "bun" -ErrorAction SilentlyContinue)) {
    Write-Error-Custom "Bun not found. Please install Bun from https://bun.sh/"
    exit 1
}
Write-Success "Bun found: $(bun --version)"

# Step 3: Build main application
Write-Step "Building main application (desktop_icon_toggler.exe)..."

Set-Location $ProjectRoot

if ($Release) {
    cargo build --release
} else {
    cargo build
}

if ($LASTEXITCODE -ne 0) {
    Write-Error-Custom "Failed to build main application"
    exit 1
}

$MainExe = "$ProjectRoot\target\$BuildType\windows_multitool.exe"
if (-not (Test-Path $MainExe)) {
    Write-Error-Custom "Main executable not found after build"
    exit 1
}

$MainSize = (Get-Item $MainExe).Length
$MainSizeKB = [math]::Round($MainSize/1KB, 1)
Write-Success "Main app built: $MainExe ($MainSizeKB KB)"

# Step 4: Build settings UI
Write-Step "Building settings UI (settings.exe)..."

Set-Location "$ProjectRoot\settings-ui"

# Install dependencies if node_modules doesn't exist
if (-not (Test-Path "node_modules")) {
    Write-Host "  Installing dependencies..." -ForegroundColor Gray
    bun install
    if ($LASTEXITCODE -ne 0) {
        Write-Error-Custom "Failed to install dependencies"
        exit 1
    }
}

# Build with Tauri
bun run tauri build

if ($LASTEXITCODE -ne 0) {
    Write-Error-Custom "Failed to build settings UI"
    exit 1
}

$SettingsExe = "$ProjectRoot\settings-ui\src-tauri\target\$BuildType\settings-ui.exe"
if (-not (Test-Path $SettingsExe)) {
    # Try alternative location
    $SettingsExe = "$ProjectRoot\settings-ui\src-tauri\target\release\settings-ui.exe"
}

if (-not (Test-Path $SettingsExe)) {
    Write-Error-Custom "Settings executable not found after build"
    exit 1
}

$SettingsSize = (Get-Item $SettingsExe).Length
$SettingsSizeMB = [math]::Round($SettingsSize/1MB, 1)
Write-Success "Settings UI built: $SettingsExe ($SettingsSizeMB MB)"

# Step 5: Copy settings.exe to main app directory
Write-Step "Deploying settings.exe to main app directory..."

$TargetSettingsExe = "$ProjectRoot\target\$BuildType\settings.exe"
Copy-Item $SettingsExe $TargetSettingsExe -Force

Write-Success "Copied settings.exe to $TargetSettingsExe"

# Step 6: Create distribution package if requested
if ($Distribute) {
    Write-Step "Creating distribution package..."
    
    $DistDir = "$ProjectRoot\dist\$BuildType"
    if (Test-Path $DistDir) {
        Remove-Item $DistDir -Recurse -Force
    }
    New-Item -ItemType Directory -Path $DistDir -Force | Out-Null
    
    # Copy executables
    Copy-Item $MainExe $DistDir
    Copy-Item $TargetSettingsExe $DistDir
    
    # Copy icon if exists
    if (Test-Path "$ProjectRoot\icon.ico") {
        Copy-Item "$ProjectRoot\icon.ico" $DistDir
    }
    
    # Create README
    $ReadmeContent = "Windows Multitool`n`n" +
        "A lightweight Windows utility with multiple system tools.`n`n" +
        "Installation:`n" +
        "1. Copy all files to a permanent location`n" +
        "2. Run windows_multitool.exe`n" +
        "3. Right-click tray icon - Settings to configure`n`n" +
        "Features:`n" +
        "- Desktop Icon Toggler - Show/hide desktop icons`n" +
        "- Cursor Hider - Auto-hide cursor after inactivity`n" +
        "- More tools coming soon!`n`n" +
        "Files:`n" +
        "- windows_multitool.exe - Main application (system tray)`n" +
        "- settings.exe - Settings UI (launched from tray menu)`n" +
        "- icon.ico - Application icon`n`n" +
        "First Run:`n" +
        "The application will create a settings file at:`n" +
        "%APPDATA%/DesktopIconToggler/settings.json`n`n" +
        "Support:`n" +
        "Check Docs/settings.md for detailed documentation."
    
    Set-Content "$DistDir\README.txt" $ReadmeContent -Encoding UTF8
    
    Write-Success "Distribution package created: $DistDir"
    
    # Show distribution contents
    Write-Host "`nDistribution contents:" -ForegroundColor Cyan
    Get-ChildItem $DistDir | ForEach-Object {
        $size = if ($_.PSIsContainer) { "folder" } else { "$([math]::Round($_.Length/1KB, 1)) KB" }
        Write-Host "  $($_.Name) ($size)" -ForegroundColor Gray
    }
}

# Step 7: Summary
Write-Host "`n===========================================" -ForegroundColor Cyan
Write-Host "  Build Complete!" -ForegroundColor Green
Write-Host "===========================================" -ForegroundColor Cyan

Write-Host "`nBuild outputs:" -ForegroundColor White
Write-Host "  Main app:      $MainExe" -ForegroundColor Gray
Write-Host "  Settings UI:   $TargetSettingsExe" -ForegroundColor Gray

if ($Distribute) {
    Write-Host "  Distribution:  $DistDir" -ForegroundColor Gray
}

Write-Host "`nReady to run:" -ForegroundColor White
Write-Host "  cd target\$BuildType" -ForegroundColor Gray
Write-Host "  .\desktop_icon_toggler.exe" -ForegroundColor Gray

Write-Host ""
