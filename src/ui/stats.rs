//! Stats view with statistics - Responsive layout

use egui::{vec2, Align, Layout, Rect, ScrollArea, Ui};

use super::components::{draw_icon, Card, CircularProgress, Icon, IconButton};
use super::theme::Theme;
use crate::core::Session;
use crate::data::{ExportFormat, Statistics};

/// Actions from stats view
#[derive(Debug, Clone, PartialEq)]
pub enum StatsAction {
    Back,
    OpenSettings,
    /// Quick start a session with given type and duration in minutes
    QuickStart {
        session_type: crate::core::SessionType,
        minutes: u32,
    },
    /// Export statistics to file
    Export {
        format: ExportFormat,
    },
    /// Undo the last completed session
    UndoLastSession,
    /// Reset all statistics
    ResetStats,
}

/// Stats view showing statistics
pub struct StatsView {
    /// Whether the export dropdown is open
    export_dropdown_open: bool,
    /// Whether the reset confirmation dialog is open
    show_reset_confirmation: bool,
}

impl StatsView {
    pub fn new() -> Self {
        Self {
            export_dropdown_open: false,
            show_reset_confirmation: false,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        stats: &Statistics,
        theme: &Theme,
        pulse: f32,
        daily_goal: u32,
    ) -> Option<StatsAction> {
        let mut action = None;
        let available = ui.available_size();

        // Responsive breakpoints
        let is_wide = available.x > 550.0;
        let is_very_wide = available.x > 750.0;

        // Responsive sizing based on available space
        let spacing = if is_wide { 16.0 } else { 12.0 };

        ui.vertical(|ui| {
            // Header with back and settings buttons - matches settings style
            ui.horizontal(|ui| {
                if IconButton::new(Icon::ArrowLeft)
                    .with_size(32.0)
                    .with_icon_scale(0.5)
                    .show(ui, theme)
                    .clicked()
                {
                    action = Some(StatsAction::Back);
                }

                ui.add_space(12.0);

                ui.label(
                    egui::RichText::new("Stats")
                        .font(theme.font_h2())
                        .color(theme.text_primary),
                );

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if IconButton::new(Icon::Settings)
                        .with_size(32.0)
                        .with_icon_scale(0.5)
                        .show(ui, theme)
                        .clicked()
                    {
                        action = Some(StatsAction::OpenSettings);
                    }

                    ui.add_space(8.0);

                    // Export button with dropdown
                    ui.scope(|ui| {
                        self.show_export_button(ui, theme, &mut action);
                    });

                    ui.add_space(8.0);

                    // Reset all stats button
                    if stats.total_pomodoros > 0 {
                        let reset_response = IconButton::new(Icon::Trash)
                            .with_size(32.0)
                            .with_icon_scale(0.5)
                            .show(ui, theme);

                        if reset_response.clicked() {
                            self.show_reset_confirmation = true;
                        }

                        reset_response.on_hover_text("Reset all statistics");
                    }

                    ui.add_space(8.0);

                    // Undo last session button
                    if stats.today_pomodoros > 0 {
                        let undo_response = IconButton::new(Icon::RotateCcw)
                            .with_size(32.0)
                            .with_icon_scale(0.5)
                            .show(ui, theme);

                        if undo_response.clicked() {
                            action = Some(StatsAction::UndoLastSession);
                        }

                        undo_response.on_hover_text("Undo last session");
                    }
                });
            });

            ui.add_space(theme.spacing_lg);

            // Main content area with scroll
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if is_wide {
                        self.show_wide_layout(
                            ui,
                            session,
                            stats,
                            theme,
                            pulse,
                            spacing,
                            is_very_wide,
                            daily_goal,
                            &mut action,
                        );
                    } else {
                        self.show_narrow_layout(
                            ui,
                            session,
                            stats,
                            theme,
                            pulse,
                            spacing,
                            daily_goal,
                            &mut action,
                        );
                    }
                });
        });

        // Reset confirmation dialog
        if self.show_reset_confirmation {
            self.show_reset_confirmation_dialog(ui, theme, &mut action);
        }

        action
    }

    /// Show the reset confirmation dialog
    fn show_reset_confirmation_dialog(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        action: &mut Option<StatsAction>,
    ) {
        let screen_rect = ui.ctx().screen_rect();

        // Dark overlay
        ui.painter().rect_filled(
            screen_rect,
            0.0,
            egui::Color32::from_black_alpha(180),
        );

        // Dialog window
        egui::Area::new(egui::Id::new("reset_confirmation_dialog"))
            .fixed_pos(screen_rect.center() - vec2(140.0, 60.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(theme.bg_secondary)
                    .stroke(egui::Stroke::new(1.0, theme.bg_tertiary))
                    .rounding(12.0)
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.set_min_width(280.0);

                        ui.vertical_centered(|ui| {
                            // Warning icon
                            let icon_rect = Rect::from_center_size(
                                ui.cursor().min + vec2(140.0, 16.0),
                                vec2(32.0, 32.0),
                            );
                            draw_icon(ui, Icon::Trash, icon_rect, theme.error);
                            ui.add_space(40.0);

                            ui.label(
                                egui::RichText::new("Reset Statistics?")
                                    .size(16.0)
                                    .strong()
                                    .color(theme.text_primary),
                            );

                            ui.add_space(8.0);

                            ui.label(
                                egui::RichText::new("This will permanently delete all\nsession history and statistics.")
                                    .size(13.0)
                                    .color(theme.text_secondary),
                            );

                            ui.add_space(16.0);

                            ui.horizontal(|ui| {
                                // Cancel button
                                let cancel_btn = ui.add_sized(
                                    vec2(100.0, 36.0),
                                    egui::Button::new(
                                        egui::RichText::new("Cancel")
                                            .size(13.0)
                                            .color(theme.text_primary),
                                    )
                                    .fill(theme.bg_tertiary)
                                    .rounding(8.0),
                                );

                                if cancel_btn.clicked() {
                                    self.show_reset_confirmation = false;
                                }

                                ui.add_space(12.0);

                                // Confirm button (red/danger)
                                let confirm_btn = ui.add_sized(
                                    vec2(100.0, 36.0),
                                    egui::Button::new(
                                        egui::RichText::new("Reset")
                                            .size(13.0)
                                            .color(egui::Color32::WHITE),
                                    )
                                    .fill(theme.error)
                                    .rounding(8.0),
                                );

                                if confirm_btn.clicked() {
                                    *action = Some(StatsAction::ResetStats);
                                    self.show_reset_confirmation = false;
                                }
                            });
                        });
                    });
            });

        // Close on Escape key
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_reset_confirmation = false;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn show_wide_layout(
        &self,
        ui: &mut Ui,
        session: &Session,
        stats: &Statistics,
        theme: &Theme,
        pulse: f32,
        spacing: f32,
        is_very_wide: bool,
        daily_goal: u32,
        action: &mut Option<StatsAction>,
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
                    self.show_quick_presets_card(ui, theme, left_col_width, action);

                    ui.add_space(spacing);

                    // Today's focus time
                    self.show_focus_card(ui, stats, theme, left_col_width, daily_goal);
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

    #[allow(clippy::too_many_arguments)]
    fn show_narrow_layout(
        &self,
        ui: &mut Ui,
        session: &Session,
        stats: &Statistics,
        theme: &Theme,
        pulse: f32,
        spacing: f32,
        daily_goal: u32,
        action: &mut Option<StatsAction>,
    ) {
        // Current Session section
        section_header(ui, theme, "Current Session");
        self.show_compact_timer_card(ui, session, theme, pulse);

        ui.add_space(spacing);

        // Statistics section
        section_header(ui, theme, "Statistics");
        self.show_compact_stats_card(ui, stats, theme, daily_goal);

        ui.add_space(spacing);

        // Week Activity section
        section_header(ui, theme, "Week Activity");
        self.show_compact_week_card(ui, stats, theme);

        ui.add_space(spacing);

        // Quick Start section
        section_header(ui, theme, "Quick Start");
        self.show_compact_presets_card(ui, theme, action);
    }

    fn show_compact_timer_card(&self, ui: &mut Ui, session: &Session, theme: &Theme, pulse: f32) {
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

    fn show_compact_stats_card(
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

    fn show_compact_week_card(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme) {
        Card::new().show(ui, theme, |ui| {
            let available = ui.available_width();
            ui.set_min_width(available);

            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("{:.1}h total", stats.week_hours()))
                        .size(11.0)
                        .color(theme.text_secondary),
                );
            });

            ui.add_space(8.0);
            self.draw_week_chart(ui, stats, theme, available);
        });
    }

    fn show_compact_presets_card(
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

    fn show_mini_timer_card(
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

    fn show_quick_presets_card(
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

    fn show_focus_card(
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

    fn show_stats_grid_wide(
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

    fn show_week_activity_card(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32) {
        let inner_width = width - 32.0; // Account for Card padding (16 * 2)

        Card::new().show(ui, theme, |ui| {
            ui.set_width(inner_width);

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
            self.draw_week_chart(ui, stats, theme, inner_width - 16.0);
        });
    }

    fn show_additional_stats(
        &self,
        ui: &mut Ui,
        stats: &Statistics,
        theme: &Theme,
        width: f32,
        spacing: f32,
    ) {
        let card_width = ((width - spacing) / 2.0).floor();

        ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.x = spacing;
            // Best streak card
            Card::new()
                .with_size(vec2(card_width, 70.0))
                .show(ui, theme, |ui| {
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
            Card::new()
                .with_size(vec2(card_width, 70.0))
                .show(ui, theme, |ui| {
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

        let (rect, _) =
            ui.allocate_exact_size(vec2(total_width, chart_height + 20.0), egui::Sense::hover());
        let (accent_start, accent_end) = theme.accent_gradient();

        for (i, (day, &value)) in days.iter().zip(values.iter()).enumerate() {
            let x = rect.left() + i as f32 * (bar_width + gap);
            let bar_height = (value / max_value) * chart_height;

            // Bar background
            let bg_rect =
                Rect::from_min_size(egui::pos2(x, rect.top()), vec2(bar_width, chart_height));
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
                    egui::pos2(
                        x + bar_width / 2.0,
                        rect.top() + chart_height - bar_height - 4.0,
                    ),
                    egui::Align2::CENTER_BOTTOM,
                    format!("{:.1}", value),
                    egui::FontId::proportional(9.0),
                    theme.text_muted,
                );
            }
        }
    }

    /// Show the export button with dropdown menu
    fn show_export_button(&mut self, ui: &mut Ui, theme: &Theme, action: &mut Option<StatsAction>) {
        let button_id = ui.make_persistent_id("export_dropdown");

        // Export button
        let button_response = IconButton::new(Icon::Download)
            .with_size(32.0)
            .with_icon_scale(0.5)
            .show(ui, theme);

        // Save rect before potentially consuming response
        let button_rect = button_response.rect;
        let was_clicked = button_response.clicked();
        let is_hovered = button_response.hovered();

        if was_clicked {
            self.export_dropdown_open = !self.export_dropdown_open;
        }

        // Show tooltip
        if is_hovered && !self.export_dropdown_open {
            button_response.on_hover_text("Export statistics");
        }

        // Dropdown menu
        if self.export_dropdown_open {
            let dropdown_pos = button_rect.left_bottom() + vec2(-60.0, 4.0);

            egui::Area::new(button_id)
                .fixed_pos(dropdown_pos)
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style())
                        .fill(theme.bg_secondary)
                        .stroke(egui::Stroke::new(1.0, theme.bg_tertiary))
                        .rounding(8.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_min_width(120.0);

                            ui.label(
                                egui::RichText::new("Export as")
                                    .size(11.0)
                                    .color(theme.text_muted),
                            );

                            ui.add_space(4.0);

                            // CSV option
                            let csv_response = ui.allocate_response(
                                vec2(ui.available_width(), 32.0),
                                egui::Sense::click(),
                            );
                            let csv_rect = csv_response.rect;

                            let bg_color = if csv_response.hovered() {
                                theme.bg_hover
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(csv_rect, 6.0, bg_color);

                            ui.painter().text(
                                csv_rect.left_center() + vec2(12.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                "CSV (.csv)",
                                egui::FontId::proportional(13.0),
                                if csv_response.hovered() {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                },
                            );

                            if csv_response.clicked() {
                                *action = Some(StatsAction::Export {
                                    format: ExportFormat::Csv,
                                });
                                self.export_dropdown_open = false;
                            }

                            // JSON option
                            let json_response = ui.allocate_response(
                                vec2(ui.available_width(), 32.0),
                                egui::Sense::click(),
                            );
                            let json_rect = json_response.rect;

                            let bg_color = if json_response.hovered() {
                                theme.bg_hover
                            } else {
                                egui::Color32::TRANSPARENT
                            };
                            ui.painter().rect_filled(json_rect, 6.0, bg_color);

                            ui.painter().text(
                                json_rect.left_center() + vec2(12.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                "JSON (.json)",
                                egui::FontId::proportional(13.0),
                                if json_response.hovered() {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                },
                            );

                            if json_response.clicked() {
                                *action = Some(StatsAction::Export {
                                    format: ExportFormat::Json,
                                });
                                self.export_dropdown_open = false;
                            }
                        });
                });

            // Close dropdown when clicking outside
            if ui.input(|i| i.pointer.any_click()) && !is_hovered {
                // Check if click is outside the dropdown area
                let click_pos = ui.input(|i| i.pointer.interact_pos());
                if let Some(pos) = click_pos {
                    let dropdown_rect = egui::Rect::from_min_size(dropdown_pos, vec2(136.0, 100.0));
                    if !dropdown_rect.contains(pos) && !button_rect.contains(pos) {
                        self.export_dropdown_open = false;
                    }
                }
            }
        }
    }
}

impl Default for StatsView {
    fn default() -> Self {
        Self::new()
    }
}

/// Section header - matches settings style
fn section_header(ui: &mut Ui, theme: &Theme, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(theme.font_body())
            .color(theme.text_primary),
    );
    ui.add_space(theme.spacing_xs);
}

/// Statistics row with icon, label and value
fn stat_row(ui: &mut Ui, theme: &Theme, icon: Icon, label: &str, value: &str) {
    ui.horizontal(|ui| {
        let icon_size = 16.0;
        let (icon_rect, _) =
            ui.allocate_exact_size(vec2(icon_size, icon_size), egui::Sense::hover());
        draw_icon(ui, icon, icon_rect, theme.text_secondary);

        ui.add_space(8.0);

        ui.label(
            egui::RichText::new(label)
                .size(13.0)
                .color(theme.text_secondary),
        );

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .size(13.0)
                    .strong()
                    .color(theme.text_primary),
            );
        });
    });
}
