//! Settings panel
//!
//! Provides the settings UI with local editing state that syncs back to Config.
//! The editing state is kept separate to allow for:
//! - Smooth slider/input interaction
//! - Validation before applying changes
//! - Consistent state management

use egui::{vec2, Color32, Layout, Rect, Ui};

use super::components::{draw_icon, Card, Icon, IconButton};
use super::theme::{AccentColor, Theme};
use crate::data::{Config, NotificationSound};

/// Actions from settings
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsAction {
    Back,
    UpdateConfig(Config),
    SelectPreset(usize),
    ResetDefaults,
    SetAlwaysOnTop(bool),
    TestSound(NotificationSound),
}

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
        }
    }

    /// Check if the editing state differs from the given config
    pub fn differs_from(&self, config: &Config) -> bool {
        self.work_duration.round() as u32 != config.timer.work_duration
            || self.short_break.round() as u32 != config.timer.short_break
            || self.long_break.round() as u32 != config.timer.long_break
            || self.sessions_before_long.round() as u32 != config.timer.sessions_before_long
            || self.volume.round() as u32 != config.sounds.volume
            || self.notification_sound != config.sounds.notification_sound
            || self.tick_enabled != config.sounds.tick_enabled
            || self.auto_start_breaks != config.timer.auto_start_breaks
            || self.auto_start_work != config.timer.auto_start_work
            || self.start_with_windows != config.system.start_with_windows
            || self.always_on_top != config.window.always_on_top
            || self.selected_accent != config.appearance.accent_color
            || self.window_opacity.round() as u32 != config.appearance.window_opacity
            || self.daily_goal.round() as u32 != config.goals.daily_target
            || self.notify_on_goal != config.goals.notify_on_goal
            || self.hotkeys_enabled != config.hotkeys.enabled
            || self.hotkey_toggle != config.hotkeys.toggle
            || self.hotkey_skip != config.hotkeys.skip
            || self.hotkey_reset != config.hotkeys.reset
            || self.high_contrast != config.accessibility.high_contrast
            || self.reduced_motion != config.accessibility.reduced_motion
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
        config
    }
}

/// Settings view
pub struct SettingsView {
    /// Local editing state, kept in sync with Config
    state: SettingsState,
}

impl SettingsView {
    pub fn new(config: &Config) -> Self {
        Self {
            state: SettingsState::from_config(config),
        }
    }

