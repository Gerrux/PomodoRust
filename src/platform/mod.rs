//! Platform-specific functionality
//!
//! This module provides cross-platform abstractions for:
//! - Audio playback
//! - System notifications
//! - Autostart configuration
//! - Window effects
//! - Global hotkeys

mod audio;

#[cfg(windows)]
mod windows;

#[cfg(windows)]
mod hotkeys;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
mod linux_hotkeys;

pub use audio::AudioPlayer;

#[cfg(windows)]
pub use windows::{
    apply_window_effects, flash_pomodorust_window, flash_window, remove_autostart, set_autostart,
    show_notification, show_pomodorust_window, stop_flash_window,
};

#[cfg(windows)]
pub use hotkeys::{HotkeyAction, HotkeyManager};

#[cfg(target_os = "linux")]
pub use linux::{
    apply_window_effects, flash_pomodorust_window, flash_window, remove_autostart, set_autostart,
    show_notification, show_pomodorust_window, stop_flash_window,
};

#[cfg(target_os = "linux")]
pub use linux_hotkeys::{HotkeyAction, HotkeyManager};

// Fallback for other platforms (not Windows, not Linux)
#[cfg(not(any(windows, target_os = "linux")))]
use crate::error::PlatformError;

#[cfg(not(any(windows, target_os = "linux")))]
pub fn show_notification(_title: &str, _body: &str) {
    tracing::info!("Notification: {} - {}", _title, _body);
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn set_autostart(_enabled: bool) -> Result<(), PlatformError> {
    // Autostart not implemented for this platform
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn remove_autostart() -> Result<(), PlatformError> {
    // Autostart not implemented for this platform
    Ok(())
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn apply_window_effects(_hwnd: isize) {
    // Window effects are platform-specific
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn flash_window(_hwnd: isize, _count: u32) {
    // Window flash is platform-specific
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn stop_flash_window(_hwnd: isize) {
    // Window flash is platform-specific
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn flash_pomodorust_window(_count: u32) -> bool {
    // Window flash is platform-specific
    false
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn show_pomodorust_window() -> bool {
    // Window show is platform-specific
    false
}

// Hotkey fallbacks for other platforms
#[cfg(not(any(windows, target_os = "linux")))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    Toggle,
    Skip,
    Reset,
}

#[cfg(not(any(windows, target_os = "linux")))]
pub struct HotkeyManager;

#[cfg(not(any(windows, target_os = "linux")))]
impl HotkeyManager {
    pub fn new() -> Self {
        Self
    }

    pub fn take_receiver(&mut self) -> Option<std::sync::mpsc::Receiver<HotkeyAction>> {
        None
    }

    pub fn start(&mut self, _toggle: &str, _skip: &str, _reset: &str) {
        tracing::info!("Global hotkeys not supported on this platform");
    }

    pub fn stop(&mut self) {}

    pub fn is_running(&self) -> bool {
        false
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}
