mod helpers;
mod item;

use egui::{self, RichText, Sense, Vec2};

use crate::data::todo::{Priority, Project, TodoItem, Workspace};
use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;

pub use helpers::truncate_text;

/// Payload carried during drag-and-drop of a todo item
#[derive(Debug, Clone)]
pub(super) struct DragTodo {
    pub id: i64,
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

        // Workspace tabs (full width)
        actions.extend(self.render_workspace_tabs(ui, theme, workspaces, current_workspace_id));

        ui.add_space(theme.spacing_xs);

        // Centered content with max width
        let max_content_width = 720.0;
        let available = ui.available_width();
        let side_margin = ((available - max_content_width) / 2.0).max(0.0);

        egui::Frame::none()
            .inner_margin(egui::Margin { left: side_margin, right: side_margin, ..Default::default() })
            .show(ui, |ui| {

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

        }); // end centered frame

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
}