    pub fn show(&mut self, ui: &mut Ui, config: &Config, theme: &Theme) -> Option<SettingsAction> {
        let mut action = None;

        // Sync always_on_top from config (may be changed externally via titlebar)
        if self.state.always_on_top != config.window.always_on_top {
            self.state.always_on_top = config.window.always_on_top;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Header
            ui.horizontal(|ui| {
                if IconButton::new(Icon::ArrowLeft)
                    .with_size(32.0)
                    .with_icon_scale(0.5)
                    .show(ui, theme)
                    .clicked()
                {
                    action = Some(SettingsAction::Back);
                }

                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new("Settings")
                        .font(theme.font_h2())
                        .color(theme.text_primary),
                );
            });

            ui.add_space(theme.spacing_lg);

            // Timer section
            section_header(ui, theme, "Timer");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                duration_row(
                    ui,
                    theme,
                    "Focus Duration",
                    &mut self.state.work_duration,
                    1.0,
                    90.0,
                );
                duration_row(
                    ui,
                    theme,
                    "Short Break",
                    &mut self.state.short_break,
                    1.0,
                    30.0,
                );
                duration_row(
                    ui,
                    theme,
                    "Long Break",
                    &mut self.state.long_break,
                    5.0,
                    60.0,
                );
                duration_row_with_unit(
                    ui,
                    theme,
                    "Sessions before long break",
                    &mut self.state.sessions_before_long,
                    2.0,
                    8.0,
                    "",
                );

                ui.add_space(theme.spacing_sm);

                toggle_row(
                    ui,
                    theme,
                    "Auto-start breaks",
                    &mut self.state.auto_start_breaks,
                );
                toggle_row(
                    ui,
                    theme,
                    "Auto-start pomodoros",
                    &mut self.state.auto_start_work,
                );
            });

            ui.add_space(theme.spacing_md);

            // Sounds section
            section_header(ui, theme, "Sounds");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Volume").color(theme.text_secondary));

                    // Use right-to-left layout for proper alignment
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}%", self.state.volume.round() as u32))
                                .color(theme.text_muted),
                        );

                        ui.add_sized(
                            vec2(120.0, 20.0),
                            egui::Slider::new(&mut self.state.volume, 0.0..=100.0)
                                .step_by(1.0)
                                .show_value(false),
                        );
                    });
                });

                ui.add_space(theme.spacing_sm);

                // Sound selection with test button
                let mut test_sound = false;
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Sound").color(theme.text_secondary));

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        // Test button
                        if ui
                            .add_sized(vec2(28.0, 22.0), egui::Button::new("\u{25B6}"))
                            .on_hover_text("Test sound")
                            .clicked()
                        {
                            test_sound = true;
                        }

                        ui.add_space(4.0);

                        egui::ComboBox::from_id_salt("notification_sound")
                            .selected_text(self.state.notification_sound.name())
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                for sound in NotificationSound::all() {
                                    ui.selectable_value(
                                        &mut self.state.notification_sound,
                                        *sound,
                                        sound.name(),
                                    );
                                }
                            });
                    });
                });
                if test_sound {
                    action = Some(SettingsAction::TestSound(self.state.notification_sound));
                }

                ui.add_space(theme.spacing_sm);

                toggle_row(
                    ui,
                    theme,
                    "Tick sound",
                    &mut self.state.tick_enabled,
                );
            });

            ui.add_space(theme.spacing_md);

            // Appearance section
            section_header(ui, theme, "Appearance");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                // Standard colors
                let standard_colors: Vec<_> = AccentColor::all()
                    .iter()
                    .filter(|c| !c.is_retro())
                    .collect();
                let retro_colors: Vec<_> =
                    AccentColor::all().iter().filter(|c| c.is_retro()).collect();

                color_picker_row(
                    ui,
                    theme,
                    "Accent Color",
                    &standard_colors,
                    &mut self.state.selected_accent,
                );

                ui.add_space(theme.spacing_sm);

                color_picker_row(
                    ui,
                    theme,
                    "Retro Themes",
                    &retro_colors,
                    &mut self.state.selected_accent,
                );

                ui.add_space(theme.spacing_sm);

                // Window opacity slider
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Window Opacity").color(theme.text_secondary));

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}%", self.state.window_opacity.round() as u32))
                                .color(theme.text_muted),
                        );

                        ui.add_sized(
                            vec2(120.0, 20.0),
                            egui::Slider::new(&mut self.state.window_opacity, 30.0..=100.0)
                                .step_by(5.0)
                                .show_value(false),
                        );
                    });
                });
            });

            ui.add_space(theme.spacing_md);

            // Accessibility section
            section_header(ui, theme, "Accessibility");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                toggle_row(
                    ui,
                    theme,
                    "High contrast mode",
                    &mut self.state.high_contrast,
                );
                toggle_row(
                    ui,
                    theme,
                    "Reduced motion",
                    &mut self.state.reduced_motion,
                );
            });

            ui.add_space(theme.spacing_md);

            // System section
            section_header(ui, theme, "System");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                toggle_row(
                    ui,
                    theme,
                    "Start with Windows",
                    &mut self.state.start_with_windows,
                );
                toggle_row(ui, theme, "Always on top", &mut self.state.always_on_top);
            });

            ui.add_space(theme.spacing_md);

            // Goals section
            section_header(ui, theme, "Goals");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                duration_row_with_unit(
                    ui,
                    theme,
                    "Daily goal",
                    &mut self.state.daily_goal,
                    1.0,
                    16.0,
                    "pomodoros",
                );

                toggle_row(
                    ui,
                    theme,
                    "Notify when goal reached",
                    &mut self.state.notify_on_goal,
                );
            });

            ui.add_space(theme.spacing_md);

            // Hotkeys section
            section_header(ui, theme, "Global Hotkeys");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                toggle_row(
                    ui,
                    theme,
                    "Enable global hotkeys",
                    &mut self.state.hotkeys_enabled,
                );

                if self.state.hotkeys_enabled {
                    ui.add_space(theme.spacing_xs);

                    // Show current hotkey bindings (read-only for now)
                    hotkey_row(ui, theme, "Toggle (start/pause)", &self.state.hotkey_toggle);
                    hotkey_row(ui, theme, "Skip session", &self.state.hotkey_skip);
                    hotkey_row(ui, theme, "Reset timer", &self.state.hotkey_reset);

                    ui.add_space(theme.spacing_xs);
                    ui.label(
                        egui::RichText::new("Restart app to apply hotkey changes")
                            .color(theme.text_muted)
                            .small(),
                    );
                }
            });

            ui.add_space(theme.spacing_md);

            // CLI Setup section
            section_header(ui, theme, "Command Line");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                ui.label(
                    egui::RichText::new("Control timer from terminal:")
                        .color(theme.text_secondary),
                );
                ui.add_space(theme.spacing_xs);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("pomodorust status")
                            .color(theme.text_muted)
                            .code(),
                    );
                });

                ui.add_space(theme.spacing_sm);

                // Get current exe path for the command
                let exe_path = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "C:\\path\\to\\pomodorust".to_string());

                let powershell_cmd = format!(
                    "$p = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($p -notlike '*{}*') {{ [Environment]::SetEnvironmentVariable('Path', \"$p;{}\", 'User') }}",
                    exe_path, exe_path
                );

                if ui
                    .add_sized(
                        vec2(ui.available_width(), 32.0),
                        egui::Button::new("Copy PATH command"),
                    )
                    .on_hover_text("Copy PowerShell command to add pomodorust to PATH")
                    .clicked()
                {
                    ui.ctx().copy_text(powershell_cmd);
                }

                ui.add_space(theme.spacing_xs);
                ui.label(
                    egui::RichText::new("Run copied command in PowerShell, then restart terminal")
                        .color(theme.text_muted)
                        .small(),
                );
            });

            ui.add_space(theme.spacing_md);

            // Presets section
            section_header(ui, theme, "Presets");
            let mut preset_clicked: Option<usize> = None;
            Card::new().show(ui, theme, |ui| {
                let card_width = ui.available_width();
                ui.set_min_width(card_width - theme.spacing_md * 2.0);

                let presets = [
                    ("Classic", "25/5/15"),
                    ("Short", "15/3/10"),
                    ("Long", "50/10/30"),
                ];

                let button_width = (card_width - theme.spacing_sm * 2.0) / 3.0;

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = theme.spacing_sm;
                    for (i, preset) in presets.iter().enumerate() {
                        if ui
                            .add_sized(
                                vec2(button_width, 48.0),
                                egui::Button::new(format!("{}\n{}", preset.0, preset.1)),
                            )
                            .clicked()
                        {
                            preset_clicked = Some(i);
                        }
                    }
                });
            });
            if let Some(index) = preset_clicked {
                action = Some(SettingsAction::SelectPreset(index));
            }

            ui.add_space(theme.spacing_xl);

            // Reset button
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 150.0) / 2.0);

                if ui
                    .add(egui::Button::new("Reset to Defaults").fill(theme.bg_tertiary))
                    .clicked()
                {
                    action = Some(SettingsAction::ResetDefaults);
                }
            });

            ui.add_space(theme.spacing_lg);
        });

        // Check if config changed and emit UpdateConfig action
        if action.is_none() && self.state.differs_from(config) {
            let new_config = self.state.apply_to(config);
            action = Some(SettingsAction::UpdateConfig(new_config));
        }

        action
    }

    /// Reset the editing state to match the given config
    pub fn reset_from_config(&mut self, config: &Config) {
        self.state = SettingsState::from_config(config);
    }
}

