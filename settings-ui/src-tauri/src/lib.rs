use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Configuration structures (must match main app)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub features: FeatureFlags,
    pub cursor_hider: CursorHiderConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeatureFlags {
    pub desktop_toggler: bool,
    pub cursor_hider: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CursorHiderConfig {
    pub timeout_seconds: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            features: FeatureFlags {
                desktop_toggler: true,
                cursor_hider: false,
            },
            cursor_hider: CursorHiderConfig {
                timeout_seconds: 5,
            },
        }
    }
}

// Get settings file path
fn get_settings_path() -> PathBuf {
    let app_data = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(app_data);
    path.push("DesktopIconToggler");
    path.push("settings.json");
    path
}

// Load configuration from file
#[tauri::command]
fn load_config() -> Result<AppConfig, String> {
    let settings_path = get_settings_path();
    
    if !settings_path.exists() {
        // Return default config if file doesn't exist
        return Ok(AppConfig::default());
    }
    
    match fs::read_to_string(&settings_path) {
        Ok(contents) => {
            match serde_json::from_str::<AppConfig>(&contents) {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to parse config: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to read config file: {}", e)),
    }
}

// Save configuration to file
#[tauri::command]
fn save_config(config: AppConfig) -> Result<String, String> {
    let settings_path = get_settings_path();
    
    // Create directory if it doesn't exist
    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    // Write config to file
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&settings_path, json)
        .map_err(|e| format!("Failed to write config file: {}", e))?;
    
    // Notify main app to reload settings
    notify_main_app_reload();
    
    Ok("Settings saved successfully".to_string())
}

// Notify main app to reload settings via Windows message
fn notify_main_app_reload() {
    unsafe {
        // Find the main app window by class name
        let class_name = windows::core::w!("DesktopTogglerClass");
        let hwnd = FindWindowW(class_name, None);
        
        if hwnd.0 != 0 {
            // WM_RELOAD_SETTINGS = WM_APP + 2 = 0x8000 + 2 = 0x8002
            const WM_RELOAD_SETTINGS: u32 = 0x8002;
            let _ = PostMessageW(hwnd, WM_RELOAD_SETTINGS, WPARAM(0), LPARAM(0));
            println!("[Settings] Sent reload signal to main app");
        } else {
            println!("[Settings] Main app window not found (main app may not be running)");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![load_config, save_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
