# Windows Multitool

A lightweight Windows utility that enhances your desktop experience with multiple system tools in one application.

## Quick Start

### Build from Source

**Option 1: Double-click (Easiest)**
```
Double-click: build.bat
```

**Option 2: PowerShell**
```powershell
.\build.ps1 -Distribute
```

**Output:** `dist/release/` folder with ready-to-use executables

### Run the Application

```powershell
cd target\release
.\windows_multitool.exe
```

Right-click the tray icon → **Settings** to configure features.

---

## Features

### ✅ Desktop Icon Toggler
- Double-click desktop to show/hide icons
- Quick access from system tray
- Zero performance impact when disabled

### ✅ Cursor Hider
- Auto-hide cursor after inactivity
- Configurable timeout (1-60 seconds)
- Instant show on mouse movement

### ✅ Lightweight Design
- Core app: only 471 KB
- Modern settings UI with Tauri + React
- Disabled features consume zero resources

### 🚀 Coming Soon
- 📋 Clipboard Manager - Save, edit, and manage clipboard history
- 🔧 More system utilities in development...

---

## Documentation

- **[Settings System](Docs/settings.md)** - Architecture, configuration, and how settings work
- **[Build Scripts](Docs/build-scripts.md)** - Complete build process documentation

---

## Project Structure

```
├── src/
│   ├── main.rs              # Main application (Win32)
│   └── cursor_hider.rs      # Cursor hider module
├── settings-ui/             # Tauri settings application
│   ├── src/                 # React frontend
│   └── src-tauri/           # Tauri backend
├── target/release/          # Build output
│   ├── windows_multitool.exe
│   └── settings.exe
├── dist/release/            # Distribution package
├── Docs/                    # Documentation
├── build.ps1                # PowerShell build script
└── build.bat                # Quick build batch file
```

---

## Tech Stack

- **Core App:** Pure Rust + Win32 API (ultra-lightweight)
- **Settings UI:** Tauri 2 + React + TypeScript
- **Package Manager:** Bun
- **Configuration:** JSON file in %APPDATA%
- **Architecture:** Modular design with dynamic feature loading

---

## License

MIT

---

## Quick Build Commands

```powershell
# Development build
.\build.ps1 -Release:$false

# Release build
.\build.ps1

# With distribution package
.\build.ps1 -Distribute

# Clean rebuild
.\build.ps1 -Clean

# Full production build
.\build.ps1 -Clean -Distribute
```
