use egui::{vec2, Align, Layout, Rect, Ui};

use super::super::components::{draw_icon, Card, CircularProgress, Icon};
use super::super::theme::Theme;
use super::{StatsAction, StatsView, stat_row};
use crate::core::Session;
use crate::data::Statistics;

impl StatsView {
    pub(crate) fn show_compact_timer_card(&self, ui: &mut Ui, session: &Session, theme: &Theme, pulse: f32) {
        let (start_color, end_color) = theme.session_gradient(session.session_type());
        let badge_color = Theme::lerp_color(start_color, end_color, 0.5);

        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(ui.available_width());

            ui.horizontal(|ui| {
                // Compact circular progress
                let radius = 28.0;
                CircularProgress::new(session.timer().progress())
                    .with_radius(radius)
                    .with_thickness(4.0)
                    .with_colors(start_color, end_color)
                    .with_bg_color(theme.bg_tertiary)
                    .with_pulse(if session.timer().is_running() && !theme.reduced_motion {
                        pulse
                    } else {
                        0.0
                    })
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                egui::RichText::new(session.timer().remaining_formatted())
                                    .size(12.0)
                                    .strong()
                                    .color(theme.text_primary),
                            );
                        });
                    });

                ui.add_space(12.0);

                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(session.session_type().label())
                            .size(14.0)
                            .strong()
                            .color(badge_color),
                    );

                    let status = if session.timer().is_running() {
                        "Running"
                    } else if session.timer().is_completed() {
                        "Completed"
                    } else {
                        "Paused"
                    };
                    ui.label(
                        egui::RichText::new(status)
                            .size(12.0)
                            .color(theme.text_muted),
                    );
                });
            });
        });
    }

    pub(crate) fn show_compact_stats_card(
        &self,
        ui: &mut Ui,
        stats: &Statistics,
        theme: &Theme,
        daily_goal: u32,
    ) {
        let goal_reached = stats.is_daily_goal_reached(daily_goal);

        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(ui.available_width());

            // Daily goal row
            let goal_value = if goal_reached {
                format!("{}/{} Done!", stats.today_pomodoros, daily_goal)
            } else {
                format!("{}/{}", stats.today_pomodoros, daily_goal)
            };
            stat_row(ui, theme, Icon::Target, "Daily Goal", &goal_value);

            ui.add_space(theme.spacing_xs);

            // Today row
            stat_row(
                ui,
                theme,
                Icon::Calendar,
                "Today",
                &format!("{:.1}h", stats.today_hours()),
            );

            ui.add_space(theme.spacing_xs);

            // Week row
            stat_row(
                ui,
                theme,
                Icon::BarChart3,
                "This Week",
                &format!("{:.1}h", stats.week_hours()),
            );

            ui.add_space(theme.spacing_xs);

            // Streak row
            stat_row(
                ui,
                theme,
                Icon::Flame,
                "Current Streak",
                &format!("{} days", stats.current_streak),
            );

            ui.add_space(theme.spacing_xs);

            // Total row
            stat_row(
                ui,
                theme,
                Icon::Timer,
                "Total",
                &format!(
                    "{}h ({} sessions)",
                    stats.total_hours(),
                    stats.total_pomodoros
                ),
            );
        });
    }

    pub(crate) fn show_compact_week_card(
        &self,
        ui: &mut Ui,
        stats: &Statistics,
        theme: &Theme,
        action: &mut Option<StatsAction>,
    ) {
        Card::new().show(ui, theme, |ui| {
            let available = ui.available_width();
            ui.set_min_width(available);

            ui.horizontal(|ui| {
                // Previous week
                let prev_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("<").size(12.0).color(theme.text_secondary),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(vec2(20.0, 20.0)),
                );
                if prev_btn.clicked() {
                    *action = Some(StatsAction::ChangeWeek {
                        offset: self.week_offset - 1,
                    });
                }

                ui.label(
                    egui::RichText::new(self.week_label())
                        .size(11.0)
                        .color(theme.text_secondary),
                );

                if self.week_offset < 0 {
                    let next_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(">").size(12.0).color(theme.text_secondary),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(vec2(20.0, 20.0)),
                    );
                    if next_btn.clicked() {
                        *action = Some(StatsAction::ChangeWeek {
                            offset: self.week_offset + 1,
                        });
                    }
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{:.1}h",
                            self.displayed_week_total(stats)
                        ))
                        .size(11.0)
                        .color(theme.text_muted),
                    );
                });
            });

            ui.add_space(8.0);
            self.draw_week_chart(ui, stats, theme, available);
        });
    }

    pub(crate) fn show_compact_presets_card(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        action: &mut Option<StatsAction>,
    ) {
        use crate::core::SessionType;

        Card::new().show(ui, theme, |ui| {
            ui.set_min_width(ui.available_width());

            for (icon, label, mins, session_type) in [
                (Icon::Coffee, "5 min break", 5, SessionType::ShortBreak),
                (Icon::Target, "25 min focus", 25, SessionType::Work),
                (Icon::Timer, "50 min deep work", 50, SessionType::Work),
            ] {
                let btn_response =
                    ui.allocate_response(vec2(ui.available_width(), 32.0), egui::Sense::click());
                let btn_rect = btn_response.rect;

                let bg_color = if btn_response.hovered() {
                    theme.bg_hover
                } else {
                    theme.bg_tertiary
                };
                ui.painter().rect_filled(btn_rect, 6.0, bg_color);

                // Icon
                let icon_rect = Rect::from_center_size(
                    egui::pos2(btn_rect.left() + 20.0, btn_rect.center().y),
                    vec2(14.0, 14.0),
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
                    egui::pos2(btn_rect.left() + 40.0, btn_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(12.0),
                    text_color,
                );

                if btn_response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if btn_response.clicked() {
                    *action = Some(StatsAction::QuickStart {
                        session_type,
                        minutes: mins,
                    });
                }

                ui.add_space(4.0);
            }
        });
    }

    pub(crate) fn show_mini_timer_card(
        &self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        width: f32,
        pulse: f32,
    ) {
        let (start_color, end_color) = theme.session_gradient(session.session_type());
        let radius = (width * 0.2).clamp(30.0, 50.0);

        Card::new().show(ui, theme, |ui| {
            ui.set_width(width - 32.0);
            ui.vertical_centered(|ui| {
                CircularProgress::new(session.timer().progress())
                    .with_radius(radius)
                    .with_thickness((radius * 0.12).clamp(3.0, 5.0))
                    .with_colors(start_color, end_color)
                    .with_bg_color(theme.bg_tertiary)
                    .with_pulse(if session.timer().is_running() && !theme.reduced_motion {
                        pulse
                    } else {
                        0.0
                    })
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            let font_size = (radius * 0.45).clamp(14.0, 20.0);
                            ui.label(
                                egui::RichText::new(session.timer().remaining_formatted())
                                    .size(font_size)
                                    .strong()
                                    .color(theme.text_primary),
                            );
                        });
                    });

                ui.add_space(4.0);

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

    pub(crate) fn show_quick_presets_card(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        width: f32,
        action: &mut Option<StatsAction>,
    ) {
        use crate::core::SessionType;

        let inner_width = width - 32.0;

        Card::new().show(ui, theme, |ui| {
            ui.set_width(inner_width);

            ui.horizontal(|ui| {
                let icon_rect =
                    Rect::from_center_size(ui.cursor().min + vec2(8.0, 8.0), vec2(14.0, 14.0));
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

            for (icon, label, mins, session_type) in [
                (Icon::Coffee, "5 min break", 5, SessionType::ShortBreak),
                (Icon::Target, "25 min focus", 25, SessionType::Work),
                (Icon::Timer, "50 min deep work", 50, SessionType::Work),
            ] {
                let btn_width = width - 40.0;
                let btn_response =
                    ui.allocate_response(vec2(btn_width, 36.0), egui::Sense::click());
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

                if btn_response.clicked() {
                    *action = Some(StatsAction::QuickStart {
                        session_type,
                        minutes: mins,
                    });
                }
            }
        });
    }

    pub(crate) fn show_focus_card(
        &self,
        ui: &mut Ui,
        stats: &Statistics,
        theme: &Theme,
        width: f32,
        daily_goal: u32,
    ) {
        let (accent_start, accent_end) = theme.accent_gradient();
        let inner_width = width - 32.0;
        let goal_progress = stats.daily_goal_progress(daily_goal);
        let goal_reached = stats.is_daily_goal_reached(daily_goal);

        Card::new().show(ui, theme, |ui| {
            ui.set_width(inner_width);

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

            // Daily goal progress
            ui.horizontal(|ui| {
                let goal_text = if goal_reached {
                    format!("{}/{} Goal reached!", stats.today_pomodoros, daily_goal)
                } else {
                    format!("{}/{} pomodoros", stats.today_pomodoros, daily_goal)
                };
                let text_color = if goal_reached {
                    theme.success
                } else {
                    theme.text_muted
                };
                ui.label(egui::RichText::new(goal_text).size(11.0).color(text_color));
            });

            ui.add_space(8.0);

            // Progress bar
            let bar_width = inner_width - 8.0;
            let bar_height = 6.0;
            let (rect, _) =
                ui.allocate_exact_size(vec2(bar_width, bar_height), egui::Sense::hover());

            // Background
            ui.painter().rect_filled(rect, 3.0, theme.bg_tertiary);

            // Fill
            let fill_width = (goal_progress.min(1.0) * bar_width).max(0.0);
            if fill_width > 0.0 {
                let fill_rect = Rect::from_min_size(rect.min, vec2(fill_width, bar_height));
                let fill_color = if goal_reached {
                    theme.success
                } else {
                    Theme::lerp_color(accent_start, accent_end, 0.5)
                };
                ui.painter().rect_filled(fill_rect, 3.0, fill_color);
            }
        });
    }

    pub(crate) fn show_stats_grid_wide(
        &self,
        ui: &mut Ui,
        stats: &Statistics,
        theme: &Theme,
        width: f32,
        spacing: f32,
    ) {
        let card_width = ((width - spacing) / 2.0).floor();
        let card_height = 90.0;

        // Row 1 - use top alignment to prevent vertical offset
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            self.stat_card_large(
                ui,
                theme,
                "Today",
                &format!("{:.1}h", stats.today_hours()),
                Some("focus time"),
                Icon::Calendar,
                card_width,
                card_height,
            );
            self.stat_card_large(
                ui,
                theme,
                "This Week",
                &format!("{:.1}h", stats.week_hours()),
                Some("total"),
                Icon::BarChart3,
                card_width,
                card_height,
            );
        });

        ui.add_space(spacing);

        // Row 2 - use top alignment to prevent vertical offset
        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            self.stat_card_large(
                ui,
                theme,
                "Current Streak",
                &format!("{} days", stats.current_streak),
                Some(&format!("Best: {}", stats.longest_streak)),
                Icon::Flame,
                card_width,
                card_height,
            );
            self.stat_card_large(
                ui,
                theme,
                "All Time",
                &format!("{}h", stats.total_hours()),
                Some(&format!("{} sessions", stats.total_pomodoros)),
                Icon::Timer,
                card_width,
                card_height,
            );
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn stat_card_large(
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
        Card::new()
            .with_size(vec2(width, height))
            .show(ui, theme, |ui| {
                ui.horizontal(|ui| {
                    // Icon on left
                    let icon_size = 24.0;
                    let (icon_rect, _) =
                        ui.allocate_exact_size(vec2(icon_size, icon_size), egui::Sense::hover());
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
                            ui.label(egui::RichText::new(sub).size(10.0).color(theme.text_muted));
                        }
                    });
                });
            });
    }
}
