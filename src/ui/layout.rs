//! Layout constants and responsive sizing utilities
//!
//! Centralizes UI dimension calculations for consistent layout across views.

use egui::Vec2;

/// Constants for responsive layout calculations
pub mod responsive {
    /// Timer view sizing constants
    pub mod timer {
        /// Timer radius as fraction of minimum dimension
        pub const RADIUS_RATIO: f32 = 0.28;
        /// Minimum timer radius
        pub const RADIUS_MIN: f32 = 60.0;
        /// Maximum timer radius
        pub const RADIUS_MAX: f32 = 120.0;

        /// Timer ring thickness as fraction of radius
        pub const THICKNESS_RATIO: f32 = 0.08;
        pub const THICKNESS_MIN: f32 = 4.0;
        pub const THICKNESS_MAX: f32 = 10.0;

        /// Timer font size as fraction of radius
        pub const FONT_RATIO: f32 = 0.55;
        pub const FONT_MIN: f32 = 28.0;
        pub const FONT_MAX: f32 = 64.0;

        /// Label font size as fraction of radius
        pub const LABEL_RATIO: f32 = 0.16;
        pub const LABEL_MIN: f32 = 11.0;
        pub const LABEL_MAX: f32 = 18.0;

        /// Control button size as fraction of minimum dimension
        pub const CONTROL_BTN_RATIO: f32 = 0.11;
        pub const CONTROL_BTN_MIN: f32 = 36.0;
        pub const CONTROL_BTN_MAX: f32 = 48.0;

        /// Navigation button width as fraction of available width
        pub const NAV_BTN_WIDTH_RATIO: f32 = 0.35;
        pub const NAV_BTN_WIDTH_MIN: f32 = 100.0;
        pub const NAV_BTN_WIDTH_MAX: f32 = 150.0;

        /// Navigation button height as fraction of minimum dimension
        pub const NAV_BTN_HEIGHT_RATIO: f32 = 0.09;
        pub const NAV_BTN_HEIGHT_MIN: f32 = 32.0;
        pub const NAV_BTN_HEIGHT_MAX: f32 = 44.0;

        /// General spacing as fraction of minimum dimension
        pub const SPACING_RATIO: f32 = 0.04;
        pub const SPACING_MIN: f32 = 8.0;
        pub const SPACING_MAX: f32 = 24.0;

        /// Icon scale within buttons
        pub const ICON_SCALE: f32 = 0.45;
    }

    /// Session dots sizing constants
    pub mod dots {
        pub const RADIUS_RATIO: f32 = 0.015;
        pub const RADIUS_MIN: f32 = 4.0;
        pub const RADIUS_MAX: f32 = 7.0;

        pub const SPACING_RATIO: f32 = 0.04;
        pub const SPACING_MIN: f32 = 12.0;
        pub const SPACING_MAX: f32 = 20.0;

        pub const CAPTION_RATIO: f32 = 0.035;
        pub const CAPTION_MIN: f32 = 10.0;
        pub const CAPTION_MAX: f32 = 14.0;
    }

    /// TUI/Retro style sizing constants
    pub mod tui {
        /// ASCII font size as fraction of minimum dimension
        pub const ASCII_FONT_RATIO: f32 = 0.045;
        pub const ASCII_FONT_MIN: f32 = 12.0;
        pub const ASCII_FONT_MAX: f32 = 20.0;

        /// Label font size as fraction of minimum dimension
        pub const LABEL_RATIO: f32 = 0.035;
        pub const LABEL_MIN: f32 = 10.0;
        pub const LABEL_MAX: f32 = 16.0;

        /// Button font size as fraction of minimum dimension
        pub const BTN_FONT_RATIO: f32 = 0.03;
        pub const BTN_FONT_MIN: f32 = 11.0;
        pub const BTN_FONT_MAX: f32 = 14.0;

        /// Spacing as fraction of minimum dimension
        pub const SPACING_RATIO: f32 = 0.03;
        pub const SPACING_MIN: f32 = 8.0;
        pub const SPACING_MAX: f32 = 20.0;

