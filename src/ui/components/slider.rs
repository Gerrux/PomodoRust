//! Custom slider component

use egui::{vec2, Color32, Pos2, Rect, Response, Sense, Stroke, Ui};

use crate::ui::animations::InteractionState;
use crate::ui::theme::Theme;

/// Custom styled slider
pub struct CustomSlider<'a> {
    value: &'a mut f32,
    range: std::ops::RangeInclusive<f32>,
    width: f32,
    height: f32,
    state: InteractionState,
}

impl<'a> CustomSlider<'a> {
    pub fn new(value: &'a mut f32, range: std::ops::RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            width: 200.0,
            height: 24.0,
            state: InteractionState::new(),
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    pub fn show(mut self, ui: &mut Ui, theme: &Theme) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(self.width, self.height), Sense::drag());

        self.state.update(response.hovered(), response.dragged());

        let hover_t = self.state.hover_t();

        // Calculate value from drag
        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let track_start = rect.left() + self.height / 2.0;
                let track_end = rect.right() - self.height / 2.0;
                let track_width = track_end - track_start;

                let t = ((pos.x - track_start) / track_width).clamp(0.0, 1.0);
                let min = *self.range.start();
                let max = *self.range.end();
                *self.value = min + t * (max - min);
            }
        }

        // Normalize value
        let min = *self.range.start();
        let max = *self.range.end();
        let t = if max > min {
            ((*self.value - min) / (max - min)).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // Draw track
        let track_height = 6.0;
        let track_rect =
            Rect::from_center_size(rect.center(), vec2(self.width - self.height, track_height));

        // Background track
        ui.painter()
            .rect_filled(track_rect, track_height / 2.0, theme.bg_tertiary);

        // Filled portion with gradient
        let (start_color, end_color) = theme.accent_gradient();
        let filled_width = track_rect.width() * t;
        if filled_width > 0.0 {
            let filled_rect =
                Rect::from_min_size(track_rect.left_top(), vec2(filled_width, track_height));
            ui.painter().rect_filled(
                filled_rect,
                track_height / 2.0,
                Theme::lerp_color(start_color, end_color, t),
            );
        }

        // Handle
        let handle_radius = (self.height / 2.0 - 2.0) * (1.0 + hover_t * 0.1);
        let handle_x = track_rect.left() + track_rect.width() * t;
        let handle_center = Pos2::new(handle_x, rect.center().y);

        // Handle shadow
        ui.painter().circle_filled(
            handle_center + vec2(0.0, 2.0),
            handle_radius,
            Color32::from_black_alpha(40),
        );

        // Handle
        let handle_color = Theme::lerp_color(start_color, end_color, t);
        ui.painter()
            .circle_filled(handle_center, handle_radius, handle_color);

        // Handle border
        ui.painter().circle_stroke(
            handle_center,
            handle_radius,
            Stroke::new(2.0, Color32::WHITE),
        );

        if self.state.is_animating() {
            ui.ctx().request_repaint();
        }

        response
    }
}
