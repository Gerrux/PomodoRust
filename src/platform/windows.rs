//! Windows-specific functionality

use std::env;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
};
use windows::Win32::UI::Controls::MARGINS;
use winreg::enums::*;
use winreg::RegKey;

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
        let corner_preference = DWMWCP_ROUND.0 as i32;
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

/// Set application to start with Windows
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_WRITE,
        )
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    if enabled {
        let exe_path = env::current_exe()
            .map_err(|e| format!("Failed to get executable path: {}", e))?;

        run_key
            .set_value("PomodoRust", &exe_path.to_string_lossy().to_string())
            .map_err(|e| format!("Failed to set registry value: {}", e))?;

        tracing::info!("Enabled autostart");
    } else {
        remove_autostart()?;
    }

    Ok(())
}

/// Remove application from Windows startup
pub fn remove_autostart() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = hkcu
        .open_subkey_with_flags(
            r"Software\Microsoft\Windows\CurrentVersion\Run",
            KEY_WRITE,
        )
        .map_err(|e| format!("Failed to open registry key: {}", e))?;

    // Ignore error if value doesn't exist
    let _ = run_key.delete_value("PomodoRust");

    tracing::info!("Disabled autostart");
    Ok(())
}