        /// Minimum height to show navigation buttons
        pub const NAV_MIN_HEIGHT: f32 = 350.0;

        /// Progress bar character ratios
        pub const PROGRESS_FONT_RATIO: f32 = 0.9;
        pub const PROGRESS_WIDTH_RATIO: f32 = 0.85;
        pub const PROGRESS_MIN_CHARS: usize = 15;
        pub const PROGRESS_MAX_CHARS: usize = 40;
        /// Characters subtracted for brackets and padding
        pub const PROGRESS_PADDING_CHARS: usize = 4;
    }

    /// Stats view sizing constants
    pub mod stats {
        pub const CARD_SPACING_RATIO: f32 = 0.02;
        pub const CARD_SPACING_MIN: f32 = 8.0;
        pub const CARD_SPACING_MAX: f32 = 16.0;

        pub const HEADER_FONT_RATIO: f32 = 0.05;
        pub const HEADER_FONT_MIN: f32 = 18.0;
        pub const HEADER_FONT_MAX: f32 = 28.0;
    }

    /// Settings view sizing constants
    pub mod settings {
        /// Icon button size
        pub const BACK_BTN_SIZE: f32 = 32.0;
        pub const BACK_ICON_SCALE: f32 = 0.5;

        /// Volume slider dimensions
        pub const VOLUME_SLIDER_WIDTH: f32 = 120.0;
        pub const VOLUME_SLIDER_HEIGHT: f32 = 20.0;

        /// Color picker dot sizes
        pub const COLOR_DOT_NORMAL: f32 = 22.0;
        pub const COLOR_DOT_SELECTED: f32 = 26.0;
        pub const COLOR_DOT_SPACING: f32 = 6.0;

        /// +/- button dimensions
        pub const STEPPER_BTN_SIZE: f32 = 32.0;
        pub const STEPPER_VALUE_WIDTH: f32 = 60.0;
        pub const STEPPER_ICON_SIZE: f32 = 14.0;

        /// Preset button height
        pub const PRESET_BTN_HEIGHT: f32 = 48.0;

        /// Reset button centering width
        pub const RESET_BTN_WIDTH: f32 = 150.0;
    }
}

/// Responsive size calculation utilities
pub struct ResponsiveSize;

impl ResponsiveSize {
    /// Calculate a responsive value clamped between min and max
    #[inline]
    pub fn calc(base: f32, ratio: f32, min: f32, max: f32) -> f32 {
        (base * ratio).clamp(min, max)
    }

    /// Calculate the minimum dimension from available size
    #[inline]
    pub fn min_dim(available: Vec2) -> f32 {
        available.x.min(available.y)
    }

    /// Calculate character width for monospace font
    #[inline]
    pub fn monospace_char_width(font_size: f32) -> f32 {
        font_size * 0.6
    }

    /// Calculate progress bar width in characters based on available space
    pub fn progress_bar_chars(available_width: f32, font_size: f32) -> usize {
        use responsive::tui::*;

        let char_width = Self::monospace_char_width(font_size);
        let max_chars = ((available_width * PROGRESS_WIDTH_RATIO) / char_width) as usize;
        max_chars
            .saturating_sub(PROGRESS_PADDING_CHARS)
            .clamp(PROGRESS_MIN_CHARS, PROGRESS_MAX_CHARS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_responsive_calc() {
        assert_eq!(ResponsiveSize::calc(100.0, 0.5, 30.0, 70.0), 50.0);
        assert_eq!(ResponsiveSize::calc(100.0, 0.2, 30.0, 70.0), 30.0); // clamped to min
        assert_eq!(ResponsiveSize::calc(100.0, 0.8, 30.0, 70.0), 70.0); // clamped to max
    }

    #[test]
    fn test_min_dim() {
        assert_eq!(ResponsiveSize::min_dim(Vec2::new(100.0, 200.0)), 100.0);
        assert_eq!(ResponsiveSize::min_dim(Vec2::new(300.0, 150.0)), 150.0);
    }
}
