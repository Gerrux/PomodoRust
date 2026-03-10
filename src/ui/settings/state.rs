use super::super::theme::{AccentColor, ThemeMode};
use crate::data::{Config, NotificationSound};
use crate::i18n::Language;

/// Editable settings state - extracted from Config for UI editing
///
/// Uses f32 for numeric fields to support smooth slider interaction.
/// These get converted back to u32 when building the Config.
#[derive(Debug, Clone)]
pub struct SettingsState {
    // Timer settings
    pub work_duration: f32,
    pub short_break: f32,
    pub long_break: f32,
    pub sessions_before_long: f32,
    // Sound settings
    pub volume: f32,
    pub notification_sound: NotificationSound,
    pub tick_enabled: bool,
    // Auto-start settings
    pub auto_start_breaks: bool,
    pub auto_start_work: bool,
    // System settings
    pub start_with_windows: bool,
    // Window settings
    pub always_on_top: bool,
    // Appearance
    pub theme_mode: ThemeMode,
    pub selected_accent: AccentColor,
    pub window_opacity: f32,
    // Goals
    pub daily_goal: f32,
    pub notify_on_goal: bool,
    // Hotkeys
    pub hotkeys_enabled: bool,
    pub hotkey_toggle: String,
    pub hotkey_skip: String,
    pub hotkey_reset: String,
    // Accessibility
    pub high_contrast: bool,
    pub reduced_motion: bool,
    // Language
    pub language: Language,
}

impl SettingsState {
    /// Create editing state from Config
    pub fn from_config(config: &Config) -> Self {
        Self {
            work_duration: config.timer.work_duration as f32,
            short_break: config.timer.short_break as f32,
            long_break: config.timer.long_break as f32,
            sessions_before_long: config.timer.sessions_before_long as f32,
            volume: config.sounds.volume as f32,
            notification_sound: config.sounds.notification_sound,
            tick_enabled: config.sounds.tick_enabled,
            auto_start_breaks: config.timer.auto_start_breaks,
            auto_start_work: config.timer.auto_start_work,
            start_with_windows: config.system.start_with_windows,
            always_on_top: config.window.always_on_top,
            theme_mode: config.appearance.theme_mode,
            selected_accent: config.appearance.accent_color,
            window_opacity: config.appearance.window_opacity as f32,
            daily_goal: config.goals.daily_target as f32,
            notify_on_goal: config.goals.notify_on_goal,
            hotkeys_enabled: config.hotkeys.enabled,
            hotkey_toggle: config.hotkeys.toggle.clone(),
            hotkey_skip: config.hotkeys.skip.clone(),
            hotkey_reset: config.hotkeys.reset.clone(),
            high_contrast: config.accessibility.high_contrast,
            reduced_motion: config.accessibility.reduced_motion,
            language: config.appearance.language,
        }
    }

    /// Check if the editing state differs from the given config
    pub fn differs_from(&self, config: &Config) -> bool {
        self.apply_to(config) != *config
    }

    /// Apply the editing state to a Config, returning a new Config
    pub fn apply_to(&self, original: &Config) -> Config {
        let mut config = original.clone();
        config.timer.work_duration = self.work_duration.round() as u32;
        config.timer.short_break = self.short_break.round() as u32;
        config.timer.long_break = self.long_break.round() as u32;
        config.timer.sessions_before_long = self.sessions_before_long.round() as u32;
        config.timer.auto_start_breaks = self.auto_start_breaks;
        config.timer.auto_start_work = self.auto_start_work;
        config.sounds.volume = self.volume.round() as u32;
        config.sounds.notification_sound = self.notification_sound;
        config.sounds.tick_enabled = self.tick_enabled;
        config.system.start_with_windows = self.start_with_windows;
        config.window.always_on_top = self.always_on_top;
        config.appearance.theme_mode = self.theme_mode;
        config.appearance.accent_color = self.selected_accent;
        config.appearance.window_opacity = self.window_opacity.round() as u32;
        config.goals.daily_target = self.daily_goal.round() as u32;
        config.goals.notify_on_goal = self.notify_on_goal;
        config.hotkeys.enabled = self.hotkeys_enabled;
        config.hotkeys.toggle = self.hotkey_toggle.clone();
        config.hotkeys.skip = self.hotkey_skip.clone();
        config.hotkeys.reset = self.hotkey_reset.clone();
        config.accessibility.high_contrast = self.high_contrast;
        config.accessibility.reduced_motion = self.reduced_motion;
        config.appearance.language = self.language;
        config
    }
}
