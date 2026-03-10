use egui::{vec2, Align, Layout, Rect, Ui};

use super::super::components::{Card, Icon, IconButton};
use super::super::theme::Theme;
use super::{StatsAction, StatsView};
use crate::data::{ExportFormat, Statistics};

impl StatsView {
    pub(crate) fn show_week_activity_card(
        &self,
        ui: &mut Ui,
        stats: &Statistics,
        theme: &Theme,
        width: f32,
        action: &mut Option<StatsAction>,
    ) {
        let inner_width = width - 32.0; // Account for Card padding (16 * 2)

        Card::new().show(ui, theme, |ui| {
            ui.set_width(inner_width);

            ui.horizontal(|ui| {
                // Previous week button
                let prev_btn = ui.add(
                    egui::Button::new(
                        egui::RichText::new("<").size(14.0).color(theme.text_secondary),
                    )
                    .fill(egui::Color32::TRANSPARENT)
                    .min_size(vec2(24.0, 24.0)),
                );
                if prev_btn.clicked() {
                    *action = Some(StatsAction::ChangeWeek {
                        offset: self.week_offset - 1,
                    });
                }

                ui.label(
                    egui::RichText::new(self.week_label())
                        .size(13.0)
                        .strong()
                        .color(theme.text_primary),
                );

                // Next week button (only if not already on current week)
                if self.week_offset < 0 {
                    let next_btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(">").size(14.0).color(theme.text_secondary),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .min_size(vec2(24.0, 24.0)),
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
                            "{:.1}h {}",
                            self.displayed_week_total(stats),
                            crate::i18n::tr().stats.total_label
                        ))
                        .size(11.0)
                        .color(theme.text_secondary),
                    );
                });
            });

            ui.add_space(12.0);
            self.draw_week_chart(ui, stats, theme, inner_width - 16.0);
        });
    }

    pub(crate) fn show_additional_stats(
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
                            egui::RichText::new(crate::i18n::tr().stats.best_streak)
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
                                egui::RichText::new(crate::i18n::tr().stats.days)
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
                            egui::RichText::new(crate::i18n::tr().stats.total_sessions)
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
                                egui::RichText::new(crate::i18n::tr().stats.completed_label)
                                    .size(12.0)
                                    .color(theme.text_muted),
                            );
                        });
                    });
                });
        });
    }

    pub(crate) fn draw_week_chart(&self, ui: &mut Ui, stats: &Statistics, theme: &Theme, width: f32) {
        let days = crate::i18n::tr().days_of_week();
        let values = self.displayed_week_hours(stats);
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
    pub(crate) fn show_export_button(&mut self, ui: &mut Ui, theme: &Theme, action: &mut Option<StatsAction>) {
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
            button_response.on_hover_text(crate::i18n::tr().stats.export_hover);
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
                                egui::RichText::new(crate::i18n::tr().stats.export_as)
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

            // Close dropdown on Escape
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.export_dropdown_open = false;
            }

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
