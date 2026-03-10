//! Design system - Vercel/shadcn inspired theme with light and dark modes

mod colors;

use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::platform::is_windows_11;

/// Theme mode options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
    // Catppuccin flavors
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
}

impl ThemeMode {
    pub fn all() -> &'static [ThemeMode] {
        &[
            ThemeMode::System,
            ThemeMode::Light,
            ThemeMode::Dark,
            ThemeMode::CatppuccinLatte,
            ThemeMode::CatppuccinFrappe,
            ThemeMode::CatppuccinMacchiato,
            ThemeMode::CatppuccinMocha,
        ]
    }

    pub fn name(&self) -> &'static str {
        crate::i18n::tr().theme_name(*self)
    }
}

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
        crate::i18n::tr().accent_name(*self)
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

    /// Get gradient colors optimized for light backgrounds
    /// Returns darker/more saturated versions that have good contrast on light
    pub fn gradient_light(&self) -> (Color32, Color32) {
        match self {
            // Standard colors - use slightly darker versions for better contrast
            AccentColor::Blue => (
                Color32::from_rgb(37, 99, 235),  // blue-600
                Color32::from_rgb(124, 58, 237), // violet-600
            ),
            AccentColor::Purple => (
                Color32::from_rgb(124, 58, 237), // violet-600
                Color32::from_rgb(219, 39, 119), // pink-600
            ),
            AccentColor::Rose => (
                Color32::from_rgb(225, 29, 72), // rose-600
                Color32::from_rgb(244, 63, 94), // rose-500
            ),
            AccentColor::Emerald => (
                Color32::from_rgb(5, 150, 105),  // emerald-600
                Color32::from_rgb(16, 185, 129), // emerald-500
            ),
            AccentColor::Amber => (
                Color32::from_rgb(217, 119, 6),  // amber-600
                Color32::from_rgb(245, 158, 11), // amber-500
            ),
            AccentColor::Cyan => (
                Color32::from_rgb(8, 145, 178), // cyan-600
                Color32::from_rgb(6, 182, 212), // cyan-500
            ),
            // Retro colors for light mode - use BLACK like printed terminal output
            AccentColor::Matrix => (
                Color32::from_rgb(0, 0, 0),    // Pure black
                Color32::from_rgb(20, 20, 20), // Near black
            ),
            AccentColor::RetroAmber => (
                Color32::from_rgb(0, 0, 0),    // Pure black
                Color32::from_rgb(30, 20, 10), // Black with warm tint
            ),
            AccentColor::Synthwave => (
                Color32::from_rgb(0, 0, 0),    // Pure black
                Color32::from_rgb(20, 10, 30), // Black with purple tint
            ),
        }
    }

    /// Get solid color optimized for light backgrounds
    pub fn solid_light(&self) -> Color32 {
        self.gradient_light().0
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

    // Mode tracking
    pub is_light: bool,
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

            // Rounding - use sharp corners on Windows 10, rounded on Windows 11/Linux/macOS
            rounding_none: 0.0,
            rounding_sm: if is_windows_11() { 4.0 } else { 0.0 },
            rounding_md: if is_windows_11() { 8.0 } else { 0.0 },
            rounding_lg: if is_windows_11() { 12.0 } else { 0.0 },
            rounding_xl: if is_windows_11() { 16.0 } else { 0.0 },
            rounding_full: 9999.0, // Keep for circular elements

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

            // Mode tracking
            is_light: false,
        }
    }

    /// Create a new light theme with the given accent color
    /// Uses high-contrast light palette for readability
    pub fn new_light(accent: AccentColor) -> Self {
        Self {
            // Backgrounds - subtle layering on light base
            bg_base: Color32::from_rgb(255, 255, 255), // Pure white
            bg_primary: Color32::from_rgb(250, 250, 250), // #FAFAFA - main bg
            bg_secondary: Color32::from_rgb(244, 244, 245), // #F4F4F5 zinc-100
            bg_tertiary: Color32::from_rgb(228, 228, 231), // #E4E4E7 zinc-200 - buttons
            bg_elevated: Color32::from_rgb(255, 255, 255), // White for elevated
            bg_hover: Color32::from_rgb(212, 212, 216), // #D4D4D8 zinc-300 - hover
            bg_active: Color32::from_rgb(161, 161, 170), // #A1A1AA zinc-400 - active

            // Borders - visible
            border_subtle: Color32::from_rgb(212, 212, 216), // #D4D4D8 zinc-300
            border_default: Color32::from_rgb(161, 161, 170), // #A1A1AA zinc-400
            border_strong: Color32::from_rgb(82, 82, 91),    // #52525B zinc-600

            // Text
            text_primary: Color32::from_rgb(9, 9, 11), // #09090B zinc-950
            text_secondary: Color32::from_rgb(63, 63, 70), // #3F3F46 zinc-700
            text_muted: Color32::from_rgb(113, 113, 122), // #71717A zinc-500
            text_disabled: Color32::from_rgb(161, 161, 170), // #A1A1AA zinc-400

            // Accent
            accent,

            // Semantic - darker for light bg contrast
            success: Color32::from_rgb(22, 163, 74), // green-600
            success_muted: Color32::from_rgb(220, 252, 231), // green-100
            warning: Color32::from_rgb(217, 119, 6), // amber-600
            warning_muted: Color32::from_rgb(254, 243, 199), // amber-100
            error: Color32::from_rgb(220, 38, 38),   // red-600
            error_muted: Color32::from_rgb(254, 226, 226), // red-100

            // Session colors - slightly darker for light mode contrast
            work_start: Color32::from_rgb(225, 29, 72), // rose-600
            work_end: Color32::from_rgb(244, 63, 94),   // rose-500
            break_start: Color32::from_rgb(5, 150, 105), // emerald-600
            break_end: Color32::from_rgb(16, 185, 129), // emerald-500
            long_break_start: Color32::from_rgb(79, 70, 229), // indigo-600
            long_break_end: Color32::from_rgb(99, 102, 241), // indigo-500

            // Spacing (in pixels) - same as dark
            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 16.0,
            spacing_lg: 24.0,
            spacing_xl: 32.0,
            spacing_2xl: 48.0,

            // Rounding - same as dark
            rounding_none: 0.0,
            rounding_sm: if is_windows_11() { 4.0 } else { 0.0 },
            rounding_md: if is_windows_11() { 8.0 } else { 0.0 },
            rounding_lg: if is_windows_11() { 12.0 } else { 0.0 },
            rounding_xl: if is_windows_11() { 16.0 } else { 0.0 },
            rounding_full: 9999.0,

            // Shadows - lighter for light mode
            shadow_sm: 10,
            shadow_md: 20,
            shadow_lg: 30,

            // Animations - same as dark
            anim_fast: 0.15,
            anim_normal: 0.3,
            anim_slow: 0.5,

            // Accessibility
            reduced_motion: false,

            // Mode tracking
            is_light: true,
        }
    }

    /// Create a theme from a Catppuccin flavor
    fn from_catppuccin(flavor: catppuccin_egui::Theme, accent: AccentColor, is_light: bool) -> Self {
        // For light flavors (Latte): layers go darker from base
        // For dark flavors: layers go lighter from base
        let (bg_tertiary, bg_hover, bg_active, bg_elevated) = if is_light {
            (flavor.surface0, flavor.surface1, flavor.surface2, flavor.base)
        } else {
            (flavor.surface0, flavor.surface1, flavor.surface2, flavor.surface1)
        };

        Self {
            // Backgrounds
            bg_base: flavor.crust,
            bg_primary: flavor.base,
            bg_secondary: flavor.mantle,
            bg_tertiary,
            bg_elevated,
            bg_hover,
            bg_active,

            // Borders
            border_subtle: if is_light { flavor.surface1 } else { flavor.surface1 },
            border_default: if is_light { flavor.surface2 } else { flavor.surface2 },
            border_strong: flavor.overlay0,

            // Text
            text_primary: flavor.text,
            text_secondary: flavor.subtext1,
            text_muted: flavor.subtext0,
            text_disabled: flavor.overlay1,

            // Accent
            accent,

            // Semantic
            success: flavor.green,
            success_muted: if is_light {
                Color32::from_rgba_unmultiplied(flavor.green.r(), flavor.green.g(), flavor.green.b(), 40)
            } else {
                Color32::from_rgba_unmultiplied(flavor.green.r(), flavor.green.g(), flavor.green.b(), 30)
            },
            warning: flavor.yellow,
            warning_muted: if is_light {
                Color32::from_rgba_unmultiplied(flavor.yellow.r(), flavor.yellow.g(), flavor.yellow.b(), 40)
            } else {
                Color32::from_rgba_unmultiplied(flavor.yellow.r(), flavor.yellow.g(), flavor.yellow.b(), 30)
            },
            error: flavor.red,
            error_muted: if is_light {
                Color32::from_rgba_unmultiplied(flavor.red.r(), flavor.red.g(), flavor.red.b(), 40)
            } else {
                Color32::from_rgba_unmultiplied(flavor.red.r(), flavor.red.g(), flavor.red.b(), 30)
            },

            // Session colors
            work_start: flavor.red,
            work_end: flavor.maroon,
            break_start: flavor.green,
            break_end: flavor.teal,
            long_break_start: flavor.blue,
            long_break_end: flavor.lavender,

            // Spacing
            spacing_xs: 4.0,
            spacing_sm: 8.0,
            spacing_md: 16.0,
            spacing_lg: 24.0,
            spacing_xl: 32.0,
            spacing_2xl: 48.0,

            // Rounding
            rounding_none: 0.0,
            rounding_sm: if is_windows_11() { 4.0 } else { 0.0 },
            rounding_md: if is_windows_11() { 8.0 } else { 0.0 },
            rounding_lg: if is_windows_11() { 12.0 } else { 0.0 },
            rounding_xl: if is_windows_11() { 16.0 } else { 0.0 },
            rounding_full: 9999.0,

            // Shadows
            shadow_sm: if is_light { 10 } else { 20 },
            shadow_md: if is_light { 20 } else { 40 },
            shadow_lg: if is_light { 30 } else { 60 },

            // Animations
            anim_fast: 0.15,
            anim_normal: 0.3,
            anim_slow: 0.5,

            // Accessibility
            reduced_motion: false,

            // Mode tracking
            is_light,
        }
    }

    /// Create a theme from mode and accent color
    /// System mode auto-detects from OS settings
    pub fn from_mode(mode: ThemeMode, accent: AccentColor) -> Self {
        match mode {
            ThemeMode::Light => Self::new_light(accent),
            ThemeMode::Dark => Self::new(accent),
            ThemeMode::System => {
                if crate::platform::system_uses_light_theme() {
                    Self::new_light(accent)
                } else {
                    Self::new(accent)
                }
            }
            ThemeMode::CatppuccinLatte => {
                Self::from_catppuccin(catppuccin_egui::LATTE, accent, true)
            }
            ThemeMode::CatppuccinFrappe => {
                Self::from_catppuccin(catppuccin_egui::FRAPPE, accent, false)
            }
            ThemeMode::CatppuccinMacchiato => {
                Self::from_catppuccin(catppuccin_egui::MACCHIATO, accent, false)
            }
            ThemeMode::CatppuccinMocha => {
                Self::from_catppuccin(catppuccin_egui::MOCHA, accent, false)
            }
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
}

impl Default for Theme {
    fn default() -> Self {
        Self::new(AccentColor::Blue)
    }
}
