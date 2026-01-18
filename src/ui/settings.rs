//! Settings panel

use egui::{vec2, Color32, Layout, Rect, Ui};

use super::components::{draw_icon, Card, Icon, IconButton};
use super::theme::{AccentColor, Theme};
use crate::data::Config;

/// Actions from settings
#[derive(Debug, Clone, PartialEq)]
pub enum SettingsAction {
    Back,
    UpdateConfig(Config),
    SelectPreset(usize),
    ResetDefaults,
}

/// Settings view
pub struct SettingsView {
    // Local copies for editing
    pub work_duration: f32,
    pub short_break: f32,
    pub long_break: f32,
    pub sessions_before_long: f32,
    pub volume: f32,
    pub auto_start_breaks: bool,
    pub auto_start_work: bool,
    pub minimize_to_tray: bool,
    pub start_with_windows: bool,
    pub selected_accent: AccentColor,
}

impl SettingsView {
    pub fn new(config: &Config) -> Self {
        Self {
            work_duration: config.timer.work_duration as f32,
            short_break: config.timer.short_break as f32,
            long_break: config.timer.long_break as f32,
            sessions_before_long: config.timer.sessions_before_long as f32,
            volume: config.sounds.volume as f32,
            auto_start_breaks: config.timer.auto_start_breaks,
            auto_start_work: config.timer.auto_start_work,
            minimize_to_tray: config.system.minimize_to_tray,
            start_with_windows: config.system.start_with_windows,
            selected_accent: config.appearance.accent_color,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        config: &Config,
        theme: &Theme,
    ) -> Option<SettingsAction> {
        let mut action = None;

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

                duration_row(ui, theme, "Focus Duration", &mut self.work_duration, 1.0, 90.0);
                duration_row(ui, theme, "Short Break", &mut self.short_break, 1.0, 30.0);
                duration_row(ui, theme, "Long Break", &mut self.long_break, 5.0, 60.0);
                duration_row_with_unit(ui, theme, "Sessions before long break", &mut self.sessions_before_long, 2.0, 8.0, "");

                ui.add_space(theme.spacing_sm);

                toggle_row(ui, theme, "Auto-start breaks", &mut self.auto_start_breaks);
                toggle_row(ui, theme, "Auto-start pomodoros", &mut self.auto_start_work);
            });

            ui.add_space(theme.spacing_md);

