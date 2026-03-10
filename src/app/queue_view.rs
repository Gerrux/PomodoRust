use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;
use crate::data::todo::QueuedTask;

/// Queue view action
pub(super) enum QueueViewAction {
    GoBack,
    Remove(i64),
    ClearAll,
    Reorder(Vec<i64>),
}

/// Render the queue page inside the main pomodoro window.
pub(super) fn render_queue_view(
    ui: &mut egui::Ui,
    theme: &Theme,
    queue: &[QueuedTask],
) -> Vec<QueueViewAction> {
    let mut actions = Vec::new();

    // Back button
    ui.horizontal(|ui| {
        let (arrow_rect, arrow_resp) =
            ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
        let ir = egui::Rect::from_center_size(arrow_rect.center(), egui::vec2(12.0, 12.0));
        draw_icon(ui, Icon::ArrowLeft, ir, theme.text_secondary);
        if arrow_resp.clicked() {
            actions.push(QueueViewAction::GoBack);
        }

        ui.label(
            egui::RichText::new("Очередь")
                .size(14.0)
                .strong()
                .color(theme.text_primary),
        );

        if !queue.is_empty() {
            let total: u32 = queue
                .iter()
                .map(|t| t.planned_pomodoros.saturating_sub(t.completed_pomodoros))
                .sum();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{} pom.", total))
                        .size(12.0)
                        .color(theme.text_muted),
                );
            });
        }
    });

    ui.add_space(theme.spacing_sm);

    if queue.is_empty() {
        ui.add_space(theme.spacing_xl);
        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new("Очередь пуста")
                    .size(16.0)
                    .color(theme.text_muted),
            );
            ui.add_space(theme.spacing_sm);
            ui.label(
                egui::RichText::new("Добавляйте задачи через меню \u{22EE} в списке задач")
                    .size(13.0)
                    .color(theme.text_muted),
            );
        });
        return actions;
    }

    // Drag & drop state
    let dnd_id = ui.id().with("queue_dnd");
    let dragged_idx: Option<usize> = ui.data(|d| d.get_temp(dnd_id));
    let mut drop_target_idx: Option<usize> = None;

    let available_height = ui.available_height();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(available_height)
        .show(ui, |ui| {
            for (i, task) in queue.iter().enumerate() {
                let _item_id = ui.id().with(("queue_item", task.id));
                let is_being_dragged = dragged_idx == Some(i);

                // Drag handle + row
                let row_resp = ui.scope(|ui| {
                    // Make the whole row semi-transparent when being dragged
                    if is_being_dragged {
                        ui.set_opacity(0.4);
                    }

                    // Top row: drag handle, indicator, progress, remove button
                    ui.horizontal(|ui| {
                        // Drag handle
                        let (handle_rect, handle_resp) = ui.allocate_exact_size(
                            egui::vec2(14.0, 18.0),
                            egui::Sense::drag(),
                        );
                        let handle_color = if handle_resp.hovered() || handle_resp.dragged() {
                            theme.text_primary
                        } else {
                            theme.text_muted
                        };
                        ui.painter().text(
                            handle_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "⠿",
                            egui::FontId::proportional(12.0),
                            handle_color,
                        );

                        if handle_resp.drag_started() {
                            ui.data_mut(|d| d.insert_temp(dnd_id, i));
                        }

                        // Current indicator
                        let (icon_rect, _) =
                            ui.allocate_exact_size(egui::vec2(14.0, 18.0), egui::Sense::hover());
                        if i == 0 && !is_being_dragged {
                            let ir = egui::Rect::from_center_size(
                                icon_rect.center(),
                                egui::vec2(10.0, 10.0),
                            );
                            draw_icon(ui, Icon::ChevronRight, ir, theme.accent.solid());
                        }

                        // Right side: progress + remove
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                let (x_rect, x_resp) = ui.allocate_exact_size(
                                    egui::vec2(18.0, 18.0),
                                    egui::Sense::click(),
                                );
                                let x_ir = egui::Rect::from_center_size(
                                    x_rect.center(),
                                    egui::vec2(10.0, 10.0),
                                );
                                let x_color = if x_resp.hovered() {
                                    theme.error
                                } else {
                                    theme.text_muted
                                };
                                draw_icon(ui, Icon::X, x_ir, x_color);
                                if x_resp.clicked() {
                                    actions.push(QueueViewAction::Remove(task.id));
                                }

                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}/{}",
                                        task.completed_pomodoros, task.planned_pomodoros
                                    ))
                                    .size(12.0)
                                    .color(theme.text_muted),
                                );
                            },
                        );
                    });

                    // Title below, with left indent matching handle+icon width
                    ui.horizontal(|ui| {
                        ui.add_space(28.0); // 14 + 14 for handle + icon
                        let color = if i == 0 {
                            theme.text_primary
                        } else {
                            theme.text_secondary
                        };
                        let mut title_text =
                            egui::RichText::new(&task.title).size(13.0).color(color);
                        if i == 0 {
                            title_text = title_text.strong();
                        }
                        ui.add(egui::Label::new(title_text).wrap());
                    });
                });

                let row_rect = row_resp.response.rect;

                // Drop target detection
                if dragged_idx.is_some() && dragged_idx != Some(i) {
                    if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                        if row_rect.contains(pointer) {
                            drop_target_idx = Some(i);
                            // Draw drop indicator line
                            let line_y = if pointer.y < row_rect.center().y {
                                row_rect.top()
                            } else {
                                row_rect.bottom()
                            };
                            ui.painter().line_segment(
                                [
                                    egui::pos2(row_rect.left(), line_y),
                                    egui::pos2(row_rect.right(), line_y),
                                ],
                                egui::Stroke::new(2.0, theme.accent.solid()),
                            );
                        }
                    }
                }

                // Hover bg (only when not dragging)
                if dragged_idx.is_none() && ui.rect_contains_pointer(row_rect) {
                    let hover_bg = if theme.is_light {
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 10)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8)
                    };
                    ui.painter()
                        .rect_filled(row_rect, theme.rounding_sm, hover_bg);
                }
            }

            // Handle drop
            if let Some(from) = dragged_idx {
                if !ui.ctx().input(|i| i.pointer.any_down()) {
                    // Drag ended
                    ui.data_mut(|d| d.remove_temp::<usize>(dnd_id));
                    if let Some(to) = drop_target_idx {
                        if from != to {
                            let mut ids: Vec<i64> = queue.iter().map(|t| t.id).collect();
                            let item = ids.remove(from);
                            ids.insert(if to > from { to } else { to }, item);
                            actions.push(QueueViewAction::Reorder(ids));
                        }
                    }
                }
            }

            // Clear all
            if queue.len() > 1 {
                ui.add_space(theme.spacing_md);
                ui.separator();
                ui.add_space(theme.spacing_xs);
                let btn = ui.add(
                    egui::Label::new(
                        egui::RichText::new("Очистить очередь")
                            .size(12.0)
                            .color(theme.error),
                    )
                    .sense(egui::Sense::click()),
                );
                if btn.clicked() {
                    actions.push(QueueViewAction::ClearAll);
                }
            }
        });

    actions
}
