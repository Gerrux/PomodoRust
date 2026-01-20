//! Custom button components with gradients and animations

use egui::{vec2, Color32, CursorIcon, Rect, Response, Rounding, Sense, Stroke, Ui, Vec2};

use super::icons::{draw_icon, Icon};
use crate::ui::animations::InteractionState;
use crate::ui::theme::Theme;

/// A button with gradient background
pub struct GradientButton {
    text: String,
    size: Vec2,
    gradient: Option<(Color32, Color32)>,
    state: InteractionState,
    id_source: egui::Id,
}

impl GradientButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            size: vec2(120.0, 40.0),
            gradient: None,
            state: InteractionState::new(),
            id_source: egui::Id::new("gradient_button"),
        }
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn with_gradient(mut self, start: Color32, end: Color32) -> Self {
        self.gradient = Some((start, end));
        self
    }

    pub fn with_id(mut self, id: impl std::hash::Hash) -> Self {
        self.id_source = egui::Id::new(id);
        self
    }

    pub fn show(mut self, ui: &mut Ui, theme: &Theme) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::click());

        self.state
            .update(response.hovered(), response.is_pointer_button_down_on());

        let hover_t = self.state.hover_t();
        let press_t = self.state.press_t();

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        // Scale effect on press
        let scale = 1.0 - press_t * 0.02;
        let scaled_size = self.size * scale;
        let scaled_rect = Rect::from_center_size(rect.center(), scaled_size);

        // Draw gradient background
        let (start_color, end_color) = self.gradient.unwrap_or_else(|| theme.accent_gradient());

        // Brighten on hover
        let start_color = brighten_color(start_color, hover_t * 0.15);
        let end_color = brighten_color(end_color, hover_t * 0.15);

        draw_gradient_rect(
            ui,
            scaled_rect,
            start_color,
            end_color,
            theme.button_rounding(),
        );

        // Glow effect on hover
        if hover_t > 0.0 {
            let glow_color = Theme::with_alpha(start_color, (hover_t * 60.0) as u8);
            let glow_rect = scaled_rect.expand(4.0 * hover_t);
            ui.painter()
                .rect_filled(glow_rect, theme.rounding_lg, glow_color);

            // Redraw the button on top of glow
            draw_gradient_rect(
                ui,
                scaled_rect,
                start_color,
                end_color,
                theme.button_rounding(),
            );
        }

        // Text - use contrasting color based on background
        let text_color = Theme::contrasting_text(start_color);
        ui.painter().text(
            scaled_rect.center(),
            egui::Align2::CENTER_CENTER,
            &self.text,
            theme.font_body(),
            text_color,
        );

        // Request repaint if animating
        if self.state.is_animating() {
            ui.ctx().request_repaint();
        }

        response
    }
}

/// Icon button (for controls) - uses Lucide-style vector icons
pub struct IconButton {
    icon: Icon,
    size: f32,
    state: InteractionState,
    filled: bool,
    icon_scale: f32,
    custom_gradient: Option<(Color32, Color32)>,
    is_light_mode: bool,
}