            // Sounds section
            section_header(ui, theme, "Sounds");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Volume")
                            .color(theme.text_secondary),
                    );

                    // Use right-to-left layout for proper alignment
                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("{}%", self.volume as u32))
                                .color(theme.text_muted),
                        );

                        ui.add_sized(
                            vec2(120.0, 20.0),
                            egui::Slider::new(&mut self.volume, 0.0..=100.0).show_value(false)
                        );
                    });
                });
            });

            ui.add_space(theme.spacing_md);

            // Appearance section
            section_header(ui, theme, "Appearance");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                // Standard colors
                let standard_colors: Vec<_> = AccentColor::all().iter().filter(|c| !c.is_retro()).collect();
                let retro_colors: Vec<_> = AccentColor::all().iter().filter(|c| c.is_retro()).collect();

                color_picker_row(ui, theme, "Accent Color", &standard_colors, &mut self.selected_accent);

                ui.add_space(theme.spacing_sm);

                color_picker_row(ui, theme, "Retro Themes", &retro_colors, &mut self.selected_accent);
            });

            ui.add_space(theme.spacing_md);

            // System section
            section_header(ui, theme, "System");
            Card::new().show(ui, theme, |ui| {
                ui.set_min_width(ui.available_width() - theme.spacing_md * 2.0);

                toggle_row(ui, theme, "Start with Windows", &mut self.start_with_windows);
                toggle_row(ui, theme, "Minimize to tray", &mut self.minimize_to_tray);
            });

            ui.add_space(theme.spacing_md);

            // Presets section
            section_header(ui, theme, "Presets");
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
                                egui::Button::new(format!("{}\n{}", preset.0, preset.1))
                            )
                            .clicked()
                        {
                            action = Some(SettingsAction::SelectPreset(i));
                        }
                    }
                });
            });

            ui.add_space(theme.spacing_xl);

            // Reset button
            ui.horizontal(|ui| {
                ui.add_space((ui.available_width() - 150.0) / 2.0);

                if ui
                    .add(
                        egui::Button::new("Reset to Defaults")
                            .fill(theme.bg_tertiary),
                    )
                    .clicked()
                {
                    action = Some(SettingsAction::ResetDefaults);
                }
            });
        });

        // Check if config changed
        if action.is_none() && self.has_changes(config) {
            let new_config = self.build_config(config);
            action = Some(SettingsAction::UpdateConfig(new_config));
        }

        action
    }

    fn has_changes(&self, config: &Config) -> bool {
        self.work_duration as u32 != config.timer.work_duration
            || self.short_break as u32 != config.timer.short_break
            || self.long_break as u32 != config.timer.long_break
            || self.sessions_before_long as u32 != config.timer.sessions_before_long
            || self.volume as u32 != config.sounds.volume
            || self.auto_start_breaks != config.timer.auto_start_breaks
            || self.auto_start_work != config.timer.auto_start_work
            || self.minimize_to_tray != config.system.minimize_to_tray
            || self.start_with_windows != config.system.start_with_windows
            || self.selected_accent != config.appearance.accent_color
    }

    fn build_config(&self, original: &Config) -> Config {
        let mut config = original.clone();
        config.timer.work_duration = self.work_duration as u32;
        config.timer.short_break = self.short_break as u32;
        config.timer.long_break = self.long_break as u32;
        config.timer.sessions_before_long = self.sessions_before_long as u32;
        config.timer.auto_start_breaks = self.auto_start_breaks;
        config.timer.auto_start_work = self.auto_start_work;
        config.sounds.volume = self.volume as u32;
        config.system.minimize_to_tray = self.minimize_to_tray;
        config.system.start_with_windows = self.start_with_windows;
        config.appearance.accent_color = self.selected_accent;
        config
    }

    /// Reset to match config
    pub fn reset_from_config(&mut self, config: &Config) {
        self.work_duration = config.timer.work_duration as f32;
        self.short_break = config.timer.short_break as f32;
        self.long_break = config.timer.long_break as f32;
        self.sessions_before_long = config.timer.sessions_before_long as f32;
        self.volume = config.sounds.volume as f32;
        self.auto_start_breaks = config.timer.auto_start_breaks;
        self.auto_start_work = config.timer.auto_start_work;
        self.minimize_to_tray = config.system.minimize_to_tray;
        self.start_with_windows = config.system.start_with_windows;
        self.selected_accent = config.appearance.accent_color;
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
fn duration_row_with_unit(ui: &mut Ui, theme: &Theme, label: &str, value: &mut f32, min: f32, max: f32, unit: &str) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(theme.text_secondary),
        );

        // Use right-to-left layout for controls alignment
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            // Plus button (appears last visually, first in RTL)
            let plus_response = ui.allocate_response(vec2(32.0, 32.0), egui::Sense::click());
            let plus_bg = if plus_response.hovered() { theme.bg_hover } else { theme.bg_tertiary };
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
            let minus_bg = if minus_response.hovered() { theme.bg_hover } else { theme.bg_tertiary };
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
fn color_picker_row(ui: &mut Ui, theme: &Theme, label: &str, colors: &[&AccentColor], selected: &mut AccentColor) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(theme.text_secondary),
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = 6.0;
            for accent in colors.iter().rev() {
                let is_selected = *selected == **accent;
                let (color, _) = accent.gradient();

                let size = if is_selected { 26.0 } else { 22.0 };
                let (rect, response) = ui.allocate_exact_size(
                    vec2(size, size),
                    egui::Sense::click(),
                );

                if response.clicked() {
                    *selected = **accent;
                }

                ui.painter().circle_filled(rect.center(), size / 2.0 - 2.0, color);

                if is_selected {
                    ui.painter().circle_stroke(
                        rect.center(),
                        size / 2.0,
                        egui::Stroke::new(2.0, Color32::WHITE),
                    );
                }

                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    egui::show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new(accent.name()), |ui| {
                        ui.label(accent.name());
                    });
                }
            }
        });
    });
}

/// Draw a toggle row with checkbox
fn toggle_row(ui: &mut Ui, theme: &Theme, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(theme.text_secondary),
        );

        // Use right-to-left layout for checkbox alignment
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add(egui::Checkbox::without_text(value));
        });
    });

    ui.add_space(theme.spacing_xs);
}
