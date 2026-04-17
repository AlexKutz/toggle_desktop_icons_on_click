# Build Scripts Documentation

## Overview

The project includes automated build scripts for Windows that compile both the main application (windows_multitool.exe) and settings UI in one command.

## Available Scripts

### 1. **build.ps1** (PowerShell Script)

Full-featured build script with multiple options.

#### Usage

```powershell
# Basic release build
.\build.ps1

# Debug build
.\build.ps1 -Release:$false

# Clean build (removes all previous builds)
.\build.ps1 -Clean

# Build with distribution package
.\build.ps1 -Distribute

# Combine options
.\build.ps1 -Clean -Distribute
```

#### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `-Release` | Switch | `$true` | Build in release mode (optimized) |
| `-Clean` | Switch | `$false` | Clean previous builds before building |
| `-Distribute` | Switch | `$false` | Create distribution package in `dist/` folder |

#### What It Does

1. ✅ Checks prerequisites (Rust/Cargo, Bun)
2. ✅ Builds main app (`desktop_icon_toggler.exe`)
3. ✅ Builds settings UI (`settings.exe`)
4. ✅ Copies settings.exe to main app directory
5. ✅ (Optional) Creates distribution package with README

#### Output Files

**Without `-Distribute`:**
```
target/release/
├── windows_multitool.exe (471 KB)
└── settings.exe (8.6 MB)
```

**With `-Distribute`:**
```
dist/release/
├── windows_multitool.exe (471 KB)
├── settings.exe (8.6 MB)
├── icon.ico (104 KB)
└── README.txt (0.5 KB)
```

---

### 2. **build.bat** (Batch File)

Simple double-click build script for quick builds.

#### Usage

Just double-click `build.bat` in Windows Explorer, or run:

```cmd
build.bat
```

#### What It Does

Runs `build.ps1` with `-Release -Distribute` flags automatically. Creates a complete distribution package ready for deployment.

---

## Prerequisites

The scripts automatically check for:

1. **Rust/Cargo** - Install from https://rustup.rs/
2. **Bun** - Install from https://bun.sh/

If either is missing, the script will show an error and exit.

---

## Build Process Details

### Step 1: Prerequisites Check
```
> Checking prerequisites...
[OK] Rust/Cargo found: cargo 1.94.1
[OK] Bun found: 1.3.10
```

### Step 2: Main App Build
```
> Building main application (windows_multitool.exe)...
   Compiling windows_multitool v0.1.0
    Finished `release` profile [optimized] target(s) in 1.54s
[OK] Main app built: ... (471.5 KB)
```

### Step 3: Settings UI Build
```
> Building settings UI (settings.exe)...
$ tauri build
     Running beforeBuildCommand `bun run build`
✓ 31 modules transformed.
   Compiling settings-ui v0.1.0
    Finished `release` profile [optimized] target(s) in 1m 10s
[OK] Settings UI built: ... (8.5 MB)
```

### Step 4: Deployment
```
> Deploying settings.exe to main app directory...
[OK] Copied settings.exe to .../target/release/settings.exe
```

### Step 5: Distribution Package (if -Distribute)
```
> Creating distribution package...
[OK] Distribution package created: .../dist/release

Distribution contents:
  desktop_icon_toggler.exe (471.5 KB)
  icon.ico (103.6 KB)
  README.txt (0.5 KB)
  settings.exe (8654 KB)
```

### Step 6: Summary
```
===========================================
  Build Complete!
===========================================

Build outputs:
  Main app:      .../target/release/windows_multitool.exe
  Settings UI:   .../target/release/settings.exe
  Distribution:  .../dist/release

Ready to run:
  cd target\release
  .\windows_multitool.exe
```

---

## Common Use Cases

### Development (Fast Iteration)

```powershell
# Quick debug build
.\build.ps1 -Release:$false
```

**Time:** ~30 seconds (if no changes)

### Release Build

```powershell
# Optimized release build
.\build.ps1
```

**Time:** ~2 minutes (first time), ~30 seconds (incremental)

### Clean Build (Troubleshooting)

```powershell
# Remove all build artifacts and rebuild
.\build.ps1 -Clean
```

**Time:** ~3-5 minutes (full rebuild)

### Prepare for Distribution

```powershell
# Build and create distributable package
.\build.ps1 -Distribute
```

**Output:** `dist/release/` folder ready to share

### Full Production Build

```powershell
# Clean + Release + Distribute
.\build.ps1 -Clean -Distribute
```

**Time:** ~5 minutes

---

## Distribution Package

When using `-Distribute` flag, the script creates a ready-to-deploy package:

### Contents

| File | Size | Purpose |
|------|------|---------|
| `windows_multitool.exe` | 471 KB | Main application |
| `settings.exe` | 8.6 MB | Settings UI |
| `icon.ico` | 104 KB | Application icon |
| `README.txt` | 0.5 KB | Quick start guide |

