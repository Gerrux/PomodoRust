//! Platform-specific functionality

mod audio;

#[cfg(windows)]
mod windows;

pub use audio::{AudioPlayer, SoundType};

#[cfg(windows)]
pub use windows::{apply_window_effects, remove_autostart, set_autostart, show_notification};

// Fallback for non-Windows platforms
#[cfg(not(windows))]
pub fn show_notification(_title: &str, _body: &str) {
    tracing::info!("Notification: {} - {}", _title, _body);
}

#[cfg(not(windows))]
pub fn set_autostart(_enabled: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub fn remove_autostart() -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub fn apply_window_effects(_hwnd: isize) {}
