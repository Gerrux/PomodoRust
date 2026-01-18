//! ASCII art components for TUI-style interface

use egui::{Color32, FontId, Ui};

/// Pixel-art style digits using full blocks (5 lines tall, 4 chars wide)
/// Using ░ for empty space to ensure consistent width
const PIXEL_DIGITS: [&[&str]; 10] = [
    // 0
    &["████", "█░░█", "█░░█", "█░░█", "████"],
    // 1
    &["░░█░", "░██░", "░░█░", "░░█░", "░███"],
    // 2
    &["████", "░░░█", "████", "█░░░", "████"],
    // 3
    &["████", "░░░█", "████", "░░░█", "████"],
    // 4
    &["█░░█", "█░░█", "████", "░░░█", "░░░█"],
    // 5
    &["████", "█░░░", "████", "░░░█", "████"],
    // 6
    &["████", "█░░░", "████", "█░░█", "████"],
    // 7
    &["████", "░░░█", "░░█░", "░░█░", "░░█░"],
    // 8
    &["████", "█░░█", "████", "█░░█", "████"],
    // 9
    &["████", "█░░█", "████", "░░░█", "████"],
];

/// Pixel colon separator for time
const PIXEL_COLON: &[&str] = &["░░░", "░█░", "░░░", "░█░", "░░░"];

/// ASCII progress bar characters
pub struct AsciiProgressBar;

impl AsciiProgressBar {
    /// Render a horizontal progress bar
    /// Returns the string representation
    pub fn render(progress: f32, width: usize) -> String {
        let filled = ((progress * width as f32) as usize).min(width);
        let empty = width - filled;

        let mut bar = String::with_capacity(width + 2);
        bar.push('▐');
        for _ in 0..filled {
            bar.push('█');
        }
        for _ in 0..empty {
            bar.push('░');
        }
        bar.push('▌');
        bar
    }

    /// Render with gradient blocks
    pub fn render_gradient(progress: f32, width: usize) -> String {
        let filled_full = ((progress * width as f32) as usize).min(width);
        let fraction = (progress * width as f32) - filled_full as f32;

        let mut bar = String::with_capacity(width + 2);
        bar.push('[');

        for i in 0..width {
            if i < filled_full {
                bar.push('█');
            } else if i == filled_full && fraction > 0.0 {
                let partial = if fraction > 0.75 {
                    '▓'
                } else if fraction > 0.5 {
                    '▒'
                } else if fraction > 0.25 {
                    '░'
                } else {
                    '·'
                };
                bar.push(partial);
            } else {
                bar.push('·');
            }
        }

        bar.push(']');
        bar
    }

    /// Draw the progress bar in UI
    pub fn draw(ui: &mut Ui, progress: f32, width: usize, color: Color32, font_size: f32) {
        let bar_text = Self::render_gradient(progress, width);
        ui.label(
            egui::RichText::new(bar_text)
                .font(FontId::monospace(font_size))
                .color(color),
        );
    }
}

/// Pixel-art time display renderer
pub struct AsciiTime;

impl AsciiTime {
    /// Get the height in lines of digits
    pub const HEIGHT: usize = 5;

    /// Render time as MM:SS in pixel art
    /// Returns 5 lines of text
    pub fn render(minutes: u32, seconds: u32) -> Vec<String> {
        let m1 = (minutes / 10) as usize % 10;
        let m2 = (minutes % 10) as usize;
        let s1 = (seconds / 10) as usize % 10;
        let s2 = (seconds % 10) as usize;

        let mut lines = Vec::with_capacity(Self::HEIGHT);

        for row in 0..Self::HEIGHT {
            let mut line = String::new();
            line.push_str(PIXEL_DIGITS[m1][row]);
            line.push(' ');
            line.push_str(PIXEL_DIGITS[m2][row]);
            line.push_str(PIXEL_COLON[row]);
            line.push_str(PIXEL_DIGITS[s1][row]);
            line.push(' ');
            line.push_str(PIXEL_DIGITS[s2][row]);
            lines.push(line);
        }

        lines
    }

    /// Draw pixel time in UI (centered, no line gaps)
    /// ░ characters are drawn invisible, █ with full color
    pub fn draw(ui: &mut Ui, minutes: u32, seconds: u32, color: Color32, font_size: f32) {
        let lines = Self::render(minutes, seconds);
        let dim_color = Color32::TRANSPARENT;

        // Calculate width for centering (each line has same char count)
        let char_count = lines.first().map(|l| l.chars().count()).unwrap_or(0);
        let approx_width = char_count as f32 * font_size * 0.6;

        ui.vertical_centered(|ui| {
            // Remove spacing between lines for solid pixel look
            ui.spacing_mut().item_spacing.y = 0.0;

            for line in lines {
                // Center each line using allocate_ui
                ui.allocate_ui_with_layout(
                    egui::vec2(approx_width, font_size * 1.1),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        for ch in line.chars() {
                            let (display_char, char_color) = if ch == '░' {
                                ('█', dim_color)
                            } else {
                                (ch, color)
                            };
                            ui.label(
                                egui::RichText::new(display_char.to_string())
                                    .font(FontId::monospace(font_size))
                                    .color(char_color),
                            );
                        }
                    },
                );
            }
        });
    }
}

