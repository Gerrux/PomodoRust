use egui::{vec2, Align, FontId, Layout, Ui};

use super::{TimerAction, TimerView};
use crate::core::Session;
use crate::data::todo::QueuedTask;
use crate::ui::components::{AsciiProgressBar, AsciiSpinner, AsciiTime};
use crate::ui::theme::Theme;

impl TimerView {
    /// TUI/Retro style with ASCII art
    pub(super) fn show_tui_style(
        &mut self,
        ui: &mut Ui,
        session: &Session,
        theme: &Theme,
        _pulse: f32,
        current_task: Option<&QueuedTask>,
        queue: &[QueuedTask],
    ) -> Option<TimerAction> {
        let t = crate::i18n::tr();
        let mut action = None;

        let available = ui.available_size();
        let min_dim = available.x.min(available.y);

        // Responsive sizing
        let ascii_font_size = (min_dim * 0.045).clamp(12.0, 20.0);
        let label_font_size = (min_dim * 0.035).clamp(10.0, 16.0);
        let btn_font_size = (min_dim * 0.03).clamp(11.0, 14.0);
        let spacing = (min_dim * 0.03).clamp(8.0, 20.0);

        // Calculate progress bar width based on available space and font size
        // Monospace character width is approximately font_size * 0.6
        let progress_font_size = ascii_font_size * 0.9;
        let char_width = progress_font_size * 0.6;
        let max_chars = ((available.x * 0.85) / char_width) as usize;
        // Subtract 4 for brackets and some padding to prevent wrapping
        let progress_width = max_chars.saturating_sub(4).clamp(15, 40);

        // Use session-based colors (like modern style)
        let (accent_start, accent_end) = theme.session_gradient(session.session_type());
        let accent = Theme::lerp_color(accent_start, accent_end, 0.5);

        // Get time values
        let remaining = session.timer().remaining();
        let minutes = (remaining.as_secs() / 60) as u32;
        let seconds = (remaining.as_secs() % 60) as u32;
        let progress = session.timer().progress();
        let is_running = session.timer().is_running();

        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(spacing * 0.3);

            // Session type with spinner (centered)
            let spinner = if is_running && !theme.reduced_motion {
                AsciiSpinner::braille_frame(self.time_offset)
            } else if is_running {
                "●"
            } else {
                "○"
            };

            let session_label = t.session_label(session.session_type());

            ui.label(
                egui::RichText::new(format!("{} {}", spinner, session_label))
                    .font(FontId::monospace(label_font_size * 1.2))
                    .color(accent),
            );

            ui.add_space(spacing * 0.3);

            // ASCII time display
            AsciiTime::draw(ui, minutes, seconds, accent, ascii_font_size);

            ui.add_space(spacing * 0.5);

            // ASCII progress bar
            let progress_bar = AsciiProgressBar::render_gradient(progress, progress_width);
            ui.label(
                egui::RichText::new(&progress_bar)
                    .font(FontId::monospace(ascii_font_size * 0.9))
                    .color(accent),
            );

            // Progress percentage
            ui.label(
                egui::RichText::new(format!("{:>3.0}%", progress * 100.0))
                    .font(FontId::monospace(label_font_size))
                    .color(theme.text_muted),
            );

            ui.add_space(spacing * 0.5);

            // ASCII session dots with colors - centered using LayoutJob
            let total = session.total_sessions_in_cycle();
            let current_idx = session.current_session_in_cycle().saturating_sub(1);

            let mut job = egui::text::LayoutJob::default();
            let font_id = FontId::monospace(label_font_size);

            for i in 0..total {
                let is_completed = i < current_idx;
                let is_current_dot = i == current_idx;

                let (symbol, color) = if is_completed {
                    ("●", theme.success)
                } else if is_current_dot {
                    ("○", accent)
                } else {
                    ("○", theme.border_default)
                };

                job.append(
                    symbol,
                    0.0,
                    egui::TextFormat {
                        font_id: font_id.clone(),
                        color,
                        ..Default::default()
                    },
                );

                // Add space between dots (except after last)
                if i < total - 1 {
                    job.append(
                        " ",
                        0.0,
                        egui::TextFormat {
                            font_id: font_id.clone(),
                            color: egui::Color32::TRANSPARENT,
                            ..Default::default()
                        },
                    );
                }
            }

            job.halign = egui::Align::Center;
            ui.label(job);

            // Session counter
            ui.label(
                egui::RichText::new(format!(
                    "{} {}/{}",
                    t.timer.session,
                    session.current_session_in_cycle(),
                    session.total_sessions_in_cycle()
                ))
                .font(FontId::monospace(label_font_size * 0.9))
                .color(theme.text_muted),
            );

            ui.add_space(spacing * 0.5);

            // ASCII control buttons - centered with spacing in the middle
            let btn_height = btn_font_size + 16.0;
            let gray = theme.text_muted;
            let half_width = ui.available_width() / 2.0;
            let btn_gap = spacing * 0.5;

            ui.horizontal(|ui| {
                // Left half - button aligned to right edge
                ui.allocate_ui_with_layout(
                    vec2(half_width - btn_gap, btn_height),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        let play_text = if is_running {
                            format!("[ \u{25A0} {} ]", t.timer.pause)
                        } else {
                            format!("[ \u{25BA} {} ]", t.timer.start)
                        };

                        let play_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new(&play_text)
                                    .font(FontId::monospace(btn_font_size))
                                    .color(gray),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, gray)),
                        );

                        if play_btn.hovered() || play_btn.has_focus() {
                            let color = if is_running { theme.warning } else { accent };
                            let rect = play_btn.rect;
                            ui.painter().rect_filled(
                                rect,
                                egui::Rounding::same(2.0),
                                theme.bg_tertiary,
                            );
                            ui.painter().rect_stroke(
                                rect,
                                egui::Rounding::same(2.0),
                                egui::Stroke::new(1.0, color),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &play_text,
                                FontId::monospace(btn_font_size),
                                color,
                            );
                        }

                        if play_btn.clicked() {
                            action = Some(TimerAction::Toggle);
                        }
                    },
                );

                // Right half - button aligned to left edge
                ui.allocate_ui_with_layout(
                    vec2(half_width - btn_gap, btn_height),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        let skip_text = format!("[ \u{00BB} {} ]", t.timer.skip);
                        let skip_btn = ui.add(
                            egui::Button::new(
                                egui::RichText::new(&skip_text)
                                    .font(FontId::monospace(btn_font_size))
                                    .color(gray),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, gray)),
                        );

                        if skip_btn.hovered() || skip_btn.has_focus() {
                            let rect = skip_btn.rect;
                            ui.painter().rect_filled(
                                rect,
                                egui::Rounding::same(2.0),
                                theme.bg_tertiary,
                            );
                            ui.painter().rect_stroke(
                                rect,
                                egui::Rounding::same(2.0),
                                egui::Stroke::new(1.0, accent),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                &skip_text,
                                FontId::monospace(btn_font_size),
                                accent,
                            );
                        }

                        if skip_btn.clicked() {
                            action = Some(TimerAction::Skip);
                        }
                    },
                );
            });

            ui.add_space(spacing * 0.3);

            // Current task display
            if let Some(task) = current_task {
                ui.label(
                    egui::RichText::new(format!(
                        "> {} [{}/{}]",
                        crate::ui::todo_view::truncate_text(&task.title, 18),
                        task.completed_pomodoros,
                        task.planned_pomodoros,
                    ))
                    .font(FontId::monospace(btn_font_size * 0.9))
                    .color(theme.text_secondary),
                );
            }

            // Navigation buttons - fade in/out on hover
            let is_hovered = ui.ctx().input(|i| {
                i.pointer.hover_pos()
                    .map(|pos| ui.max_rect().contains(pos))
                    .unwrap_or(false)
            });
            let nav_alpha = ui.ctx().animate_bool_with_time(
                ui.id().with("tui_nav_fade"),
                is_hovered,
                0.25,
            );

            if nav_alpha > 0.0 {
                let fade = |c: egui::Color32| -> egui::Color32 {
                    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (c.a() as f32 * nav_alpha) as u8)
                };

                ui.add_space(spacing * 0.3 * nav_alpha);

                // Separator
                ui.label(
                    egui::RichText::new("─────────────────────")
                        .font(FontId::monospace(btn_font_size))
                        .color(fade(theme.border_subtle)),
                );

                ui.add_space(spacing * 0.3 * nav_alpha);

                let nav_half_width = ui.available_width() / 2.0;
                let nav_btn_gap = spacing * 0.25;

                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_btn_gap, btn_height),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            let dash_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(format!("[ {} ]", t.nav.statistics))
                                        .font(FontId::monospace(btn_font_size))
                                        .color(fade(theme.text_secondary)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            );
                            if dash_btn.clicked() {
                                action = Some(TimerAction::OpenStats);
                            }
                        },
                    );

                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_btn_gap, btn_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let settings_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(format!("[ {} ]", t.nav.settings))
                                        .font(FontId::monospace(btn_font_size))
                                        .color(fade(theme.text_secondary)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            );
                            if settings_btn.clicked() {
                                action = Some(TimerAction::OpenSettings);
                            }
                        },
                    );
                });

                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_btn_gap, btn_height),
                        Layout::right_to_left(Align::Center),
                        |ui| {
                            let todo_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(format!("[ {} ]", t.nav.tasks))
                                        .font(FontId::monospace(btn_font_size))
                                        .color(fade(theme.text_secondary)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            );
                            if todo_btn.clicked() {
                                action = Some(TimerAction::OpenTodo);
                            }
                        },
                    );

                    ui.allocate_ui_with_layout(
                        vec2(nav_half_width - nav_btn_gap, btn_height),
                        Layout::left_to_right(Align::Center),
                        |ui| {
                            let queue_text = if queue.is_empty() {
                                format!("[ {} ]", t.nav.queue)
                            } else {
                                format!("[ {} {} ]", t.nav.queue, queue.len())
                            };
                            let queue_btn = ui.add(
                                egui::Button::new(
                                    egui::RichText::new(&queue_text)
                                        .font(FontId::monospace(btn_font_size))
                                        .color(fade(theme.text_secondary)),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .stroke(egui::Stroke::NONE),
                            );
                            if queue_btn.clicked() {
                                action = Some(TimerAction::OpenQueue);
                            }
                        },
                    );
                });

                ui.add_space(spacing * 0.3 * nav_alpha);

                // Request repaint during animation
                if nav_alpha < 1.0 {
                    ui.ctx().request_repaint();
                }
            }
        });

        // Request continuous repaint for animations
        if is_running {
            ui.ctx().request_repaint();
        }

        action
    }
}
