//! Settings panel
//!
//! Provides the settings UI with local editing state that syncs back to Config.
//! The editing state is kept separate to allow for:
//! - Smooth slider/input interaction
//! - Validation before applying changes
//! - Consistent state management

mod components;
mod state;

use egui::{vec2, Layout, Ui};

use super::components::{draw_icon, Card, Icon, IconButton};
use super::theme::{AccentColor, Theme, ThemeMode};
use crate::data::{Config, NotificationSound};
use components::{
    color_picker_row, duration_row, duration_row_with_unit, hotkey_row, section_header, toggle_row,
};
pub use state::SettingsState;

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

        // Max-width container - centered with limited width like web pages
        let max_content_width = 600.0;
        let available_width = ui.available_width();
        let content_width = available_width.min(max_content_width);
        let horizontal_margin = ((available_width - content_width) / 2.0).max(0.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Apply horizontal margins for centering
            let margin = egui::Margin {
                left: horizontal_margin,
                right: horizontal_margin,
                top: 0.0,
                bottom: 0.0,
            };
            egui::Frame::none().inner_margin(margin).show(ui, |ui| {
            // Force dark theme styles for all widgets (fixes Windows 10 light theme issues)
            let visuals = &mut ui.style_mut().visuals;
            visuals.widgets.inactive.bg_fill = theme.bg_tertiary;
            visuals.widgets.inactive.weak_bg_fill = theme.bg_tertiary;
            visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, theme.text_primary);
            visuals.widgets.hovered.bg_fill = theme.bg_hover;
            visuals.widgets.hovered.weak_bg_fill = theme.bg_hover;
            visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, theme.text_primary);
            visuals.widgets.active.bg_fill = theme.bg_active;
            visuals.widgets.active.weak_bg_fill = theme.bg_active;
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, theme.text_primary);
            visuals.widgets.open.bg_fill = theme.bg_tertiary;
            visuals.widgets.open.weak_bg_fill = theme.bg_tertiary;
            visuals.widgets.open.fg_stroke = egui::Stroke::new(1.0, theme.text_primary);
            visuals.widgets.noninteractive.bg_fill = theme.bg_secondary;
            visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, theme.text_secondary);
            visuals.selection.bg_fill = Theme::with_alpha(theme.accent.solid(), 100);
            visuals.selection.stroke = egui::Stroke::new(1.0, theme.accent.solid());

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
                    egui::RichText::new("Настройки")
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
                        // Test button with play icon
                        let (btn_rect, btn_resp) =
                            ui.allocate_exact_size(vec2(28.0, 22.0), egui::Sense::click());
                        ui.painter().rect_filled(btn_rect, theme.rounding_sm, theme.bg_tertiary);
                        ui.painter().rect_stroke(btn_rect, theme.rounding_sm, egui::Stroke::new(1.0, theme.border_subtle));
                        let icon_rect = egui::Rect::from_center_size(btn_rect.center(), vec2(12.0, 12.0));
                        draw_icon(ui, Icon::Play, icon_rect, theme.text_primary);
                        if btn_resp.on_hover_text("Test sound").clicked() {
                            test_sound = true;
                        }

                        ui.add_space(4.0);

                        // Apply dark theme styles for ComboBox button
                        ui.style_mut().visuals.widgets.inactive.bg_fill = theme.bg_tertiary;
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = theme.bg_tertiary;
                        ui.style_mut().visuals.widgets.hovered.bg_fill = theme.bg_hover;
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = theme.bg_hover;
                        ui.style_mut().visuals.widgets.active.bg_fill = theme.bg_active;
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = theme.bg_active;
                        ui.style_mut().visuals.widgets.open.bg_fill = theme.bg_tertiary;
                        ui.style_mut().visuals.widgets.open.weak_bg_fill = theme.bg_tertiary;

                        egui::ComboBox::from_id_salt("notification_sound")
                            .selected_text(
                                egui::RichText::new(self.state.notification_sound.name())
                                    .color(theme.text_primary),
                            )
                            .width(120.0)
                            .show_ui(ui, |ui| {
                                ui.style_mut().visuals.widgets.inactive.bg_fill = theme.bg_secondary;
                                ui.style_mut().visuals.widgets.hovered.bg_fill = theme.bg_hover;
                                for sound in NotificationSound::all() {
                                    ui.selectable_value(
                                        &mut self.state.notification_sound,
                                        *sound,
                                        egui::RichText::new(sound.name()).color(theme.text_primary),
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

                // Theme mode selector
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Theme").color(theme.text_secondary));

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        // Apply dark theme styles for ComboBox button
                        ui.style_mut().visuals.widgets.inactive.bg_fill = theme.bg_tertiary;
                        ui.style_mut().visuals.widgets.inactive.weak_bg_fill = theme.bg_tertiary;
                        ui.style_mut().visuals.widgets.hovered.bg_fill = theme.bg_hover;
                        ui.style_mut().visuals.widgets.hovered.weak_bg_fill = theme.bg_hover;
                        ui.style_mut().visuals.widgets.active.bg_fill = theme.bg_active;
                        ui.style_mut().visuals.widgets.active.weak_bg_fill = theme.bg_active;
                        ui.style_mut().visuals.widgets.open.bg_fill = theme.bg_tertiary;
                        ui.style_mut().visuals.widgets.open.weak_bg_fill = theme.bg_tertiary;

                        egui::ComboBox::from_id_salt("theme_mode")
                            .selected_text(
                                egui::RichText::new(self.state.theme_mode.name())
                                    .color(theme.text_primary),
                            )
                            .width(100.0)
                            .show_ui(ui, |ui| {
                                ui.style_mut().visuals.widgets.inactive.bg_fill = theme.bg_secondary;
                                ui.style_mut().visuals.widgets.hovered.bg_fill = theme.bg_hover;
                                for mode in ThemeMode::all() {
                                    ui.selectable_value(
                                        &mut self.state.theme_mode,
                                        *mode,
                                        egui::RichText::new(mode.name()).color(theme.text_primary),
                                    );
                                }
                            });
                    });
                });

                ui.add_space(theme.spacing_sm);

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

                let copy_btn = egui::Button::new(
                    egui::RichText::new("Copy PATH command").color(theme.text_primary),
                )
                .fill(theme.bg_tertiary)
                .stroke(egui::Stroke::new(1.0, theme.border_subtle));

                if ui
                    .add_sized(vec2(ui.available_width(), 32.0), copy_btn)
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
                        let preset_btn = egui::Button::new(
                            egui::RichText::new(format!("{}\n{}", preset.0, preset.1))
                                .color(theme.text_primary),
                        )
                        .fill(theme.bg_tertiary)
                        .stroke(egui::Stroke::new(1.0, theme.border_subtle));

                        if ui
                            .add_sized(vec2(button_width, 48.0), preset_btn)
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

                let reset_btn = egui::Button::new(
                    egui::RichText::new("Reset to Defaults").color(theme.text_primary),
                )
                .fill(theme.bg_tertiary)
                .stroke(egui::Stroke::new(1.0, theme.border_subtle));

                if ui.add(reset_btn).clicked() {
                    action = Some(SettingsAction::ResetDefaults);
                }
            });

            ui.add_space(theme.spacing_lg);
            }); // Frame
        }); // ScrollArea

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
