use egui::{self, Color32, FontId, Rounding, Stroke};

use crate::core::SessionType;
use super::{AccentColor, Theme};

impl Theme {
    /// Get gradient colors for current session type
    /// Work sessions use the accent color, breaks use complementary colors
    pub fn session_gradient(&self, session_type: SessionType) -> (Color32, Color32) {
        // For retro themes, use theme-appropriate colors
        if self.accent.is_retro() {
            return self.retro_session_gradient(session_type);
        }

        match session_type {
            // Work sessions use the accent color (light-adjusted if needed)
            SessionType::Work => {
                if self.is_light {
                    self.accent.gradient_light()
                } else {
                    self.accent.gradient()
                }
            }
            SessionType::ShortBreak => (self.break_start, self.break_end),
            SessionType::LongBreak => (self.long_break_start, self.long_break_end),
        }
    }

    /// Get retro-themed session colors
    /// Returns darker colors for light mode, bright neon for dark mode
    fn retro_session_gradient(&self, session_type: SessionType) -> (Color32, Color32) {
        if self.is_light {
            self.retro_session_gradient_light(session_type)
        } else {
            self.retro_session_gradient_dark(session_type)
        }
    }

    /// Retro session colors for dark mode (bright neon)
    fn retro_session_gradient_dark(
        &self,
        session_type: SessionType,
    ) -> (Color32, Color32) {
        match self.accent {
            AccentColor::Matrix => match session_type {
                SessionType::Work => {
                    (Color32::from_rgb(0, 255, 65), Color32::from_rgb(0, 200, 50))
                }
                SessionType::ShortBreak => (
                    Color32::from_rgb(0, 200, 200),
                    Color32::from_rgb(0, 160, 160),
                ),
                SessionType::LongBreak => (
                    Color32::from_rgb(0, 150, 255),
                    Color32::from_rgb(0, 120, 200),
                ),
            },
            AccentColor::RetroAmber => match session_type {
                SessionType::Work => (
                    Color32::from_rgb(255, 176, 0),
                    Color32::from_rgb(255, 204, 0),
                ),
                SessionType::ShortBreak => (
                    Color32::from_rgb(255, 140, 60),
                    Color32::from_rgb(255, 160, 80),
                ),
                SessionType::LongBreak => (
                    Color32::from_rgb(255, 100, 50),
                    Color32::from_rgb(255, 120, 70),
                ),
            },
            AccentColor::Synthwave => match session_type {
                SessionType::Work => (
                    Color32::from_rgb(255, 0, 128),
                    Color32::from_rgb(255, 50, 150),
                ),
                SessionType::ShortBreak => (
                    Color32::from_rgb(0, 255, 255),
                    Color32::from_rgb(50, 200, 255),
                ),
                SessionType::LongBreak => (
                    Color32::from_rgb(180, 0, 255),
                    Color32::from_rgb(140, 50, 255),
                ),
            },
            _ => self.accent.gradient(),
        }
    }

    /// Retro session colors for light mode - BLACK like printed terminal
    fn retro_session_gradient_light(
        &self,
        session_type: SessionType,
    ) -> (Color32, Color32) {
        // All retro themes use black in light mode - the retro feel comes from
        // the ASCII art and monospace font, not from colored text
        match self.accent {
            AccentColor::Matrix => match session_type {
                SessionType::Work => {
                    (Color32::from_rgb(0, 0, 0), Color32::from_rgb(20, 20, 20))
                }
                SessionType::ShortBreak => (
                    Color32::from_rgb(20, 30, 20), // Slight green tint
                    Color32::from_rgb(10, 20, 10),
                ),
                SessionType::LongBreak => (
                    Color32::from_rgb(20, 30, 30), // Slight teal tint
                    Color32::from_rgb(10, 20, 20),
                ),
            },
            AccentColor::RetroAmber => match session_type {
                SessionType::Work => (
                    Color32::from_rgb(0, 0, 0),
                    Color32::from_rgb(30, 20, 10), // Warm black
                ),
                SessionType::ShortBreak => (
                    Color32::from_rgb(40, 25, 10), // Dark brown
                    Color32::from_rgb(30, 20, 5),
                ),
                SessionType::LongBreak => (
                    Color32::from_rgb(50, 20, 10), // Dark red-brown
                    Color32::from_rgb(40, 15, 5),
                ),
            },
            AccentColor::Synthwave => match session_type {
                SessionType::Work => (
                    Color32::from_rgb(0, 0, 0),
                    Color32::from_rgb(20, 10, 30), // Purple-black
                ),
                SessionType::ShortBreak => (
                    Color32::from_rgb(10, 20, 30), // Cyan-black
                    Color32::from_rgb(5, 15, 25),
                ),
                SessionType::LongBreak => (
                    Color32::from_rgb(30, 10, 40), // Deep purple
                    Color32::from_rgb(20, 5, 30),
                ),
            },
            _ => self.accent.gradient_light(),
        }
    }

