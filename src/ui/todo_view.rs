use egui::{self, Color32, RichText, Sense, Vec2};

use crate::data::todo::{Priority, Project, TodoItem, Workspace};
use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;

/// Payload carried during drag-and-drop of a todo item
#[derive(Debug, Clone)]
struct DragTodo {
    id: i64,
}

/// Actions produced by the todo UI
#[derive(Debug, Clone)]
pub enum TodoAction {
    // Workspaces
    CreateWorkspace { name: String },
    RenameWorkspace { id: i64, name: String },
    DeleteWorkspace { id: i64 },
    SwitchWorkspace { id: i64 },

    // Projects
    CreateProject { workspace_id: i64, name: String },
    RenameProject { id: i64, name: String },
    DeleteProject { id: i64 },
    ToggleProjectCollapse { id: i64 },

    // Todos
    CreateTodo {
        workspace_id: i64,
        project_id: Option<i64>,
        title: String,
    },
    CreateTodoWithBody {
        workspace_id: i64,
        project_id: Option<i64>,
        title: String,
        body: String,
    },
    UpdateTodo {
        id: i64,
        title: String,
        body: Option<String>,
    },
    ToggleComplete { id: i64 },
    ToggleCollapse { id: i64 },
    DeleteTodo { id: i64 },
    MoveTodo { id: i64, project_id: Option<i64> },
    ReorderTodo { id: i64, project_id: Option<i64>, new_position: i32 },
    SetPriority { id: i64, priority: Priority },

    // Queue
    AddToQueue { todo_id: i64, planned_pomodoros: u32 },
    RemoveFromQueue { id: i64 },
    ClearQueue,

    // Settings
    ToggleShowCompleted,

    // Window
    Close,
}

/// Main todo view rendered inside the todo viewport
pub struct TodoView {
    // Editing state
    editing_task_id: Option<i64>,
    editing_title: String,
    editing_body: String,
    editing_focus_title: bool,

    // New task input (unassigned)
    new_task_title: String,
    focus_new_task: bool,

    // Per-project new task input: project_id -> title
    project_task_titles: std::collections::HashMap<i64, String>,

    // New workspace
    adding_workspace: bool,
    new_workspace_name: String,

    // New project
    adding_project_workspace: Option<i64>,
    new_project_name: String,

    // Renaming
    renaming_workspace_id: Option<i64>,
    rename_workspace_buffer: String,
    renaming_project_id: Option<i64>,
    rename_project_buffer: String,

}

impl TodoView {
    pub fn new() -> Self {
        Self {
            editing_task_id: None,
            editing_title: String::new(),
            editing_body: String::new(),
            editing_focus_title: false,
            new_task_title: String::new(),
            focus_new_task: false,
            project_task_titles: std::collections::HashMap::new(),
            adding_workspace: false,
            new_workspace_name: String::new(),
            adding_project_workspace: None,
            new_project_name: String::new(),
            renaming_workspace_id: None,
            rename_workspace_buffer: String::new(),
            renaming_project_id: None,
            rename_project_buffer: String::new(),
        }
    }

