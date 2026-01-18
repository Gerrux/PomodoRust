//! Minimal window title bar - appears on hover

use egui::{vec2, Color32, CursorIcon, Rect, Rounding, Sense, Ui};

use super::animations::InteractionState;
use super::components::{draw_icon, Icon};
use super::theme::Theme;

/// Title bar button type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleBarButton {
    Minimize,
    Maximize,
    Close,
}

/// Minimal title bar component - shows controls on hover
pub struct TitleBar {
    minimize_state: InteractionState,
    maximize_state: InteractionState,
    close_state: InteractionState,
    bar_hover_state: InteractionState,
}

impl TitleBar {
    pub fn new() -> Self {
        Self {
            minimize_state: InteractionState::new(),
            maximize_state: InteractionState::new(),
            close_state: InteractionState::new(),
            bar_hover_state: InteractionState::new(),
        }
    }

    /// Height of the title bar (minimal)
    pub const HEIGHT: f32 = 32.0;

    /// Render the title bar
    /// Returns: (should_drag, clicked_button)
    pub fn show(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        is_maximized: bool,
    ) -> (bool, Option<TitleBarButton>) {
        let mut clicked_button = None;
        let mut should_drag = false;

        let available_width = ui.available_width();
        let title_bar_rect = ui.allocate_space(vec2(available_width, Self::HEIGHT)).1;

        // Check if mouse is in title bar area
        let is_hovered = ui.rect_contains_pointer(title_bar_rect);
        self.bar_hover_state.update(is_hovered, false);
        let hover_t = self.bar_hover_state.hover_t();

        // Background - only visible on hover, with smooth transition
        let bg_alpha = (hover_t * 0.6 * 255.0) as u8;
        if bg_alpha > 0 {
            let bg_color = Color32::from_rgba_unmultiplied(
                theme.bg_secondary.r(),
                theme.bg_secondary.g(),
                theme.bg_secondary.b(),
                bg_alpha,
            );

            let rounding = if is_maximized {
                Rounding::ZERO
            } else {
                Rounding {
                    nw: theme.rounding_lg,
                    ne: theme.rounding_lg,
                    sw: 0.0,
                    se: 0.0,
                }
            };

            ui.painter().rect_filled(title_bar_rect, rounding, bg_color);
        }

        // Window control buttons (right side) - only visible on hover
        let button_size = vec2(40.0, Self::HEIGHT);
        let buttons_width = button_size.x * 3.0;

        let buttons_rect = Rect::from_min_size(
            title_bar_rect.right_top() - vec2(buttons_width, 0.0),
            vec2(buttons_width, Self::HEIGHT),
        );

        // Drag area is everything except the buttons
        let drag_rect = Rect::from_min_max(
            title_bar_rect.min,
            title_bar_rect.max - vec2(buttons_width, 0.0),
        );

        let drag_response = ui.interact(drag_rect, ui.id().with("titlebar_drag"), Sense::drag());

        if drag_response.dragged() {
            should_drag = true;
        }

        // Double-click to maximize
        if drag_response.double_clicked() {
            clicked_button = Some(TitleBarButton::Maximize);
        }

        // Draw buttons only if hovering or animating
        if hover_t > 0.01 {
            let mut button_x = buttons_rect.left();

            // Minimize button
            let min_rect = Rect::from_min_size(egui::pos2(button_x, buttons_rect.top()), button_size);
            if let Some(btn) = self.draw_button(
                ui,
                min_rect,
                TitleBarButton::Minimize,
                &mut self.minimize_state.clone(),
                theme,
                is_maximized,
                hover_t,
            ) {
                clicked_button = Some(btn);
            }
            button_x += button_size.x;

            // Maximize button
            let max_rect = Rect::from_min_size(egui::pos2(button_x, buttons_rect.top()), button_size);
            if let Some(btn) = self.draw_button(
                ui,
                max_rect,
                TitleBarButton::Maximize,
                &mut self.maximize_state.clone(),
                theme,
                is_maximized,
                hover_t,
            ) {
                clicked_button = Some(btn);
            }
            button_x += button_size.x;

            // Close button
            let close_rect = Rect::from_min_size(egui::pos2(button_x, buttons_rect.top()), button_size);
            if let Some(btn) = self.draw_button(
                ui,
                close_rect,
                TitleBarButton::Close,
                &mut self.close_state.clone(),
                theme,
                is_maximized,
                hover_t,
            ) {
                clicked_button = Some(btn);
            }
        }

        // Request repaint if animating
        if self.bar_hover_state.is_animating()
            || self.minimize_state.is_animating()
            || self.maximize_state.is_animating()
            || self.close_state.is_animating()
        {
            ui.ctx().request_repaint();
        }

        (should_drag, clicked_button)
    }

    fn draw_button(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        button_type: TitleBarButton,
        state: &mut InteractionState,
        theme: &Theme,
        is_maximized: bool,
        bar_hover_t: f32,
    ) -> Option<TitleBarButton> {
        let response = ui.interact(rect, ui.id().with(button_type as i32), Sense::click());

        state.update(response.hovered(), response.is_pointer_button_down_on());
        let hover_t = state.hover_t();
        let press_t = state.press_t();

        // Button opacity based on bar hover
        let base_alpha = (bar_hover_t * 255.0) as u8;

        // Background color
        let bg_color = if button_type == TitleBarButton::Close {
            if hover_t > 0.0 {
                let red = Color32::from_rgb(220, 38, 38);
                Color32::from_rgba_unmultiplied(
                    red.r(),
                    red.g(),
                    red.b(),
                    (hover_t * base_alpha as f32) as u8,
                )
            } else {
                Color32::TRANSPARENT
            }
        } else {
            let hover_bg = theme.bg_hover;
            Color32::from_rgba_unmultiplied(
                hover_bg.r(),
                hover_bg.g(),
                hover_bg.b(),
                (hover_t * base_alpha as f32) as u8,
            )
        };

        // Draw background
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // Icon color with opacity
        let icon_alpha = base_alpha;
        let icon_color = if button_type == TitleBarButton::Close && hover_t > 0.0 {
            Color32::from_rgba_unmultiplied(255, 255, 255, icon_alpha)
        } else {
            let base = Theme::lerp_color(theme.text_muted, theme.text_primary, hover_t);
            Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), icon_alpha)
        };

        // Draw icon
        let icon_size = 12.0 - press_t * 1.0;
        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_size, icon_size));

        let icon = match button_type {
            TitleBarButton::Minimize => Icon::Minimize,
            TitleBarButton::Maximize => {
                if is_maximized {
                    Icon::Restore
                } else {
                    Icon::Maximize
                }
            }
            TitleBarButton::Close => Icon::X,
        };

        draw_icon(ui, icon, icon_rect, icon_color);

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
        }

        if response.clicked() {
            Some(button_type)
        } else {
            None
        }
    }
}

impl Default for TitleBar {
    fn default() -> Self {
        Self::new()
    }
}
