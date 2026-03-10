//! Compact timer view (main widget) - Responsive layout with TUI mode

mod tui_style;

use egui::{vec2, Align, Layout, RichText, Ui};

use super::components::{CircularProgress, Icon, IconButton};
use super::theme::Theme;
use crate::core::Session;
use crate::data::todo::QueuedTask;

/// Actions that can be triggered from the timer view
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerAction {
    Toggle,
    Skip,
    Reset,
    OpenStats,
    OpenSettings,
    OpenTodo,
    OpenQueue,
}

// Layout constants for responsive sizing
const RADIUS_FACTOR: f32 = 0.28;
const THICKNESS_RATIO: f32 = 0.08;
const CONTROL_BTN_FACTOR: f32 = 0.11;
const SPACING_FACTOR: f32 = 0.04;
const TIMER_FONT_RATIO: f32 = 0.55;
const LABEL_FONT_RATIO: f32 = 0.16;
const NAV_BTN_WIDTH_FACTOR: f32 = 0.35;
const NAV_BTN_HEIGHT_FACTOR: f32 = 0.08;
const DOT_RADIUS_FACTOR: f32 = 0.015;
const DOT_SPACING_FACTOR: f32 = 0.04;
const DOT_CAPTION_FACTOR: f32 = 0.035;

/// Maximum time_offset before wrapping (avoids float precision loss)
const TIME_OFFSET_WRAP: f32 = 1000.0;

/// The compact timer view with responsive layout
pub struct TimerView {
    time_offset: f32,
}

impl TimerView {
    pub fn new() -> Self {
        Self {
            time_offset: 0.0,
        }
    }

    /// Show the timer view and return any action triggered
    pub fn show(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        pulse: f32,
        window_opacity: u32,
        current_task: Option<&QueuedTask>,
        queue: &[QueuedTask],
    ) -> Option<TimerAction> {
        // Update animation time (wrap to avoid float precision loss)
        self.time_offset = (self.time_offset + ui.ctx().input(|i| i.unstable_dt)) % TIME_OFFSET_WRAP;

        // Check if we should use TUI/retro style
        if theme.accent.is_retro() {
            self.show_tui_style(ui, session, theme, pulse, current_task, queue)
        } else {
            self.show_modern_style(ui, session, theme, pulse, window_opacity, current_task, queue)
        }
    }

