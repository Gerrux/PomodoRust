//! Windows-specific functionality
//!
//! Provides Windows-specific features including:
//! - DWM (Desktop Window Manager) effects for dark mode and rounded corners
//! - Native toast notifications
//! - Autostart via registry
//! - Window flash for timer completion
//!
//! ## Windows Version Compatibility
//!
//! | Build      | Version | Dark Mode Attr | Rounded Corners | Shadow |
//! |------------|---------|----------------|-----------------|--------|
//! | < 17763    | < 1809  | None           | No              | DWM    |
//! | 17763-18985| 1809-1903| Attr 19       | No              | DWM    |
//! | 18986-21999| 1903-21H2| Attr 20       | No              | DWM    |
//! | >= 22000   | Win 11  | Attr 20        | Native          | Native |

use std::env;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DwmSetWindowAttribute;
use windows::Win32::UI::WindowsAndMessaging::{
    FlashWindowEx, FLASHWINFO, FLASHW_ALL, FLASHW_TIMERNOFG,
};
use winreg::enums::*;
use winreg::RegKey;

use crate::error::PlatformError;

use std::sync::OnceLock;

/// Cached Windows build number (avoids repeated registry reads)
static WINDOWS_BUILD: OnceLock<u32> = OnceLock::new();

/// Get Windows build number (cached after first call)
fn get_windows_build() -> u32 {
    *WINDOWS_BUILD.get_or_init(|| {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(nt_key) = hklm.open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion") {
            if let Ok(build_str) = nt_key.get_value::<String, _>("CurrentBuild") {
                if let Ok(build) = build_str.parse::<u32>() {
                    return build;
                }
            }
        }
        // Fallback to a safe default (Windows 10 1809)
        17763
    })
}

/// Check if running on Windows 11 (build 22000+)
/// Result is cached after first call for better performance.
pub fn is_windows_11() -> bool {
    get_windows_build() >= 22000
}

/// Check if the system supports DWMWA_USE_IMMERSIVE_DARK_MODE (attribute 20)
/// Requires Windows 10 build 18985+ (20H1 preview) or Windows 11
fn supports_dark_mode_attr_20() -> bool {
    get_windows_build() >= 18985
}

/// Check if the system supports dark mode title bar at all
/// Requires Windows 10 build 17763+ (1809)
fn supports_dark_mode() -> bool {
    get_windows_build() >= 17763
}

/// Apply Windows DWM effects (shadow and rounded corners)
///
/// This function handles different Windows 10 builds:
/// - Build < 17763 (pre-1809): No dark mode support, only shadow
/// - Build 17763-18985 (1809-early 20H1): Dark mode via undocumented attribute 19
/// - Build 18986+ (20H1+): Dark mode via DWMWA_USE_IMMERSIVE_DARK_MODE (20)
/// - Build 22000+ (Windows 11): Also enables rounded corners
pub fn apply_window_effects(hwnd: isize) {
    unsafe {
        let hwnd = HWND(hwnd as *mut std::ffi::c_void);
        let build = get_windows_build();

        // Apply dark mode for window frame
        if supports_dark_mode() {
            let dark_mode: i32 = 1;

            if supports_dark_mode_attr_20() {
                // Windows 10 build 18985+ and Windows 11: Use standard attribute 20
                use windows::Win32::Graphics::Dwm::DWMWA_USE_IMMERSIVE_DARK_MODE;
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE,
                    &dark_mode as *const _ as *const std::ffi::c_void,
                    std::mem::size_of::<i32>() as u32,
                );
            } else {
                // Windows 10 build 17763-18985: Use undocumented attribute 19
                // This was the original dark mode attribute before it was standardized
                use windows::Win32::Graphics::Dwm::DWMWINDOWATTRIBUTE;
                const DWMWA_USE_IMMERSIVE_DARK_MODE_LEGACY: DWMWINDOWATTRIBUTE =
                    DWMWINDOWATTRIBUTE(19);
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_USE_IMMERSIVE_DARK_MODE_LEGACY,
                    &dark_mode as *const _ as *const std::ffi::c_void,
                    std::mem::size_of::<i32>() as u32,
                );
            }
        }

        // Windows 11-specific: Enable rounded corners
        if is_windows_11() {
            use windows::Win32::Graphics::Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND};

            let corner_preference = DWMWCP_ROUND.0;
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &corner_preference as *const _ as *const std::ffi::c_void,
                std::mem::size_of::<i32>() as u32,
            );
        }

        // Apply shadow via DwmExtendFrameIntoClientArea for all versions
        // This provides a native shadow effect for borderless windows
        use windows::Win32::Graphics::Dwm::DwmExtendFrameIntoClientArea;
        use windows::Win32::UI::Controls::MARGINS;

        let margins = MARGINS {
            cxLeftWidth: 1,
            cxRightWidth: 1,
            cyTopHeight: 1,
            cyBottomHeight: 1,
        };
        let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);

        tracing::info!(
            "Applied DWM effects for Windows build {} (dark_mode={}, rounded={})",
            build,
            supports_dark_mode(),
            is_windows_11()
        );
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

/// Check if Windows is configured to use light theme for apps
/// Reads from registry: HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize
/// Returns true if AppsUseLightTheme = 1, false otherwise (defaults to dark)
pub fn system_uses_light_theme() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) =
        hkcu.open_subkey(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize")
    {
        if let Ok(value) = key.get_value::<u32, _>("AppsUseLightTheme") {
            return value == 1;
        }
    }
    false // Default to dark theme
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
