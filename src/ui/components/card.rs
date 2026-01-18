//! Card component

use egui::{vec2, Color32, Response, Stroke, Ui, Vec2};

use crate::ui::theme::Theme;

/// A card component with consistent styling
pub struct Card {
    size: Option<Vec2>,
}

impl Card {
    pub fn new() -> Self {
        Self { size: None }
    }

    pub fn with_size(mut self, size: Vec2) -> Self {
        self.size = Some(size);
        self
    }

    pub fn show(
        self,
        ui: &mut Ui,
        theme: &Theme,
        content: impl FnOnce(&mut Ui),
    ) -> Response {
        egui::Frame::none()
            .fill(theme.bg_secondary)
            .rounding(theme.card_rounding())
            .stroke(Stroke::new(1.0, theme.border_subtle))
            .inner_margin(theme.spacing_md)
            .shadow(egui::epaint::Shadow {
                spread: 0.0,
                blur: 8.0,
                offset: vec2(0.0, 4.0),
                color: Color32::from_black_alpha(20),
            })
            .show(ui, |ui| {
                if let Some(size) = self.size {
                    ui.set_min_size(size - vec2(theme.spacing_md * 2.0, theme.spacing_md * 2.0));
                }
                content(ui);
            })
            .response
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}
