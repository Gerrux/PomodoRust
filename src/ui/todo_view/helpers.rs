use egui::{self, Color32, RichText};

use crate::data::todo::Priority;
use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;
use super::{TodoAction, TodoView};

impl TodoView {
    pub(super) fn handle_clipboard_paste(
        &mut self,
        ctx: &egui::Context,
        workspace_id: i64,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        let paste_text: Option<String> = ctx.input(|i| {
            i.events.iter().find_map(|e| {
                if let egui::Event::Paste(text) = e {
                    Some(text.clone())
                } else {
                    None
                }
            })
        });

        if let Some(text) = paste_text {
            let text = text.trim().to_string();
            if text.is_empty() {
                return actions;
            }

            let lines: Vec<&str> = text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

            if lines.len() == 1 {
                actions.push(TodoAction::CreateTodo {
                    workspace_id,
                    project_id: None,
                    title: lines[0].to_string(),
                });
            } else {
                let looks_like_list = lines.iter().skip(1).all(|l| {
                    l.starts_with("- ") || l.starts_with("* ") || l.starts_with("- [ ]")
                });

                if looks_like_list {
                    actions.push(TodoAction::CreateTodoWithBody {
                        workspace_id,
                        project_id: None,
                        title: lines[0].to_string(),
                        body: lines[1..].join("\n"),
                    });
                } else {
                    for line in &lines {
                        let title = line
                            .trim_start_matches("- [ ] ")
                            .trim_start_matches("- [x] ")
                            .trim_start_matches("- ")
                            .trim_start_matches("* ")
                            .to_string();
                        actions.push(TodoAction::CreateTodo {
                            workspace_id,
                            project_id: None,
                            title,
                        });
                    }
                }
            }
        }

        actions
    }
}

/// Simple markdown renderer (fallback without egui_commonmark)
pub(super) fn render_markdown_simple(ui: &mut egui::Ui, theme: &Theme, text: &str) {
    for line in text.lines() {
        if line.starts_with("# ") {
            ui.label(
                RichText::new(&line[2..])
                    .size(18.0)
                    .strong()
                    .color(theme.text_primary),
            );
        } else if line.starts_with("## ") {
            ui.label(
                RichText::new(&line[3..])
                    .size(16.0)
                    .strong()
                    .color(theme.text_primary),
            );
        } else if line.starts_with("### ") {
            ui.label(
                RichText::new(&line[4..])
                    .size(14.0)
                    .strong()
                    .color(theme.text_primary),
            );
        } else if line.starts_with("- [x] ") || line.starts_with("- [X] ") {
            ui.horizontal(|ui| {
                let (cb_rect, _) =
                    ui.allocate_exact_size(egui::Vec2::new(14.0, 14.0), egui::Sense::hover());
                let ir = egui::Rect::from_center_size(cb_rect.center(), egui::Vec2::splat(12.0));
                draw_icon(ui, Icon::CheckSquare, ir, theme.success);
                ui.label(
                    RichText::new(&line[6..])
                        .strikethrough()
                        .color(theme.text_muted),
                );
            });
        } else if line.starts_with("- [ ] ") {
            ui.horizontal(|ui| {
                let (cb_rect, _) =
                    ui.allocate_exact_size(egui::Vec2::new(14.0, 14.0), egui::Sense::hover());
                let ir = egui::Rect::from_center_size(cb_rect.center(), egui::Vec2::splat(12.0));
                draw_icon(ui, Icon::Square, ir, theme.text_secondary);
                ui.label(RichText::new(&line[6..]).color(theme.text_primary));
            });
        } else if line.starts_with("- ") || line.starts_with("* ") {
            ui.horizontal(|ui| {
                let dot_center = ui.allocate_exact_size(egui::Vec2::new(14.0, 14.0), egui::Sense::hover()).0;
                ui.painter().circle_filled(dot_center.center(), 2.5, theme.text_muted);
                ui.label(RichText::new(&line[2..]).color(theme.text_primary));
            });
        } else if line.starts_with("> ") {
            egui::Frame::none()
                .inner_margin(egui::Margin {
                    left: 8.0,
                    ..Default::default()
                })
                .stroke(egui::Stroke::new(2.0, theme.border_default))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(&line[2..])
                            .color(theme.text_secondary)
                            .italics(),
                    );
                });
        } else if line.is_empty() {
            ui.add_space(4.0);
        } else {
            // Render inline bold: **text**
            render_inline_formatted(ui, theme, line);
        }
    }
}

/// Render text with basic inline formatting (bold)
fn render_inline_formatted(ui: &mut egui::Ui, theme: &Theme, text: &str) {
    // Simple bold detection: if entire line is **wrapped**, render bold
    if text.starts_with("**") && text.ends_with("**") && text.len() > 4 {
        ui.label(
            RichText::new(&text[2..text.len() - 2])
                .strong()
                .color(theme.text_primary),
        );
    } else {
        ui.label(RichText::new(text).color(theme.text_primary));
    }
}

/// Parse hex color string like "#7C3AED" to Color32
pub(super) fn parse_hex_color(hex: &str) -> Option<Color32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color32::from_rgb(r, g, b))
}

/// Get color for a priority level
pub(super) fn priority_color(priority: Priority, theme: &Theme) -> Color32 {
    match priority {
        Priority::None => theme.text_muted,
        Priority::Low => theme.success,
        Priority::Medium => Color32::from_rgb(59, 130, 246), // blue-500
        Priority::High => theme.warning,
        Priority::Urgent => theme.error,
    }
}

/// Truncate text safely for Cyrillic (by chars, not bytes)
pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        let truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}\u{2026}", truncated)
    }
}
