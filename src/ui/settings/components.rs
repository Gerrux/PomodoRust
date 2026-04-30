use egui::{vec2, Layout, Rect, Ui};

use super::super::components::{draw_icon, Icon};
use super::super::theme::{AccentColor, Theme};

/// Draw section header
pub(super) fn section_header(ui: &mut Ui, theme: &Theme, title: &str) {
    ui.label(
        egui::RichText::new(title)
            .font(theme.font_body())
            .color(theme.text_primary),
    );
    ui.add_space(theme.spacing_xs);
}

/// Draw a duration row with +/- buttons and unit label
pub(super) fn duration_row(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    value: &mut f32,
    min: f32,
    max: f32,
) {
    duration_row_with_unit(ui, theme, label, value, min, max, "min");
}

/// Draw a duration row with +/- buttons, custom unit
pub(super) fn duration_row_with_unit(
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
pub(super) fn color_picker_row(
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
                // Show light-mode colors when in light theme
                let (color, _) = if theme.is_light {
                    accent.gradient_light()
                } else {
                    accent.gradient()
                };

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
                        egui::Stroke::new(2.0, theme.text_primary),
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
pub(super) fn toggle_row(ui: &mut Ui, theme: &Theme, label: &str, value: &mut bool) {
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
pub(super) fn hotkey_row(ui: &mut Ui, theme: &Theme, label: &str, hotkey: &str) {
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