    /// Modern style with circular progress
    fn show_modern_style(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        pulse: f32,
        window_opacity: u32,
        current_task: Option<&QueuedTask>,
        queue: &[QueuedTask],
    ) -> Option<TimerAction> {
        let t = crate::i18n::tr();
        let mut action = None;

        // Get available size for responsive calculations
        let available = ui.available_size();
        let min_dim = available.x.min(available.y);

        // Responsive sizing based on available space
        let timer_radius = (min_dim * RADIUS_FACTOR).clamp(60.0, 120.0);
        let timer_thickness = (timer_radius * THICKNESS_RATIO).clamp(4.0, 10.0);
        let control_btn_size = (min_dim * CONTROL_BTN_FACTOR).clamp(36.0, 48.0);
        let spacing = (min_dim * SPACING_FACTOR).clamp(8.0, 24.0);

        // Responsive font sizes - larger timer text
        let timer_font_size = (timer_radius * TIMER_FONT_RATIO).clamp(28.0, 64.0);
        let label_font_size = (timer_radius * LABEL_FONT_RATIO).clamp(11.0, 18.0);

        // Use centered vertical layout
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(spacing);

            // Circular progress with timer
            let (start_color, end_color) = theme.session_gradient(session.session_type());
            let progress = session.timer().progress();

            // Adjust colors for light mode visibility at lower window opacity.
            // Maps opacity 100% -> 0.0 (normal) down to 30% -> 1.0 (fully darkened).
            let opacity_factor = ((100 - window_opacity.min(100)) as f32 / 70.0).min(1.0);

            let ring_bg_color = if theme.is_light {
                // Darken to black as opacity decreases
                let black = egui::Color32::from_rgb(20, 20, 20);
                Theme::lerp_color(theme.bg_tertiary, black, opacity_factor)
            } else {
                theme.bg_tertiary
            };

            CircularProgress::new(progress)
                .with_radius(timer_radius)
                .with_thickness(timer_thickness)
                .with_colors(start_color, end_color)
                .with_bg_color(ring_bg_color)
                .with_pulse(if session.timer().is_running() && !theme.reduced_motion {
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

                        // Session type label - darken to black as opacity decreases (light mode)
                        let base_label_color = Theme::lerp_color(start_color, end_color, 0.5);
                        let label_color = if theme.is_light {
                            let black = egui::Color32::from_rgb(10, 10, 10);
                            Theme::lerp_color(base_label_color, black, opacity_factor)
                        } else {
                            base_label_color
                        };
                        ui.label(
                            egui::RichText::new(t.session_label(session.session_type()))
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
                            .light_mode(theme.is_light)
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
                            .light_mode(theme.is_light)
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
            self.show_session_dots(ui, session, theme, min_dim, opacity_factor);

            ui.add_space(spacing * 0.5);

            // Current task display (if pinned)
            if let Some(task) = current_task {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(">")
                            .size(11.0)
                            .color(theme.accent.solid()),
                    );
                    let max_chars = ((ui.available_width() - 50.0) / (12.0 * 0.5)) as usize;
                    let title = crate::ui::todo_view::truncate_text(&task.title, max_chars.max(10));
                    ui.label(
                        RichText::new(&title)
                            .size(12.0)
                            .color(theme.text_secondary),
                    );
                    ui.label(
                        RichText::new(format!("{}/{}", task.completed_pomodoros, task.planned_pomodoros))
                            .size(11.0)
                            .color(theme.text_muted),
                    );
                });
            }

            // Navigation buttons - fade in/out on hover
            let is_hovered = ui.ctx().input(|i| {
                i.pointer.hover_pos()
                    .map(|pos| ui.max_rect().contains(pos))
                    .unwrap_or(false)
            });
            let nav_alpha = ui.ctx().animate_bool_with_time(
                ui.id().with("modern_nav_fade"),
                is_hovered,
                0.25,
            );

            if nav_alpha > 0.0 {
                let fade = |c: egui::Color32| -> egui::Color32 {
                    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as f32 * nav_alpha) as u8)
                };

                ui.add_space(spacing * 0.25);

                let nav_gap = spacing * 0.25;
                let nav_half_width = ui.available_width() / 2.0;
                let nav_btn_width = (available.x * NAV_BTN_WIDTH_FACTOR).clamp(80.0, 140.0);
                let nav_btn_height = (min_dim * NAV_BTN_HEIGHT_FACTOR).clamp(28.0, 38.0);
                let nav_btn_fill = fade(if theme.is_light {
                    egui::Color32::WHITE
                } else {
                    theme.bg_tertiary
                });

