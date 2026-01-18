//! Configuration management using TOML

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::ui::theme::AccentColor;

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
    pub work_end_sound: String,
    pub break_end_sound: String,
    pub tick_enabled: bool,
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            volume: 80,
            work_end_sound: "bell".into(),
            break_end_sound: "chime".into(),
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

/// Main configuration struct
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Config {
    pub timer: TimerConfig,
    pub sounds: SoundConfig,
    pub appearance: AppearanceConfig,
    pub system: SystemConfig,
    pub window: WindowConfig,
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
    pub fn save(&self) -> Result<(), String> {
        let Some(dir) = Self::config_dir() else {
            return Err("Could not determine config directory".into());
        };

        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&dir) {
            return Err(format!("Failed to create config directory: {}", e));
        }

        let path = dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

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
