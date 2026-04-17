use std::time::Instant;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Global flag to control the cursor hider thread
static CURSOR_HIDER_RUNNING: AtomicBool = AtomicBool::new(false);

/// Start the cursor hider in a separate thread
/// This will hide the cursor after the specified timeout of inactivity
pub fn start_cursor_hider(timeout_seconds: u64) {
    // Stop existing thread if running
    stop_cursor_hider();
    
    // Mark as running
    CURSOR_HIDER_RUNNING.store(true, Ordering::SeqCst);
    
    thread::spawn(move || {
        let timeout = Duration::from_secs(timeout_seconds);
        let mut last_activity = Instant::now();
        let mut cursor_hidden = false;
        let mut last_position = POINT::default();
        
        // Get initial cursor position
        unsafe {
            let _ = GetCursorPos(&mut last_position);
        }
        
        println!("[CursorHider] Started with timeout: {} seconds", timeout_seconds);
        
        while CURSOR_HIDER_RUNNING.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(100)); // Check every 100ms
            
            unsafe {
                let mut current_position = POINT::default();
                if GetCursorPos(&mut current_position).is_ok() {
                    // Check if cursor moved
                    if current_position.x != last_position.x || current_position.y != last_position.y {
                        // Cursor moved - show it if hidden
                        if cursor_hidden {
                            show_cursor();
                            cursor_hidden = false;
                            println!("[CursorHider] Cursor shown (movement detected)");
                        }
                        last_activity = Instant::now();
                        last_position = current_position;
                    }
                }
                
                // Check if timeout elapsed
                if !cursor_hidden && last_activity.elapsed() >= timeout {
                    hide_cursor();
                    cursor_hidden = true;
                    println!("[CursorHider] Cursor hidden (timeout reached)");
                }
            }
        }
        
        // Make sure cursor is shown when thread stops
        if cursor_hidden {
            show_cursor();
        }
        
        println!("[CursorHider] Thread stopped");
    });
}

/// Stop the cursor hider thread
pub fn stop_cursor_hider() {
    if CURSOR_HIDER_RUNNING.load(Ordering::SeqCst) {
        CURSOR_HIDER_RUNNING.store(false, Ordering::SeqCst);
        println!("[CursorHider] Stopping thread...");
    }
}

/// Hide the cursor
fn hide_cursor() {
    // ShowCursor returns negative values when hiding, positive when showing
    // We need to call it until it returns negative (cursor is hidden)
    unsafe {
        while ShowCursor(FALSE) >= 0 {
            // Keep calling until cursor is hidden
        }
    }
}

/// Show the cursor
fn show_cursor() {
    // ShowCursor returns positive values when showing
    // We need to call it until it returns non-negative (cursor is shown)
    unsafe {
        while ShowCursor(TRUE) < 0 {
            // Keep calling until cursor is shown
        }
    }
}