                ui.horizontal(|ui| {
                    // Left half - Stats aligned to right
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_gap, nav_btn_height),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            let btn = egui::Button::new(
                                RichText::new(t.nav.statistics).size(label_font_size * 0.85).color(fade(theme.text_primary)),
                            )
                            .fill(nav_btn_fill)
                            .stroke(egui::Stroke::new(1.0, fade(theme.border_default)))
                            .rounding(theme.rounding_md)
                            .min_size(vec2(nav_btn_width, nav_btn_height));
                            if ui.add(btn).clicked() {
                                action = Some(TimerAction::OpenStats);
                            }
                        },
                    );

                    // Right half - Settings aligned to left
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_gap, nav_btn_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let btn = egui::Button::new(
                                RichText::new(t.nav.settings).size(label_font_size * 0.85).color(fade(theme.text_primary)),
                            )
                            .fill(nav_btn_fill)
                            .stroke(egui::Stroke::new(1.0, fade(theme.border_default)))
                            .rounding(theme.rounding_md)
                            .min_size(vec2(nav_btn_width, nav_btn_height));
                            if ui.add(btn).clicked() {
                                action = Some(TimerAction::OpenSettings);
                            }
                        },
                    );
                });

                ui.add_space(spacing * 0.15);

                ui.horizontal(|ui| {
                    // Left half - Todo aligned to right
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_gap, nav_btn_height),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            let btn = egui::Button::new(
                                RichText::new(t.nav.tasks).size(label_font_size * 0.85).color(fade(theme.text_primary)),
                            )
                            .fill(nav_btn_fill)
                            .stroke(egui::Stroke::new(1.0, fade(theme.border_default)))
                            .rounding(theme.rounding_md)
                            .min_size(vec2(nav_btn_width, nav_btn_height));
                            if ui.add(btn).clicked() {
                                action = Some(TimerAction::OpenTodo);
                            }
                        },
                    );

                    // Right half - Queue aligned to left
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_gap, nav_btn_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let queue_text = if queue.is_empty() {
                                t.nav.queue.to_string()
                            } else {
                                format!("{} ({})", t.nav.queue, queue.len())
                            };
                            let btn = egui::Button::new(
                                RichText::new(&queue_text).size(label_font_size * 0.85).color(fade(theme.text_primary)),
                            )
                            .fill(nav_btn_fill)
                            .stroke(egui::Stroke::new(1.0, fade(theme.border_default)))
                            .rounding(theme.rounding_md)
                            .min_size(vec2(nav_btn_width, nav_btn_height));
                            if ui.add(btn).clicked() {
                                action = Some(TimerAction::OpenQueue);
                            }
                        },
                    );
                });

                ui.add_space(spacing * 0.5);

                // Request repaint during animation
                if nav_alpha < 1.0 {
                    ui.ctx().request_repaint();
                }
            }
        });

        action
    }

    fn show_session_dots(
        &self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        scale: f32,
        opacity_factor: f32,
    ) {
        let total = session.total_sessions_in_cycle() as usize;
        // Current session index (0-based)
        let current_idx = (session.current_session_in_cycle() as usize).saturating_sub(1);

        // Responsive dot sizing
        let dot_radius = (scale * DOT_RADIUS_FACTOR).clamp(4.0, 7.0);
        let dot_spacing = (scale * DOT_SPACING_FACTOR).clamp(12.0, 20.0);
        let caption_size = (scale * DOT_CAPTION_FACTOR).clamp(10.0, 14.0);

        // Calculate total width and allocate centered rect
        let dots_width = dot_spacing * (total - 1) as f32 + dot_radius * 2.0;
        let height = dot_radius * 3.0;
        let (rect, _) = ui.allocate_exact_size(vec2(dots_width, height), egui::Sense::hover());

        let black = egui::Color32::from_rgb(20, 20, 20);

        // Draw dots manually for perfect centering
        // Only completed sessions are filled, current/future are outlined
        let start_x = rect.left() + dot_radius;
        let center_y = rect.center().y;
        let stroke_width = (dot_radius * 0.3).clamp(1.5, 2.5);

        for i in 0..total {
            let is_completed = i < current_idx;
            let is_current = i == current_idx;

            let base_color = if is_completed {
                theme.success
            } else if is_current {
                let (start, end) = theme.session_gradient(session.session_type());
                Theme::lerp_color(start, end, 0.5)
            } else {
                theme.border_default
            };

            // Darken to black in light mode as opacity decreases
            let color = if theme.is_light {
                Theme::lerp_color(base_color, black, opacity_factor)
            } else {
                base_color
            };

            let center = egui::pos2(start_x + dot_spacing * i as f32, center_y);

            if is_completed {
                // Filled circle for completed
                ui.painter().circle_filled(center, dot_radius, color);
            } else {
                // Outline only for current and future
                ui.painter().circle_stroke(
                    center,
                    dot_radius,
                    egui::Stroke::new(stroke_width, color),
                );
            }
        }

        ui.add_space(4.0);

        // Session text - darken to black in light mode
        let text_color = if theme.is_light {
            Theme::lerp_color(theme.text_muted, black, opacity_factor)
        } else {
            theme.text_muted
        };

        let t = crate::i18n::tr();
        ui.label(
            egui::RichText::new(format!(
                "{} {}/{}",
                t.timer.session,
                session.current_session_in_cycle(),
                session.total_sessions_in_cycle()
            ))
            .size(caption_size)
            .color(text_color),
        );
    }
}

impl Default for TimerView {
    fn default() -> Self {
        Self::new()
    }
}
