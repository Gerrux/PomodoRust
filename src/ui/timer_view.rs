//! Compact timer view (main widget) - Responsive layout with TUI mode

use egui::{vec2, Align, FontId, Layout, Ui};

use super::components::{
    AsciiProgressBar, AsciiSessionDots, AsciiSpinner, AsciiTime, CircularProgress, GradientButton,
    Icon, IconButton,
};
use super::theme::Theme;
use crate::core::{Session, SessionType};

/// Actions that can be triggered from the timer view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerAction {
    Toggle,
    Skip,
    Reset,
    OpenStats,
    OpenSettings,
}

/// The compact timer view with responsive layout
pub struct TimerView {
    time_offset: f32, // For animations
}

impl TimerView {
    pub fn new() -> Self {
        Self { time_offset: 0.0 }
    }

    /// Show the timer view and return any action triggered
    pub fn show(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        pulse: f32,
    ) -> Option<TimerAction> {
        // Update animation time
        self.time_offset += ui.ctx().input(|i| i.unstable_dt);

        // Check if we should use TUI/retro style
        if theme.accent.is_retro() {
            self.show_tui_style(ui, session, theme, pulse)
        } else {
            self.show_modern_style(ui, session, theme, pulse)
        }
    }

    /// Modern style with circular progress
    fn show_modern_style(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        pulse: f32,
    ) -> Option<TimerAction> {
        let mut action = None;

        // Get available size for responsive calculations
        let available = ui.available_size();
        let min_dim = available.x.min(available.y);

        // Responsive sizing based on available space
        let timer_radius = (min_dim * 0.28).clamp(60.0, 120.0);
        let timer_thickness = (timer_radius * 0.08).clamp(4.0, 10.0);
        let control_btn_size = (min_dim * 0.11).clamp(36.0, 48.0); // Smaller buttons
        let nav_btn_width = (available.x * 0.35).clamp(100.0, 150.0);
        let nav_btn_height = (min_dim * 0.09).clamp(32.0, 44.0);
        let spacing = (min_dim * 0.04).clamp(8.0, 24.0);

        // Responsive font sizes - larger timer text
        let timer_font_size = (timer_radius * 0.55).clamp(28.0, 64.0);
        let label_font_size = (timer_radius * 0.16).clamp(11.0, 18.0);

        // Use centered vertical layout
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(spacing);

            // Circular progress with timer
            let (start_color, end_color) = theme.session_gradient(session.session_type());
            let progress = session.timer().progress();

            CircularProgress::new(progress)
                .with_radius(timer_radius)
                .with_thickness(timer_thickness)
                .with_colors(start_color, end_color)
                .with_bg_color(theme.bg_tertiary)
                .with_pulse(if session.timer().is_running() {
                    pulse
                } else {
                    0.0
                })
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        // Push content down slightly within the circle
                        ui.add_space(timer_radius * 0.15);

                        // Timer display
                        ui.label(
                            egui::RichText::new(session.timer().remaining_formatted())
                                .size(timer_font_size)
                                .color(theme.text_primary),
                        );

                        ui.add_space(2.0);

                        // Session type label
                        let label_color = Theme::lerp_color(start_color, end_color, 0.5);
                        ui.label(
                            egui::RichText::new(session.session_type().label())
                                .size(label_font_size)
                                .color(label_color),
                        );
                    });
                });

            ui.add_space(spacing * 0.5);

            // Control buttons - centered with spacing in the middle
            let btn_spacing = spacing * 0.75;
            let half_width = ui.available_width() / 2.0;
            let btn_gap = btn_spacing / 2.0;

            ui.horizontal(|ui| {
                // Left half - play/pause aligned to right
                ui.allocate_ui_with_layout(
                    vec2(half_width - btn_gap, control_btn_size),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        let is_running = session.timer().is_running();
                        let play_icon = if is_running { Icon::Pause } else { Icon::Play };

                        if IconButton::new(play_icon)
                            .with_size(control_btn_size)
                            .with_icon_scale(0.45)
                            .filled(false)
                            .with_gradient(start_color, end_color)
                            .show(ui, theme)
                            .clicked()
                        {
                            action = Some(TimerAction::Toggle);
                        }
                    },
                );

                // Right half - skip aligned to left
                ui.allocate_ui_with_layout(
                    vec2(half_width - btn_gap, control_btn_size),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        if IconButton::new(Icon::SkipForward)
                            .with_size(control_btn_size)
                            .with_icon_scale(0.45)
                            .filled(false)
                            .with_gradient(start_color, end_color)
                            .show(ui, theme)
                            .clicked()
                        {
                            action = Some(TimerAction::Skip);
                        }
                    },
                );
            });

            ui.add_space(spacing * 1.5);

            // Session progress dots
            self.show_session_dots(ui, session, theme, min_dim);

            ui.add_space(spacing * 0.75);

            // Bottom navigation buttons - centered with spacing in the middle
            let nav_gap = spacing * 0.25;
            let nav_half_width = ui.available_width() / 2.0;

            ui.horizontal(|ui| {
                // Left half - Stats aligned to right
                ui.allocate_ui_with_layout(
                    vec2(nav_half_width - nav_gap, nav_btn_height),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        if GradientButton::new("Stats")
                            .with_size(vec2(nav_btn_width, nav_btn_height))
                            .with_gradient(theme.bg_tertiary, theme.bg_hover)
                            .with_id("stats_btn")
                            .show(ui, theme)
                            .clicked()
                        {
                            action = Some(TimerAction::OpenStats);
                        }
                    },
                );

                // Right half - Settings aligned to left
                ui.allocate_ui_with_layout(
                    vec2(nav_half_width - nav_gap, nav_btn_height),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        if GradientButton::new("Settings")
                            .with_size(vec2(nav_btn_width, nav_btn_height))
                            .with_gradient(theme.bg_tertiary, theme.bg_hover)
                            .with_id("settings_btn")
                            .show(ui, theme)
                            .clicked()
                        {
                            action = Some(TimerAction::OpenSettings);
                        }
                    },
                );
            });

            ui.add_space(spacing);
        });

        action
    }

    /// TUI/Retro style with ASCII art
    fn show_tui_style(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        _pulse: f32,
    ) -> Option<TimerAction> {
        let mut action = None;

        let available = ui.available_size();
        let min_dim = available.x.min(available.y);

        // Responsive sizing
        let ascii_font_size = (min_dim * 0.045).clamp(12.0, 20.0);
        let label_font_size = (min_dim * 0.035).clamp(10.0, 16.0);
        let btn_font_size = (min_dim * 0.03).clamp(11.0, 14.0);
        let spacing = (min_dim * 0.03).clamp(8.0, 20.0);

        // Calculate progress bar width based on available space and font size
        // Monospace character width is approximately font_size * 0.6
        let progress_font_size = ascii_font_size * 0.9;
        let char_width = progress_font_size * 0.6;
        let max_chars = ((available.x * 0.85) / char_width) as usize;
        // Subtract 4 for brackets and some padding to prevent wrapping
        let progress_width = max_chars.saturating_sub(4).clamp(15, 40);

        // Use session-based colors (like modern style)
        let (accent_start, accent_end) = theme.session_gradient(session.session_type());
        let accent = Theme::lerp_color(accent_start, accent_end, 0.5);

        // Get time values
        let remaining = session.timer().remaining();
        let minutes = (remaining.as_secs() / 60) as u32;
        let seconds = (remaining.as_secs() % 60) as u32;
        let progress = session.timer().progress();
        let is_running = session.timer().is_running();

        // Check if we have enough height for navigation buttons
        let show_nav = available.y > 350.0;

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(spacing * 0.3);

            // Session type with spinner (centered)
            let spinner = if is_running {
                AsciiSpinner::braille_frame(self.time_offset)
            } else {
                "○"
            };

            let session_label = match session.session_type() {
                SessionType::Work => "FOCUS",
                SessionType::ShortBreak => "SHORT BREAK",
                SessionType::LongBreak => "LONG BREAK",
            };

            ui.label(
                egui::RichText::new(format!("{} {}", spinner, session_label))
                    .font(FontId::monospace(label_font_size * 1.2))
                    .color(accent),
            );

            ui.add_space(spacing * 0.3);

            // ASCII time display
            AsciiTime::draw(ui, minutes, seconds, accent, ascii_font_size);

            ui.add_space(spacing * 0.5);

            // ASCII progress bar
            let progress_bar = AsciiProgressBar::render_gradient(progress, progress_width);
            ui.label(
                egui::RichText::new(&progress_bar)
                    .font(FontId::monospace(ascii_font_size * 0.9))
                    .color(accent),
            );

            // Progress percentage
            ui.label(
                egui::RichText::new(format!("{:>3.0}%", progress * 100.0))
                    .font(FontId::monospace(label_font_size))
                    .color(theme.text_muted),
            );

            ui.add_space(spacing * 0.5);

            // ASCII session dots
            let session_dots = AsciiSessionDots::render(
                session.completed_work_sessions(),
                session.total_sessions_in_cycle(),
            );
            ui.label(
                egui::RichText::new(&session_dots)
                    .font(FontId::monospace(label_font_size))
                    .color(theme.text_secondary),
            );

            // Session counter
            ui.label(
                egui::RichText::new(format!(
                    "Session {}/{}",
                    session.current_session_in_cycle(),
                    session.total_sessions_in_cycle()
                ))
                .font(FontId::monospace(label_font_size * 0.9))
                .color(theme.text_muted),
            );

            ui.add_space(spacing * 0.5);

            // ASCII control buttons - centered with spacing in the middle
            let btn_height = btn_font_size + 16.0;
            let gray = theme.text_muted;
            let half_width = ui.available_width() / 2.0;
            let btn_gap = spacing * 0.5;

            ui.horizontal(|ui| {
                // Left half - button aligned to right edge
                ui.allocate_ui_with_layout(
                    vec2(half_width - btn_gap, btn_height),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        let play_text = if is_running {
                            "[ ■ PAUSE ]"
                        } else {
                            "[ ► START ]"
                        };

                        let play_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new(play_text)
                                    .font(FontId::monospace(btn_font_size))
                                    .color(gray),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, gray)),
                        );

                        if play_btn.hovered() || play_btn.has_focus() {
                            let color = if is_running { theme.warning } else { accent };
                            let rect = play_btn.rect;
                            ui.painter()
                                .rect_filled(rect, egui::Rounding::same(2.0), theme.bg_tertiary);
                            ui.painter().rect_stroke(
                                rect,
                                egui::Rounding::same(2.0),
                                egui::Stroke::new(1.0, color),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                play_text,
                                FontId::monospace(btn_font_size),
                                color,
                            );
                        }

                        if play_btn.clicked() {
                            action = Some(TimerAction::Toggle);
                        }
                    },
                );

                // Right half - button aligned to left edge
                ui.allocate_ui_with_layout(
                    vec2(half_width - btn_gap, btn_height),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        let skip_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new("[ » SKIP ]")
                                    .font(FontId::monospace(btn_font_size))
                                    .color(gray),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, gray)),
                        );

                        if skip_btn.hovered() || skip_btn.has_focus() {
                            let rect = skip_btn.rect;
                            ui.painter()
                                .rect_filled(rect, egui::Rounding::same(2.0), theme.bg_tertiary);
                            ui.painter().rect_stroke(
                                rect,
                                egui::Rounding::same(2.0),
                                egui::Stroke::new(1.0, accent),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "[ » SKIP ]",
                                FontId::monospace(btn_font_size),
                                accent,
                            );
                        }

                        if skip_btn.clicked() {
                            action = Some(TimerAction::Skip);
                        }
                    },
                );
            });

            // Navigation (only show if enough vertical space)
            if show_nav {
                ui.add_space(spacing * 0.5);

                // Separator
                ui.label(
                    egui::RichText::new("─────────────────────")
                        .font(FontId::monospace(btn_font_size))
                        .color(theme.border_subtle),
                );

                ui.add_space(spacing * 0.3);

                // Navigation - centered with spacing in the middle
                let nav_half_width = ui.available_width() / 2.0;

                ui.horizontal(|ui| {
                    // Left half - Statistics aligned to right
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - btn_gap, btn_height),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            let dash_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("[ Statistics ]")
                                        .font(FontId::monospace(btn_font_size))
                                        .color(theme.text_secondary),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            );

                            if dash_btn.clicked() {
                                action = Some(TimerAction::OpenStats);
                            }
                        },
                    );

                    // Right half - Settings aligned to left
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - btn_gap, btn_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let settings_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new("[ Settings ]")
                                        .font(FontId::monospace(btn_font_size))
                                        .color(theme.text_secondary),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            );

                            if settings_btn.clicked() {
                                action = Some(TimerAction::OpenSettings);
                            }
                        },
                    );
                });
            }

            ui.add_space(spacing * 0.3);
        });

        // Request continuous repaint for animations
        if is_running {
            ui.ctx().request_repaint();
        }

        action
    }

    fn show_session_dots(&self, ui: &mut Ui, session: &Session, theme: &Theme, scale: f32) {
        let total = session.total_sessions_in_cycle() as usize;
        // Current session index (0-based)
        let current_idx = (session.current_session_in_cycle() as usize).saturating_sub(1);

        // Responsive dot sizing
        let dot_radius = (scale * 0.015).clamp(4.0, 7.0);
        let dot_spacing = (scale * 0.04).clamp(12.0, 20.0);
        let caption_size = (scale * 0.035).clamp(10.0, 14.0);

        // Calculate total width and allocate centered rect
        let dots_width = dot_spacing * (total - 1) as f32 + dot_radius * 2.0;
        let height = dot_radius * 3.0;
        let (rect, _) = ui.allocate_exact_size(vec2(dots_width, height), egui::Sense::hover());

        // Draw dots manually for perfect centering
        let start_x = rect.left() + dot_radius;
        let center_y = rect.center().y;
        for i in 0..total {
            let is_completed = i < current_idx;
            let is_current = i == current_idx;

            let color = if is_completed {
                theme.success
            } else if is_current {
                let (start, end) = theme.session_gradient(session.session_type());
                Theme::lerp_color(start, end, 0.5)
            } else {
                theme.bg_hover
            };

            let center = egui::pos2(start_x + dot_spacing * i as f32, center_y);
            ui.painter().circle_filled(center, dot_radius, color);
        }

        ui.add_space(4.0);

        // Session text
        ui.label(
            egui::RichText::new(format!(
                "Session {}/{}",
                session.current_session_in_cycle(),
                session.total_sessions_in_cycle()
            ))
            .size(caption_size)
            .color(theme.text_muted),
        );
    }
}

impl Default for TimerView {
    fn default() -> Self {
        Self::new()
    }
}
