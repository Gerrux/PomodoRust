//! Design system - Vercel/shadcn inspired dark theme

use egui::{Color32, FontFamily, FontId, Rounding, Stroke, Vec2};
use serde::{Deserialize, Serialize};

/// Accent color options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AccentColor {
    #[default]
    Blue,
    Purple,
    Rose,
    Emerald,
    Amber,
    Cyan,
    // Retro terminal colors
    Matrix,     // Green phosphor
    RetroAmber, // Amber CRT
    Synthwave,  // Pink/cyan retro
}

impl AccentColor {
    pub fn all() -> &'static [AccentColor] {
        &[
            AccentColor::Blue,
            AccentColor::Purple,
            AccentColor::Rose,
            AccentColor::Emerald,
            AccentColor::Amber,
            AccentColor::Cyan,
            AccentColor::Matrix,
            AccentColor::RetroAmber,
            AccentColor::Synthwave,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            AccentColor::Blue => "Blue",
            AccentColor::Purple => "Purple",
            AccentColor::Rose => "Rose",
            AccentColor::Emerald => "Emerald",
            AccentColor::Amber => "Amber",
            AccentColor::Cyan => "Cyan",
            AccentColor::Matrix => "Matrix",
            AccentColor::RetroAmber => "Retro Amber",
            AccentColor::Synthwave => "Synthwave",
        }
    }

    pub fn gradient(&self) -> (Color32, Color32) {
        match self {
            AccentColor::Blue => (
                Color32::from_rgb(59, 130, 246), // blue-500
                Color32::from_rgb(139, 92, 246), // violet-500
            ),
            AccentColor::Purple => (
                Color32::from_rgb(139, 92, 246), // violet-500
                Color32::from_rgb(236, 72, 153), // pink-500
            ),
            AccentColor::Rose => (
                Color32::from_rgb(244, 63, 94),   // rose-500
                Color32::from_rgb(251, 113, 133), // rose-400
            ),
            AccentColor::Emerald => (
                Color32::from_rgb(16, 185, 129), // emerald-500
                Color32::from_rgb(52, 211, 153), // emerald-400
            ),
            AccentColor::Amber => (
                Color32::from_rgb(245, 158, 11), // amber-500
                Color32::from_rgb(251, 191, 36), // amber-400
            ),
            AccentColor::Cyan => (
                Color32::from_rgb(6, 182, 212),  // cyan-500
                Color32::from_rgb(34, 211, 238), // cyan-400
            ),
            // Retro terminal colors
            AccentColor::Matrix => (
                Color32::from_rgb(0, 255, 65), // Matrix green
                Color32::from_rgb(0, 200, 50), // Darker green
            ),
            AccentColor::RetroAmber => (
                Color32::from_rgb(255, 176, 0), // Amber
                Color32::from_rgb(255, 204, 0), // Lighter amber
            ),
            AccentColor::Synthwave => (
                Color32::from_rgb(255, 0, 128), // Hot pink
                Color32::from_rgb(0, 255, 255), // Cyan
            ),
        }
    }

    pub fn solid(&self) -> Color32 {
        self.gradient().0
    }

    /// Check if this is a retro/TUI style
    pub fn is_retro(&self) -> bool {
        matches!(
            self,
            AccentColor::Matrix | AccentColor::RetroAmber | AccentColor::Synthwave
        )
    }

    /// Get glow color for retro styles
    pub fn glow(&self) -> Color32 {
        match self {
            AccentColor::Matrix => Color32::from_rgba_unmultiplied(0, 255, 65, 60),
            AccentColor::RetroAmber => Color32::from_rgba_unmultiplied(255, 176, 0, 60),
            AccentColor::Synthwave => Color32::from_rgba_unmultiplied(255, 0, 128, 60),
            _ => Color32::TRANSPARENT,
        }
    }
}

/// The main theme struct containing all design tokens
#[derive(Debug, Clone)]
pub struct Theme {
    // Backgrounds
    pub bg_base: Color32,
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub bg_tertiary: Color32,
    pub bg_elevated: Color32,
    pub bg_hover: Color32,
    pub bg_active: Color32,

    // Borders
    pub border_subtle: Color32,
    pub border_default: Color32,
    pub border_strong: Color32,

    // Text
    pub text_primary: Color32,
    pub text_secondary: Color32,
    pub text_muted: Color32,
    pub text_disabled: Color32,

    // Accent
    pub accent: AccentColor,

    // Semantic colors
    pub success: Color32,
    pub success_muted: Color32,
    pub warning: Color32,
    pub warning_muted: Color32,
    pub error: Color32,
    pub error_muted: Color32,

    // Session type colors
    pub work_start: Color32,
    pub work_end: Color32,
    pub break_start: Color32,
    pub break_end: Color32,
    pub long_break_start: Color32,
    pub long_break_end: Color32,