    /// Get accent gradient (light-adjusted if in light mode)
    pub fn accent_gradient(&self) -> (Color32, Color32) {
        if self.is_light {
            self.accent.gradient_light()
        } else {
            self.accent.gradient()
        }
    }

    /// Get window rounding
    pub fn window_rounding(&self) -> Rounding {
        Rounding::same(self.rounding_lg)
    }

    /// Get card rounding
    pub fn card_rounding(&self) -> Rounding {
        Rounding::same(self.rounding_lg)
    }

    /// Get button rounding
    pub fn button_rounding(&self) -> Rounding {
        Rounding::same(self.rounding_md)
    }

    /// Get subtle border stroke
    pub fn subtle_stroke(&self) -> Stroke {
        Stroke::new(1.0, self.border_subtle)
    }

    /// Get default border stroke
    pub fn default_stroke(&self) -> Stroke {
        Stroke::new(1.0, self.border_default)
    }

    /// Interpolate between two colors (gamma-correct)
    pub fn lerp_color(c1: Color32, c2: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        // Use gamma-correct blending for better visual interpolation
        let r = Self::gamma_lerp(c1.r(), c2.r(), t);
        let g = Self::gamma_lerp(c1.g(), c2.g(), t);
        let b = Self::gamma_lerp(c1.b(), c2.b(), t);
        let alpha = (c1.a() as f32 + (c2.a() as f32 - c1.a() as f32) * t) as u8;
        Color32::from_rgba_unmultiplied(r, g, b, alpha)
    }

    /// Gamma-correct lerp for a single channel
    fn gamma_lerp(a: u8, b: u8, t: f32) -> u8 {
        let a = (a as f32 / 255.0).powf(2.2);
        let b = (b as f32 / 255.0).powf(2.2);
        let result = a + (b - a) * t;
        (result.powf(1.0 / 2.2) * 255.0) as u8
    }

