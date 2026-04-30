//! Todo viewport: separate OS window for the todo list.
//!
//! Architecture:
//! - `TodoBridge` connects the main app thread and the deferred viewport thread.
//! - `TodoData` (RwLock): main writes, deferred reads — data + display settings.
//! - `TodoSignals` (Mutex): deferred writes, main reads — actions + window events.
//! - UI state (`TodoView`, `TitleBar`) lives in a thread-local, not shared.

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, RwLock,
};

use crate::data::todo::{Project, QueuedTask, TodoItem, Workspace};
use crate::ui::theme::Theme;

use super::titlebar::{TitleBar, TitleBarButton};
use super::todo_view::{TodoAction, TodoView};

// ── Window handle ─────────────────────────────────────────────────

/// Manages the todo viewport (separate window).
pub struct TodoWindow {
    pub is_open: bool,
    pub viewport_id: egui::ViewportId,
}

impl Default for TodoWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl TodoWindow {
    pub fn new() -> Self {
        Self {
            is_open: false,
            viewport_id: egui::ViewportId::from_hash_of("todo_viewport"),
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

// ── Bridge: main thread ↔ deferred viewport thread ───────────────

/// Data flowing from main app → todo viewport.
/// Main thread writes, deferred thread reads.
/// Uses Arc for vectors so RenderCache can share without cloning.
pub struct TodoData {
    pub workspaces: Arc<Vec<Workspace>>,
    pub current_workspace_id: i64,
    pub projects: Arc<Vec<Project>>,
    pub todos: Arc<Vec<TodoItem>>,
    pub queue: Vec<QueuedTask>,
    pub show_completed: bool,
    pub theme: Theme,
    pub is_always_on_top: bool,
    /// Set to `true` at init; main resets to `false` after first refresh.
    pub needs_refresh: bool,
    generation: u64,
}

impl TodoData {
    /// Get mutable access to workspaces, making a unique copy if shared.
    pub fn workspaces_mut(&mut self) -> &mut Vec<Workspace> {
        Arc::make_mut(&mut self.workspaces)
    }
    /// Get mutable access to projects, making a unique copy if shared.
    pub fn projects_mut(&mut self) -> &mut Vec<Project> {
        Arc::make_mut(&mut self.projects)
    }
    /// Get mutable access to todos, making a unique copy if shared.
    pub fn todos_mut(&mut self) -> &mut Vec<TodoItem> {
        Arc::make_mut(&mut self.todos)
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn bump_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }
}

/// Signals flowing from todo viewport → main app.
/// Deferred thread writes, main thread reads and drains.
#[derive(Default)]
pub struct TodoSignals {
    pub pending_actions: Vec<TodoAction>,
    pub should_close: bool,
    /// Deferred thread toggled always-on-top; main should flip its state.
    pub always_on_top_toggled: bool,
    pub last_window_pos: Option<egui::Pos2>,
    pub last_window_size: Option<egui::Vec2>,
}

/// Bridge between the main app and the todo deferred viewport.
///
/// Replaces the monolithic `SharedTodoState` with separate read/write channels
/// so the deferred viewport only holds a read lock during rendering.
pub struct TodoBridge {
    /// Main → deferred (RwLock: main writes data, deferred reads)
    pub data: RwLock<TodoData>,
    /// Deferred → main (Mutex: brief locks for signal exchange)
    pub signals: Mutex<TodoSignals>,
    /// Whether DWM effects have been applied (atomic, no lock needed)
    pub dwm_applied: AtomicBool,
    /// Parent viewport context — used to wake the main event loop when
    /// the deferred viewport produces actions (so they are processed promptly
    /// even when the main window is hidden to tray).
    pub parent_ctx: Mutex<Option<egui::Context>>,
}

pub type SharedTodo = Arc<TodoBridge>;

pub fn new_shared_todo(theme: Theme) -> SharedTodo {
    Arc::new(TodoBridge {
        data: RwLock::new(TodoData {
            workspaces: Arc::new(Vec::new()),
            current_workspace_id: 0,
            projects: Arc::new(Vec::new()),
            todos: Arc::new(Vec::new()),
            queue: Vec::new(),
            show_completed: true,
            theme,
            is_always_on_top: false,
            needs_refresh: true,
            generation: 0,
        }),
        signals: Mutex::new(TodoSignals::default()),
        dwm_applied: AtomicBool::new(false),
        parent_ctx: Mutex::new(None),
    })
}

// ── Deferred viewport rendering ───────────────────────────────────

/// Thread-local UI state for the deferred viewport (NOT shared across threads).
struct ViewportUiState {
    todo_view: TodoView,
    titlebar: TitleBar,
}

/// Cached snapshot of todo data, shared via Arc (cheap clone on generation change).
struct RenderCache {
    generation: u64,
    workspaces: Arc<Vec<Workspace>>,
    projects: Arc<Vec<Project>>,
    todos: Arc<Vec<TodoItem>>,
}

thread_local! {
    static VIEWPORT_UI: std::cell::RefCell<Option<ViewportUiState>> = const { std::cell::RefCell::new(None) };
    static RENDER_CACHE: std::cell::RefCell<Option<RenderCache>> = const { std::cell::RefCell::new(None) };
}

/// Snapshot of data read under a brief read lock.
struct DataSnapshot {
    theme: Theme,
    is_always_on_top: bool,
    ws_id: i64,
    show_completed: bool,
}

/// Render the todo viewport content. Called from the deferred viewport callback.
pub fn render_todo_viewport(ctx: &egui::Context, bridge: &TodoBridge) {
    // ── 1. Read data snapshot (brief read lock) ──────────────────
    let snapshot = {
        let Ok(data) = bridge.data.read() else {
            return;
        };

        // Update render cache only when data generation changed
        let gen = data.generation();
        RENDER_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();
            if cache.as_ref().is_none_or(|c| c.generation != gen) {
                *cache = Some(RenderCache {
                    generation: gen,
                    workspaces: Arc::clone(&data.workspaces),
                    projects: Arc::clone(&data.projects),
                    todos: Arc::clone(&data.todos),
                });
            }
        });

        DataSnapshot {
            theme: data.theme.clone(),
            is_always_on_top: data.is_always_on_top,
            ws_id: data.current_workspace_id,
            show_completed: data.show_completed,
        }
    }; // ← read lock released

    // ── 2. Track window position/size (no lock needed) ───────────
    let (window_pos, window_size) = ctx.input(|i| {
        (
            i.viewport().outer_rect.map(|r| r.min),
            i.viewport().inner_rect.map(|r| r.size()),
        )
    });

    // ── 3. DWM effects (atomic flag, no lock) ────────────────────
    #[cfg(windows)]
    if !bridge.dwm_applied.load(Ordering::Relaxed) {
        bridge.dwm_applied.store(true, Ordering::Relaxed);
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_millis(100));
            unsafe {
                use windows::core::PCWSTR;
                use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
                let title: Vec<u16> = "PomodoRust - Todo\0".encode_utf16().collect();
                if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) {
                    if !hwnd.is_invalid() {
                        crate::platform::apply_window_effects(hwnd.0 as isize);
                    }
                }
            }
        });
    }

    // ── 4. Apply theme and check viewport state ──────────────────
    snapshot.theme.apply(ctx);

    let close_requested = ctx.input(|i| i.viewport().close_requested());
    if close_requested {
        if let Ok(mut sig) = bridge.signals.lock() {
            sig.should_close = true;
            sig.last_window_pos = window_pos;
            sig.last_window_size = window_size;
        }
        return;
    }

    let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

    // ── 5. Handle resize zones ───────────────────────────────────
    if !is_maximized {
        handle_resize_zones(ctx);
    }

    // ── 6. Render UI with thread-local state (NO shared lock) ────
    let mut collected_actions = Vec::new();
    let mut should_close = false;
    let mut aot_toggled = false;

    egui::CentralPanel::default()
        .frame(
            egui::Frame::none()
                .fill(snapshot.theme.bg_primary)
                .rounding(if is_maximized {
                    egui::Rounding::ZERO
                } else {
                    snapshot.theme.window_rounding()
                }),
        )
        .show(ctx, |ui| {
            VIEWPORT_UI.with(|ui_state| {
                let mut ui_state = ui_state.borrow_mut();
                let vui = ui_state.get_or_insert_with(|| ViewportUiState {
                    todo_view: TodoView::new(),
                    titlebar: TitleBar::with_id("todo"),
                });

                // Titlebar
                let (drag, button) =
                    vui.titlebar
                        .show(ui, &snapshot.theme, is_maximized, snapshot.is_always_on_top);

                if drag {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }
                if let Some(button) = button {
                    match button {
                        TitleBarButton::Close => {
                            should_close = true;
                        }
                        TitleBarButton::Minimize => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        TitleBarButton::Maximize => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                        }
                        TitleBarButton::AlwaysOnTop => {
                            aot_toggled = true;
                            // Apply immediately to this viewport
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                if !snapshot.is_always_on_top {
                                    egui::WindowLevel::AlwaysOnTop
                                } else {
                                    egui::WindowLevel::Normal
                                },
                            ));
                        }
                    }
                }

                // Content
                let actions = RENDER_CACHE.with(|cache| {
                    let cache = cache.borrow();
                    let Some(c) = cache.as_ref() else {
                        return Vec::new();
                    };
                    let mut actions = Vec::new();
                    egui::Frame::none()
                        .inner_margin(egui::Margin::symmetric(
                            snapshot.theme.spacing_md,
                            snapshot.theme.spacing_sm,
                        ))
                        .show(ui, |ui| {
                            actions = vui.todo_view.show_inner(
                                ui,
                                &snapshot.theme,
                                &c.workspaces,
                                &c.projects,
                                &c.todos,
                                snapshot.ws_id,
                                snapshot.show_completed,
                            );
                        });
                    actions
                });

                collected_actions.extend(actions);
            });
        });

    // ── 7. Push signals back (brief mutex lock) ──────────────────
    let has_signals = !collected_actions.is_empty() || should_close || aot_toggled;
    if let Ok(mut sig) = bridge.signals.lock() {
        sig.pending_actions.extend(collected_actions);
        sig.should_close |= should_close;
        if aot_toggled {
            sig.always_on_top_toggled = true;
        }
        sig.last_window_pos = window_pos;
        sig.last_window_size = window_size;
    }

    // ── 8. Wake parent viewport so it processes actions promptly ──
    if has_signals {
        if let Ok(guard) = bridge.parent_ctx.lock() {
            if let Some(ref pctx) = *guard {
                pctx.request_repaint();
            }
        }
    }
}

