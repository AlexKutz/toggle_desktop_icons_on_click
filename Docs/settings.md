# Settings System Documentation

## Overview

The Desktop Icon Toggler application uses a dual-application architecture with a lightweight core and a separate settings UI. This design keeps the main application minimal while providing a modern, user-friendly configuration interface.

## Architecture

### Two Separate Applications

1. **desktop_icon_toggler.exe** (471 KB)
   - Lightweight Win32 core application
   - Runs in system tray
   - Handles all desktop functionality (icon toggling, cursor hiding)
   - Reads settings from JSON file at startup and on-demand
   - Zero UI overhead - completely headless

2. **settings.exe** (8.6 MB)
   - Tauri 2 application with React frontend
   - Modern settings interface
   - Only runs when user opens settings
   - Reads/writes JSON configuration file
   - Communicates with main app via Windows messages

### Communication Flow

```
┌─────────────────────┐
│   settings.exe      │
│   (Tauri + React)   │
│                     │
│  1. Load config     │◄─── Read from JSON file
│  2. User changes    │
│  3. Save config     │──── Write to JSON file
│  4. Send WM message │──── PostMessageW() to main app
└─────────────────────┘

┌─────────────────────┐
│ desktop_icon_toggler│
│   (Win32 Core)      │
│                     │
│  5. Receive WM msg  │◄─── WM_RELOAD_SETTINGS (0x8002)
│  6. Reload config   │──── Read from JSON file
│  7. Apply settings  │──── Start/stop features
└─────────────────────┘
```

## Configuration File

### Location

```
%APPDATA%/DesktopIconToggler/settings.json
```

Typically: `C:\Users\[username]\AppData\Roaming\DesktopIconToggler\settings.json`

### Structure

```json
{
  "features": {
    "desktop_toggler": true,
    "cursor_hider": false
  },
  "cursor_hider": {
    "timeout_seconds": 5
  }
}
```

### Configuration Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `features.desktop_toggler` | boolean | `true` | Enable/disable desktop icon toggling on double-click |
| `features.cursor_hider` | boolean | `false` | Enable/disable automatic cursor hiding |
| `cursor_hider.timeout_seconds` | number | `5` | Seconds of inactivity before hiding cursor (1-60) |

## Implementation Details

### Main App (desktop_icon_toggler.exe)

#### 1. Settings Loading (`src/main.rs`)

