#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness};
use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

use winreg::RegKey;
use winreg::enums::*;

mod cursor_hider;

static mut LAST_CLICK_TIME: Option<Instant> = None;

const WM_TRAYICON: u32 = WM_APP + 1;
const TRAY_EXIT_ID: usize = 1001;
const TRAY_AUTORUN_ID: usize = 1002;
const APP_REGISTRY_NAME: &str = "DesktopIconTogglerApp";

const TRAY_SETTINGS_ID: usize = 1004; // ID для кнопки "Настройки"
const WM_RELOAD_SETTINGS: u32 = WM_APP + 2; // Секретный сигнал от UI к Ядру

// Configuration structures
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

// Глобальное состояние (включены ли функции)
static mut IS_DESKTOP_TOGGLER_ENABLED: bool = true;
static mut IS_CURSOR_HIDER_ENABLED: bool = false;
static mut CURSOR_HIDER_TIMEOUT: u64 = 5;

// Get settings file path
fn get_settings_path() -> PathBuf {
    let app_data = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(app_data);
    path.push("DesktopIconToggler");
    path.push("settings.json");
    path
}

// Create default settings file
fn create_default_settings(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let default_config = AppConfig::default();
    if let Ok(json) = serde_json::to_string_pretty(&default_config) {
        let _ = fs::write(path, json);
    }
}

// Функция для чтения файла settings.json
fn load_settings() {
    let settings_path = get_settings_path();
    
    // Create default if not exists
    if !settings_path.exists() {
        create_default_settings(&settings_path);
    }
    
    // Read and parse settings
    match fs::read_to_string(&settings_path) {
        Ok(contents) => {
            match serde_json::from_str::<AppConfig>(&contents) {
                Ok(config) => {
                    unsafe {
                        IS_DESKTOP_TOGGLER_ENABLED = config.features.desktop_toggler;
                        IS_CURSOR_HIDER_ENABLED = config.features.cursor_hider;
                        CURSOR_HIDER_TIMEOUT = config.cursor_hider.timeout_seconds;
                    }
                    println!("[Ядро] Настройки загружены успешно:");
                    println!("  - Desktop Toggler: {}", config.features.desktop_toggler);
                    println!("  - Cursor Hider: {} (timeout: {}s)", 
                             config.features.cursor_hider, 
                             config.cursor_hider.timeout_seconds);
                    
                    // Start or stop cursor hider based on settings
                    if config.features.cursor_hider {
                        cursor_hider::start_cursor_hider(config.cursor_hider.timeout_seconds);
                    } else {
                        cursor_hider::stop_cursor_hider();
                    }
                }
                Err(e) => {
                    eprintln!("[Ядро] Ошибка парсинга настроек: {}", e);
                    eprintln!("[Ядро] Используются настройки по умолчанию");
                    create_default_settings(&settings_path);
                }
            }
        }
        Err(e) => {
            eprintln!("[Ядро] Ошибка чтения файла настроек: {}", e);
            create_default_settings(&settings_path);
        }
    }
}

fn is_autorun_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run_key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
        if let Ok(value) = run_key.get_value::<String, _>(APP_REGISTRY_NAME) {
            if let Ok(exe_path) = env::current_exe() {
                return value == exe_path.to_string_lossy().to_string();
            }
        }
    }
    false
}

fn toggle_autorun() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok((run_key, _)) =
        hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
    {
        if is_autorun_enabled() {
            let _ = run_key.delete_value(APP_REGISTRY_NAME);
        } else {
            if let Ok(exe_path) = env::current_exe() {
                let _ =
                    run_key.set_value(APP_REGISTRY_NAME, &exe_path.to_string_lossy().to_string());
            }
        }
    }
}

