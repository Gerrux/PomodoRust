mod helpers;
mod item;

use egui::{self, RichText, Sense, Vec2};

use crate::data::todo::{Priority, Project, TodoItem, Workspace};
use crate::ui::components::{draw_icon, Icon};
use crate::ui::theme::Theme;

pub use helpers::truncate_text;

/// Actions produced by the todo UI
#[derive(Debug, Clone)]
pub enum TodoAction {
    // Workspaces
    CreateWorkspace {
        name: String,
    },
    RenameWorkspace {
        id: i64,
        name: String,
    },
    DeleteWorkspace {
        id: i64,
    },
    SwitchWorkspace {
        id: i64,
    },

    // Projects
    CreateProject {
        workspace_id: i64,
        name: String,
    },
    RenameProject {
        id: i64,
        name: String,
    },
    DeleteProject {
        id: i64,
    },
    ToggleProjectCollapse {
        id: i64,
    },

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
    ToggleComplete {
        id: i64,
    },
    ToggleCollapse {
        id: i64,
    },
    DeleteTodo {
        id: i64,
    },
    MoveTodo {
        id: i64,
        project_id: Option<i64>,
    },
    ReorderTodo {
        id: i64,
        project_id: Option<i64>,
        new_position: i32,
    },
    SetPriority {
        id: i64,
        priority: Priority,
    },

    // Queue
    AddToQueue {
        todo_id: i64,
        planned_pomodoros: u32,
    },
    RemoveFromQueue {
        id: i64,
    },
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

    // Todo item popup (rendered outside DnD to avoid click conflicts)
    popup_todo_id: Option<i64>,
    popup_dots_rect: Option<egui::Rect>,
}

