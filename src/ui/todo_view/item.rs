use egui::{self, RichText, Sense, Vec2};

use crate::data::todo::{Priority, Project, TodoItem};
use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;
use super::{DragTodo, TodoAction, TodoView};
use super::helpers::{parse_hex_color, priority_color, render_markdown_simple};

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

        // Project header
        ui.add_space(8.0);
        let header_response = ui.horizontal(|ui| {
            // Collapse arrow (vector icon)
            let arrow_icon = if project.collapsed { Icon::ChevronRight } else { Icon::ChevronDown };
            let (arrow_rect, arrow_resp) =
                ui.allocate_exact_size(Vec2::new(18.0, 22.0), Sense::click());
            let icon_rect = egui::Rect::from_center_size(arrow_rect.center(), Vec2::splat(12.0));
            draw_icon(ui, arrow_icon, icon_rect, theme.text_secondary);
            if arrow_resp.clicked() {
                actions.push(TodoAction::ToggleProjectCollapse { id: project.id });
            }

            // Color dot
            if let Some(color_str) = &project.color {
                if let Some(color) = parse_hex_color(color_str) {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(Vec2::new(10.0, 10.0), Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 5.0, color);
                }
            }

            // Renaming
            if self.renaming_project_id == Some(project.id) {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.rename_project_buffer)
                        .desired_width(140.0)
                        .font(egui::FontId::proportional(15.0)),
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
                // Project name
                ui.label(
                    RichText::new(&project.name)
                        .size(15.0)
                        .color(theme.text_primary)
                        .strong(),
                );
            }

            // Active count badge
            if active_count > 0 {
                ui.label(
                    RichText::new(format!("{}", active_count))
                        .size(12.0)
                        .color(theme.text_muted),
                );
            }

            // Spacer to push dots to the right
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Three-dot menu button for project
                let dots_id = ui.id().with(("project_dots", project.id));
                let (dots_rect, dots_resp) =
                    ui.allocate_exact_size(Vec2::new(18.0, 22.0), Sense::click());
                let dots_icon_rect = egui::Rect::from_center_size(dots_rect.center(), Vec2::splat(14.0));
                let dots_color = if dots_resp.hovered() { theme.text_primary } else { theme.text_muted };
                draw_icon(ui, Icon::MoreVertical, dots_icon_rect, dots_color);
                if dots_resp.clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(dots_id));
                }
                let close_popup = |ui: &mut egui::Ui| {
                    ui.memory_mut(|mem| mem.close_popup());
                };
                egui::popup_below_widget(ui, dots_id, &dots_resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                    ui.set_min_width(160.0);
                    if ui.button("Переименовать").clicked() {
                        self.renaming_project_id = Some(project.id);
                        self.rename_project_buffer = project.name.clone();
                        close_popup(ui);
                    }
                    let total_in_project = todos.iter().filter(|t| t.project_id == Some(project.id)).count();
                    let delete_label = if total_in_project > 0 {
                        format!("Удалить проект ({} задач)", total_in_project)
                    } else {
                        "Удалить проект".to_string()
                    };
                    if ui.button(&delete_label).clicked() {
                        actions.push(TodoAction::DeleteProject { id: project.id });
                        close_popup(ui);
                    }
                });
            });
        });

        // Right-click context menu on project header
        header_response.response.context_menu(|ui| {
            if ui.button("Переименовать").clicked() {
                self.renaming_project_id = Some(project.id);
                self.rename_project_buffer = project.name.clone();
                ui.close_menu();
            }
            let total_in_project = todos.iter().filter(|t| t.project_id == Some(project.id)).count();
            let delete_label = if total_in_project > 0 {
                format!("Удалить проект ({} задач)", total_in_project)
            } else {
                "Удалить проект".to_string()
            };
            if ui.button(&delete_label).clicked() {
                actions.push(TodoAction::DeleteProject { id: project.id });
                ui.close_menu();
            }
        });

        // Project body (todos)
        if !project.collapsed {
            ui.indent(egui::Id::new(("project_indent", project.id)), |ui| {
                let is_dragging = egui::DragAndDrop::has_payload_of_type::<DragTodo>(ui.ctx());

                if !project_todos.is_empty() {
                    for todo in &project_todos {
                        // Drop indicator before this item (insert at this item's position)
                        if is_dragging {
                            actions.extend(self.render_drop_indicator(
                                ui, theme, Some(project.id), todo.position,
                            ));
                        }

                        actions.extend(self.render_todo_item(ui, theme, todo, all_projects));
                    }

                    // Drop indicator after last item
                    if is_dragging {
                        let last_pos = project_todos.last().unwrap().position;
                        actions.extend(self.render_drop_indicator(
                            ui, theme, Some(project.id), last_pos + 1,
                        ));
                    }
                } else if is_dragging {
                    // Drop zone on empty project
                    actions.extend(self.render_drop_indicator(ui, theme, Some(project.id), 0));
                } else {
                    ui.label(
                        RichText::new("Нет задач")
                            .size(13.0)
                            .color(theme.text_muted)
                            .italics(),
                    );
                }

                // Inline new task input for this project
                ui.add_space(4.0);
                let title_buf = self.project_task_titles.entry(project.id).or_default();
                ui.horizontal(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 22.0), Sense::hover());
                    let ir = egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(12.0));
                    draw_icon(ui, Icon::Plus, ir, theme.text_muted);

                    let response = ui.add(
                        egui::TextEdit::singleline(title_buf)
                            .hint_text("Новая задача...")
                            .desired_width(ui.available_width())
                            .font(egui::FontId::proportional(13.0))
                            .frame(false),
                    );

                    let submitted = response.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter));
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
        }

        actions
    }

    pub(super) fn render_todo_item(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        todo: &TodoItem,
        projects: &[Project],
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        // Editing mode
        if self.editing_task_id == Some(todo.id) {
            return self.render_editing(ui, theme, todo);
        }

        // Row content - push unique ID so egui tracks each item correctly
        ui.add_space(2.0);
        ui.push_id(("todo_item", todo.id), |ui| {
        let row_resp = ui.horizontal_top(|ui| {
            // Drag handle
            let (grip_rect, grip_resp) =
                ui.allocate_exact_size(Vec2::new(14.0, 22.0), Sense::drag());
            let grip_icon_rect = egui::Rect::from_center_size(grip_rect.center(), Vec2::splat(12.0));
            let grip_color = if grip_resp.hovered() || grip_resp.dragged() {
                theme.text_secondary
            } else {
                theme.text_muted.linear_multiply(0.5)
            };
            draw_icon(ui, Icon::GripVertical, grip_icon_rect, grip_color);
            if grip_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
            }
            if grip_resp.drag_started() {
                egui::DragAndDrop::set_payload(ui.ctx(), DragTodo {
                    id: todo.id,
                });
            }
            if grip_resp.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                // Paint floating label at cursor
                if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                    let layer_id = egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag_overlay"));
                    let painter = ui.ctx().layer_painter(layer_id);
                    let text = egui::WidgetText::from(
                        RichText::new(&todo.title).size(13.0).color(theme.text_primary)
                    );
                    let galley = text.into_galley(ui, Some(egui::TextWrapMode::Truncate), 200.0, egui::FontSelection::Default);
                    let text_rect = egui::Rect::from_min_size(
                        pointer + Vec2::new(10.0, -10.0),
                        galley.size(),
                    ).expand(4.0);
                    painter.rect_filled(text_rect, 4.0, theme.bg_elevated);
                    painter.rect_stroke(text_rect, 4.0, egui::Stroke::new(1.0, theme.border_default));
                    painter.galley(text_rect.min + Vec2::new(4.0, 4.0), galley, theme.text_primary);
                }
            }

            // Collapse arrow (if has body)
            if todo.body.is_some() {
                let icon = if todo.collapsed {
                    Icon::ChevronRight
                } else {
                    Icon::ChevronDown
                };
                let (arrow_rect, arrow_resp) =
                    ui.allocate_exact_size(Vec2::new(16.0, 22.0), Sense::click());
                let icon_rect =
                    egui::Rect::from_center_size(arrow_rect.center(), Vec2::splat(12.0));
                draw_icon(ui, icon, icon_rect, theme.text_muted);
                if arrow_resp.clicked() {
                    actions.push(TodoAction::ToggleCollapse { id: todo.id });
                }
            }

            // Checkbox icon (larger hit area for stability)
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
                ui.allocate_exact_size(Vec2::new(22.0, 22.0), Sense::click());
            let cb_icon_rect = egui::Rect::from_center_size(cb_rect.center(), Vec2::splat(16.0));
            draw_icon(ui, checkbox_icon, cb_icon_rect, checkbox_color);
            if cb_resp.clicked() {
                actions.push(TodoAction::ToggleComplete { id: todo.id });
            }

            // Priority indicator
            if todo.priority != Priority::None && !todo.completed {
                let priority_color = priority_color(todo.priority, theme);
                let (dot_rect, _) =
                    ui.allocate_exact_size(Vec2::new(10.0, 22.0), Sense::hover());
                ui.painter()
                    .circle_filled(dot_rect.center(), 4.0, priority_color);
            }

            // Title
            let title_color = if todo.completed {
                theme.text_muted
            } else {
                theme.text_primary
            };
            let mut title_text = RichText::new(&todo.title)
                .size(14.0)
                .color(title_color);
            if todo.completed {
                title_text = title_text.strikethrough();
            }
            let title_response =
                ui.add(egui::Label::new(title_text).wrap().sense(Sense::click()));

            // Double click title to edit
            if title_response.double_clicked() {
                self.editing_task_id = Some(todo.id);
                self.editing_title = todo.title.clone();
                self.editing_body = todo.body.clone().unwrap_or_default();
                self.editing_focus_title = true;
            }

            // Three-dot menu button (left-click opens menu)
            let dots_id = ui.id().with(("dots", todo.id));
            let (dots_rect, dots_resp) =
                ui.allocate_exact_size(Vec2::new(18.0, 22.0), Sense::click());
            let dots_icon_rect = egui::Rect::from_center_size(dots_rect.center(), Vec2::splat(14.0));
            let dots_color = if dots_resp.hovered() { theme.text_primary } else { theme.text_muted };
            draw_icon(ui, Icon::MoreVertical, dots_icon_rect, dots_color);
            if dots_resp.clicked() {
                ui.memory_mut(|mem| mem.toggle_popup(dots_id));
            }
            let close_popup = |ui: &mut egui::Ui| {
                ui.memory_mut(|mem| mem.close_popup());
            };
            egui::popup_below_widget(ui, dots_id, &dots_resp, egui::PopupCloseBehavior::CloseOnClickOutside, |ui| {
                ui.set_min_width(160.0);
                if ui.button("Редактировать").clicked() {
                    self.editing_task_id = Some(todo.id);
                    self.editing_title = todo.title.clone();
                    self.editing_body = todo.body.clone().unwrap_or_default();
                    self.editing_focus_title = true;
                    close_popup(ui);
                }
                if !todo.completed {
                    if ui.button("В очередь").clicked() {
                        actions.push(TodoAction::AddToQueue {
                            todo_id: todo.id,
                            planned_pomodoros: 1,
                        });
                        close_popup(ui);
                    }
                }
                // Move to project submenu
                if !projects.is_empty() || todo.project_id.is_some() {
                    ui.menu_button("Переместить в...", |ui| {
                        if todo.project_id.is_some() {
                            if ui.button("Без проекта").clicked() {
                                actions.push(TodoAction::MoveTodo {
                                    id: todo.id,
                                    project_id: None,
                                });
                                ui.close_menu();
                            }
                            if !projects.is_empty() {
                                ui.separator();
                            }
                        }
                        for p in projects {
                            if Some(p.id) != todo.project_id {
                                if ui.button(&p.name).clicked() {
                                    actions.push(TodoAction::MoveTodo {
                                        id: todo.id,
                                        project_id: Some(p.id),
                                    });
                                    ui.close_menu();
                                }
                            }
                        }
                    });
                }
                // Priority submenu
                if !todo.completed {
                    ui.menu_button("Приоритет", |ui| {
                        for p in Priority::all() {
                            let is_current = todo.priority == *p;
                            let label = if is_current {
                                format!("● {}", p.label())
                            } else {
                                p.label().to_string()
                            };
                            if ui.button(&label).clicked() {
                                actions.push(TodoAction::SetPriority {
                                    id: todo.id,
                                    priority: *p,
                                });
                                ui.close_menu();
                            }
                        }
                    });
                }
                ui.separator();
                if ui.button("Удалить").clicked() {
                    actions.push(TodoAction::DeleteTodo { id: todo.id });
                    close_popup(ui);
                }
            });
        });

        // Right-click context menu on the row
        row_resp.response.context_menu(|ui| {
            if ui.button("Редактировать").clicked() {
                self.editing_task_id = Some(todo.id);
                self.editing_title = todo.title.clone();
                self.editing_body = todo.body.clone().unwrap_or_default();
                self.editing_focus_title = true;
                ui.close_menu();
            }
            if !todo.completed {
                if ui.button("В очередь").clicked() {
                    actions.push(TodoAction::AddToQueue {
                        todo_id: todo.id,
                        planned_pomodoros: 1,
                    });
                    ui.close_menu();
                }
            }
            // Move to project submenu
            if !projects.is_empty() || todo.project_id.is_some() {
                ui.menu_button("Переместить в...", |ui| {
                    if todo.project_id.is_some() {
                        if ui.button("Без проекта").clicked() {
                            actions.push(TodoAction::MoveTodo {
                                id: todo.id,
                                project_id: None,
                            });
                            ui.close_menu();
                        }
                        if !projects.is_empty() {
                            ui.separator();
                        }
                    }
                    for p in projects {
                        if Some(p.id) != todo.project_id {
                            if ui.button(&p.name).clicked() {
                                actions.push(TodoAction::MoveTodo {
                                    id: todo.id,
                                    project_id: Some(p.id),
                                });
                                ui.close_menu();
                            }
                        }
                    }
                });
            }
            // Priority submenu
            if !todo.completed {
                ui.menu_button("Приоритет", |ui| {
                    for p in Priority::all() {
                        let is_current = todo.priority == *p;
                        let label = if is_current {
                            format!("● {}", p.label())
                        } else {
                            p.label().to_string()
                        };
                        if ui.button(&label).clicked() {
                            actions.push(TodoAction::SetPriority {
                                id: todo.id,
                                priority: *p,
                            });
                            ui.close_menu();
                        }
                    }
                });
            }
            ui.separator();
            if ui.button("Удалить").clicked() {
                actions.push(TodoAction::DeleteTodo { id: todo.id });
                ui.close_menu();
            }
        });

        // Markdown body (expanded)
        if !todo.collapsed {
            if let Some(body) = &todo.body {
                ui.indent(egui::Id::new(("todo_body", todo.id)), |ui| {
                    render_markdown_simple(ui, theme, body);
                });
            }
        }

        ui.add_space(1.0);
        }); // push_id

        actions
    }

    fn render_editing(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        todo: &TodoItem,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        egui::Frame::none()
            .inner_margin(egui::Margin::same(8.0))
            .rounding(theme.rounding_sm)
            .stroke(egui::Stroke::new(1.0, theme.border_default))
            .show(ui, |ui| {
                // Title edit
                let title_resp = ui.add(
                    egui::TextEdit::singleline(&mut self.editing_title)
                        .font(egui::FontId::proportional(15.0))
                        .desired_width(ui.available_width() - 10.0)
                        .hint_text("Заголовок..."),
                );
                if self.editing_focus_title {
                    title_resp.request_focus();
                    self.editing_focus_title = false;
                }

                ui.add_space(4.0);

                // Body edit
                ui.add(
                    egui::TextEdit::multiline(&mut self.editing_body)
                        .font(egui::FontId::proportional(14.0))
                        .desired_width(ui.available_width() - 10.0)
                        .desired_rows(4)
                        .hint_text("Описание (markdown)..."),
                );

                // Buttons
                ui.horizontal(|ui| {
                    let save = ui.small_button("Сохранить");
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
                    if ui.small_button("Отмена").clicked() {
                        self.editing_task_id = None;
                    }
                });
            });

        actions
    }

}