/// Box drawing helper for TUI-style borders
pub struct AsciiBox;

impl AsciiBox {
    /// Box drawing characters
    pub const TOP_LEFT: char = '┌';
    pub const TOP_RIGHT: char = '┐';
    pub const BOTTOM_LEFT: char = '└';
    pub const BOTTOM_RIGHT: char = '┘';
    pub const HORIZONTAL: char = '─';
    pub const VERTICAL: char = '│';
    pub const T_DOWN: char = '┬';
    pub const T_UP: char = '┴';
    pub const T_RIGHT: char = '├';
    pub const T_LEFT: char = '┤';
    pub const CROSS: char = '┼';

    /// Double line variants
    pub const DOUBLE_HORIZONTAL: char = '═';
    pub const DOUBLE_VERTICAL: char = '║';
    pub const DOUBLE_TOP_LEFT: char = '╔';
    pub const DOUBLE_TOP_RIGHT: char = '╗';
    pub const DOUBLE_BOTTOM_LEFT: char = '╚';
    pub const DOUBLE_BOTTOM_RIGHT: char = '╝';

    /// Render a box top line
    pub fn top_line(width: usize) -> String {
        let mut s = String::with_capacity(width + 2);
        s.push(Self::TOP_LEFT);
        for _ in 0..width {
            s.push(Self::HORIZONTAL);
        }
        s.push(Self::TOP_RIGHT);
        s
    }

    /// Render a box bottom line
    pub fn bottom_line(width: usize) -> String {
        let mut s = String::with_capacity(width + 2);
        s.push(Self::BOTTOM_LEFT);
        for _ in 0..width {
            s.push(Self::HORIZONTAL);
        }
        s.push(Self::BOTTOM_RIGHT);
        s
    }

    /// Render a box middle line with content
    pub fn middle_line(content: &str, width: usize) -> String {
        let content_len = content.chars().count();
        let padding = width.saturating_sub(content_len);
        let mut s = String::with_capacity(width + 2);
        s.push(Self::VERTICAL);
        s.push_str(content);
        for _ in 0..padding {
            s.push(' ');
        }
        s.push(Self::VERTICAL);
        s
    }

    /// Draw a complete box with title
    pub fn draw_titled(ui: &mut Ui, title: &str, width: usize, color: Color32, font_size: f32) {
        let title_line = format!(
            "{}{}{}{}",
            Self::TOP_LEFT,
            Self::HORIZONTAL,
            title,
            Self::HORIZONTAL
                .to_string()
                .repeat(width.saturating_sub(title.len() + 1))
        ) + &Self::TOP_RIGHT.to_string();

        ui.label(
            egui::RichText::new(title_line)
                .font(FontId::monospace(font_size))
                .color(color),
        );
    }
}

/// ASCII spinner animation frames
pub struct AsciiSpinner;

impl AsciiSpinner {
    const FRAMES: &'static [&'static str] = &["◐", "◓", "◑", "◒"];
    const BRAILLE_FRAMES: &'static [&'static str] =
        &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

    /// Get spinner frame based on time
    pub fn frame(t: f32) -> &'static str {
        let idx = ((t * 8.0) as usize) % Self::FRAMES.len();
        Self::FRAMES[idx]
    }

    /// Get braille spinner frame
    pub fn braille_frame(t: f32) -> &'static str {
        let idx = ((t * 10.0) as usize) % Self::BRAILLE_FRAMES.len();
        Self::BRAILLE_FRAMES[idx]
    }
}

/// ASCII tomato art for pomodoro
pub const ASCII_TOMATO: &[&str] = &[
    "    \\|/    ",
    "   .-^-.   ",
    "  /     \\  ",
    " |  ___  | ",
    " | |   | | ",
    "  \\_____/  ",
];

/// Small ASCII tomato
pub const ASCII_TOMATO_SMALL: &[&str] = &["  ~  ", " /-\\ ", "|   |", " \\_/ "];

/// Session indicator dots
pub struct AsciiSessionDots;

impl AsciiSessionDots {
    pub fn render(current: u32, total: u32) -> String {
        let mut s = String::new();
        for i in 0..total {
            if i < current {
                s.push('●');
            } else {
                s.push('○');
            }
            if i < total - 1 {
                s.push(' ');
            }
        }
        s
    }
}
