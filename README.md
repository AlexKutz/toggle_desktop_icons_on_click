# Desktop Icon Toggler

Toggle desktop icons visibility and manage system preferences with a lightweight Windows application.

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
.\desktop_icon_toggler.exe
```

Right-click the tray icon → **Settings** to configure features.

---

## Features

- ✅ **Desktop Icon Toggler** - Double-click desktop to show/hide icons
- ✅ **Cursor Hider** - Auto-hide cursor after inactivity
- ✅ **Lightweight** - Only 471 KB core application
- ✅ **Modern Settings UI** - Beautiful Tauri + React interface
- ✅ **Zero overhead** - Disabled features consume no resources

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
│   ├── desktop_icon_toggler.exe
│   └── settings.exe
├── dist/release/            # Distribution package
├── Docs/                    # Documentation
├── build.ps1                # PowerShell build script
└── build.bat                # Quick build batch file
```

---

## Tech Stack

- **Main App:** Pure Rust + Win32 API
- **Settings UI:** Tauri 2 + React + TypeScript
- **Package Manager:** Bun
- **Configuration:** JSON file in %APPDATA%

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