    /// Create a color with adjusted alpha
    pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
    }

    /// Brighten a color by a factor (0.0 to 1.0)
    pub fn brighten(color: Color32, factor: f32) -> Color32 {
        let factor = factor.clamp(0.0, 1.0);
        Color32::from_rgb(
            (color.r() as f32 + (255.0 - color.r() as f32) * factor) as u8,
            (color.g() as f32 + (255.0 - color.g() as f32) * factor) as u8,
            (color.b() as f32 + (255.0 - color.b() as f32) * factor) as u8,
        )
    }

    /// Darken a color by a factor (0.0 to 1.0)
    pub fn darken(color: Color32, factor: f32) -> Color32 {
        let factor = 1.0 - factor.clamp(0.0, 1.0);
        Color32::from_rgb(
            (color.r() as f32 * factor) as u8,
            (color.g() as f32 * factor) as u8,
            (color.b() as f32 * factor) as u8,
        )
    }

    /// Get relative luminance (for contrast calculations)
    pub fn luminance(color: Color32) -> f32 {
        let r = Self::gamma_to_linear(color.r());
        let g = Self::gamma_to_linear(color.g());
        let b = Self::gamma_to_linear(color.b());
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn gamma_to_linear(value: u8) -> f32 {
        let v = value as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Check if text color has sufficient contrast against background
    /// Returns true if contrast ratio >= 4.5:1 (WCAG AA)
    pub fn has_sufficient_contrast(foreground: Color32, background: Color32) -> bool {
        let l1 = Self::luminance(foreground);
        let l2 = Self::luminance(background);
        let lighter = l1.max(l2);
        let darker = l1.min(l2);
        let contrast = (lighter + 0.05) / (darker + 0.05);
        contrast >= 4.5
    }

    /// Get a contrasting text color for any background
    pub fn contrasting_text(background: Color32) -> Color32 {
        let lum = Self::luminance(background);
        if lum > 0.179 {
            Color32::from_rgb(10, 10, 10)
        } else {
            Color32::from_rgb(250, 250, 250)
        }
    }

    /// Apply theme to egui context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        // Visuals
        style.visuals.dark_mode = !self.is_light;
        style.visuals.override_text_color = Some(self.text_primary);
        style.visuals.panel_fill = self.bg_primary;
        style.visuals.window_fill = self.bg_primary;
        style.visuals.extreme_bg_color = self.bg_base;
        style.visuals.faint_bg_color = self.bg_tertiary;
        style.visuals.code_bg_color = self.bg_tertiary;

        // Widget visuals
        let rounding = self.button_rounding();

        style.visuals.widgets.noninteractive.bg_fill = self.bg_secondary;
        style.visuals.widgets.noninteractive.weak_bg_fill = self.bg_secondary;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_subtle);
        style.visuals.widgets.noninteractive.rounding = rounding;

        style.visuals.widgets.inactive.bg_fill = self.bg_tertiary;
        style.visuals.widgets.inactive.weak_bg_fill = self.bg_tertiary;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border_subtle);
        style.visuals.widgets.inactive.rounding = rounding;

        style.visuals.widgets.hovered.bg_fill = self.bg_hover;
        style.visuals.widgets.hovered.weak_bg_fill = self.bg_hover;
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.border_default);
        style.visuals.widgets.hovered.rounding = rounding;

        style.visuals.widgets.active.bg_fill = self.bg_active;
        style.visuals.widgets.active.weak_bg_fill = self.bg_active;
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.border_strong);
        style.visuals.widgets.active.rounding = rounding;

        style.visuals.widgets.open.bg_fill = self.bg_tertiary;
        style.visuals.widgets.open.weak_bg_fill = self.bg_tertiary;
        style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.border_default);
        style.visuals.widgets.open.rounding = rounding;

        // Selection
        let accent_solid = if self.is_light {
            self.accent.solid_light()
        } else {
            self.accent.solid()
        };
        style.visuals.selection.bg_fill = Self::with_alpha(accent_solid, 100);
        style.visuals.selection.stroke = Stroke::new(1.0, accent_solid);

        // Window
        style.visuals.window_rounding = self.window_rounding();
        style.visuals.window_stroke = self.subtle_stroke();
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: egui::Vec2::new(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
            color: Color32::from_black_alpha(self.shadow_lg),
        };

        // Spacing
        style.spacing.item_spacing = egui::Vec2::new(self.spacing_sm, self.spacing_sm);
        style.spacing.window_margin = egui::Margin::same(self.spacing_md);
        style.spacing.button_padding = egui::Vec2::new(self.spacing_md, self.spacing_sm);

        ctx.set_style(style);
    }

    /// Get font ID for different text styles
    pub fn font_display(&self) -> FontId {
        FontId::new(48.0, egui::FontFamily::Proportional)
    }

    pub fn font_h1(&self) -> FontId {
        FontId::new(32.0, egui::FontFamily::Proportional)
    }

    pub fn font_h2(&self) -> FontId {
        FontId::new(24.0, egui::FontFamily::Proportional)
    }

    pub fn font_body(&self) -> FontId {
        FontId::new(16.0, egui::FontFamily::Proportional)
    }

    pub fn font_small(&self) -> FontId {
        FontId::new(14.0, egui::FontFamily::Proportional)
    }

    pub fn font_caption(&self) -> FontId {
        FontId::new(12.0, egui::FontFamily::Proportional)
    }

    /// Create a high contrast version of the theme for accessibility
    pub fn with_high_contrast(mut self) -> Self {
        if self.is_light {
            // Light high contrast: pure white background, black text
            self.bg_base = Color32::WHITE;
            self.bg_primary = Color32::WHITE;
            self.bg_secondary = Color32::from_rgb(245, 245, 245);
            self.bg_tertiary = Color32::from_rgb(230, 230, 230);
            self.bg_elevated = Color32::WHITE;
            self.bg_hover = Color32::from_rgb(220, 220, 220);
            self.bg_active = Color32::from_rgb(200, 200, 200);

            // Dark borders for light theme
            self.border_subtle = Color32::from_rgb(150, 150, 150);
            self.border_default = Color32::from_rgb(80, 80, 80);
            self.border_strong = Color32::BLACK;

            // Maximum contrast dark text
            self.text_primary = Color32::BLACK;
            self.text_secondary = Color32::from_rgb(30, 30, 30);
            self.text_muted = Color32::from_rgb(60, 60, 60);
            self.text_disabled = Color32::from_rgb(120, 120, 120);

            // Darker semantic colors for light background
            self.success = Color32::from_rgb(0, 140, 60);
            self.warning = Color32::from_rgb(180, 130, 0);
            self.error = Color32::from_rgb(200, 0, 0);

            // Darker accent colors for high contrast on light
            let (start, end) = self.accent.gradient();
            self.work_start = Self::darken(start, 0.2);
            self.work_end = Self::darken(end, 0.2);
        } else {
            // Dark high contrast: pure black background, white text
            self.bg_base = Color32::BLACK;
            self.bg_primary = Color32::BLACK;
            self.bg_secondary = Color32::from_rgb(10, 10, 10);
            self.bg_tertiary = Color32::from_rgb(20, 20, 20);
            self.bg_elevated = Color32::from_rgb(25, 25, 25);
            self.bg_hover = Color32::from_rgb(40, 40, 40);
            self.bg_active = Color32::from_rgb(50, 50, 50);

            // High contrast borders
            self.border_subtle = Color32::from_rgb(100, 100, 100);
            self.border_default = Color32::from_rgb(180, 180, 180);
            self.border_strong = Color32::WHITE;

            // Maximum contrast text
            self.text_primary = Color32::WHITE;
            self.text_secondary = Color32::from_rgb(220, 220, 220);
            self.text_muted = Color32::from_rgb(180, 180, 180);
            self.text_disabled = Color32::from_rgb(120, 120, 120);

            // Brighter semantic colors
            self.success = Color32::from_rgb(0, 255, 100);
            self.warning = Color32::from_rgb(255, 220, 0);
            self.error = Color32::from_rgb(255, 80, 80);

            // Brighter accent colors for high contrast
            let (start, end) = self.accent.gradient();
            self.work_start = Self::brighten(start, 0.3);
            self.work_end = Self::brighten(end, 0.3);
        }

        self
    }
}
