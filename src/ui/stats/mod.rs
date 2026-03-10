//! Stats view with statistics - Responsive layout

mod cards;
mod charts;

use egui::{vec2, Align, Layout, Rect, ScrollArea, Ui};

use super::components::{draw_icon, Icon, IconButton};
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
    /// Change the displayed week (offset from current week)
    ChangeWeek {
        offset: i32,
    },
}

/// Stats view showing statistics
pub struct StatsView {
    /// Whether the export dropdown is open
    export_dropdown_open: bool,
    /// Whether the reset confirmation dialog is open
    show_reset_confirmation: bool,
    /// Week offset for chart navigation (0 = current week, -1 = previous, etc.)
    pub week_offset: i32,
    /// Cached weekly hours for the selected week
    pub selected_week_hours: Option<Vec<f32>>,
}

impl StatsView {
    pub fn new() -> Self {
        Self {
            export_dropdown_open: false,
            show_reset_confirmation: false,
            week_offset: 0,
            selected_week_hours: None,
        }
    }

    /// Get the week label for the current offset
    fn week_label(&self) -> String {
        use chrono::{Datelike, Local};
        let today = Local::now().date_naive();
        let reference = today + chrono::Duration::weeks(self.week_offset as i64);
        let start = reference
            - chrono::Duration::days(reference.weekday().num_days_from_monday() as i64);
        let end = start + chrono::Duration::days(6);
        if self.week_offset == 0 {
            "This Week".to_string()
        } else {
            format!("{} — {}", start.format("%d %b"), end.format("%d %b"))
        }
    }

    /// Get the hours data for the currently displayed week
    fn displayed_week_hours<'a>(&'a self, stats: &'a Statistics) -> &'a [f32] {
        if self.week_offset == 0 {
            &stats.week_daily_hours
        } else {
            self.selected_week_hours.as_deref().unwrap_or(&stats.week_daily_hours)
        }
    }

    /// Total hours for the displayed week
    fn displayed_week_total(&self, stats: &Statistics) -> f32 {
        let hours = self.displayed_week_hours(stats);
        (hours.iter().sum::<f32>() * 10.0).round() / 10.0
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

        // Max-width container - centered with limited width like web pages
        let max_content_width = 800.0;
        let available_width = ui.available_width();
        let content_width = available_width.min(max_content_width);
        let horizontal_margin = ((available_width - content_width) / 2.0).max(0.0);

        // Calculate responsive breakpoints based on content width
        let is_wide = content_width > 550.0;
        let is_very_wide = content_width > 750.0;

        // Responsive sizing based on available space
        let spacing = if is_wide { 16.0 } else { 12.0 };

        ui.vertical(|ui| {
            // Apply horizontal margins for centering
            let margin = egui::Margin {
                left: horizontal_margin,
                right: horizontal_margin,
                top: 0.0,
                bottom: 0.0,
            };
            egui::Frame::none().inner_margin(margin).show(ui, |ui| {
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
                        egui::RichText::new("Статистика")
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
            }); // Frame
        }); // vertical

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
        ui.painter()
            .rect_filled(screen_rect, 0.0, egui::Color32::from_black_alpha(180));

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
                    self.show_week_activity_card(ui, stats, theme, right_col_width, action);

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
        self.show_compact_week_card(ui, stats, theme, action);

        ui.add_space(spacing);

        // Quick Start section
        section_header(ui, theme, "Quick Start");
        self.show_compact_presets_card(ui, theme, action);
    }
}

impl Default for StatsView {
    fn default() -> Self {
        Self::new()
    }
}

/// Section header - matches settings style
pub(super) fn section_header(ui: &mut Ui, theme: &Theme, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(theme.font_body())
            .color(theme.text_primary),
    );
    ui.add_space(theme.spacing_xs);
}

/// Statistics row with icon, label and value
pub(super) fn stat_row(ui: &mut Ui, theme: &Theme, icon: Icon, label: &str, value: &str) {
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