/// Handle window resize zones for custom decorated window (no system frame).
fn handle_resize_zones(ctx: &egui::Context) {
    let resize_margin = 8.0;
    let screen_rect = ctx.screen_rect();

    let (hover_pos, primary_pressed) =
        ctx.input(|input| (input.pointer.hover_pos(), input.pointer.primary_pressed()));

    let Some(pos) = hover_pos else { return };

    let left = pos.x < screen_rect.left() + resize_margin;
    let right = pos.x > screen_rect.right() - resize_margin;
    let top = pos.y < screen_rect.top() + resize_margin;
    let bottom = pos.y > screen_rect.bottom() - resize_margin;

    let resize_direction = match (left, right, top, bottom) {
        (true, false, true, false) => Some(egui::ResizeDirection::NorthWest),
        (false, true, true, false) => Some(egui::ResizeDirection::NorthEast),
        (true, false, false, true) => Some(egui::ResizeDirection::SouthWest),
        (false, true, false, true) => Some(egui::ResizeDirection::SouthEast),
        (true, false, false, false) => Some(egui::ResizeDirection::West),
        (false, true, false, false) => Some(egui::ResizeDirection::East),
        (false, false, true, false) => Some(egui::ResizeDirection::North),
        (false, false, false, true) => Some(egui::ResizeDirection::South),
        _ => None,
    };

    if let Some(dir) = resize_direction {
        let cursor = match dir {
            egui::ResizeDirection::North | egui::ResizeDirection::South => {
                egui::CursorIcon::ResizeVertical
            }
            egui::ResizeDirection::East | egui::ResizeDirection::West => {
                egui::CursorIcon::ResizeHorizontal
            }
            egui::ResizeDirection::NorthWest | egui::ResizeDirection::SouthEast => {
                egui::CursorIcon::ResizeNwSe
            }
            egui::ResizeDirection::NorthEast | egui::ResizeDirection::SouthWest => {
                egui::CursorIcon::ResizeNeSw
            }
        };
        ctx.set_cursor_icon(cursor);

        if primary_pressed {
            ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(dir));
        }
    }
}
