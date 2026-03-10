use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::data::todo::{Priority, Project, QueuedTask, TodoItem};
use crate::ui::todo_view::TodoAction;
use crate::ui::todo_window::render_todo_viewport;

use super::PomodoRustApp;

impl PomodoRustApp {
    /// Refresh todo data from database into shared state
    pub(super) fn refresh_todo_data(&mut self) {
        let Some(db) = &self.database else { return };

        if let Ok(mut state) = self.shared_todo.data.write() {
            state.workspaces = Arc::new(db.get_workspaces().unwrap_or_default());

            // Set current workspace if not set or invalid
            if state.current_workspace_id == 0
                || !state
                    .workspaces
                    .iter()
                    .any(|w| w.id == state.current_workspace_id)
            {
                if let Some(saved_id) = self.config.todo.last_workspace_id {
                    if state.workspaces.iter().any(|w| w.id == saved_id) {
                        state.current_workspace_id = saved_id;
                    } else if let Some(first) = state.workspaces.first() {
                        state.current_workspace_id = first.id;
                    }
                } else if let Some(first) = state.workspaces.first() {
                    state.current_workspace_id = first.id;
                }
            }

            let ws_id = state.current_workspace_id;
            state.projects = Arc::new(db.get_projects(ws_id).unwrap_or_default());
            state.todos = Arc::new(db.get_todos(ws_id).unwrap_or_default());
            state.queue = db.get_queue().unwrap_or_default();
            state.needs_refresh = false;
            state.bump_generation();
        }
    }