    // Spacing
    pub spacing_xs: f32,
    pub spacing_sm: f32,
    pub spacing_md: f32,
    pub spacing_lg: f32,
    pub spacing_xl: f32,
    pub spacing_2xl: f32,

    // Rounding
    pub rounding_none: f32,
    pub rounding_sm: f32,
    pub rounding_md: f32,
    pub rounding_lg: f32,
    pub rounding_xl: f32,
    pub rounding_full: f32,

    // Shadows (as alpha values for shadow color)
    pub shadow_sm: u8,
    pub shadow_md: u8,
    pub shadow_lg: u8,

    // Animation durations (in seconds)
    pub anim_fast: f32,
    pub anim_normal: f32,
    pub anim_slow: f32,

    // Accessibility
    pub reduced_motion: bool,
}

impl Theme {
    pub fn new(accent: AccentColor) -> Self {
        Self {
            // Backgrounds - Vercel-inspired deep blacks
            bg_base: Color32::from_rgb(0, 0, 0), // Pure black for window bg
            bg_primary: Color32::from_rgb(10, 10, 11), // #0A0A0B
            bg_secondary: Color32::from_rgb(17, 17, 19), // #111113
            bg_tertiary: Color32::from_rgb(24, 24, 27), // #18181B
            bg_elevated: Color32::from_rgb(28, 28, 31), // #1C1C1F
            bg_hover: Color32::from_rgb(39, 39, 42), // #27272A
            bg_active: Color32::from_rgb(48, 48, 54), // #30303A

            // Borders
            border_subtle: Color32::from_rgb(39, 39, 42), // #27272A
            border_default: Color32::from_rgb(63, 63, 70), // #3F3F46
            border_strong: Color32::from_rgb(82, 82, 91), // #52525B

            // Text
            text_primary: Color32::from_rgb(250, 250, 250), // #FAFAFA
            text_secondary: Color32::from_rgb(161, 161, 170), // #A1A1AA
            text_muted: Color32::from_rgb(113, 113, 122),   // #71717A
            text_disabled: Color32::from_rgb(82, 82, 91),   // #52525B

            // Accent
            accent,

            // Semantic
            success: Color32::from_rgb(34, 197, 94), // green-500
            success_muted: Color32::from_rgb(22, 78, 39), // green-900/50
            warning: Color32::from_rgb(245, 158, 11), // amber-500
            warning_muted: Color32::from_rgb(78, 53, 5), // amber-900/50
            error: Color32::from_rgb(239, 68, 68),   // red-500
            error_muted: Color32::from_rgb(69, 26, 26), // red-900/50

            // Session colors
            work_start: Color32::from_rgb(244, 63, 94), // rose-500
            work_end: Color32::from_rgb(251, 113, 133), // rose-400
            break_start: Color32::from_rgb(16, 185, 129), // emerald-500
            break_end: Color32::from_rgb(52, 211, 153), // emerald-400
            long_break_start: Color32::from_rgb(99, 102, 241), // indigo-500
            long_break_end: Color32::from_rgb(129, 140, 248), // indigo-400

            // Spacing (in pixels)
            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 16.0,
            spacing_lg: 24.0,
            spacing_xl: 32.0,
            spacing_2xl: 48.0,

            // Rounding
            rounding_none: 0.0,
            rounding_sm: 4.0,
            rounding_md: 8.0,
            rounding_lg: 12.0,
            rounding_xl: 16.0,
            rounding_full: 9999.0,

            // Shadows
            shadow_sm: 20,
            shadow_md: 40,
            shadow_lg: 60,

            // Animations
            anim_fast: 0.15,
            anim_normal: 0.3,
            anim_slow: 0.5,

            // Accessibility
            reduced_motion: false,
        }
    }

    /// Create a theme with reduced motion enabled
    pub fn with_reduced_motion(mut self) -> Self {
        self.reduced_motion = true;
        // Set faster animation durations for reduced motion
        self.anim_fast = 0.0;
        self.anim_normal = 0.05;
        self.anim_slow = 0.1;
        self
    }

    /// Get gradient colors for current session type
    /// Work sessions use the accent color, breaks use complementary colors
    pub fn session_gradient(&self, session_type: crate::core::SessionType) -> (Color32, Color32) {
        // For retro themes, use theme-appropriate colors
        if self.accent.is_retro() {
            return self.retro_session_gradient(session_type);
        }

        match session_type {
            // Work sessions use the accent color
            crate::core::SessionType::Work => self.accent.gradient(),
            crate::core::SessionType::ShortBreak => (self.break_start, self.break_end),
            crate::core::SessionType::LongBreak => (self.long_break_start, self.long_break_end),
        }
    }

