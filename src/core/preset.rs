//! Timer presets management

use serde::{Deserialize, Serialize};

/// A timer preset configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    /// Preset name
    pub name: String,
    /// Work duration in minutes
    pub work_duration: u32,
    /// Short break duration in minutes
    pub short_break: u32,
    /// Long break duration in minutes
    pub long_break: u32,
    /// Number of work sessions before a long break
    pub sessions_before_long_break: u32,
    /// Whether this is a built-in preset
    #[serde(default)]
    pub is_builtin: bool,
}

impl Preset {
    /// Create a custom preset
    pub fn custom(
        name: impl Into<String>,
        work: u32,
        short_break: u32,
        long_break: u32,
        sessions: u32,
    ) -> Self {
        Self {
            name: name.into(),
            work_duration: work,
            short_break,
            long_break,
            sessions_before_long_break: sessions,
            is_builtin: false,
        }
    }

    /// Classic Pomodoro (25/5/15)
    pub fn classic() -> Self {
        Self {
            name: "Classic".into(),
            work_duration: 25,
            short_break: 5,
            long_break: 15,
            sessions_before_long_break: 4,
            is_builtin: true,
        }
    }

    /// Short sessions (15/3/10)
    pub fn short() -> Self {
        Self {
            name: "Short".into(),
            work_duration: 15,
            short_break: 3,
            long_break: 10,
            sessions_before_long_break: 4,
            is_builtin: true,
        }
    }

    /// Long focus sessions (50/10/30)
    pub fn long() -> Self {
        Self {
            name: "Long Focus".into(),
            work_duration: 50,
            short_break: 10,
            long_break: 30,
            sessions_before_long_break: 2,
            is_builtin: true,
        }
    }

    /// 52/17 method
    pub fn fifty_two_seventeen() -> Self {
        Self {
            name: "52/17".into(),
            work_duration: 52,
            short_break: 17,
            long_break: 30,
            sessions_before_long_break: 2,
            is_builtin: true,
        }
    }
}

impl Default for Preset {
    fn default() -> Self {
        Self::classic()
    }
}

/// Manages available presets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetManager {
    /// All available presets
    presets: Vec<Preset>,
    /// Index of currently selected preset
    selected_index: usize,
}

impl PresetManager {
    /// Create a new preset manager with default presets
    pub fn new() -> Self {
        Self {
            presets: vec![
                Preset::classic(),
                Preset::short(),
                Preset::long(),
                Preset::fifty_two_seventeen(),
            ],
            selected_index: 0,
        }
    }

    /// Get all presets
    pub fn presets(&self) -> &[Preset] {
        &self.presets
    }

    /// Get current preset
    pub fn current(&self) -> &Preset {
        &self.presets[self.selected_index]
    }

    /// Get current preset index
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Select a preset by index
    pub fn select(&mut self, index: usize) {
        if index < self.presets.len() {
            self.selected_index = index;
        }
    }

    /// Select a preset by name
    pub fn select_by_name(&mut self, name: &str) {
        if let Some(idx) = self.presets.iter().position(|p| p.name == name) {
            self.selected_index = idx;
        }
    }

    /// Add a custom preset
    pub fn add_custom(&mut self, preset: Preset) {
        self.presets.push(preset);
    }

    /// Remove a preset by index (only non-builtin)
    pub fn remove(&mut self, index: usize) -> bool {
        if index < self.presets.len() && !self.presets[index].is_builtin {
            self.presets.remove(index);
            if self.selected_index >= self.presets.len() {
                self.selected_index = self.presets.len().saturating_sub(1);
            }
            true
        } else {
            false
        }
    }

    /// Update a preset by index
    pub fn update(&mut self, index: usize, preset: Preset) {
        if index < self.presets.len() {
            self.presets[index] = preset;
        }
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}