    /// Handle todo action from the todo viewport.
    /// Returns `true` if a full refresh is needed (deferred for batching).
    pub(super) fn handle_todo_action(&mut self, action: TodoAction) -> bool {
        let Some(db) = &self.database else { return false };

        // Helper macro to log DB errors instead of silently ignoring them
        macro_rules! db_op {
            ($expr:expr, $op:literal) => {
                if let Err(e) = $expr {
                    tracing::warn!("DB {}: {e}", $op);
                }
            };
        }
        // Helper macro: DB operation that returns a value, None on error
        macro_rules! db_try {
            ($expr:expr, $op:literal) => {
                match $expr {
                    Ok(v) => Some(v),
                    Err(e) => {
                        tracing::warn!("DB {}: {e}", $op);
                        None
                    }
                }
            };
        }

        match action {
            // ── Fast-path: no refresh needed ──────────────────────────

            TodoAction::ToggleShowCompleted => {
                if let Ok(mut state) = self.shared_todo.data.write() {
                    state.show_completed = !state.show_completed;
                    state.bump_generation();
                }
                self.config.todo.show_completed = !self.config.todo.show_completed;
                db_op!(self.config.save(), "save_config");
                false
            }
            TodoAction::Close => {
                self.todo_window.close();
                false
            }
            TodoAction::ToggleProjectCollapse { id } => {
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if let Some(proj) = state.projects_mut().iter_mut().find(|p| p.id == id) {
                        proj.collapsed = !proj.collapsed;
                        db_op!(db.update_project(proj), "update_project");
                        state.bump_generation();
                    }
                }
                false
            }
            TodoAction::ToggleComplete { id } => {
                db_op!(db.toggle_todo(id), "toggle_todo");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    let mut became_completed = false;
                    if let Some(todo) = state.todos_mut().iter_mut().find(|t| t.id == id) {
                        todo.completed = !todo.completed;
                        todo.completed_at = if todo.completed {
                            became_completed = true;
                            Some(chrono::Utc::now())
                        } else {
                            None
                        };
                    }
                    if became_completed {
                        if let Some(queue_item) = state.queue.iter().find(|q| q.todo_id == id) {
                            let queue_id = queue_item.id;
                            db_op!(db.remove_from_queue(queue_id), "remove_from_queue");
                            state.queue.retain(|q| q.id != queue_id);
                        }
                    }
                    state.bump_generation();
                }
                false
            }
            TodoAction::ToggleCollapse { id } => {
                db_op!(db.toggle_todo_collapsed(id), "toggle_collapsed");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if let Some(todo) = state.todos_mut().iter_mut().find(|t| t.id == id) {
                        todo.collapsed = !todo.collapsed;
                    }
                    state.bump_generation();
                }
                false
            }
            TodoAction::SetPriority { id, priority } => {
                db_op!(db.set_todo_priority(id, priority), "set_priority");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if let Some(todo) = state.todos_mut().iter_mut().find(|t| t.id == id) {
                        todo.priority = priority;
                    }
                    state.bump_generation();
                }
                false
            }
            TodoAction::RenameWorkspace { id, name } => {
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if let Some(ws) = state.workspaces_mut().iter_mut().find(|w| w.id == id) {
                        ws.name = name;
                        db_op!(db.update_workspace(ws), "update_workspace");
                        state.bump_generation();
                    }
                }
                false
            }
            TodoAction::RenameProject { id, name } => {
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if let Some(proj) = state.projects_mut().iter_mut().find(|p| p.id == id) {
                        proj.name = name;
                        db_op!(db.update_project(proj), "update_project");
                        state.bump_generation();
                    }
                }
                false
            }
            TodoAction::UpdateTodo { id, title, body } => {
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if let Some(todo) = state.todos_mut().iter_mut().find(|t| t.id == id) {
                        todo.title = title;
                        todo.body = body;
                        db_op!(db.update_todo(todo), "update_todo");
                        state.bump_generation();
                    }
                }
                false
            }
            TodoAction::MoveTodo { id, project_id } => {
                db_op!(db.move_todo(id, project_id), "move_todo");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if let Some(todo) = state.todos_mut().iter_mut().find(|t| t.id == id) {
                        todo.project_id = project_id;
                    }
                    state.bump_generation();
                }
                false
            }
            // Reorder: needs full refresh to get correct sort order from DB
            TodoAction::ReorderTodo { id, project_id, new_position } => {
                db_op!(db.reorder_todo_to(id, project_id, new_position), "reorder_todo");
                true
            }

            // ── Fast-path: create todo with cache insert ─────────────

            TodoAction::CreateTodo { workspace_id, project_id, title } => {
                if let Some(new_id) = db_try!(db.create_todo(workspace_id, project_id, &title), "create_todo") {
                    if let Ok(mut state) = self.shared_todo.data.write() {
                        let max_pos = state.todos.iter()
                            .filter(|t| t.workspace_id == workspace_id)
                            .map(|t| t.position)
                            .max()
                            .unwrap_or(-1);
                        state.todos_mut().push(TodoItem {
                            id: new_id,
                            project_id,
                            workspace_id,
                            title,
                            body: None,
                            completed: false,
                            collapsed: false,
                            priority: Priority::None,
                            position: max_pos + 1,
                            created_at: chrono::Utc::now(),
                            completed_at: None,
                        });
                        state.bump_generation();
                    }
                }
                false
            }
            TodoAction::CreateTodoWithBody { workspace_id, project_id, title, body } => {
                if let Some(new_id) = db_try!(db.create_todo_with_body(workspace_id, project_id, &title, &body), "create_todo_with_body") {
                    if let Ok(mut state) = self.shared_todo.data.write() {
                        let max_pos = state.todos.iter()
                            .filter(|t| t.workspace_id == workspace_id)
                            .map(|t| t.position)
                            .max()
                            .unwrap_or(-1);
                        state.todos_mut().push(TodoItem {
                            id: new_id,
                            project_id,
                            workspace_id,
                            title,
                            body: Some(body),
                            completed: false,
                            collapsed: false,
                            priority: Priority::None,
                            position: max_pos + 1,
                            created_at: chrono::Utc::now(),
                            completed_at: None,
                        });
                        state.bump_generation();
                    }
                }
                false
            }

            // ── Fast-path: delete todo with cache removal ────────────

            TodoAction::DeleteTodo { id } => {
                db_op!(db.delete_todo(id), "delete_todo");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    state.todos_mut().retain(|t| t.id != id);
                    state.queue.retain(|q| q.todo_id != id);
                    state.bump_generation();
                }
                false
            }

            // ── Fast-path: queue operations with cache update ────────

            TodoAction::AddToQueue { todo_id, planned_pomodoros } => {
                if let Some(new_id) = db_try!(db.add_to_queue(todo_id, planned_pomodoros), "add_to_queue") {
                    if new_id > 0 {
                        if let Ok(mut state) = self.shared_todo.data.write() {
                            let title = state.todos.iter()
                                .find(|t| t.id == todo_id)
                                .map(|t| t.title.clone())
                                .unwrap_or_default();
                            let max_pos = state.queue.iter()
                                .map(|q| q.position)
                                .max()
                                .unwrap_or(-1);
                            state.queue.push(QueuedTask {
                                id: new_id,
                                todo_id,
                                title,
                                planned_pomodoros,
                                completed_pomodoros: 0,
                                position: max_pos + 1,
                            });
                            state.bump_generation();
                        }
                    }
                }
                false
            }
            TodoAction::RemoveFromQueue { id } => {
                db_op!(db.remove_from_queue(id), "remove_from_queue");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    state.queue.retain(|q| q.id != id);
                    state.bump_generation();
                }
                false
            }
            TodoAction::ClearQueue => {
                db_op!(db.clear_queue(), "clear_queue");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    state.queue.clear();
                    state.bump_generation();
                }
                false
            }

            // ── Fast-path: create project with cache insert ──────────

            TodoAction::CreateProject { workspace_id, name } => {
                if let Some(new_id) = db_try!(db.create_project(workspace_id, &name, None), "create_project") {
                    if let Ok(mut state) = self.shared_todo.data.write() {
                        let max_pos = state.projects.iter()
                            .filter(|p| p.workspace_id == workspace_id)
                            .map(|p| p.position)
                            .max()
                            .unwrap_or(-1);
                        state.projects_mut().push(Project {
                            id: new_id,
                            workspace_id,
                            name,
                            color: None,
                            collapsed: false,
                            position: max_pos + 1,
                        });
                        state.bump_generation();
                    }
                }
                false
            }
            TodoAction::DeleteProject { id } => {
                db_op!(db.delete_project(id), "delete_project");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    state.projects_mut().retain(|p| p.id != id);
                    // Unassign todos from deleted project
                    for todo in state.todos_mut().iter_mut() {
                        if todo.project_id == Some(id) {
                            todo.project_id = None;
                        }
                    }
                    state.bump_generation();
                }
                false
            }

            // ── Slow-path: structural changes needing full refresh ───

            TodoAction::CreateWorkspace { name } => {
                db_op!(db.create_workspace(&name, None, None), "create_workspace");
                true // need refresh to get all workspaces
            }
            TodoAction::DeleteWorkspace { id } => {
                db_op!(db.delete_workspace(id), "delete_workspace");
                if let Ok(mut state) = self.shared_todo.data.write() {
                    if state.current_workspace_id == id {
                        state.current_workspace_id = state
                            .workspaces
                            .iter()
                            .find(|w| w.id != id)
                            .map(|w| w.id)
                            .unwrap_or(0);
                    }
                }
                true // workspace deletion cascades, need full refresh
            }
            TodoAction::SwitchWorkspace { id } => {
                if let Ok(mut state) = self.shared_todo.data.write() {
                    state.current_workspace_id = id;
                }
                self.config.todo.last_workspace_id = Some(id);
                db_op!(self.config.save(), "save_config");
                true // need to load projects/todos for new workspace
            }
        }
    }

    /// Show the todo as a separate OS window using show_viewport_deferred.
    pub(super) fn show_todo_viewport(&mut self, ctx: &egui::Context) {
        // ── 0. Ensure parent context is set for wakeup ────────────
        if let Ok(mut guard) = self.shared_todo.parent_ctx.lock() {
            if guard.is_none() {
                *guard = Some(ctx.clone());
            }
        }

        // ── 1. Sync theme to data only when changed ──────────────
        if self.todo_theme_dirty {
            if let Ok(mut data) = self.shared_todo.data.write() {
                data.theme = self.theme.clone();
            }
            self.todo_theme_dirty = false;
        }

        // ── 2. Read data flags (brief read lock) ──────────────────
        let (needs_refresh, todo_pinned) = {
            let Ok(data) = self.shared_todo.data.read() else {
                return;
            };
            (data.needs_refresh, data.is_always_on_top)
        };

        // ── 3. Drain signals (brief mutex lock) ──────────────────
        let (should_close, aot_toggled, actions) = {
            let Ok(mut sig) = self.shared_todo.signals.lock() else {
                return;
            };
            let actions: Vec<TodoAction> = sig.pending_actions.drain(..).collect();
            let sc = sig.should_close;
            let aot = sig.always_on_top_toggled;
            if sc {
                sig.should_close = false;
                self.shared_todo.dwm_applied.store(false, Ordering::Relaxed);
            }
            sig.always_on_top_toggled = false;
            (sc, aot, actions)
        };

        if needs_refresh {
            self.refresh_todo_data();
        }

        if should_close {
            self.todo_window.close();
        }

        // ── 4. Handle AOT toggle from todo viewport ──────────────
        if aot_toggled {
            self.set_always_on_top(!todo_pinned, ctx);
        }

        // Process pending actions (batch: single refresh for all structural actions)
        if !actions.is_empty() {
            let mut needs_refresh = false;
            for action in actions {
                needs_refresh |= self.handle_todo_action(action);
            }
            if needs_refresh {
                self.refresh_todo_data();
            }
            ctx.request_repaint();
        }

        // Show viewport
        let todo_config = &self.config.todo;
        let shared = self.shared_todo.clone();

        // Always set window_level explicitly to avoid egui builder patch issues.
        // When window_level is None (not set), the patch() diff skips the update,
        // leaving the window stuck in its previous z-order state. This also avoids
        // repeated DWM SetWindowPos calls that can freeze transparent windows on Windows.
        let mut todo_vp = egui::ViewportBuilder::default()
            .with_title("PomodoRust - Todo")
            .with_inner_size([todo_config.window_width, todo_config.window_height])
            .with_min_inner_size([320.0, 400.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(true)
            .with_window_level(if todo_pinned {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            });
        if let (Some(x), Some(y)) = (todo_config.window_x, todo_config.window_y) {
            todo_vp = todo_vp.with_position([x, y]);
        }
        ctx.show_viewport_deferred(
            self.todo_window.viewport_id,
            todo_vp,
            move |ctx, _class| {
                render_todo_viewport(ctx, &shared);
            },
        );
    }
}