**Configuration Structures:**

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AppConfig {
    features: FeatureFlags,
    cursor_hider: CursorHiderConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct FeatureFlags {
    desktop_toggler: bool,
    cursor_hider: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CursorHiderConfig {
    timeout_seconds: u64,
}
```

**Global State:**

```rust
static mut IS_DESKTOP_TOGGLER_ENABLED: bool = true;
static mut IS_CURSOR_HIDER_ENABLED: bool = false;
static mut CURSOR_HIDER_TIMEOUT: u64 = 5;
```

**Loading Process (`load_settings()`):**

1. Get settings file path from `%APPDATA%`
2. If file doesn't exist, create with default values
3. Read and parse JSON file
4. Update global flags with new values
5. Start/stop cursor hider based on settings
6. Log success or errors to console

**Error Handling:**
- If file read fails → create default settings
- If JSON parsing fails → log error, create default settings
- Always falls back to safe defaults

#### 2. Feature Flag Checks

**Mouse Hook Optimization (`src/main.rs`):**

```rust
unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // Early return if code < 0 (required by Windows API)
    if code < 0 {
        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
    }
    
    // If desktop toggler disabled, skip all processing
    if !IS_DESKTOP_TOGGLER_ENABLED {
        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
    }
    
    // ... process desktop clicks only if enabled
}
```

**Performance Impact:**
- When feature disabled: ~1 microsecond (just flag check)
- When feature enabled: full processing
- Zero overhead for disabled features

#### 3. Cursor Hider Module (`src/cursor_hider.rs`)

**How It Works:**

1. Runs in separate background thread
2. Checks cursor position every 100ms
3. Tracks last movement time
4. Hides cursor after timeout of inactivity
5. Shows cursor immediately on movement

**Thread Control:**

```rust
// Atomic flag for thread-safe control
static CURSOR_HIDER_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn start_cursor_hider(timeout_seconds: u64) {
    stop_cursor_hider(); // Stop existing thread
    CURSOR_HIDER_RUNNING.store(true, Ordering::SeqCst);
    
    thread::spawn(move || {
        // Monitor cursor position
        // Hide after timeout
        // Show on movement
    });
}

pub fn stop_cursor_hider() {
    CURSOR_HIDER_RUNNING.store(false, Ordering::SeqCst);
    // Thread exits loop and ensures cursor is shown
}
```

**Lifecycle:**
- Started when app loads settings with `cursor_hider: true`
- Stopped when settings change to `cursor_hider: false`
- Automatically shows cursor when stopped
- Re-started with new timeout if settings change

#### 4. Settings Reload via Windows Message

**Message Handler (`wnd_proc`):**

```rust
const WM_RELOAD_SETTINGS: u32 = WM_APP + 2; // 0x8002

WM_RELOAD_SETTINGS => {
    load_settings(); // Reload from JSON file
    LRESULT(0)
}
```

**When Triggered:**
- Settings app sends message after saving
- Main app receives in window procedure
- Calls `load_settings()` to apply new config
- Features start/stop immediately

### Settings UI (settings.exe)

#### 1. Tauri Backend (`settings-ui/src-tauri/src/lib.rs`)

**Commands:**

```rust
#[tauri::command]
fn load_config() -> Result<AppConfig, String> {
    // Read from %APPDATA%/DesktopIconToggler/settings.json
    // Return parsed config or default
}

#[tauri::command]
fn save_config(config: AppConfig) -> Result<String, String> {
    // Write config to JSON file
    // Call notify_main_app_reload()
    // Return success message
}
```

**Notification Function:**

```rust
fn notify_main_app_reload() {
    unsafe {
        // Find main app window by class name
        let class_name = w!("DesktopTogglerClass");
        let hwnd = FindWindowW(class_name, None);
        
        if hwnd.0 != 0 {
            // Send reload message
            const WM_RELOAD_SETTINGS: u32 = 0x8002;
            PostMessageW(hwnd, WM_RELOAD_SETTINGS, WPARAM(0), LPARAM(0));
        }
    }
}
```

#### 2. React Frontend (`settings-ui/src/App.tsx`)

**State Management:**

```typescript
interface AppConfig {
  features: {
    desktop_toggler: boolean;
    cursor_hider: boolean;
  };
  cursor_hider: {
    timeout_seconds: number;
  };
}

const [config, setConfig] = useState<AppConfig | null>(null);
```

**Loading Config:**

```typescript
useEffect(() => {
  loadConfig(); // Called on component mount
}, []);

async function loadConfig() {
  const loadedConfig = await invoke<AppConfig>("load_config");
  setConfig(loadedConfig);
}
```

**Saving Config:**

```typescript
async function saveConfig() {
  const result = await invoke<string>("save_config", { config });
  // Shows success message
  // Main app receives WM_RELOAD_SETTINGS automatically
}
```

**UI Components:**

1. **Desktop Icon Toggler Toggle**
   - Checkbox styled as toggle switch
   - Controls `config.features.desktop_toggler`

2. **Cursor Hider Toggle**
   - Checkbox styled as toggle switch
   - Controls `config.features.cursor_hider`
   - Shows timeout input when enabled

3. **Cursor Hide Timeout**
   - Number input (1-60 seconds)
   - Only visible when cursor hider is enabled
   - Controls `config.cursor_hider.timeout_seconds`

4. **Save Button**
   - Triggers `save_config()` Tauri command
   - Shows loading state while saving
   - Displays success/error message

## Build Process

### Main App

```bash
cargo build --release
```

Output: `target/release/desktop_icon_toggler.exe`

### Settings UI

```bash
cd settings-ui
bun install
bun run tauri build
```

Output: `settings-ui/src-tauri/target/release/settings-ui.exe`

### Deployment

Copy settings executable to main app directory:

```bash
Copy-Item "settings-ui/src-tauri/target/release/settings-ui.exe" "target/release/settings.exe"
```

Both executables must be in the same directory for the tray menu to launch settings.

## User Workflow

### First Run

1. User runs `desktop_icon_toggler.exe`
2. App checks for settings file → not found
3. Creates default settings:
   ```json
   {
     "features": {
       "desktop_toggler": true,
       "cursor_hider": false
     },
     "cursor_hider": {
       "timeout_seconds": 5
     }
   }
   ```
4. Applies default settings
5. Shows tray icon

### Changing Settings

1. User right-clicks tray icon
2. Selects "Settings..."
3. `settings.exe` launches
4. UI loads current config from JSON file
5. User toggles features or adjusts timeout
6. User clicks "Save Settings"
7. Settings app:
   - Writes new config to JSON file
   - Sends `WM_RELOAD_SETTINGS` to main app
8. Main app:
   - Receives Windows message
   - Reloads settings from JSON file
   - Starts/stops features as needed
9. User sees success message in settings UI

### Feature Examples

**Disable Desktop Toggler:**
- Set `desktop_toggler: false`
- Mouse hook returns immediately
- Double-click on desktop does nothing
- Near-zero CPU usage from hook

**Enable Cursor Hider:**
- Set `cursor_hider: true`
- Set `timeout_seconds: 3`
- Background thread starts
- Cursor hides after 3 seconds of no movement
- Cursor shows immediately on mouse move

**Change Timeout:**
- Modify `timeout_seconds` from 5 to 10
- Cursor hider thread stops
- Restarts with new 10-second timeout
- No app restart needed

## Technical Details

### Windows API Usage

| Function | Purpose | Used In |
|----------|---------|---------|
| `FindWindowW` | Find main app window | Settings app notification |
| `PostMessageW` | Send reload message | Settings app → Main app |
| `GetCursorPos` | Track cursor position | Cursor hider module |
| `ShowCursor` | Hide/show cursor | Cursor hider module |
| `SetWindowsHookExW` | Install mouse hook | Main app mouse monitoring |

### Message Constants

```rust
WM_APP = 0x8000
WM_RELOAD_SETTINGS = WM_APP + 2 = 0x8002
```

### Thread Safety

- Cursor hider uses `AtomicBool` for thread-safe control
- Global settings flags are `static mut` (accessed only from main thread)
- Settings file I/O happens on separate threads
- Windows message passing is thread-safe

### Error Handling

**Settings File Missing:**
- Creates file with default values
- Logs to console
- Continues with defaults

**Invalid JSON:**
- Logs parsing error
- Overwrites with default settings
- Prevents app crash

**Main App Not Running:**
- Settings app saves file successfully
- Warning logged if can't send message
- Next main app startup will load new settings

## Future Extensibility

### Adding New Features

1. **Add to config structure:**
   ```rust
   struct FeatureFlags {
       desktop_toggler: bool,
       cursor_hider: bool,
       new_feature: bool, // Add here
   }
   ```

2. **Add to settings UI:**
   ```tsx
   <Toggle 
     checked={config.features.new_feature}
     onChange={(e) => setConfig({
       ...config,
       features: { ...config.features, new_feature: e.target.checked }
     })}
   />
   ```

3. **Add flag check in main app:**
   ```rust
   if !IS_NEW_FEATURE_ENABLED {
       return; // Early return if disabled
   }
   ```

4. **Update default settings** in `impl Default for AppConfig`

### Module Structure

```
src/
├── main.rs              # Entry point, tray, hooks
├── cursor_hider.rs      # Cursor hider module
└── [new_feature].rs     # Future feature modules

settings-ui/
├── src/
│   ├── App.tsx          # React UI
│   └── App.css          # Styles
└── src-tauri/
    └── src/
        └── lib.rs       # Tauri commands
```

## Troubleshooting

### Settings Not Applying

**Problem:** Changes in settings UI don't affect main app

**Solutions:**
1. Check if main app is running (tray icon visible)
2. Restart main app - it will load latest settings
3. Check console output for error messages
4. Verify settings file exists at `%APPDATA%/DesktopIconToggler/settings.json`

### Settings File Location

Find the file:
```powershell
Get-Content "$env:APPDATA\DesktopIconToggler\settings.json"
```

Reset to defaults:
```powershell
Remove-Item "$env:APPDATA\DesktopIconToggler\settings.json" -Force
# Restart main app to recreate with defaults
```

### Settings App Won't Launch

**Problem:** Clicking "Settings..." does nothing

**Solutions:**
1. Verify `settings.exe` is in same directory as `desktop_icon_toggler.exe`
2. Check console output: `[Ядро] Settings app not found at: ...`
3. Rebuild and copy settings executable

### Cursor Not Hiding

**Problem:** Cursor hider enabled but cursor stays visible

**Solutions:**
1. Check settings: `cursor_hider` must be `true`
2. Wait for timeout period (default 5 seconds)
3. Move mouse to reset timer
4. Check console for: `[CursorHider] Started with timeout: X seconds`

## Performance Characteristics

### Memory Usage

| Component | Memory |
|-----------|--------|
| Main app (idle) | ~5 MB |
| Main app (with cursor hider) | ~6 MB |
| Settings app | ~50 MB |

### CPU Usage

| Scenario | CPU |
|----------|-----|
| Main app (all features off) | < 0.1% |
| Main app (desktop toggler on) | < 0.1% |
| Main app (cursor hider on) | ~0.2% |
| Settings app (open) | ~1-2% |

### Why This Architecture?

**Separate Settings App Benefits:**
- Main app stays lightweight (471 KB vs 8.6 MB)
- No UI framework overhead in main app
- Settings UI only uses resources when open
- Modern React UI without bloating core app
- Easy to update UI without touching core logic

**JSON Config Benefits:**
- Human-readable format
- Easy to backup/restore
- Can edit manually if needed
- Cross-platform compatible
- No registry dependencies

**Windows Message IPC Benefits:**
- Zero dependencies
- Extremely fast (< 1ms)
- Built into Windows API
- No network ports needed
- Secure (local only)
