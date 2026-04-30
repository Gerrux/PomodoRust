use crate::data::todo::QueuedTask;
use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;

/// Queue view action
pub(super) enum QueueViewAction {
    GoBack,
    Remove(i64),
    ClearAll,
    Reorder(Vec<i64>),
}

/// Wrapper for DnD: implements DragDropItem via unique queue id.
#[derive(Hash)]
struct DndQueueItem {
    id: i64,
    todo_id: i64,
    title: String,
    completed_pomodoros: u32,
    planned_pomodoros: u32,
}

impl DndQueueItem {
    fn from_task(task: &QueuedTask) -> Self {
        Self {
            id: task.id,
            todo_id: task.todo_id,
            title: task.title.clone(),
            completed_pomodoros: task.completed_pomodoros,
            planned_pomodoros: task.planned_pomodoros,
        }
    }
}

/// Render the queue page inside the main pomodoro window.
pub(super) fn render_queue_view(
    ui: &mut egui::Ui,
    theme: &Theme,
    queue: &[QueuedTask],
) -> Vec<QueueViewAction> {
    let mut actions = Vec::new();
    let t = crate::i18n::tr();

    // Header
    ui.horizontal(|ui| {
        let (arrow_rect, arrow_resp) =
            ui.allocate_exact_size(egui::vec2(18.0, 18.0), egui::Sense::click());
        let ir = egui::Rect::from_center_size(arrow_rect.center(), egui::vec2(12.0, 12.0));
        draw_icon(ui, Icon::ArrowLeft, ir, theme.text_secondary);
        if arrow_resp.clicked() {
            actions.push(QueueViewAction::GoBack);
        }

        ui.label(
            egui::RichText::new(t.queue.title)
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
                egui::RichText::new(t.queue.empty)
                    .size(16.0)
                    .color(theme.text_muted),
            );
            ui.add_space(theme.spacing_sm);
            ui.label(
                egui::RichText::new(t.queue.empty_hint)
                    .size(13.0)
                    .color(theme.text_muted),
            );
        });
        return actions;
    }

    let mut items: Vec<DndQueueItem> = queue.iter().map(DndQueueItem::from_task).collect();

    let hover_bg = if theme.is_light {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 12)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8)
    };

    let available_height = ui.available_height();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(available_height)
        .show(ui, |ui| {
            let response =
                egui_dnd::dnd(ui, "queue_dnd").show_vec(&mut items, |ui, item, handle, state| {
                    let i = state.index;
                    let is_current = i == 0 && !state.dragged;

                    // Item card frame
                    let item_resp = egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                        .rounding(theme.rounding_sm)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Drag handle
                                handle.ui(ui, |ui| {
                                    let (handle_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(12.0, 18.0),
                                        egui::Sense::hover(),
                                    );
                                    let ir = egui::Rect::from_center_size(
                                        handle_rect.center(),
                                        egui::vec2(10.0, 10.0),
                                    );
                                    draw_icon(
                                        ui,
                                        Icon::GripVertical,
                                        ir,
                                        theme.text_muted.linear_multiply(0.4),
                                    );
                                });

                                // Current indicator
                                if is_current {
                                    let (icon_rect, _) = ui.allocate_exact_size(
                                        egui::vec2(12.0, 18.0),
                                        egui::Sense::hover(),
                                    );
                                    let ir = egui::Rect::from_center_size(
                                        icon_rect.center(),
                                        egui::vec2(9.0, 9.0),
                                    );
                                    draw_icon(ui, Icon::ChevronRight, ir, theme.accent.solid());
                                }

                                // Title
                                let title_color = if is_current {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                };
                                let mut title_text = egui::RichText::new(&item.title)
                                    .size(13.0)
                                    .color(title_color);
                                if is_current {
                                    title_text = title_text.strong();
                                }
                                ui.add(egui::Label::new(title_text).wrap());

                                // Right side: progress + remove
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        // Remove button
                                        let (x_rect, x_resp) = ui.allocate_exact_size(
                                            egui::vec2(16.0, 18.0),
                                            egui::Sense::click(),
                                        );
                                        let x_ir = egui::Rect::from_center_size(
                                            x_rect.center(),
                                            egui::vec2(9.0, 9.0),
                                        );
                                        let x_color = if x_resp.hovered() {
                                            theme.error
                                        } else {
                                            theme.text_muted.linear_multiply(0.5)
                                        };
                                        draw_icon(ui, Icon::X, x_ir, x_color);
                                        if x_resp.clicked() {
                                            actions.push(QueueViewAction::Remove(item.id));
                                        }

                                        // Progress badge
                                        let badge_bg = if theme.is_light {
                                            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 25)
                                        } else {
                                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25)
                                        };
                                        egui::Frame::none()
                                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                                            .rounding(theme.rounding_sm)
                                            .fill(badge_bg)
                                            .show(ui, |ui| {
                                                ui.label(
                                                    egui::RichText::new(format!(
                                                        "{}/{}",
                                                        item.completed_pomodoros,
                                                        item.planned_pomodoros
                                                    ))
                                                    .size(11.0)
                                                    .color(theme.text_secondary),
                                                );
                                            });
                                    },
                                );
                            });
                        });

                    let item_rect = item_resp.response.rect;

                    // Hover bg
                    if ui.rect_contains_pointer(item_rect) {
                        ui.painter()
                            .rect_filled(item_rect, theme.rounding_sm, hover_bg);
                    }

                    // Current item accent stripe
                    if is_current {
                        ui.painter().line_segment(
                            [
                                egui::pos2(item_rect.left() + 1.0, item_rect.top() + 3.0),
                                egui::pos2(item_rect.left() + 1.0, item_rect.bottom() - 3.0),
                            ],
                            egui::Stroke::new(2.5, theme.accent.solid()),
                        );
                    }
                });

            // Reorder
            if response.final_update().is_some() {
                let ids: Vec<i64> = items.iter().map(|item| item.id).collect();
                actions.push(QueueViewAction::Reorder(ids));
            }

            // Clear all
            if queue.len() > 1 {
                ui.add_space(theme.spacing_md);
                ui.separator();
                ui.add_space(theme.spacing_xs);
                let btn = ui.add(
                    egui::Label::new(
                        egui::RichText::new(t.queue.clear)
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