/// Draw section header
fn section_header(ui: &mut Ui, theme: &Theme, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(theme.font_body())
            .color(theme.text_primary),
    );
    ui.add_space(theme.spacing_xs);
}

/// Draw a duration row with +/- buttons and unit label
fn duration_row(ui: &mut Ui, theme: &Theme, label: &str, value: &mut f32, min: f32, max: f32) {
    duration_row_with_unit(ui, theme, label, value, min, max, "min");
}

/// Draw a duration row with +/- buttons, custom unit
fn duration_row_with_unit(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
    unit: &str,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme.text_secondary));

        // Use right-to-left layout for controls alignment
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            // Plus button (appears last visually, first in RTL)
            let plus_response = ui.allocate_response(vec2(32.0, 32.0), egui::Sense::click());
            let plus_bg = if plus_response.hovered() {
                theme.bg_hover
            } else {
                theme.bg_tertiary
            };
            ui.painter().rect_filled(plus_response.rect, 6.0, plus_bg);
            if plus_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let plus_color = if plus_response.hovered() {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            let plus_rect = Rect::from_center_size(plus_response.rect.center(), vec2(14.0, 14.0));
            draw_icon(ui, Icon::Plus, plus_rect, plus_color);
            if plus_response.clicked() {
                *value = (*value + 1.0).min(max);
            }

            // Value display with unit
            let display_text = if unit.is_empty() {
                format!("{}", *value as u32)
            } else {
                format!("{} {}", *value as u32, unit)
            };
            ui.add_sized(
                vec2(60.0, 32.0),
                egui::Label::new(
                    egui::RichText::new(display_text)
                        .color(theme.text_primary)
                        .strong(),
                ),
            );

            // Minus button
            let minus_response = ui.allocate_response(vec2(32.0, 32.0), egui::Sense::click());
            let minus_bg = if minus_response.hovered() {
                theme.bg_hover
            } else {
                theme.bg_tertiary
            };
            ui.painter().rect_filled(minus_response.rect, 6.0, minus_bg);
            if minus_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            let minus_color = if minus_response.hovered() {
                theme.text_primary
            } else {
                theme.text_secondary
            };
            let minus_rect = Rect::from_center_size(minus_response.rect.center(), vec2(14.0, 14.0));
            draw_icon(ui, Icon::Minus, minus_rect, minus_color);
            if minus_response.clicked() {
                *value = (*value - 1.0).max(min);
            }
        });
    });

    ui.add_space(theme.spacing_sm);
}

