#![windows_subsystem = "windows"]

use std::time::Instant;
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{PROCESS_PER_MONITOR_DPI_AWARE, SetProcessDpiAwareness};
use windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

static mut LAST_CLICK_TIME: Option<Instant> = None;

const WM_TRAYICON: u32 = WM_APP + 1;
const TRAY_EXIT_ID: usize = 1001;

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
            println!(">>> Icons toggled! <<<");
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
                        let _ = InsertMenuW(
                            hmenu,
                            0,
                            MF_BYPOSITION | MF_STRING,
                            TRAY_EXIT_ID,
                            w!("Exit!"),
                        );

                        SetForegroundWindow(hwnd);

                        let _ = TrackPopupMenu(
                            hmenu,
                            TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                            pt.x,
                            pt.y,
                            0,
                            hwnd,
                            None,
                        );
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
    if code >= 0 && wparam.0 as u32 == WM_LBUTTONDOWN {
        unsafe {
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
        let text = "Toggle icons\0".encode_utf16().collect::<Vec<_>>();
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

        println!("Started");

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