impl IconButton {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: 48.0,
            state: InteractionState::new(),
            filled: false,
            icon_scale: 0.5,
            custom_gradient: None,
            is_light_mode: false,
        }
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn filled(mut self, filled: bool) -> Self {
        self.filled = filled;
        self
    }

    /// Set icon scale relative to button size (default: 0.5)
    pub fn with_icon_scale(mut self, scale: f32) -> Self {
        self.icon_scale = scale;
        self
    }

    /// Set custom gradient colors for filled button
    pub fn with_gradient(mut self, start: Color32, end: Color32) -> Self {
        self.custom_gradient = Some((start, end));
        self
    }

    /// Enable light mode styling (beige bg, black border, black icon)
    pub fn light_mode(mut self, enabled: bool) -> Self {
        self.is_light_mode = enabled;
        self
    }

    pub fn show(mut self, ui: &mut Ui, theme: &Theme) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(self.size, self.size), Sense::click());

        self.state
            .update(response.hovered(), response.is_pointer_button_down_on());

        let hover_t = self.state.hover_t();
        let press_t = self.state.press_t();

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        // Scale effect
        let scale = 1.0 - press_t * 0.05;
        let scaled_size = self.size * scale;
        let scaled_rect = Rect::from_center_size(rect.center(), vec2(scaled_size, scaled_size));

        // Background and icon color
        let icon_color = if self.is_light_mode {
            // Light mode: beige background, black border, black icon
            let beige = Color32::from_rgb(250, 247, 240);
            let beige_hover = Color32::from_rgb(245, 241, 232);
            let bg_color = Theme::lerp_color(beige, beige_hover, hover_t);
            ui.painter()
                .rect_filled(scaled_rect, scaled_size / 2.0, bg_color);

            // Black border
            let border_color = Color32::from_rgb(60, 60, 60);
            ui.painter().rect_stroke(
                scaled_rect,
                scaled_size / 2.0,
                Stroke::new(1.5, border_color),
            );

            // Black icon
            Color32::from_rgb(20, 20, 20)
        } else if self.filled {
            let (start, end) = self
                .custom_gradient
                .unwrap_or_else(|| theme.accent_gradient());
            let start = brighten_color(start, hover_t * 0.15);
            let end = brighten_color(end, hover_t * 0.15);
            draw_gradient_rect(
                ui,
                scaled_rect,
                start,
                end,
                Rounding::same(scaled_size / 2.0),
            );
            Theme::contrasting_text(start)
        } else if let Some((start, _end)) = self.custom_gradient {
            // Outline style with accent color - subtle bg, colored border and icon
            let bg_color = Theme::lerp_color(theme.bg_tertiary, theme.bg_hover, hover_t);
            ui.painter()
                .rect_filled(scaled_rect, scaled_size / 2.0, bg_color);

            // Colored border (muted)
            let border_color = Theme::with_alpha(start, (120.0 + hover_t * 60.0) as u8);
            ui.painter().rect_stroke(
                scaled_rect,
                scaled_size / 2.0,
                Stroke::new(1.5, border_color),
            );

            // Icon uses accent color (muted when not hovered)
            let alpha = (140.0 + hover_t * 115.0) as u8;
            Theme::with_alpha(start, alpha)
        } else {
            // Default style - gray
            let bg_color = Theme::lerp_color(theme.bg_tertiary, theme.bg_hover, hover_t);
            ui.painter()
                .rect_filled(scaled_rect, scaled_size / 2.0, bg_color);

            // Border
            let border_color =
                Theme::lerp_color(theme.border_subtle, theme.border_default, hover_t);
            ui.painter().rect_stroke(
                scaled_rect,
                scaled_size / 2.0,
                Stroke::new(1.0, border_color),
            );

            Theme::lerp_color(theme.text_secondary, theme.text_primary, hover_t)
        };

        // Draw vector icon
        let icon_size = scaled_size * self.icon_scale;
        let icon_rect = Rect::from_center_size(scaled_rect.center(), vec2(icon_size, icon_size));
        draw_icon(ui, self.icon, icon_rect, icon_color);

        if self.state.is_animating() {
            ui.ctx().request_repaint();
        }

        response
    }
}

/// Draw a horizontal gradient rectangle using segmented approach
fn draw_gradient_rect(ui: &mut Ui, rect: Rect, start: Color32, end: Color32, rounding: Rounding) {
    // Draw base with mid-color
    let mid_color = Theme::lerp_color(start, end, 0.5);
    ui.painter().rect_filled(rect, rounding, mid_color);

    // Overlay gradient segments for smoother appearance
    let steps = 4;
    let step_width = rect.width() / steps as f32;

    for i in 0..steps {
        let t = i as f32 / (steps - 1) as f32;
        let color = Theme::lerp_color(start, end, t);
        let x = rect.left() + step_width * i as f32;

        let step_rect =
            Rect::from_min_size(egui::pos2(x, rect.top()), vec2(step_width, rect.height()));

        // Apply rounding only to first and last segments
        let step_rounding = if i == 0 {
            Rounding {
                nw: rounding.nw,
                sw: rounding.sw,
                ne: 0.0,
                se: 0.0,
            }
        } else if i == steps - 1 {
            Rounding {
                nw: 0.0,
                sw: 0.0,
                ne: rounding.ne,
                se: rounding.se,
            }
        } else {
            Rounding::ZERO
        };

        ui.painter()
            .rect_filled(step_rect, step_rounding, Theme::with_alpha(color, 80));
    }
}

/// Brighten a color by a factor (0.0 to 1.0)
fn brighten_color(color: Color32, factor: f32) -> Color32 {
    let factor = factor.clamp(0.0, 1.0);
    Color32::from_rgb(
        (color.r() as f32 + (255.0 - color.r() as f32) * factor) as u8,
        (color.g() as f32 + (255.0 - color.g() as f32) * factor) as u8,
        (color.b() as f32 + (255.0 - color.b() as f32) * factor) as u8,
    )
}
