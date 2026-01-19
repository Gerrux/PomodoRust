//! Configuration management using TOML
//!
//! Handles loading, saving, and validating application configuration
//! stored in TOML format at the platform-specific config directory.

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::error::ConfigError;
use crate::ui::theme::AccentColor;

/// Available notification sounds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NotificationSound {
    #[default]
    SoftBell,
    LevelUp,
    DigitalAlert,
}

impl NotificationSound {
    /// Get all available sounds
    pub fn all() -> &'static [NotificationSound] {
        &[
            NotificationSound::SoftBell,
            NotificationSound::LevelUp,
            NotificationSound::DigitalAlert,
        ]
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            NotificationSound::SoftBell => "Soft Bell",
            NotificationSound::LevelUp => "Level Up",
            NotificationSound::DigitalAlert => "Digital Alert",
        }
    }
}

/// Timer configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TimerConfig {
    pub work_duration: u32,
    pub short_break: u32,
    pub long_break: u32,
    pub sessions_before_long: u32,
    pub auto_start_breaks: bool,
    pub auto_start_work: bool,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            work_duration: 25,
            short_break: 5,
            long_break: 15,
            sessions_before_long: 4,
            auto_start_breaks: false,
            auto_start_work: false,
        }
    }
}

/// Sound configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoundConfig {
    pub enabled: bool,
    pub volume: u32,
    pub notification_sound: NotificationSound,
    pub tick_enabled: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 80,
            notification_sound: NotificationSound::SoftBell,
            tick_enabled: false,
        }
    }
}

/// Appearance configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppearanceConfig {
    pub accent_color: AccentColor,
    pub compact_mode: bool,
    pub window_opacity: u32,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            accent_color: AccentColor::Blue,
            compact_mode: false,
            window_opacity: 100,
        }
    }
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemConfig {
    pub start_with_windows: bool,
    pub minimize_to_tray: bool,
    pub show_in_taskbar: bool,
    pub notifications_enabled: bool,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            start_with_windows: false,
            minimize_to_tray: true,
            show_in_taskbar: true,
            notifications_enabled: true,
        }
    }
}

/// Window position configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WindowConfig {
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub always_on_top: bool,
    pub maximized: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            width: 360.0,
            height: 480.0,
            x: None,
            y: None,
            always_on_top: false,
            maximized: false,
        }
    }
}

/// Goals configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GoalsConfig {
    pub daily_target: u32,
    pub weekly_target: u32,
    pub notify_on_goal: bool,
}

impl Default for GoalsConfig {
    fn default() -> Self {
        Self {
            daily_target: 8,
            weekly_target: 40,
            notify_on_goal: true,
        }
    }
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HotkeysConfig {
    pub enabled: bool,
    /// Toggle timer (start/pause) - default: Ctrl+Alt+Space
    pub toggle: String,
    /// Skip to next session - default: Ctrl+Alt+S
    pub skip: String,
    /// Reset timer - default: Ctrl+Alt+R
    pub reset: String,
}

impl Default for HotkeysConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            toggle: "Ctrl+Alt+Space".to_string(),
            skip: "Ctrl+Alt+S".to_string(),
            reset: "Ctrl+Alt+R".to_string(),
        }
    }
}

/// Main configuration struct
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Config {
    pub timer: TimerConfig,
    pub sounds: SoundConfig,
    pub appearance: AppearanceConfig,
    pub system: SystemConfig,
    pub window: WindowConfig,
    #[serde(default)]
    pub goals: GoalsConfig,
    #[serde(default)]
    pub hotkeys: HotkeysConfig,
}

impl Config {
    /// Get the configuration directory path
    pub fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "pomodorust", "PomodoRust")
            .map(|dirs| dirs.config_dir().to_path_buf())
    }

    /// Get the configuration file path
    pub fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    /// Load configuration from file or create default
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            tracing::warn!("Could not determine config path, using defaults");
            return Self::default();
        };

        if !path.exists() {
            tracing::info!("No config file found, creating default at {:?}", path);
            let config = Self::default();
            let _ = config.save();
            return config;
        }

        match fs::read_to_string(&path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => {
                    tracing::info!("Loaded config from {:?}", path);
                    config
                }
                Err(e) => {
                    tracing::error!("Failed to parse config: {}, using defaults", e);
                    Self::default()
                }
            },
            Err(e) => {
                tracing::error!("Failed to read config file: {}, using defaults", e);
                Self::default()
            }
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), ConfigError> {
        let dir = Self::config_dir().ok_or(ConfigError::DirectoryNotFound)?;

        // Create directory if it doesn't exist
        fs::create_dir_all(&dir).map_err(|e| ConfigError::DirectoryCreation {
            path: dir.clone(),
            source: e,
        })?;

        let path = dir.join("config.toml");
        let content = toml::to_string_pretty(self).map_err(|e| ConfigError::Serialize {
            message: e.to_string(),
        })?;

        fs::write(&path, &content).map_err(|e| ConfigError::WriteFile {
            path: path.clone(),
            source: e,
        })?;

        tracing::info!("Saved config to {:?}", path);
        Ok(())
    }

    /// Reset to defaults
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Convert to preset
    pub fn to_preset(&self) -> crate::core::Preset {
        crate::core::Preset::custom(
            "Custom",
            self.timer.work_duration,
            self.timer.short_break,
            self.timer.long_break,
            self.timer.sessions_before_long,
        )
    }

    /// Apply preset to timer config
    pub fn apply_preset(&mut self, preset: &crate::core::Preset) {
        self.timer.work_duration = preset.work_duration;
        self.timer.short_break = preset.short_break;
        self.timer.long_break = preset.long_break;
        self.timer.sessions_before_long = preset.sessions_before_long_break;
    }
}
