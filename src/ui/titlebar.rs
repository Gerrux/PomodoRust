//! Minimal window title bar - appears on hover

use egui::{vec2, Color32, CursorIcon, Rect, Rounding, Sense, Ui};

use super::animations::InteractionState;
use super::components::{draw_icon, Icon};
use super::theme::Theme;

/// Title bar button type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleBarButton {
    AlwaysOnTop,
    Minimize,
    Maximize,
    Close,
}

/// Minimal title bar component - shows controls on hover
pub struct TitleBar {
    pin_state: InteractionState,
    minimize_state: InteractionState,
    maximize_state: InteractionState,
    close_state: InteractionState,
    bar_hover_state: InteractionState,
}

impl TitleBar {
    pub fn new() -> Self {
        Self {
            pin_state: InteractionState::new(),
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
        is_always_on_top: bool,
    ) -> (bool, Option<TitleBarButton>) {
        self.show_with_status(ui, theme, is_maximized, is_always_on_top, None)
    }

    /// Render the title bar with optional status message
    /// Returns: (should_drag, clicked_button)
    pub fn show_with_status(
        &mut self,
        ui: &mut Ui,
        theme: &Theme,
        is_maximized: bool,
        is_always_on_top: bool,
        status_message: Option<&str>,
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
        // Use higher opacity for light theme for better visibility
        let bg_alpha = if theme.is_light {
            (hover_t * 0.85 * 255.0) as u8
        } else {
            (hover_t * 0.6 * 255.0) as u8
        };
        if bg_alpha > 0 {
            // Use darker bg for light theme
            let base_bg = if theme.is_light {
                Color32::from_rgb(230, 230, 235)
            } else {
                theme.bg_secondary
            };
            let bg_color =
                Color32::from_rgba_unmultiplied(base_bg.r(), base_bg.g(), base_bg.b(), bg_alpha);

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
        let buttons_width = button_size.x * 4.0; // Pin + Minimize + Maximize + Close

        let buttons_rect = Rect::from_min_size(
            title_bar_rect.right_top() - vec2(buttons_width, 0.0),
            vec2(buttons_width, Self::HEIGHT),
        );

        // Drag area is everything except the buttons
        let drag_rect = Rect::from_min_max(
            title_bar_rect.min,
            title_bar_rect.max - vec2(buttons_width, 0.0),
        );

        let drag_response = ui.interact(
            drag_rect,
            egui::Id::new("titlebar_drag_area"),
            Sense::drag(),
        );

        if drag_response.drag_started() {
            should_drag = true;
        }

        // Double-click to maximize
        if drag_response.double_clicked() {
            clicked_button = Some(TitleBarButton::Maximize);
        }

        // Draw status message in the center of drag area
        if let Some(message) = status_message {
            let text = egui::RichText::new(message).color(theme.success).small();
            let galley = ui.painter().layout_no_wrap(
                text.text().to_string(),
                egui::FontId::new(12.0, egui::FontFamily::Proportional),
                theme.success,
            );
            let text_pos = egui::pos2(
                drag_rect.center().x - galley.size().x / 2.0,
                drag_rect.center().y - galley.size().y / 2.0,
            );
            ui.painter().galley(text_pos, galley, theme.success);
        }

        // Draw buttons only if hovering or animating
        if hover_t > 0.01 {
            let mut button_x = buttons_rect.left();

            // Always on top (pin) button
            let pin_rect =
                Rect::from_min_size(egui::pos2(button_x, buttons_rect.top()), button_size);
            if let Some(btn) = self.draw_pin_button(ui, pin_rect, theme, is_always_on_top, hover_t)
            {
                clicked_button = Some(btn);
            }
            button_x += button_size.x;

            // Minimize button
            let min_rect =
                Rect::from_min_size(egui::pos2(button_x, buttons_rect.top()), button_size);
            if let Some(btn) = self.draw_button(
                ui,
                min_rect,
                TitleBarButton::Minimize,
                theme,
                is_maximized,
                hover_t,
            ) {
                clicked_button = Some(btn);
            }
            button_x += button_size.x;

            // Maximize button
            let max_rect =
                Rect::from_min_size(egui::pos2(button_x, buttons_rect.top()), button_size);
            if let Some(btn) = self.draw_button(
                ui,
                max_rect,
                TitleBarButton::Maximize,
                theme,
                is_maximized,
                hover_t,
            ) {
                clicked_button = Some(btn);
            }
            button_x += button_size.x;

            // Close button
            let close_rect =
                Rect::from_min_size(egui::pos2(button_x, buttons_rect.top()), button_size);
            if let Some(btn) = self.draw_button(
                ui,
                close_rect,
                TitleBarButton::Close,
                theme,
                is_maximized,
                hover_t,
            ) {
                clicked_button = Some(btn);
            }
        }

        // Request repaint if animating
        if self.bar_hover_state.is_animating()
            || self.pin_state.is_animating()
            || self.minimize_state.is_animating()
            || self.maximize_state.is_animating()
            || self.close_state.is_animating()
        {
            ui.ctx().request_repaint();
        }

        (should_drag, clicked_button)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_button(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        button_type: TitleBarButton,
        theme: &Theme,
        is_maximized: bool,
        bar_hover_t: f32,
    ) -> Option<TitleBarButton> {
        let button_id = match button_type {
            TitleBarButton::AlwaysOnTop => "titlebar_btn_pin",
            TitleBarButton::Minimize => "titlebar_btn_minimize",
            TitleBarButton::Maximize => "titlebar_btn_maximize",
            TitleBarButton::Close => "titlebar_btn_close",
        };
        let response = ui.interact(rect, egui::Id::new(button_id), Sense::click());

        // Get the state for this button type
        let state = match button_type {
            TitleBarButton::AlwaysOnTop => &mut self.pin_state,
            TitleBarButton::Minimize => &mut self.minimize_state,
            TitleBarButton::Maximize => &mut self.maximize_state,
            TitleBarButton::Close => &mut self.close_state,
        };
        state.update(response.hovered(), response.is_pointer_button_down_on());
        let hover_t = state.hover_t();
        let press_t = state.press_t();

        // Button opacity based on bar hover
        let base_alpha = (bar_hover_t * 255.0) as u8;

        // Background color - more visible hover for light theme
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
            let hover_bg = if theme.is_light {
                Color32::from_rgb(200, 200, 210) // Darker hover for light theme
            } else {
                theme.bg_hover
            };
            Color32::from_rgba_unmultiplied(
                hover_bg.r(),
                hover_bg.g(),
                hover_bg.b(),
                (hover_t * base_alpha as f32) as u8,
            )
        };

        // Draw background
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // Icon color with opacity - use dark icons for light theme
        let icon_alpha = base_alpha;
        let icon_color = if button_type == TitleBarButton::Close && hover_t > 0.0 {
            Color32::from_rgba_unmultiplied(255, 255, 255, icon_alpha)
        } else if theme.is_light {
            // Dark icons for light theme
            let dark = Color32::from_rgb(60, 60, 70);
            let darker = Color32::from_rgb(20, 20, 30);
            let base = Theme::lerp_color(dark, darker, hover_t);
            Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), icon_alpha)
        } else {
            let base = Theme::lerp_color(theme.text_muted, theme.text_primary, hover_t);
            Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), icon_alpha)
        };

        // Draw icon
        let icon_size = 12.0 - press_t * 1.0;
        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_size, icon_size));

        let icon = match button_type {
            TitleBarButton::AlwaysOnTop => Icon::Pin, // handled by draw_pin_button
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

    fn draw_pin_button(
        &mut self,
        ui: &mut Ui,
        rect: Rect,
        theme: &Theme,
        is_pinned: bool,
        bar_hover_t: f32,
    ) -> Option<TitleBarButton> {
        let response = ui.interact(rect, egui::Id::new("titlebar_pin_button"), Sense::click());

        self.pin_state
            .update(response.hovered(), response.is_pointer_button_down_on());
        let hover_t = self.pin_state.hover_t();
        let press_t = self.pin_state.press_t();

        // Button opacity based on bar hover
        let base_alpha = (bar_hover_t * 255.0) as u8;

        // Background color - highlight when pinned
        let accent_color = theme.accent.solid();
        let bg_color = if is_pinned {
            Color32::from_rgba_unmultiplied(
                accent_color.r(),
                accent_color.g(),
                accent_color.b(),
                ((0.3 + hover_t * 0.3) * base_alpha as f32) as u8,
            )
        } else {
            let hover_bg = if theme.is_light {
                Color32::from_rgb(200, 200, 210)
            } else {
                theme.bg_hover
            };
            Color32::from_rgba_unmultiplied(
                hover_bg.r(),
                hover_bg.g(),
                hover_bg.b(),
                (hover_t * base_alpha as f32) as u8,
            )
        };

        // Draw background
        ui.painter().rect_filled(rect, 0.0, bg_color);

        // Icon color with opacity - brighter when pinned
        let icon_color = if is_pinned {
            Color32::from_rgba_unmultiplied(
                accent_color.r(),
                accent_color.g(),
                accent_color.b(),
                base_alpha,
            )
        } else if theme.is_light {
            // Dark icons for light theme
            let dark = Color32::from_rgb(60, 60, 70);
            let darker = Color32::from_rgb(20, 20, 30);
            let base = Theme::lerp_color(dark, darker, hover_t);
            Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), base_alpha)
        } else {
            let base = Theme::lerp_color(theme.text_muted, theme.text_primary, hover_t);
            Color32::from_rgba_unmultiplied(base.r(), base.g(), base.b(), base_alpha)
        };

        // Draw icon
        let icon_size = 12.0 - press_t * 1.0;
        let icon_rect = Rect::from_center_size(rect.center(), vec2(icon_size, icon_size));

        let icon = if is_pinned { Icon::Pin } else { Icon::PinOff };
        draw_icon(ui, icon, icon_rect, icon_color);

        if response.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);

            // Show tooltip
            let tooltip_text = if is_pinned {
                "Unpin window (disable always on top)"
            } else {
                "Pin window (always on top)"
            };
            egui::show_tooltip_at_pointer(
                ui.ctx(),
                egui::LayerId::new(
                    egui::Order::Tooltip,
                    egui::Id::new("titlebar_tooltip_layer"),
                ),
                egui::Id::new("titlebar_pin_tooltip"),
                |ui| {
                    ui.label(tooltip_text);
                },
            );
        }

        if response.clicked() {
            Some(TitleBarButton::AlwaysOnTop)
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