/// Draw a color picker row
fn color_picker_row(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    colors: &[&AccentColor],
    selected: &mut AccentColor,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme.text_secondary));

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for accent in colors.iter().rev() {
                let is_selected = *selected == **accent;
                let (color, _) = accent.gradient();

                let size = if is_selected { 26.0 } else { 22.0 };
                let (rect, response) =
                    ui.allocate_exact_size(vec2(size, size), egui::Sense::click());

                if response.clicked() {
                    *selected = **accent;
                }

                ui.painter()
                    .circle_filled(rect.center(), size / 2.0 - 2.0, color);

                if is_selected {
                    ui.painter().circle_stroke(
                        rect.center(),
                        size / 2.0,
                        egui::Stroke::new(2.0, Color32::WHITE),
                    );
                }

                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    egui::show_tooltip(
                        ui.ctx(),
                        ui.layer_id(),
                        egui::Id::new(accent.name()),
                        |ui| {
                            ui.label(accent.name());
                        },
                    );
                }
            }
        });
    });
}

/// Draw a toggle row with checkbox
fn toggle_row(ui: &mut Ui, theme: &Theme, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme.text_secondary));

        // Use right-to-left layout for checkbox alignment
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Checkbox::without_text(value));
        });
    });

    ui.add_space(theme.spacing_xs);
}

/// Draw a hotkey display row (read-only)
fn hotkey_row(ui: &mut Ui, theme: &Theme, label: &str, hotkey: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme.text_secondary));

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            // Display hotkey in a styled box
            let hotkey_text = egui::RichText::new(hotkey)
                .color(theme.text_primary)
                .strong()
                .small();

            egui::Frame::none()
                .fill(theme.bg_tertiary)
                .rounding(4.0)
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.label(hotkey_text);
                });
        });
    });

    ui.add_space(theme.spacing_xs);
}