    /// Show the todo content inside an egui::Window (no titlebar/CentralPanel needed)
    pub fn show_inner(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        workspaces: &[Workspace],
        projects: &[Project],
        todos: &[TodoItem],
        current_workspace_id: i64,
        show_completed: bool,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        // Workspace tabs
        actions.extend(self.render_workspace_tabs(ui, theme, workspaces, current_workspace_id));

        // Filter bar
        ui.horizontal(|ui| {
            let completed_count = todos.iter().filter(|t| t.completed).count();
            if completed_count > 0 {
                let resp = ui.horizontal(|ui| {
                    let chevron = if show_completed {
                        Icon::ChevronDown
                    } else {
                        Icon::ChevronRight
                    };
                    let (icon_rect, _) =
                        ui.allocate_exact_size(Vec2::new(12.0, 12.0), Sense::hover());
                    let ir = egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(10.0));
                    draw_icon(ui, chevron, ir, theme.text_muted);
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("Завершённые ({})", completed_count))
                                .size(12.0)
                                .color(theme.text_muted),
                        )
                        .sense(Sense::click()),
                    )
                });
                if resp.inner.clicked() || resp.response.clicked() {
                    actions.push(TodoAction::ToggleShowCompleted);
                }
            }
        });

        ui.add_space(theme.spacing_sm);

        // Task list
        actions.extend(self.render_task_list(
            ui,
            theme,
            projects,
            todos,
            current_workspace_id,
            show_completed,
        ));

        // Hotkeys
        let any_text_focused = ui.ctx().memory(|m| m.focused().is_some());
        ui.ctx().input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::N) {
                self.focus_new_task = true;
            }
            if i.key_pressed(egui::Key::Escape) {
                if self.editing_task_id.is_some() {
                    self.editing_task_id = None;
                } else if self.adding_workspace {
                    self.adding_workspace = false;
                } else if self.adding_project_workspace.is_some() {
                    self.adding_project_workspace = None;
                }
            }
        });

        // Clipboard paste (only when no text field is focused)
        if self.editing_task_id.is_none() && !any_text_focused {
            actions.extend(self.handle_clipboard_paste(ui.ctx(), current_workspace_id));
        }

        actions
    }

    fn render_workspace_tabs(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        workspaces: &[Workspace],
        current_id: i64,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            egui::ScrollArea::horizontal()
                .max_width(ui.available_width() - 30.0)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .show(ui, |ui| {
                    for ws in workspaces {
                        // Renaming mode
                        if self.renaming_workspace_id == Some(ws.id) {
                            let response =
                                ui.add(egui::TextEdit::singleline(&mut self.rename_workspace_buffer)
                                    .desired_width(80.0)
                                    .font(egui::FontId::proportional(13.0)));
                            if response.lost_focus()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                let name = self.rename_workspace_buffer.trim().to_string();
                                if !name.is_empty() {
                                    actions.push(TodoAction::RenameWorkspace { id: ws.id, name });
                                }
                                self.renaming_workspace_id = None;
                            }
                            continue;
                        }

                        let is_active = ws.id == current_id;
                        let label = ws.name.clone();

                        let text = RichText::new(&label)
                            .size(14.0)
                            .color(if is_active {
                                theme.text_primary
                            } else {
                                theme.text_muted
                            });

                        let response = ui.add(egui::Label::new(text).sense(Sense::click()));

                        // Active underline
                        if is_active {
                            let rect = response.rect;
                            ui.painter().line_segment(
                                [
                                    egui::pos2(rect.left(), rect.bottom()),
                                    egui::pos2(rect.right(), rect.bottom()),
                                ],
                                egui::Stroke::new(2.0, theme.accent.solid()),
                            );
                        }

                        if response.clicked() {
                            actions.push(TodoAction::SwitchWorkspace { id: ws.id });
                        }

                        response.context_menu(|ui| {
                            if ui.button("Переименовать").clicked() {
                                self.renaming_workspace_id = Some(ws.id);
                                self.rename_workspace_buffer = ws.name.clone();
                                ui.close_menu();
                            }
                            if workspaces.len() > 1 {
                                if ui.button("Удалить").clicked() {
                                    actions.push(TodoAction::DeleteWorkspace { id: ws.id });
                                    ui.close_menu();
                                }
                            }
                        });

                        ui.add_space(theme.spacing_sm);
                    }
                });

            // Add workspace button / input
            if self.adding_workspace {
                let response =
                    ui.add(egui::TextEdit::singleline(&mut self.new_workspace_name)
                        .desired_width(80.0)
                        .hint_text("Название...")
                        .font(egui::FontId::proportional(13.0)));
                if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let name = self.new_workspace_name.trim().to_string();
                    if !name.is_empty() {
                        actions.push(TodoAction::CreateWorkspace { name });
                    }
                    self.new_workspace_name.clear();
                    self.adding_workspace = false;
                }
            } else {
                let (plus_rect, plus_resp) =
                    ui.allocate_exact_size(Vec2::new(18.0, 18.0), Sense::click());
                let ir = egui::Rect::from_center_size(plus_rect.center(), Vec2::splat(14.0));
                draw_icon(ui, Icon::Plus, ir, theme.text_muted);
                if plus_resp.clicked() {
                    self.adding_workspace = true;
                }
                plus_resp.on_hover_text("Новый workspace");
            }
        });

        actions
    }

    fn render_task_list(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        projects: &[Project],
        todos: &[TodoItem],
        current_workspace_id: i64,
        show_completed: bool,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Projects with their todos
                for project in projects {
                    actions.extend(self.render_project(ui, theme, project, todos, projects, show_completed));
                }

                // Unassigned todos
                let unassigned: Vec<&TodoItem> = todos
                    .iter()
                    .filter(|t| t.project_id.is_none())
                    .filter(|t| show_completed || !t.completed)
                    .collect();

                if !unassigned.is_empty() || projects.is_empty() {
                    let is_dragging = egui::DragAndDrop::has_payload_of_type::<DragTodo>(ui.ctx());

                    if !projects.is_empty() && !unassigned.is_empty() {
                        ui.add_space(theme.spacing_md);
                        ui.horizontal(|ui| {
                            let line_color = theme.border_subtle;
                            let rect = ui.available_rect_before_wrap();
                            let y = rect.center().y;
                            let text_response = ui.label(
                                RichText::new("Разное")
                                    .size(13.0)
                                    .color(theme.text_muted),
                            );
                            let after_text = text_response.rect.right() + 8.0;
                            ui.painter().line_segment(
                                [
                                    egui::pos2(after_text, y),
                                    egui::pos2(rect.right(), y),
                                ],
                                egui::Stroke::new(1.0, line_color),
                            );
                        });
                        ui.add_space(6.0);
                    }

                    if !unassigned.is_empty() {
                        for todo in &unassigned {
                            // Drop indicator before this item
                            if is_dragging {
                                actions.extend(self.render_drop_indicator(
                                    ui, theme, None, todo.position,
                                ));
                            }
                            actions.extend(self.render_todo_item(ui, theme, todo, projects));
                        }

                        // Drop indicator after last item
                        if is_dragging {
                            let last_pos = unassigned.last().unwrap().position;
                            actions.extend(self.render_drop_indicator(
                                ui, theme, None, last_pos + 1,
                            ));
                        }
                    }

                    // Drop on empty unassigned area
                    if is_dragging && unassigned.is_empty() && !projects.is_empty() {
                        actions.extend(self.render_drop_indicator(ui, theme, None, 0));
                    }
                }

                // Empty state
                if todos.is_empty() && projects.is_empty() {
                    ui.add_space(theme.spacing_xl);
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("Нет задач")
                                .size(16.0)
                                .color(theme.text_muted),
                        );
                        ui.add_space(theme.spacing_sm);
                        ui.label(
                            RichText::new("Создайте первую задачу ниже")
                                .size(13.0)
                                .color(theme.text_muted),
                        );
                    });
                }

                ui.add_space(theme.spacing_sm);

                // New task input
                actions.extend(self.render_new_task_input(
                    ui,
                    theme,
                    current_workspace_id,
                    None,
                ));

                ui.add_space(theme.spacing_xs);

                // Add project button
                if self.adding_project_workspace == Some(current_workspace_id) {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("+").size(14.0).color(theme.text_muted),
                        );
                        let response = ui.add(
                            egui::TextEdit::singleline(&mut self.new_project_name)
                                .hint_text("Название проекта...")
                                .desired_width(ui.available_width() - 10.0)
                                .font(egui::FontId::proportional(13.0))
                                .frame(false),
                        );
                        if response.lost_focus()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            let name = self.new_project_name.trim().to_string();
                            if !name.is_empty() {
                                actions.push(TodoAction::CreateProject {
                                    workspace_id: current_workspace_id,
                                    name,
                                });
                            }
                            self.new_project_name.clear();
                            self.adding_project_workspace = None;
                        }
                    });
                } else {
                    let resp = ui.horizontal(|ui| {
                        let (icon_rect, _) =
                            ui.allocate_exact_size(Vec2::new(16.0, 18.0), Sense::hover());
                        let ir = egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(12.0));
                        draw_icon(ui, Icon::CirclePlus, ir, theme.text_muted);
                        ui.add(
                            egui::Label::new(
                                RichText::new("Проект")
                                    .size(13.0)
                                    .color(theme.text_muted),
                            )
                            .sense(Sense::click()),
                        )
                    });
                    if resp.inner.clicked() || resp.response.clicked() {
                        self.adding_project_workspace = Some(current_workspace_id);
                    }
                }
            });

        actions
    }

    fn render_project(
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
        });

        // Context menu on header
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

    fn render_todo_item(
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

    fn render_new_task_input(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        workspace_id: i64,
        project_id: Option<i64>,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        ui.horizontal(|ui| {
            // Plus icon
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(18.0, 22.0), Sense::hover());
            let ir = egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(14.0));
            draw_icon(ui, Icon::Plus, ir, theme.accent.solid());

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.new_task_title)
                    .hint_text("Новая задача...")
                    .desired_width(ui.available_width())
                    .font(egui::FontId::proportional(14.0))
                    .frame(false),
            );

            if self.focus_new_task {
                response.request_focus();
                self.focus_new_task = false;
            }

            // Singleline TextEdit loses focus on Enter
            let submitted = response.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if submitted {
                let title = self.new_task_title.trim().to_string();
                if !title.is_empty() {
                    actions.push(TodoAction::CreateTodo {
                        workspace_id,
                        project_id,
                        title,
                    });
                    self.new_task_title.clear();
                    response.request_focus();
                }
            }
        });

        actions
    }

    /// Render a drop indicator line between todo items. Returns action if dropped here.
    fn render_drop_indicator(
        &self,
        ui: &mut egui::Ui,
        theme: &Theme,
        target_project_id: Option<i64>,
        target_position: i32,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        if !egui::DragAndDrop::has_payload_of_type::<DragTodo>(ui.ctx()) {
            return actions;
        }

        // Allocate a thin drop zone
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ui.available_width(), 6.0),
            Sense::hover(),
        );

        let is_hovered = ui.ctx().pointer_latest_pos()
            .map_or(false, |pos| rect.contains(pos));

        if is_hovered {
            // Draw accent line
            let y = rect.center().y;
            ui.painter().line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(2.0, theme.accent.solid()),
            );
            // Draw small circles at ends
            ui.painter().circle_filled(egui::pos2(rect.left() + 3.0, y), 3.0, theme.accent.solid());
            ui.painter().circle_filled(egui::pos2(rect.right() - 3.0, y), 3.0, theme.accent.solid());

            let pointer_released = ui.input(|i| i.pointer.any_released());
            if pointer_released {
                if let Some(payload) = egui::DragAndDrop::take_payload::<DragTodo>(ui.ctx()) {
                    let drag = payload.as_ref();
                    actions.push(TodoAction::ReorderTodo {
                        id: drag.id,
                        project_id: target_project_id,
                        new_position: target_position,
                    });
                }
            }
        }

        actions
    }

    fn handle_clipboard_paste(
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
fn render_markdown_simple(ui: &mut egui::Ui, theme: &Theme, text: &str) {
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
fn parse_hex_color(hex: &str) -> Option<Color32> {
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
fn priority_color(priority: Priority, theme: &Theme) -> Color32 {
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
