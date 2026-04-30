use egui::{self, RichText, Sense, Vec2};

use super::helpers::{parse_hex_color, priority_color, render_markdown_simple};
use super::{TodoAction, TodoView};
use crate::data::todo::{Priority, Project, TodoItem};
use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;

/// Shared hover background color
fn hover_bg(theme: &Theme) -> egui::Color32 {
    if theme.is_light {
        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 12)
    } else {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 8)
    }
}

impl TodoView {
    pub(super) fn render_project(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        project: &Project,
        todos: &[TodoItem],
        all_projects: &[Project],
        show_completed: bool,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();
        let project_todos: Vec<&TodoItem> = todos
            .iter()
            .filter(|t| t.project_id == Some(project.id))
            .filter(|t| show_completed || !t.completed)
            .collect();

        let active_count = project_todos.iter().filter(|t| !t.completed).count();

        ui.add_space(6.0);

        // Project header with hover background
        let header_response = egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(4.0, 3.0))
            .rounding(theme.rounding_sm)
            .show(ui, |ui| {
                let resp = ui.horizontal(|ui| {
                    // Collapse arrow
                    let arrow_icon = if project.collapsed {
                        Icon::ChevronRight
                    } else {
                        Icon::ChevronDown
                    };
                    let (arrow_rect, arrow_resp) =
                        ui.allocate_exact_size(Vec2::new(16.0, 20.0), Sense::click());
                    let icon_rect =
                        egui::Rect::from_center_size(arrow_rect.center(), Vec2::splat(11.0));
                    draw_icon(ui, arrow_icon, icon_rect, theme.text_secondary);
                    if arrow_resp.clicked() {
                        actions.push(TodoAction::ToggleProjectCollapse { id: project.id });
                    }

                    // Color dot
                    if let Some(color_str) = &project.color {
                        if let Some(color) = parse_hex_color(color_str) {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(Vec2::new(8.0, 8.0), Sense::hover());
                            ui.painter().circle_filled(dot_rect.center(), 4.0, color);
                            ui.add_space(2.0);
                        }
                    }

                    // Renaming
                    if self.renaming_project_id == Some(project.id) {
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.rename_project_buffer)
                                .desired_width(140.0)
                                .font(egui::FontId::proportional(14.0)),
                        );
                        if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let name = self.rename_project_buffer.trim().to_string();
                            if !name.is_empty() {
                                actions.push(TodoAction::RenameProject {
                                    id: project.id,
                                    name,
                                });
                            }
                            self.renaming_project_id = None;
                        }
                    } else {
                        let name_resp = ui.add(
                            egui::Label::new(
                                RichText::new(&project.name)
                                    .size(14.0)
                                    .color(theme.text_primary)
                                    .strong(),
                            )
                            .sense(Sense::click()),
                        );
                        if name_resp.clicked() {
                            actions.push(TodoAction::ToggleProjectCollapse { id: project.id });
                        }
                    }

                    // Active count badge
                    if active_count > 0 {
                        let badge_text = format!("{}", active_count);
                        let badge_bg = if theme.is_light {
                            egui::Color32::from_rgba_unmultiplied(0, 0, 0, 25)
                        } else {
                            egui::Color32::from_rgba_unmultiplied(255, 255, 255, 25)
                        };
                        egui::Frame::none()
                            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                            .rounding(theme.rounding_full)
                            .fill(badge_bg)
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(badge_text)
                                        .size(11.0)
                                        .color(theme.text_secondary),
                                );
                            });
                    }

                    // Push dots menu to the right (popup rendered outside frame)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let (dots_rect, dots_resp) =
                            ui.allocate_exact_size(Vec2::new(18.0, 20.0), Sense::click());
                        let dots_icon_rect =
                            egui::Rect::from_center_size(dots_rect.center(), Vec2::splat(13.0));
                        let dots_color = if dots_resp.hovered() {
                            theme.text_primary
                        } else {
                            theme.text_muted
                        };
                        draw_icon(ui, Icon::MoreVertical, dots_icon_rect, dots_color);
                        if dots_resp.clicked() {
                            if self.popup_todo_id == Some(-project.id) {
                                self.popup_todo_id = None;
                                self.popup_dots_rect = None;
                            } else {
                                self.popup_todo_id = Some(-project.id);
                                self.popup_dots_rect = Some(dots_rect);
                            }
                        }
                    });
                });
                resp
            });

        // Hover bg on project header
        let header_rect = header_response.response.rect;
        if ui.rect_contains_pointer(header_rect) {
            ui.painter()
                .rect_filled(header_rect, theme.rounding_sm, hover_bg(theme));
        }

        // Project dots popup (rendered outside header frame to avoid blocking clicks)
        if self.popup_todo_id == Some(-project.id) {
            if let Some(anchor) = self.popup_dots_rect {
                let area_id = egui::Id::new("project_popup").with(project.id);
                let popup_pos = anchor.left_bottom() + egui::vec2(-140.0, 4.0);

                let area_resp = egui::Area::new(area_id)
                    .order(egui::Order::Foreground)
                    .fixed_pos(popup_pos)
                    .show(ui.ctx(), |ui| {
                        egui::Frame::popup(ui.style())
                            .fill(theme.bg_secondary)
                            .stroke(egui::Stroke::new(1.0, theme.border_subtle))
                            .rounding(theme.rounding_md)
                            .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                            .show(ui, |ui| {
                                ui.set_min_width(160.0);
                                if ui.button(crate::i18n::tr().todo.rename).clicked() {
                                    self.renaming_project_id = Some(project.id);
                                    self.rename_project_buffer = project.name.clone();
                                    self.popup_todo_id = None;
                                }
                                let total_in_project = todos
                                    .iter()
                                    .filter(|t| t.project_id == Some(project.id))
                                    .count();
                                let delete_label = if total_in_project > 0 {
                                    crate::i18n::tr()
                                        .todo
                                        .delete_project_n
                                        .replace("{}", &total_in_project.to_string())
                                } else {
                                    crate::i18n::tr().todo.delete_project.to_string()
                                };
                                if ui.button(&delete_label).clicked() {
                                    actions.push(TodoAction::DeleteProject { id: project.id });
                                    self.popup_todo_id = None;
                                }
                            });
                    });

                let popup_rect = area_resp.response.rect;
                if ui.input(|i| i.pointer.any_pressed()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        if !popup_rect.contains(pos) && !anchor.contains(pos) {
                            self.popup_todo_id = None;
                        }
                    }
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.popup_todo_id = None;
                }
            }
        }

        // Project body (todos)
        if !project.collapsed {
            // Subtle left border for project group
            let indent_color = if let Some(color_str) = &project.color {
                parse_hex_color(color_str)
                    .map(|c| egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 30))
                    .unwrap_or(theme.border_subtle)
            } else {
                theme.border_subtle
            };

            let body_resp = egui::Frame::none()
                .inner_margin(egui::Margin {
                    left: 12.0,
                    top: 2.0,
                    bottom: 2.0,
                    right: 0.0,
                })
                .show(ui, |ui| {
                    if !project_todos.is_empty() {
                        actions.extend(self.render_dnd_todo_list(
                            ui,
                            theme,
                            &project_todos,
                            all_projects,
                            Some(project.id),
                        ));
                    } else {
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(crate::i18n::tr().todo.no_tasks)
                                .size(12.0)
                                .color(theme.text_muted)
                                .italics(),
                        );
                        ui.add_space(2.0);
                    }

                    // Inline new task input
                    ui.add_space(2.0);
                    let title_buf = self.project_task_titles.entry(project.id).or_default();
                    ui.horizontal(|ui| {
                        let (icon_rect, _) =
                            ui.allocate_exact_size(Vec2::new(14.0, 20.0), Sense::hover());
                        let ir =
                            egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(11.0));
                        draw_icon(ui, Icon::Plus, ir, theme.text_muted.linear_multiply(0.6));

                        let response = ui.add(
                            egui::TextEdit::singleline(title_buf)
                                .hint_text(crate::i18n::tr().todo.new_task_hint)
                                .desired_width(ui.available_width())
                                .font(egui::FontId::proportional(13.0))
                                .frame(false),
                        );

                        let submitted =
                            response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if submitted {
                            let title = title_buf.trim().to_string();
                            if !title.is_empty() {
                                actions.push(TodoAction::CreateTodo {
                                    workspace_id: project.workspace_id,
                                    project_id: Some(project.id),
                                    title,
                                });
                                title_buf.clear();
                                response.request_focus();
                            }
                        }
                    });
                });

            // Draw left accent border using actual content rect
            let body_rect = body_resp.response.rect;
            ui.painter().line_segment(
                [
                    egui::pos2(body_rect.left() + 6.0, body_rect.top() + 2.0),
                    egui::pos2(body_rect.left() + 6.0, body_rect.bottom() - 2.0),
                ],
                egui::Stroke::new(2.0, indent_color),
            );
        }

        actions
    }

    /// Render a single todo item row with DnD handle provided by egui_dnd.
    pub(super) fn render_todo_item(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        todo: &TodoItem,
        _projects: &[Project],
        handle: egui_dnd::Handle,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        // Editing mode — full-width card
        if self.editing_task_id == Some(todo.id) {
            return self.render_editing(ui, theme, todo);
        }

        ui.push_id(("todo_item", todo.id), |ui| {
            // Priority left-accent stripe color
            let accent_stripe = if todo.priority != Priority::None && !todo.completed {
                Some(priority_color(todo.priority, theme))
            } else {
                None
            };

            // Item card frame
            let frame_resp = egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(4.0, 3.0))
                .rounding(theme.rounding_sm)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Drag handle
                        handle.ui(ui, |ui| {
                            let (grip_rect, _) =
                                ui.allocate_exact_size(Vec2::new(12.0, 20.0), Sense::hover());
                            let grip_icon_rect =
                                egui::Rect::from_center_size(grip_rect.center(), Vec2::splat(10.0));
                            draw_icon(
                                ui,
                                Icon::GripVertical,
                                grip_icon_rect,
                                theme.text_muted.linear_multiply(0.4),
                            );
                        });

                        // Collapse arrow (if has body)
                        if todo.body.is_some() {
                            let icon = if todo.collapsed {
                                Icon::ChevronRight
                            } else {
                                Icon::ChevronDown
                            };
                            let (arrow_rect, arrow_resp) =
                                ui.allocate_exact_size(Vec2::new(14.0, 20.0), Sense::click());
                            let icon_rect = egui::Rect::from_center_size(
                                arrow_rect.center(),
                                Vec2::splat(10.0),
                            );
                            draw_icon(ui, icon, icon_rect, theme.text_muted);
                            if arrow_resp.clicked() {
                                actions.push(TodoAction::ToggleCollapse { id: todo.id });
                            }
                        }

                        // Checkbox
                        let checkbox_icon = if todo.completed {
                            Icon::CheckSquare
                        } else {
                            Icon::Square
                        };
                        let checkbox_color = if todo.completed {
                            theme.success
                        } else {
                            theme.text_secondary
                        };
                        let (cb_rect, cb_resp) =
                            ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::click());
                        let cb_icon_rect =
                            egui::Rect::from_center_size(cb_rect.center(), Vec2::splat(15.0));
                        draw_icon(ui, checkbox_icon, cb_icon_rect, checkbox_color);
                        if cb_resp.clicked() {
                            actions.push(TodoAction::ToggleComplete { id: todo.id });
                        }

                        ui.add_space(2.0);

                        // Title — takes all remaining space
                        let title_color = if todo.completed {
                            theme.text_muted
                        } else {
                            theme.text_primary
                        };
                        let mut title_text =
                            RichText::new(&todo.title).size(13.5).color(title_color);
                        if todo.completed {
                            title_text = title_text.strikethrough();
                        }
                        let title_response =
                            ui.add(egui::Label::new(title_text).wrap().sense(Sense::click()));

                        if title_response.double_clicked() {
                            self.editing_task_id = Some(todo.id);
                            self.editing_title = todo.title.clone();
                            self.editing_body = todo.body.clone().unwrap_or_default();
                            self.editing_focus_title = true;
                        }

                        // Push dots to the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Three-dot menu (popup rendered outside DnD)
                            let (dots_rect, dots_resp) =
                                ui.allocate_exact_size(Vec2::new(16.0, 20.0), Sense::click());
                            let dots_icon_rect =
                                egui::Rect::from_center_size(dots_rect.center(), Vec2::splat(12.0));
                            let is_popup_open = self.popup_todo_id == Some(todo.id);
                            let dots_color = if dots_resp.hovered() || is_popup_open {
                                theme.text_primary
                            } else {
                                theme.text_muted.linear_multiply(0.6)
                            };
                            draw_icon(ui, Icon::MoreVertical, dots_icon_rect, dots_color);
                            if dots_resp.clicked() {
                                if is_popup_open {
                                    self.popup_todo_id = None;
                                    self.popup_dots_rect = None;
                                } else {
                                    self.popup_todo_id = Some(todo.id);
                                    self.popup_dots_rect = Some(dots_rect);
                                }
                            }
                        });
                    });
                });

            let item_rect = frame_resp.response.rect;

            // Hover bg
            if ui.rect_contains_pointer(item_rect) {
                ui.painter()
                    .rect_filled(item_rect, theme.rounding_sm, hover_bg(theme));
            }

            // Priority accent stripe on the left
            if let Some(stripe_color) = accent_stripe {
                ui.painter().line_segment(
                    [
                        egui::pos2(item_rect.left() + 1.0, item_rect.top() + 2.0),
                        egui::pos2(item_rect.left() + 1.0, item_rect.bottom() - 2.0),
                    ],
                    egui::Stroke::new(2.5, stripe_color),
                );
            }

            // Markdown body (expanded)
            if !todo.collapsed {
                if let Some(body) = &todo.body {
                    egui::Frame::none()
                        .inner_margin(egui::Margin {
                            left: 52.0,
                            top: 0.0,
                            bottom: 4.0,
                            right: 8.0,
                        })
                        .show(ui, |ui| {
                            render_markdown_simple(ui, theme, body);
                        });
                }
            }
        }); // push_id

        actions
    }

    /// Render the popup menu for a todo item as an egui::Area (outside DnD context).
    pub(super) fn render_todo_popup_area(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        todo: &TodoItem,
        projects: &[Project],
        actions: &mut Vec<TodoAction>,
    ) {
        let anchor_rect = match self.popup_dots_rect {
            Some(r) => r,
            None => return,
        };

        let area_id = egui::Id::new("todo_popup").with(todo.id);
        let popup_pos = anchor_rect.left_bottom() + egui::vec2(-140.0, 4.0);

        let area_resp = egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .fixed_pos(popup_pos)
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(theme.bg_secondary)
                    .stroke(egui::Stroke::new(1.0, theme.border_subtle))
                    .rounding(theme.rounding_md)
                    .inner_margin(egui::Margin::symmetric(4.0, 4.0))
                    .show(ui, |ui| {
                        ui.set_min_width(160.0);
                        if ui.button(crate::i18n::tr().todo.edit).clicked() {
                            self.editing_task_id = Some(todo.id);
                            self.editing_title = todo.title.clone();
                            self.editing_body = todo.body.clone().unwrap_or_default();
                            self.editing_focus_title = true;
                            self.popup_todo_id = None;
                        }
                        if !todo.completed
                            && ui.button(crate::i18n::tr().todo.add_to_queue).clicked()
                        {
                            actions.push(TodoAction::AddToQueue {
                                todo_id: todo.id,
                                planned_pomodoros: 1,
                            });
                            self.popup_todo_id = None;
                        }
                        if Self::render_move_menu(ui, todo, projects, actions) {
                            self.popup_todo_id = None;
                        }
                        if Self::render_priority_menu(ui, todo, actions) {
                            self.popup_todo_id = None;
                        }
                        ui.separator();
                        if ui.button(crate::i18n::tr().todo.delete).clicked() {
                            actions.push(TodoAction::DeleteTodo { id: todo.id });
                            self.popup_todo_id = None;
                        }
                    });
            });

        // Close on click outside the popup area or anchor button
        let popup_rect = area_resp.response.rect;
        if ui.input(|i| i.pointer.any_pressed()) {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if !popup_rect.contains(pos) && !anchor_rect.contains(pos) {
                    self.popup_todo_id = None;
                }
            }
        }

        // Close on Escape
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.popup_todo_id = None;
        }
    }

    /// Shared "Move to..." submenu. Returns true if an action was taken.
    fn render_move_menu(
        ui: &mut egui::Ui,
        todo: &TodoItem,
        projects: &[Project],
        actions: &mut Vec<TodoAction>,
    ) -> bool {
        let mut acted = false;
        if !projects.is_empty() || todo.project_id.is_some() {
            ui.menu_button(crate::i18n::tr().todo.move_to, |ui| {
                if todo.project_id.is_some() {
                    if ui.button(crate::i18n::tr().todo.no_project).clicked() {
                        actions.push(TodoAction::MoveTodo {
                            id: todo.id,
                            project_id: None,
                        });
                        acted = true;
                        ui.close_menu();
                    }
                    if !projects.is_empty() {
                        ui.separator();
                    }
                }
                for p in projects {
                    if Some(p.id) != todo.project_id && ui.button(&p.name).clicked() {
                        actions.push(TodoAction::MoveTodo {
                            id: todo.id,
                            project_id: Some(p.id),
                        });
                        acted = true;
                        ui.close_menu();
                    }
                }
            });
        }
        acted
    }

    /// Shared priority submenu. Returns true if an action was taken.
    fn render_priority_menu(
        ui: &mut egui::Ui,
        todo: &TodoItem,
        actions: &mut Vec<TodoAction>,
    ) -> bool {
        let mut acted = false;
        if !todo.completed {
            ui.menu_button(crate::i18n::tr().todo.priority, |ui| {
                for p in Priority::all() {
                    let is_current = todo.priority == *p;
                    let label = if is_current {
                        format!("● {}", crate::i18n::tr().priority_label(*p))
                    } else {
                        crate::i18n::tr().priority_label(*p).to_string()
                    };
                    if ui.button(&label).clicked() {
                        actions.push(TodoAction::SetPriority {
                            id: todo.id,
                            priority: *p,
                        });
                        acted = true;
                        ui.close_menu();
                    }
                }
            });
        }
        acted
    }

    fn render_editing(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        todo: &TodoItem,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(10.0, 8.0))
            .rounding(theme.rounding_md)
            .stroke(egui::Stroke::new(
                1.0,
                theme.accent.solid().linear_multiply(0.5),
            ))
            .fill(if theme.is_light {
                egui::Color32::from_rgba_unmultiplied(0, 0, 0, 5)
            } else {
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, 5)
            })
            .show(ui, |ui| {
                // Title edit
                let title_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.editing_title)
                        .font(egui::FontId::proportional(14.0))
                        .desired_width(ui.available_width())
                        .hint_text(crate::i18n::tr().todo.title_hint),
                );
                if self.editing_focus_title {
                    title_resp.request_focus();
                    self.editing_focus_title = false;
                }

                ui.add_space(4.0);

                // Body edit
                ui.add(
                    egui::TextEdit::multiline(&mut self.editing_body)
                        .font(egui::FontId::proportional(13.0))
                        .desired_width(ui.available_width())
                        .desired_rows(3)
                        .hint_text(crate::i18n::tr().todo.description_hint),
                );

                ui.add_space(4.0);

                // Buttons row
                ui.horizontal(|ui| {
                    let save = ui.small_button(crate::i18n::tr().todo.save);
                    let ctrl_enter =
                        ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter));
                    if save.clicked() || ctrl_enter {
                        let title = self.editing_title.trim().to_string();
                        if !title.is_empty() {
                            let body = if self.editing_body.trim().is_empty() {
                                None
                            } else {
                                Some(self.editing_body.clone())
                            };
                            actions.push(TodoAction::UpdateTodo {
                                id: todo.id,
                                title,
                                body,
                            });
                        }
                        self.editing_task_id = None;
                    }
                    if ui.small_button(crate::i18n::tr().common.cancel).clicked() {
                        self.editing_task_id = None;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Ctrl+Enter")
                                .size(11.0)
                                .color(theme.text_muted),
                        );
                    });
                });
            });

        actions
    }
}