impl Default for TodoView {
    fn default() -> Self {
        Self::new()
    }
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
            popup_todo_id: None,
            popup_dots_rect: None,
        }
    }

    /// Show the todo content inside an egui::Window (no titlebar/CentralPanel needed)
    #[allow(clippy::too_many_arguments)]
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
        let t = crate::i18n::tr();

        // Workspace tabs (full width)
        actions.extend(self.render_workspace_tabs(ui, theme, workspaces, current_workspace_id));

        ui.add_space(theme.spacing_xs);

        // Centered content with max width
        let max_content_width = 720.0;
        let available = ui.available_width();
        let side_margin = ((available - max_content_width) / 2.0).max(0.0);
        let center_margin = egui::Margin {
            left: side_margin,
            right: side_margin,
            ..Default::default()
        };

        // Filter bar (inside centering frame)
        egui::Frame::none()
            .inner_margin(center_margin)
            .show(ui, |ui| {
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
                            let ir =
                                egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(10.0));
                            draw_icon(ui, chevron, ir, theme.text_muted);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(format!(
                                        "{} ({})",
                                        t.todo.completed_label, completed_count
                                    ))
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
            });

        ui.add_space(theme.spacing_sm);

        // Task list — ScrollArea OUTSIDE centering Frame so available_height() is correct
        let scroll_height = ui.available_height();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(scroll_height)
            .show(ui, |ui| {
                egui::Frame::none()
                    .inner_margin(center_margin)
                    .show(ui, |ui| {
                        actions.extend(self.render_task_list_content(
                            ui,
                            theme,
                            projects,
                            todos,
                            current_workspace_id,
                            show_completed,
                        ));
                    });
            });

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
                .enable_scrolling(false)
                .show(ui, |ui| {
                    for ws in workspaces {
                        // Renaming mode
                        if self.renaming_workspace_id == Some(ws.id) {
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut self.rename_workspace_buffer)
                                    .desired_width(80.0)
                                    .font(egui::FontId::proportional(13.0)),
                            );
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

                        let text = RichText::new(&label).size(14.0).color(if is_active {
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
                            if ui.button(crate::i18n::tr().todo.rename).clicked() {
                                self.renaming_workspace_id = Some(ws.id);
                                self.rename_workspace_buffer = ws.name.clone();
                                ui.close_menu();
                            }
                            if workspaces.len() > 1
                                && ui.button(crate::i18n::tr().todo.delete).clicked()
                            {
                                actions.push(TodoAction::DeleteWorkspace { id: ws.id });
                                ui.close_menu();
                            }
                        });

                        ui.add_space(theme.spacing_sm);
                    }
                });

            // Add workspace button / input
            if self.adding_workspace {
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.new_workspace_name)
                        .desired_width(80.0)
                        .hint_text(crate::i18n::tr().todo.name_hint)
                        .font(egui::FontId::proportional(13.0)),
                );
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
                plus_resp.on_hover_text(crate::i18n::tr().todo.new_workspace);
            }
        });

        actions
    }

    /// Render task list content (projects, unassigned todos, new task input).
    /// Called inside a ScrollArea — do NOT add another ScrollArea here.
    fn render_task_list_content(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        projects: &[Project],
        todos: &[TodoItem],
        current_workspace_id: i64,
        show_completed: bool,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();
        let t = crate::i18n::tr();

        // Projects with their todos
        for project in projects {
            actions.extend(self.render_project(
                ui,
                theme,
                project,
                todos,
                projects,
                show_completed,
            ));
        }

        // Unassigned todos
        let unassigned: Vec<&TodoItem> = todos
            .iter()
            .filter(|todo| todo.project_id.is_none())
            .filter(|todo| show_completed || !todo.completed)
            .collect();

        if !unassigned.is_empty() || projects.is_empty() {
            if !projects.is_empty() && !unassigned.is_empty() {
                ui.add_space(theme.spacing_md);
                ui.horizontal(|ui| {
                    let line_color = theme.border_subtle;
                    let rect = ui.available_rect_before_wrap();
                    let y = rect.center().y;
                    let text_response = ui.label(
                        RichText::new(t.todo.miscellaneous)
                            .size(13.0)
                            .color(theme.text_muted),
                    );
                    let after_text = text_response.rect.right() + 8.0;
                    ui.painter().line_segment(
                        [egui::pos2(after_text, y), egui::pos2(rect.right(), y)],
                        egui::Stroke::new(1.0, line_color),
                    );
                });
                ui.add_space(6.0);
            }

            if !unassigned.is_empty() {
                // DnD for unassigned todos
                actions.extend(self.render_dnd_todo_list(ui, theme, &unassigned, projects, None));
            }
        }

        // Empty state
        if todos.is_empty() && projects.is_empty() {
            ui.add_space(theme.spacing_xl);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(t.todo.no_tasks)
                        .size(16.0)
                        .color(theme.text_muted),
                );
                ui.add_space(theme.spacing_sm);
                ui.label(
                    RichText::new(t.todo.create_first_task)
                        .size(13.0)
                        .color(theme.text_muted),
                );
            });
        }

        ui.add_space(theme.spacing_sm);

        // New task input
        actions.extend(self.render_new_task_input(ui, theme, current_workspace_id, None));

        ui.add_space(theme.spacing_xs);

        // Add project button
        if self.adding_project_workspace == Some(current_workspace_id) {
            ui.horizontal(|ui| {
                ui.label(RichText::new("+").size(14.0).color(theme.text_muted));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.new_project_name)
                        .hint_text(crate::i18n::tr().todo.project_name_hint)
                        .desired_width(ui.available_width() - 10.0)
                        .font(egui::FontId::proportional(13.0))
                        .frame(false),
                );
                if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
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
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(16.0, 18.0), Sense::hover());
                let ir = egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(12.0));
                draw_icon(ui, Icon::CirclePlus, ir, theme.text_muted);
                ui.add(
                    egui::Label::new(
                        RichText::new(t.todo.project)
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

        actions
    }

    /// Render a sortable list of todos using egui_dnd.
    /// `project_id` is None for unassigned todos.
    fn render_dnd_todo_list(
        &mut self,
        ui: &mut egui::Ui,
        theme: &Theme,
        todo_list: &[&TodoItem],
        projects: &[Project],
        project_id: Option<i64>,
    ) -> Vec<TodoAction> {
        let mut actions = Vec::new();

        // Build lightweight DnD items with stable ids
        let mut dnd_items: Vec<DndTodoItem> = todo_list
            .iter()
            .map(|t| DndTodoItem {
                todo_id: t.id,
                position: t.position,
            })
            .collect();

        let dnd_id = match project_id {
            Some(pid) => ("todo_dnd_project", pid),
            None => ("todo_dnd_unassigned", 0),
        };

        let response =
            egui_dnd::dnd(ui, dnd_id).show_vec(&mut dnd_items, |ui, dnd_item, handle, _state| {
                // Find the full todo item for rendering
                if let Some(todo) = todo_list.iter().find(|t| t.id == dnd_item.todo_id) {
                    actions.extend(self.render_todo_item(ui, theme, todo, projects, handle));
                }
            });

        // On drag finished, emit reorder actions
        if response.final_update().is_some() {
            for (new_pos, dnd_item) in dnd_items.iter().enumerate() {
                let original = todo_list.iter().find(|t| t.id == dnd_item.todo_id);
                if let Some(todo) = original {
                    if new_pos as i32 != todo.position {
                        actions.push(TodoAction::ReorderTodo {
                            id: dnd_item.todo_id,
                            project_id,
                            new_position: new_pos as i32,
                        });
                    }
                }
            }
        }

        // Render todo popup OUTSIDE DnD to avoid click conflicts
        if let Some(popup_id) = self.popup_todo_id {
            if let Some(todo) = todo_list.iter().find(|t| t.id == popup_id) {
                self.render_todo_popup_area(ui, theme, todo, projects, &mut actions);
            }
        }

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

        // Subtle dashed-border card for new task input
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(4.0, 3.0))
            .rounding(theme.rounding_sm)
            .stroke(egui::Stroke::new(1.0, theme.border_subtle))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Plus icon
                    let (icon_rect, _) =
                        ui.allocate_exact_size(Vec2::new(14.0, 20.0), Sense::hover());
                    let ir = egui::Rect::from_center_size(icon_rect.center(), Vec2::splat(11.0));
                    draw_icon(
                        ui,
                        Icon::Plus,
                        ir,
                        theme.accent.solid().linear_multiply(0.7),
                    );

                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.new_task_title)
                            .hint_text(crate::i18n::tr().todo.new_task_hint)
                            .desired_width(ui.available_width())
                            .font(egui::FontId::proportional(13.0))
                            .frame(false),
                    );

                    if self.focus_new_task {
                        response.request_focus();
                        self.focus_new_task = false;
                    }

                    let submitted =
                        response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
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
            });

        actions
    }
}

/// Lightweight struct for DnD tracking — only needs id + position for reorder.
#[derive(Hash)]
struct DndTodoItem {
    todo_id: i64,
    position: i32,
}