fn toggle_icons() {
    unsafe {
        let desktop_hwnd = FindWindowW(w!("Progman"), None);
        let mut def_view = FindWindowExW(desktop_hwnd, HWND(0), w!("SHELLDLL_DefView"), None);

        if def_view.0 == 0 {
            let mut worker_w = FindWindowExW(HWND(0), HWND(0), w!("WorkerW"), None);
            while worker_w.0 != 0 {
                def_view = FindWindowExW(worker_w, HWND(0), w!("SHELLDLL_DefView"), None);
                if def_view.0 != 0 {
                    break;
                }
                worker_w = FindWindowExW(HWND(0), worker_w, w!("WorkerW"), None);
            }
        }

        if def_view.0 != 0 {
            let _ = SendMessageW(def_view, WM_COMMAND, WPARAM(0x7402), LPARAM(0));
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_RELOAD_SETTINGS => {
                // Если UI прислал этот сигнал, значит пользователь нажал "Сохранить"
                load_settings(); 
                LRESULT(0)
            }

            WM_TRAYICON => {
                let event = lparam.0 as u32;
                match event {
                    WM_LBUTTONUP => {
                        toggle_icons();
                    }
                    WM_RBUTTONUP => {
                        let mut pt = POINT::default();
                        let _ = GetCursorPos(&mut pt);

                        let hmenu = CreatePopupMenu().unwrap();

                        let mut autorun_flags = MF_BYPOSITION | MF_STRING;
                        if is_autorun_enabled() {
                            autorun_flags |= MF_CHECKED;
                        } else {
                            autorun_flags |= MF_UNCHECKED;
                        }

                        // 1. Кнопка "Настройки"
                        let _ = InsertMenuW(hmenu, 0, MF_BYPOSITION | MF_STRING, TRAY_SETTINGS_ID, w!("Settings..."));
                        let _ = InsertMenuW(hmenu, 1, MF_BYPOSITION | MF_SEPARATOR, 0, w!(""));

                        // 2. Кнопка автозагрузки
                        let mut autorun_flags = MF_BYPOSITION | MF_STRING;
                        if is_autorun_enabled() { autorun_flags |= MF_CHECKED; } else { autorun_flags |= MF_UNCHECKED; }
                        let _ = InsertMenuW(hmenu, 2, autorun_flags, TRAY_AUTORUN_ID, w!("Run at startup"));
                        let _ = InsertMenuW(hmenu, 3, MF_BYPOSITION | MF_SEPARATOR, 0, w!(""));

                        // 3. Кнопка выхода
                        let _ = InsertMenuW(hmenu, 4, MF_BYPOSITION | MF_STRING, TRAY_EXIT_ID, w!("Exit"));

                        SetForegroundWindow(hwnd);

                        let _ = TrackPopupMenu(hmenu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, pt.x, pt.y, 0, hwnd, None);
                        let _ = DestroyMenu(hmenu);
                    }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as usize;
                if id == TRAY_EXIT_ID {
                    PostQuitMessage(0);
                } else if id == TRAY_AUTORUN_ID {
                    toggle_autorun();
                } else if id == TRAY_SETTINGS_ID {
                    // Launch settings UI in a separate thread
                    std::thread::spawn(|| {
                        if let Ok(exe_path) = env::current_exe() {
                            if let Some(exe_dir) = exe_path.parent() {
                                let settings_path = exe_dir.join("settings.exe");
                                if settings_path.exists() {
                                    let _ = std::process::Command::new(settings_path).spawn();
                                } else {
                                    eprintln!("[Ядро] Settings app not found at: {:?}", settings_path);
                                }
                            }
                        }
                    });
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        // If code < 0, we must pass to next hook without processing
        if code < 0 {
            return CallNextHookEx(HHOOK(0), code, wparam, lparam);
        }
        
        // Track mouse activity for cursor hider (regardless of which features are enabled)
        if IS_CURSOR_HIDER_ENABLED {
            // Mouse activity detected - cursor hider thread will detect position changes
            // This is handled by the cursor_hider module's position tracking
        }
        
        // Check if desktop toggler is enabled before processing desktop clicks
        if !IS_DESKTOP_TOGGLER_ENABLED {
            // Desktop toggler is disabled, pass through without processing
            return CallNextHookEx(HHOOK(0), code, wparam, lparam);
        }
        
        // Process desktop icon toggler logic
        if wparam.0 as u32 == WM_LBUTTONDOWN {
            let hook_struct = *(lparam.0 as *const MSLLHOOKSTRUCT);
            let hwnd_under_cursor = WindowFromPoint(hook_struct.pt);

            let mut class_name = [0u16; 256];
            let len = GetClassNameW(hwnd_under_cursor, &mut class_name);
            let class_string = String::from_utf16_lossy(&class_name[..len as usize]);

            if class_string == "SysListView32"
                || class_string == "WorkerW"
                || class_string == "Progman"
                || class_string == "SHELLDLL_DefView"
            {
                let now = Instant::now();
                let dc_time = GetDoubleClickTime();

                if let Some(last) = LAST_CLICK_TIME {
                    let diff = now.duration_since(last).as_millis();
                    if diff < dc_time as u128 {
                        let mut selected_count = 0;
                        if class_string == "SysListView32" {
                            selected_count =
                                SendMessageW(hwnd_under_cursor, 0x1032, WPARAM(0), LPARAM(0)).0;
                        }

                        if selected_count == 0 {
                            toggle_icons();
                        }

                        LAST_CLICK_TIME = None;
                        return CallNextHookEx(HHOOK(0), code, wparam, lparam);
                    }
                }
                LAST_CLICK_TIME = Some(now);
            } else {
                LAST_CLICK_TIME = None;
            }
        }
    }
    unsafe { CallNextHookEx(HHOOK(0), code, wparam, lparam) }
}

fn main() -> Result<()> {
    unsafe {
        // Load settings at startup
        load_settings();
        
        let _ = SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE);
        let module = GetModuleHandleW(None)?;

        let class_name = w!("DesktopTogglerClass");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: module.into(),
            lpszClassName: class_name,
            ..Default::default()
        };
        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("DesktopTogglerWindow"),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            module,
            None,
        );

        let mut tooltip = [0u16; 128];
        let text = "Double click on desktop\0"
            .encode_utf16()
            .collect::<Vec<_>>();
        tooltip[..text.len()].copy_from_slice(&text);

        let nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: hwnd,
            uID: 1,
            uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: LoadIconW(module, PCWSTR(1 as *const u16)).unwrap(),
            szTip: tooltip,
            ..Default::default()
        };

        Shell_NotifyIconW(NIM_ADD, &nid);

        let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), module, 0)?;

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnhookWindowsHookEx(hook);
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }
    Ok(())
}