### How to Distribute

1. **Zip the folder:**
   ```powershell
   Compress-Archive -Path "dist\release\*" -DestinationPath "DesktopIconToggler-v0.1.0.zip"
   ```

2. **Create installer:** (future enhancement)
   - Use Inno Setup
   - Use NSIS
   - Use WiX Toolset

3. **Deploy to users:**
   - Users extract to any folder
   - Run `desktop_icon_toggler.exe`
   - Both executables must be in same folder

---

## Troubleshooting

### "Access is denied" Error

**Problem:**
```
error: failed to remove file `...\windows_multitool.exe`
Access is denied. (os error 5)
```

**Solution:**
The application is still running. Close it first:
```powershell
Stop-Process -Name "windows_multitool" -Force
Stop-Process -Name "settings" -Force
```

Then rebuild:
```powershell
.\build.ps1
```

---

### "Rust/Cargo not found" Error

**Problem:**
```
[ERROR] Rust/Cargo not found. Please install Rust from https://rustup.rs/
```

**Solution:**
1. Install Rust: https://rustup.rs/
2. Restart terminal
3. Run build again

---

### "Bun not found" Error

**Problem:**
```
[ERROR] Bun not found. Please install Bun from https://bun.sh/
```

**Solution:**
1. Install Bun: https://bun.sh/
   ```powershell
   powershell -c "irm bun.sh/install.ps1|iex"
   ```
2. Restart terminal
3. Run build again

---

### Settings UI Build Fails

**Problem:**
Tauri build fails with dependency errors.

**Solution:**
Clean and rebuild:
```powershell
.\build.ps1 -Clean
```

Or manually:
```powershell
cd settings-ui
Remove-Item node_modules -Recurse -Force
Remove-Item src-tauri\target -Recurse -Force
bun install
cd ..
.\build.ps1
```

---

### PowerShell Execution Policy Error

**Problem:**
```
cannot be loaded because running scripts is disabled on this system
```

**Solution:**
Use the execution policy bypass:
```powershell
powershell -ExecutionPolicy Bypass -File ".\build.ps1"
```

Or set execution policy (admin required):
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

## Integration with IDE

### VS Code Tasks

Add to `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Build Release",
      "type": "shell",
      "command": "powershell -ExecutionPolicy Bypass -File ${workspaceFolder}/build.ps1 -Release",
      "group": "build",
      "presentation": {
        "reveal": "always",
        "clear": true
      }
    },
    {
      "label": "Build Distribution",
      "type": "shell",
      "command": "powershell -ExecutionPolicy Bypass -File ${workspaceFolder}/build.ps1 -Release -Distribute",
      "group": "build"
    },
    {
      "label": "Clean Build",
      "type": "shell",
      "command": "powershell -ExecutionPolicy Bypass -File ${workspaceFolder}/build.ps1 -Clean",
      "group": "build"
    }
  ]
}
```

Now you can:
- Press `Ctrl+Shift+B` to build
- Select task from Command Palette

---

## Performance

### Build Times (Approximate)

| Scenario | First Build | Incremental |
|----------|-------------|-------------|
| Debug | 2 min | 30 sec |
| Release | 3 min | 45 sec |
| Clean | 5 min | 5 min |

### Disk Space

| Component | Size |
|-----------|------|
| Main app (debug) | ~5 MB |
| Main app (release) | 471 KB |
| Settings UI (debug) | ~100 MB |
| Settings UI (release) | 8.6 MB |
| Full build artifacts | ~500 MB |

---

## Future Enhancements

Potential improvements to build scripts:

1. **Version management:**
   - Auto-increment version in Cargo.toml
   - Tag git commits
   - Generate changelog

2. **Installer creation:**
   - Auto-generate MSI/NSIS installers
   - Code signing integration

3. **Testing integration:**
   - Run tests before build
   - Generate test coverage reports

4. **CI/CD integration:**
   - GitHub Actions workflow
   - AppVeyor configuration
   - Azure Pipelines support

5. **Package managers:**
   - Winget package
   - Scoop manifest
   - Chocolatey package

---

## Quick Reference

```powershell
# Most common commands:

# Build for development
.\build.ps1 -Release:$false

# Build for release
.\build.ps1

# Build and create distributable
.\build.ps1 -Distribute

# Clean everything and rebuild
.\build.ps1 -Clean

# Full production build
.\build.ps1 -Clean -Distribute

# Just double-click for quick build
build.bat
```

---

## Support

If you encounter issues:

1. Check prerequisites are installed
2. Try clean build: `.\build.ps1 -Clean`
3. Check console output for specific errors
4. Refer to `Docs/settings.md` for troubleshooting

For more help, check project documentation or open an issue.
