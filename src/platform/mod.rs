//! Platform-specific functionality
//!
//! This module provides cross-platform abstractions for:
//! - Audio playback
//! - System notifications
//! - Autostart configuration
//! - Window effects
//! - System tray

mod audio;

#[cfg(windows)]
mod windows;

pub use audio::{AudioPlayer, SoundType};

#[cfg(windows)]
pub use windows::{
    apply_window_effects, flash_pomodorust_window, flash_window, remove_autostart, set_autostart,
    show_notification, show_pomodorust_window, stop_flash_window,
};

#[cfg(not(windows))]
use crate::error::PlatformError;

// Fallback for non-Windows platforms
#[cfg(not(windows))]
pub fn show_notification(_title: &str, _body: &str) {
    tracing::info!("Notification: {} - {}", _title, _body);
}

#[cfg(not(windows))]
pub fn set_autostart(_enabled: bool) -> Result<(), PlatformError> {
    // Autostart not implemented for non-Windows platforms
    Ok(())
}

#[cfg(not(windows))]
pub fn remove_autostart() -> Result<(), PlatformError> {
    // Autostart not implemented for non-Windows platforms
    Ok(())
}

#[cfg(not(windows))]
pub fn apply_window_effects(_hwnd: isize) {
    // DWM effects are Windows-specific
}

#[cfg(not(windows))]
pub fn flash_window(_hwnd: isize, _count: u32) {
    // Window flash is Windows-specific
}

#[cfg(not(windows))]
pub fn stop_flash_window(_hwnd: isize) {
    // Window flash is Windows-specific
}

#[cfg(not(windows))]
pub fn flash_pomodorust_window(_count: u32) -> bool {
    // Window flash is Windows-specific
    false
}

#[cfg(not(windows))]
pub fn show_pomodorust_window() -> bool {
    // Window show is Windows-specific
    false
}