    /// Get retro-themed session colors
    fn retro_session_gradient(&self, session_type: crate::core::SessionType) -> (Color32, Color32) {
        match self.accent {
            AccentColor::Matrix => {
                match session_type {
                    // Work: Bright green
                    crate::core::SessionType::Work => {
                        (Color32::from_rgb(0, 255, 65), Color32::from_rgb(0, 200, 50))
                    }
                    // Short break: Cyan/teal
                    crate::core::SessionType::ShortBreak => (
                        Color32::from_rgb(0, 200, 200),
                        Color32::from_rgb(0, 160, 160),
                    ),
                    // Long break: Blue-green
                    crate::core::SessionType::LongBreak => (
                        Color32::from_rgb(0, 150, 255),
                        Color32::from_rgb(0, 120, 200),
                    ),
                }
            }
            AccentColor::RetroAmber => {
                match session_type {
                    // Work: Bright amber
                    crate::core::SessionType::Work => (
                        Color32::from_rgb(255, 176, 0),
                        Color32::from_rgb(255, 204, 0),
                    ),
                    // Short break: Soft orange
                    crate::core::SessionType::ShortBreak => (
                        Color32::from_rgb(255, 140, 60),
                        Color32::from_rgb(255, 160, 80),
                    ),
                    // Long break: Warm red-orange
                    crate::core::SessionType::LongBreak => (
                        Color32::from_rgb(255, 100, 50),
                        Color32::from_rgb(255, 120, 70),
                    ),
                }
            }
            AccentColor::Synthwave => {
                match session_type {
                    // Work: Hot pink
                    crate::core::SessionType::Work => (
                        Color32::from_rgb(255, 0, 128),
                        Color32::from_rgb(255, 50, 150),
                    ),
                    // Short break: Cyan
                    crate::core::SessionType::ShortBreak => (
                        Color32::from_rgb(0, 255, 255),
                        Color32::from_rgb(50, 200, 255),
                    ),
                    // Long break: Purple
                    crate::core::SessionType::LongBreak => (
                        Color32::from_rgb(180, 0, 255),
                        Color32::from_rgb(140, 50, 255),
                    ),
                }
            }
            // Fallback (shouldn't reach here)
            _ => self.accent.gradient(),
        }
    }

    /// Get accent gradient
    pub fn accent_gradient(&self) -> (Color32, Color32) {
        self.accent.gradient()
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
        style.visuals.dark_mode = true;
        style.visuals.override_text_color = Some(self.text_primary);
        style.visuals.panel_fill = self.bg_primary;
        style.visuals.window_fill = self.bg_primary;
        style.visuals.extreme_bg_color = self.bg_base;
        style.visuals.faint_bg_color = self.bg_tertiary;
        style.visuals.code_bg_color = self.bg_tertiary;

        // Widget visuals
        style.visuals.widgets.noninteractive.bg_fill = self.bg_secondary;
        style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_secondary);
        style.visuals.widgets.noninteractive.rounding = self.button_rounding();

        style.visuals.widgets.inactive.bg_fill = self.bg_tertiary;
        style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.inactive.rounding = self.button_rounding();

        style.visuals.widgets.hovered.bg_fill = self.bg_hover;
        style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.hovered.rounding = self.button_rounding();

        style.visuals.widgets.active.bg_fill = self.bg_active;
        style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.text_primary);
        style.visuals.widgets.active.rounding = self.button_rounding();

        // Selection
        style.visuals.selection.bg_fill = Self::with_alpha(self.accent.solid(), 100);
        style.visuals.selection.stroke = Stroke::new(1.0, self.accent.solid());

        // Window
        style.visuals.window_rounding = self.window_rounding();
        style.visuals.window_stroke = self.subtle_stroke();
        style.visuals.window_shadow = egui::epaint::Shadow {
            offset: Vec2::new(0.0, 4.0),
            blur: 16.0,
            spread: 0.0,
            color: Color32::from_black_alpha(self.shadow_lg),
        };

        // Spacing
        style.spacing.item_spacing = Vec2::new(self.spacing_sm, self.spacing_sm);
        style.spacing.window_margin = egui::Margin::same(self.spacing_md);
        style.spacing.button_padding = Vec2::new(self.spacing_md, self.spacing_sm);

        ctx.set_style(style);
    }

    /// Get font ID for different text styles
    pub fn font_display(&self) -> FontId {
        FontId::new(48.0, FontFamily::Proportional)
    }

    pub fn font_h1(&self) -> FontId {
        FontId::new(32.0, FontFamily::Proportional)
    }

    pub fn font_h2(&self) -> FontId {
        FontId::new(24.0, FontFamily::Proportional)
    }

    pub fn font_body(&self) -> FontId {
        FontId::new(16.0, FontFamily::Proportional)
    }

    pub fn font_small(&self) -> FontId {
        FontId::new(14.0, FontFamily::Proportional)
    }

    pub fn font_caption(&self) -> FontId {
        FontId::new(12.0, FontFamily::Proportional)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(AccentColor::Blue)
    }
}

impl Theme {
    /// Create a high contrast version of the theme for accessibility
    pub fn with_high_contrast(mut self) -> Self {
        // Pure black background for maximum contrast
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

        self
    }
}
