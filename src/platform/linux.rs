//! Linux-specific functionality
//!
//! Provides Linux-specific features including:
//! - Desktop notifications via D-Bus (notify-rust)
//! - Autostart via XDG Desktop Entry specification
//! - Window effects (no-op on Linux)

use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::PlatformError;

/// Show a desktop notification using D-Bus
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

/// Get the XDG autostart directory path
fn get_autostart_dir() -> Option<PathBuf> {
    env::var_os("XDG_CONFIG_HOME")
        .map(|config_home| PathBuf::from(config_home).join("autostart"))
        .or_else(|| {
            env::var_os("HOME").map(|home| PathBuf::from(home).join(".config").join("autostart"))
        })
}

/// Get the path to the autostart .desktop file
fn get_desktop_file_path() -> Option<PathBuf> {
    get_autostart_dir().map(|dir| dir.join("pomodorust.desktop"))
}

/// Create the XDG Desktop Entry content
fn create_desktop_entry(exec_path: &str) -> String {
    format!(
        r#"[Desktop Entry]
Type=Application
Name=PomodoRust
Comment=A modern, lightweight Pomodoro timer
Exec={}
Icon=pomodorust
Terminal=false
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
"#,
        exec_path
    )
}

/// Set application to start on login
pub fn set_autostart(enabled: bool) -> Result<(), PlatformError> {
    if enabled {
        let autostart_dir = get_autostart_dir().ok_or(PlatformError::Unsupported {
            feature: "autostart (HOME not set)",
        })?;

        // Create autostart directory if it doesn't exist
        if !autostart_dir.exists() {
            fs::create_dir_all(&autostart_dir).map_err(|e| PlatformError::Notification {
                message: format!("Failed to create autostart directory: {}", e),
            })?;
        }

        let exe_path =
            env::current_exe().map_err(|e| PlatformError::ExecutablePath { source: e })?;

        let desktop_content = create_desktop_entry(&exe_path.to_string_lossy());

        let desktop_file_path = get_desktop_file_path().ok_or(PlatformError::Unsupported {
            feature: "autostart (HOME not set)",
        })?;

        let mut file =
            fs::File::create(&desktop_file_path).map_err(|e| PlatformError::Notification {
                message: format!("Failed to create desktop file: {}", e),
            })?;

        file.write_all(desktop_content.as_bytes())
            .map_err(|e| PlatformError::Notification {
                message: format!("Failed to write desktop file: {}", e),
            })?;

        tracing::info!("Enabled autostart: {:?}", desktop_file_path);
    } else {
        remove_autostart()?;
    }

    Ok(())
}

/// Remove application from login startup
pub fn remove_autostart() -> Result<(), PlatformError> {
    if let Some(desktop_file_path) = get_desktop_file_path() {
        if desktop_file_path.exists() {
            fs::remove_file(&desktop_file_path).map_err(|e| PlatformError::Notification {
                message: format!("Failed to remove desktop file: {}", e),
            })?;
            tracing::info!("Disabled autostart: {:?}", desktop_file_path);
        }
    }

    Ok(())
}

/// Apply window effects (no-op on Linux)
/// DWM effects are Windows-specific
pub fn apply_window_effects(_hwnd: isize) {
    // No-op on Linux
    // Window decorations and effects are handled by the window manager
}

/// Flash the window in taskbar (no-op on Linux)
pub fn flash_window(_hwnd: isize, _count: u32) {
    // No-op on Linux
    // Taskbar flash is platform-specific
}

/// Stop flashing the window (no-op on Linux)
pub fn stop_flash_window(_hwnd: isize) {
    // No-op on Linux
}

/// Flash the PomodoRust window by finding it by title (no-op on Linux)
/// Returns false as this feature is not implemented
pub fn flash_pomodorust_window(_count: u32) -> bool {
    // Window flash not implemented on Linux
    false
}

/// Show and bring the PomodoRust window to foreground (no-op on Linux)
/// Returns false as this feature is not implemented
pub fn show_pomodorust_window() -> bool {
    // Window show not implemented on Linux
    // Would require X11/Wayland-specific implementation
    false
}
