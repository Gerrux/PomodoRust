//! Dashboard view with statistics - Responsive layout

use egui::{vec2, Align, Layout, Rect, Ui, ScrollArea};

use super::components::{draw_icon, Card, CircularProgress, Icon, IconButton};
use super::theme::Theme;
use crate::core::Session;
use crate::data::Statistics;

/// Actions from dashboard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardAction {
    Back,
    OpenSettings,
}

/// Dashboard view showing statistics
pub struct DashboardView;

impl DashboardView {
    pub fn new() -> Self {
        Self
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        stats: &Statistics,
        theme: &Theme,
        pulse: f32,
    ) -> Option<DashboardAction> {
        let mut action = None;
        let available = ui.available_size();

        // Responsive breakpoints
        let is_wide = available.x > 550.0;
        let is_very_wide = available.x > 750.0;

        // Responsive sizing based on available space
        let spacing = if is_wide { 16.0 } else { 12.0 };

        ui.vertical(|ui| {
            // Header with back and settings buttons
            ui.horizontal(|ui| {
                if IconButton::new(Icon::ArrowLeft)
                    .with_size(36.0)
                    .with_icon_scale(0.5)
                    .show(ui, theme)
                    .clicked()
                {
                    action = Some(DashboardAction::Back);
                }

                ui.add_space(spacing);

                ui.label(
                    egui::RichText::new("Dashboard")
                        .size(20.0)
                        .strong()
                        .color(theme.text_primary),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if IconButton::new(Icon::Settings)
                        .with_size(36.0)
                        .with_icon_scale(0.5)
                        .show(ui, theme)
                        .clicked()
                    {
                        action = Some(DashboardAction::OpenSettings);
                    }
                });
            });

            ui.add_space(spacing);

            // Main content area with scroll
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if is_wide {
                        self.show_wide_layout(ui, session, stats, theme, pulse, spacing, is_very_wide);
                    } else {
                        self.show_narrow_layout(ui, session, stats, theme, pulse, spacing);
                    }
                });
        });

        action
    }

    fn show_wide_layout(
        &self,
        ui: &mut Ui,
        session: &Session,
        stats: &Statistics,
        theme: &Theme,
        pulse: f32,
        spacing: f32,
        is_very_wide: bool,
    ) {
        let available_width = ui.available_width();

        // Calculate column widths - left column is narrower, right column gets more space
        let left_col_width = if is_very_wide {
            (available_width * 0.35).clamp(200.0, 280.0)
        } else {
            (available_width * 0.4).clamp(180.0, 240.0)
        };
        let right_col_width = available_width - left_col_width - spacing;

        ui.horizontal(|ui| {
            // Left Column - Timer & Quick Actions
            ui.allocate_ui(vec2(left_col_width, ui.available_height()), |ui| {
                ui.vertical(|ui| {
                    // Mini timer card
                    self.show_mini_timer_card(ui, session, theme, left_col_width, pulse);

                    ui.add_space(spacing);

                    // Quick presets
                    self.show_quick_presets_card(ui, theme, left_col_width);

                    ui.add_space(spacing);

                    // Today's focus time
                    self.show_focus_card(ui, stats, theme, left_col_width);
                });
            });

            ui.add_space(spacing);

            // Right Column - Statistics
            ui.allocate_ui(vec2(right_col_width, ui.available_height()), |ui| {
                ui.vertical(|ui| {
                    // Stats grid - 2x2
                    self.show_stats_grid_wide(ui, stats, theme, right_col_width, spacing);

                    ui.add_space(spacing);

                    // Week activity chart
                    self.show_week_activity_card(ui, stats, theme, right_col_width);

                    ui.add_space(spacing);

                    // Additional stats row
                    self.show_additional_stats(ui, stats, theme, right_col_width, spacing);
                });
            });
        });
    }

    fn show_narrow_layout(
        &self,
        ui: &mut Ui,
        session: &Session,
        stats: &Statistics,
        theme: &Theme,
        pulse: f32,
        spacing: f32,
    ) {
        let available_width = ui.available_width();
        let card_width = (available_width - spacing).max(200.0);

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            // Mini timer
            self.show_mini_timer_card(ui, session, theme, card_width.min(280.0), pulse);

            ui.add_space(spacing);

            // Stats grid - 2x2
            self.show_stats_grid_narrow(ui, stats, theme, card_width, spacing);

            ui.add_space(spacing);

            // Week chart
            self.show_week_activity_card(ui, stats, theme, card_width);

            ui.add_space(spacing);

            // Today's sessions
            self.show_today_sessions_card(ui, stats, theme, card_width);

            ui.add_space(spacing);

            // Quick presets
            self.show_quick_presets_card(ui, theme, card_width.min(280.0));
        });
    }

    fn show_mini_timer_card(
        &self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        width: f32,
        pulse: f32,
    ) {
        let (start_color, end_color) = theme.session_gradient(session.session_type());
        let radius = (width * 0.25).clamp(35.0, 55.0);

        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(width - 24.0);
            ui.vertical_centered(|ui| {
                CircularProgress::new(session.timer().progress())
                    .with_radius(radius)
                    .with_thickness((radius * 0.12).clamp(3.0, 6.0))
                    .with_colors(start_color, end_color)
                    .with_bg_color(theme.bg_tertiary)
                    .with_pulse(if session.timer().is_running() { pulse } else { 0.0 })
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let font_size = (radius * 0.45).clamp(14.0, 22.0);
                            ui.label(
                                egui::RichText::new(session.timer().remaining_formatted())
                                    .size(font_size)
                                    .strong()
                                    .color(theme.text_primary),
                            );
                        });
                    });

                ui.add_space(8.0);

                // Session type badge
                let badge_color = Theme::lerp_color(start_color, end_color, 0.5);
                ui.label(
                    egui::RichText::new(session.session_type().label())
                        .size(11.0)
                        .color(badge_color),
                );

                // Status
                let status = if session.timer().is_running() {
                    "Running"
                } else if session.timer().is_completed() {
                    "Completed"
                } else {
                    "Paused"
                };
                ui.label(
                    egui::RichText::new(status)
                        .size(10.0)
                        .color(theme.text_muted),
                );
            });
        });
    }

    fn show_quick_presets_card(&self, ui: &mut Ui, theme: &Theme, width: f32) {
        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(width - 24.0);

            ui.horizontal(|ui| {
                let icon_rect = Rect::from_center_size(
                    ui.cursor().min + vec2(8.0, 8.0),
                    vec2(14.0, 14.0),
                );
                draw_icon(ui, Icon::Zap, icon_rect, theme.text_secondary);
                ui.add_space(20.0);
                ui.label(
                    egui::RichText::new("Quick Start")
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                );
            });

            ui.add_space(8.0);

            for (icon, label, _mins) in [
                (Icon::Coffee, "5 min break", 5),
                (Icon::Target, "25 min focus", 25),
                (Icon::Timer, "50 min deep work", 50),
            ] {
                let btn_width = width - 40.0;
                let btn_response = ui.allocate_response(vec2(btn_width, 36.0), egui::Sense::click());
                let btn_rect = btn_response.rect;

                let bg_color = if btn_response.hovered() {
                    theme.bg_hover
                } else {
                    theme.bg_tertiary
                };
                ui.painter().rect_filled(btn_rect, 8.0, bg_color);

                // Icon
                let icon_rect = Rect::from_center_size(
                    egui::pos2(btn_rect.left() + 22.0, btn_rect.center().y),
                    vec2(16.0, 16.0),
                );
                let icon_color = if btn_response.hovered() {
                    theme.accent.solid()
                } else {
                    theme.text_secondary
                };
                draw_icon(ui, icon, icon_rect, icon_color);

                // Label
                let text_color = if btn_response.hovered() {
                    theme.text_primary
                } else {
                    theme.text_secondary
                };
                ui.painter().text(
                    egui::pos2(btn_rect.left() + 44.0, btn_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(13.0),
                    text_color,
                );

                if btn_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
            }
        });
    }

    fn show_focus_card(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32) {
        let (accent_start, accent_end) = theme.accent_gradient();

        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(width - 24.0);

            ui.label(
                egui::RichText::new("Today's Focus")
                    .size(12.0)
                    .color(theme.text_secondary),
            );

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{:.1}", stats.today_hours()))
                        .size(32.0)
                        .strong()
                        .color(Theme::lerp_color(accent_start, accent_end, 0.5)),
                );
                ui.label(
                    egui::RichText::new("hours")
                        .size(14.0)
                        .color(theme.text_muted),
                );
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{} pomodoros completed", stats.today_pomodoros))
                        .size(11.0)
                        .color(theme.text_muted),
                );
            });
        });
    }

    fn show_stats_grid_wide(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32, spacing: f32) {
        // Account for ui.horizontal's default item spacing
        let item_spacing = ui.spacing().item_spacing.x;
        let card_width = ((width - spacing - item_spacing) / 2.0).floor();
        let card_height = 90.0;

        // Row 1 - use top alignment to prevent vertical offset
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            self.stat_card_large(ui, theme, "Today", &format!("{:.1}h", stats.today_hours()),
                Some("focus time"), Icon::Calendar, card_width, card_height);
            self.stat_card_large(ui, theme, "This Week", &format!("{:.1}h", stats.week_hours()),
                Some("total"), Icon::BarChart3, card_width, card_height);
        });

        ui.add_space(spacing);

        // Row 2 - use top alignment to prevent vertical offset
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            self.stat_card_large(ui, theme, "Current Streak", &format!("{} days", stats.current_streak),
                Some(&format!("Best: {}", stats.longest_streak)), Icon::Flame, card_width, card_height);
            self.stat_card_large(ui, theme, "All Time", &format!("{}h", stats.total_hours()),
                Some(&format!("{} sessions", stats.total_pomodoros)), Icon::Timer, card_width, card_height);
        });
    }

    fn show_stats_grid_narrow(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32, spacing: f32) {
        let item_spacing = ui.spacing().item_spacing.x;
        let card_size = ((width - spacing - item_spacing) / 2.0).clamp(80.0, 120.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            stat_card_small(ui, theme, "Today", &format!("{:.1}h", stats.today_hours()), Some(Icon::Calendar), card_size);
            stat_card_small(ui, theme, "Week", &format!("{:.1}h", stats.week_hours()), Some(Icon::BarChart3), card_size);
        });

        ui.add_space(spacing);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            stat_card_small(ui, theme, "Streak", &format!("{}", stats.current_streak), Some(Icon::Flame), card_size);
            stat_card_small(ui, theme, "Total", &format!("{}h", stats.total_hours()), Some(Icon::Timer), card_size);
        });
    }

    fn stat_card_large(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        title: &str,
        value: &str,
        subtitle: Option<&str>,
        icon: Icon,
        width: f32,
        height: f32,
    ) {
        Card::new().with_size(vec2(width, height)).show(ui, theme, |ui| {
            ui.horizontal(|ui| {
                // Icon on left
                let icon_size = 24.0;
                let (icon_rect, _) = ui.allocate_exact_size(vec2(icon_size, icon_size), egui::Sense::hover());
                draw_icon(ui, icon, icon_rect, theme.text_muted);

                ui.add_space(12.0);

                // Text on right
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(title)
                            .size(11.0)
                            .color(theme.text_secondary),
                    );

                    ui.label(
                        egui::RichText::new(value)
                            .size(22.0)
                            .strong()
                            .color(theme.text_primary),
                    );

                    if let Some(sub) = subtitle {
                        ui.label(
                            egui::RichText::new(sub)
                                .size(10.0)
                                .color(theme.text_muted),
                        );
                    }
                });
            });
        });
    }

    fn show_week_activity_card(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32) {
        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(width - 24.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Week Activity")
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!("{:.1}h total", stats.week_hours()))
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                });
            });

            ui.add_space(12.0);
            self.draw_week_chart(ui, stats, theme, width - 48.0);
        });
    }

    fn show_today_sessions_card(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32) {
        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(width - 24.0);

            ui.label(
                egui::RichText::new("Today's Sessions")
                    .size(12.0)
                    .color(theme.text_secondary),
            );

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{}", stats.today_pomodoros))
                        .size(36.0)
                        .strong()
                        .color(theme.text_primary),
                );
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("pomodoros")
                            .size(13.0)
                            .color(theme.text_muted),
                    );
                    ui.label(
                        egui::RichText::new(format!("{:.1} hours", stats.today_hours()))
                            .size(11.0)
                            .color(theme.text_muted),
                    );
                });
            });
        });
    }

    fn show_additional_stats(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32, spacing: f32) {
        let item_spacing = ui.spacing().item_spacing.x;
        let card_width = ((width - spacing - item_spacing) / 2.0).floor();

        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            // Best streak card
            Card::new().with_size(vec2(card_width, 70.0)).show(ui, theme, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Best Streak")
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", stats.longest_streak))
                                .size(24.0)
                                .strong()
                                .color(theme.success),
                        );
                        ui.label(
                            egui::RichText::new("days")
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                    });
                });
            });

            // Total sessions card
            Card::new().with_size(vec2(card_width, 70.0)).show(ui, theme, |ui| {
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Total Sessions")
                            .size(11.0)
                            .color(theme.text_secondary),
                    );
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}", stats.total_pomodoros))
                                .size(24.0)
                                .strong()
                                .color(theme.accent.solid()),
                        );
                        ui.label(
                            egui::RichText::new("completed")
                                .size(12.0)
                                .color(theme.text_muted),
                        );
                    });
                });
            });
        });
    }

    fn draw_week_chart(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32) {
        let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
        let values = &stats.week_daily_hours;
        let max_value = values.iter().cloned().fold(1.0_f32, f32::max);

        let chart_height = 60.0;
        let bar_width = ((width - 12.0) / 7.0).clamp(16.0, 32.0);
        let gap = ((width - bar_width * 7.0) / 6.0).clamp(4.0, 12.0);
        let total_width = 7.0 * bar_width + 6.0 * gap;

        let (rect, _) = ui.allocate_exact_size(vec2(total_width, chart_height + 20.0), egui::Sense::hover());
        let (accent_start, accent_end) = theme.accent_gradient();

        for (i, (day, &value)) in days.iter().zip(values.iter()).enumerate() {
            let x = rect.left() + i as f32 * (bar_width + gap);
            let bar_height = (value / max_value) * chart_height;

            // Bar background
            let bg_rect = Rect::from_min_size(
                egui::pos2(x, rect.top()),
                vec2(bar_width, chart_height),
            );
            ui.painter().rect_filled(bg_rect, 4.0, theme.bg_tertiary);

            // Bar fill
            if bar_height > 0.0 {
                let fill_rect = Rect::from_min_size(
                    egui::pos2(x, rect.top() + chart_height - bar_height),
                    vec2(bar_width, bar_height),
                );
                let t = value / max_value;
                let color = Theme::lerp_color(accent_start, accent_end, t);
                ui.painter().rect_filled(fill_rect, 4.0, color);
            }

            // Day label
            ui.painter().text(
                egui::pos2(x + bar_width / 2.0, rect.bottom() - 4.0),
                egui::Align2::CENTER_BOTTOM,
                day,
                egui::FontId::proportional(10.0),
                theme.text_muted,
            );

            // Value on hover (optional: show hours)
            if value > 0.0 {
                ui.painter().text(
                    egui::pos2(x + bar_width / 2.0, rect.top() + chart_height - bar_height - 4.0),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{:.1}", value),
                    egui::FontId::proportional(9.0),
                    theme.text_muted,
                );
            }
        }
    }
}

impl Default for DashboardView {
    fn default() -> Self {
        Self::new()
    }
}

/// Small statistics card for narrow layout
fn stat_card_small(ui: &mut Ui, theme: &Theme, label: &str, value: &str, icon: Option<Icon>, size: f32) {
    Card::new().with_size(vec2(size, size)).show(ui, theme, |ui| {
        ui.vertical_centered(|ui| {
            if let Some(icon) = icon {
                let icon_size = (size * 0.22).clamp(14.0, 20.0);
                let (icon_rect, _) = ui.allocate_exact_size(vec2(icon_size, icon_size), egui::Sense::hover());
                draw_icon(ui, icon, icon_rect, theme.text_secondary);
                ui.add_space(2.0);
            }

            ui.label(
                egui::RichText::new(value)
                    .size((size * 0.25).clamp(16.0, 24.0))
                    .strong()
                    .color(theme.text_primary),
            );

            ui.label(
                egui::RichText::new(label)
                    .size((size * 0.14).clamp(9.0, 12.0))
                    .color(theme.text_muted),
            );
        });
    });
}
