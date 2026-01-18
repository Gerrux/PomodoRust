//! Windows-specific functionality
//!
//! Provides Windows-specific features including:
//! - DWM (Desktop Window Manager) effects for dark mode and rounded corners
//! - Native toast notifications
//! - Autostart via registry
//! - Window flash for timer completion

use std::env;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
};
use windows::Win32::UI::Controls::MARGINS;
use windows::Win32::UI::WindowsAndMessaging::{
    FlashWindowEx, FLASHWINFO, FLASHW_ALL, FLASHW_TIMERNOFG,
};
use winreg::enums::*;
use winreg::RegKey;

use crate::error::PlatformError;

/// Apply Windows DWM effects (shadow and rounded corners)
pub fn apply_window_effects(hwnd: isize) {
    unsafe {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

        // Enable dark mode for window frame
        let dark_mode: i32 = 1;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        );

        // Enable rounded corners (Windows 11)
        let corner_preference = DWMWCP_ROUND.0;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner_preference as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<i32>() as u32,
        );

        // Extend frame into client area to enable shadow
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

        tracing::info!("Applied Windows DWM effects");
    }
}

/// Show a Windows toast notification
pub fn show_notification(title: &str, body: &str) {
    if let Err(e) = notify_rust::Notification::new()
        .summary(title)
        .body(body)
        .appname("PomodoRust")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show()
    {
        tracing::error!("Failed to show notification: {}", e);
    }
}

/// Flash the window in taskbar to get user attention
/// This is called when timer completes to notify the user
pub fn flash_window(hwnd: isize, count: u32) {
    unsafe {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

        let flash_info = FLASHWINFO {
            cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
            hwnd,
            dwFlags: FLASHW_ALL | FLASHW_TIMERNOFG,
            uCount: count,
            dwTimeout: 0, // Use default cursor blink rate
        };

        let _ = FlashWindowEx(&flash_info);
        tracing::info!("Flashing window {} times", count);
    }
}

/// Stop flashing the window
pub fn stop_flash_window(hwnd: isize) {
    unsafe {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);

        let flash_info = FLASHWINFO {
            cbSize: std::mem::size_of::<FLASHWINFO>() as u32,
            hwnd,
            dwFlags: windows::Win32::UI::WindowsAndMessaging::FLASHW_STOP,
            uCount: 0,
            dwTimeout: 0,
        };

        let _ = FlashWindowEx(&flash_info);
    }
}

/// Flash the PomodoRust window by finding it by title
/// Returns true if window was found and flashed
pub fn flash_pomodorust_window(count: u32) -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

    unsafe {
        let title: Vec<u16> = "PomodoRust\0".encode_utf16().collect();
        if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) {
            if !hwnd.is_invalid() {
                flash_window(hwnd.0 as isize, count);
                return true;
            }
        }
    }
    false
}

/// Show and bring the PomodoRust window to foreground
/// Returns true if window was found and shown
pub fn show_pomodorust_window() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::UI::WindowsAndMessaging::{
        FindWindowW, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };

    unsafe {
        let title: Vec<u16> = "PomodoRust\0".encode_utf16().collect();
        if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) {
            if !hwnd.is_invalid() {
                // Restore window if minimized
                let _ = ShowWindow(hwnd, SW_RESTORE);
                // Bring to foreground
                let _ = SetForegroundWindow(hwnd);
                tracing::info!("Restored PomodoRust window");
                return true;
            }
        }
    }
    tracing::warn!("Could not find PomodoRust window");
    false
}

/// Registry key path for Windows autostart
const AUTOSTART_REGISTRY_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Application name in registry
const APP_REGISTRY_NAME: &str = "PomodoRust";

/// Set application to start with Windows
pub fn set_autostart(enabled: bool) -> Result<(), PlatformError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(AUTOSTART_REGISTRY_KEY, KEY_WRITE)
        .map_err(|e| PlatformError::Registry {
            operation: "open",
            message: e.to_string(),
        })?;

    if enabled {
        let exe_path =
            env::current_exe().map_err(|e| PlatformError::ExecutablePath { source: e })?;

        run_key
            .set_value(APP_REGISTRY_NAME, &exe_path.to_string_lossy().to_string())
            .map_err(|e| PlatformError::Registry {
                operation: "set_value",
                message: e.to_string(),
            })?;

        tracing::info!("Enabled autostart");
    } else {
        remove_autostart()?;
    }

    Ok(())
}

/// Remove application from Windows startup
pub fn remove_autostart() -> Result<(), PlatformError> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(AUTOSTART_REGISTRY_KEY, KEY_WRITE)
        .map_err(|e| PlatformError::Registry {
            operation: "open",
            message: e.to_string(),
        })?;

    // Ignore error if value doesn't exist
    let _ = run_key.delete_value(APP_REGISTRY_NAME);

    tracing::info!("Disabled autostart");
    Ok(())
}
