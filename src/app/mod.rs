//! Main application struct and logic

mod actions;
mod queue_view;
mod system;
mod todo_handler;

use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;

use chrono::Utc;

use crate::core::{Session, TimerEvent};
use crate::data::{Config, Database, Statistics};
use crate::ipc::{IpcCommand, IpcServer};
use crate::platform::{AudioPlayer, HotkeyAction, HotkeyManager, SystemTray};
use crate::ui::{
    animations::AnimationState,
    settings::{SettingsAction, SettingsView},
    stats::StatsView,
    theme::Theme,
    timer_view::{TimerAction, TimerView},
    titlebar::{TitleBar, TitleBarButton},
    todo_window::{new_shared_todo, SharedTodo, TodoWindow},
};

/// Application view state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Timer,
    Queue,
    Stats,
    Settings,
}

/// Main application struct
pub struct PomodoRustApp {
    // Core state
    session: Session,
    config: Config,
    theme: Theme,

    // Data
    database: Option<Database>,
    statistics: Statistics,

    // UI components
    titlebar: TitleBar,
    timer_view: TimerView,
    stats_view: StatsView,
    settings_view: Option<SettingsView>,

    // Animation state
    animations: AnimationState,

    // Current view
    current_view: View,

    // Audio
    audio: Option<AudioPlayer>,

    // Session tracking
    session_start_time: Option<chrono::DateTime<Utc>>,

    // IPC for CLI integration
    ipc_server: IpcServer,
    ipc_receiver: Option<Receiver<IpcCommand>>,

    // Global hotkeys (manager kept alive to maintain registrations)
    #[allow(dead_code)]
    hotkey_manager: HotkeyManager,
    hotkey_receiver: Option<Receiver<HotkeyAction>>,

    // Window state tracking for persistence
    last_window_pos: Option<egui::Pos2>,
    last_window_size: Option<egui::Vec2>,
    last_window_maximized: bool,

    // Toast notifications
    toasts: egui_notify::Toasts,

    // Todo
    todo_window: TodoWindow,
    shared_todo: SharedTodo,
    todo_theme_dirty: bool,

    // System tray
    system_tray: Option<SystemTray>,
    hidden_to_tray: bool,

    // Close confirmation dialog
    show_close_dialog: bool,
    force_quit: bool,
}

/// Duration to show toast notifications
const TOAST_DURATION: std::time::Duration = std::time::Duration::from_secs(2);

impl PomodoRustApp {
    /// Create a new application instance with the given config and optional tray
    pub fn with_config(cc: &eframe::CreationContext<'_>, config: Config, system_tray: Option<SystemTray>) -> Self {
        Self::init(cc, config, system_tray)
    }

    /// Create a new application instance (loads config from disk)
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load();
        Self::init(cc, config, None)
    }

    /// Internal initialization with a config
    fn init(cc: &eframe::CreationContext<'_>, config: Config, system_tray: Option<SystemTray>) -> Self {
        // Setup fonts with emoji fallback
        Self::setup_fonts(&cc.egui_ctx);

        // Ensure Start Menu shortcut for Windows toast notifications
        #[cfg(windows)]
        crate::platform::ensure_notification_shortcut();

        // Create theme from config
        let mut theme =
            Theme::from_mode(config.appearance.theme_mode, config.appearance.accent_color);
        if config.accessibility.high_contrast {
            theme = theme.with_high_contrast();
        }
        if config.accessibility.reduced_motion {
            theme = theme.with_reduced_motion();
        }
        theme.apply(&cc.egui_ctx);

        // Create session with config preset
        let preset = config.to_preset();
        let mut session = Session::with_preset(preset);
        session.set_auto_start(config.timer.auto_start_breaks, config.timer.auto_start_work);

        // Initialize database
        let database = match Database::open() {
            Ok(db) => {
                tracing::info!("Database initialized");
                Some(db)
            }
            Err(e) => {
                tracing::error!("Failed to initialize database: {}", e);
                None
            }
        };

        // Load statistics
        let statistics = database
            .as_ref()
            .map(Statistics::load)
            .unwrap_or_else(Statistics::empty);

        // Initialize audio
        let mut audio = AudioPlayer::new();
        if let Some(ref mut player) = audio {
            player.set_volume(config.sounds.volume as f32 / 100.0);
        }

        // Initialize IPC server for CLI
        let mut ipc_server = IpcServer::new();
        let ipc_receiver = ipc_server.take_receiver();
        ipc_server.start();

        // Initialize global hotkeys
        let mut hotkey_manager = HotkeyManager::new();
        let hotkey_receiver = hotkey_manager.take_receiver();
        if config.hotkeys.enabled {
            hotkey_manager.start(
                &config.hotkeys.toggle,
                &config.hotkeys.skip,
                &config.hotkeys.reset,
            );
        }

        let shared_todo = new_shared_todo(theme.clone());

        let todo_auto_open = config.todo.auto_open;

        let mut app = Self {
            session,
            config,
            theme,
            database,
            statistics,
            titlebar: TitleBar::new(),
            timer_view: TimerView::new(),
            stats_view: StatsView::new(),
            settings_view: None,
            animations: AnimationState::new(),
            current_view: View::Timer,
            audio,
            session_start_time: None,
            ipc_server,
            ipc_receiver,
            hotkey_manager,
            hotkey_receiver,
            last_window_pos: None,
            last_window_size: None,
            last_window_maximized: false,
            toasts: egui_notify::Toasts::default()
                .with_anchor(egui_notify::Anchor::BottomRight)
                .with_margin(egui::vec2(10.0, 10.0)),
            todo_window: TodoWindow::new(),
            shared_todo,
            todo_theme_dirty: true,
            system_tray,
            hidden_to_tray: false,
            show_close_dialog: false,
            force_quit: false,
        };

        if todo_auto_open {
            app.todo_window.open();
        }

        // Set show_completed from config
        if let Ok(mut state) = app.shared_todo.data.write() {
            state.show_completed = app.config.todo.show_completed;
        }

        // Initial data load for todo
        app.refresh_todo_data();

        app
    }

    /// Show a success toast notification
    fn show_status(&mut self, message: impl Into<String>) {
        self.toasts.success(message.into())
            .duration(Some(TOAST_DURATION))
            .closable(true);
    }

    /// Centralized always-on-top toggle: updates config, main viewport, and todo bridge.
    fn set_always_on_top(&mut self, enabled: bool, ctx: &egui::Context) {
        self.config.window.always_on_top = enabled;
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if enabled {
            egui::WindowLevel::AlwaysOnTop
        } else {
            egui::WindowLevel::Normal
        }));
        if let Ok(mut data) = self.shared_todo.data.write() {
            data.is_always_on_top = enabled;
        }
        let _ = self.config.save();
    }
}

impl eframe::App for PomodoRustApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Transparent for rounded corners
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Track window state for persistence (only when not maximized to preserve normal size)
        ctx.input(|i| {
            let maximized = i.viewport().maximized.unwrap_or(false);
            self.last_window_maximized = maximized;

            // Only save position/size when not maximized (to preserve "normal" window state)
            if !maximized {
                if let Some(rect) = i.viewport().inner_rect {
                    self.last_window_pos = Some(rect.min);
                    self.last_window_size = Some(rect.size());
                }
            }
        });

        // Handle IPC commands from CLI
        self.handle_ipc_commands();

        // Handle global hotkey events
        self.handle_hotkey_events();

        // Handle system tray events
        self.handle_tray_events(ctx);
        self.update_tray_state();

        // Keep polling when hidden to tray
        if self.hidden_to_tray {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }

        // Intercept native close (Alt+F4, taskbar close) when tray is available
        if ctx.input(|i| i.viewport().close_requested()) && self.system_tray.is_some() && !self.force_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_close_dialog = true;
        }

        // Apply theme
        self.theme.apply(ctx);

        // Handle window resize zones (for custom decorated window)
        self.handle_resize_zones(ctx);

        // Update timer
        let (event, should_auto_start) = self.session.update();
        if let Some(TimerEvent::Completed) = event {
            self.on_timer_completed();
            if should_auto_start {
                self.session.start();
                self.session_start_time = Some(Utc::now());
            }
        }

        // Manage tick sound
        if let Some(ref mut audio) = self.audio {
            let should_tick = self.config.sounds.enabled
                && self.config.sounds.tick_enabled
                && self.session.timer().is_running();

            if should_tick && !audio.is_tick_playing() {
                audio.start_tick();
            } else if !should_tick && audio.is_tick_playing() {
                audio.stop_tick();
            }
        }

        // Update animations
        self.animations.update(self.session.timer().is_running());

        // Request continuous repaint when timer is running or animating
        if self.session.timer().is_running() || self.animations.needs_repaint() {
            ctx.request_repaint();
        }

        // Check if maximized for rounding
        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

        // Collect settings action to handle after UI
        let mut settings_action: Option<SettingsAction> = None;

        // Show toast notifications
        self.toasts.show(ctx);

        // Calculate background color with opacity
        let bg_alpha = (self.config.appearance.window_opacity as f32 / 100.0 * 255.0) as u8;
        let bg_color = egui::Color32::from_rgba_unmultiplied(
            self.theme.bg_primary.r(),
            self.theme.bg_primary.g(),
            self.theme.bg_primary.b(),
            bg_alpha,
        );

        // Main panel with custom frame - no rounding or border when maximized
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(bg_color)
                    .rounding(if is_maximized {
                        egui::Rounding::ZERO
                    } else {
                        self.theme.window_rounding()
                    }),
            )
            .show(ctx, |ui| {
                // Title bar
                let (should_drag, button) = self.titlebar.show(
                    ui,
                    &self.theme,
                    is_maximized,
                    self.config.window.always_on_top,
                );

                if should_drag {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                if let Some(button) = button {
                    match button {
                        TitleBarButton::AlwaysOnTop => {
                            self.set_always_on_top(!self.config.window.always_on_top, ctx);
                        }
                        TitleBarButton::Minimize => {
                            if self.config.system.minimize_to_tray && self.system_tray.is_some() {
                                self.hide_to_tray(ctx);
                            } else {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            }
                        }
                        TitleBarButton::Maximize => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                                !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
                            ));
                        }
                        TitleBarButton::Close => {
                            if self.system_tray.is_some() {
                                self.show_close_dialog = true;
                            } else {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }
                }

                // Content area with padding
                egui::Frame::none()
                    .inner_margin(egui::Margin::symmetric(
                        self.theme.spacing_md,
                        self.theme.spacing_sm,
                    ))
                    .show(ui, |ui| {
                        // Show current view
                        match self.current_view {
                            View::Timer => {
                                let (current_task, queue) = self
                                    .shared_todo
                                    .data
                                    .read()
                                    .map(|s| (s.queue.first().cloned(), s.queue.clone()))
                                    .unwrap_or_default();
                                if let Some(action) = self.timer_view.show(
                                    ui,
                                    &self.session,
                                    &self.theme,
                                    self.animations.pulse_value(),
                                    self.config.appearance.window_opacity,
                                    current_task.as_ref(),
                                    &queue,
                                ) {
                                    self.handle_timer_action(action);
                                }
                            }
                            View::Queue => {
                                let queue = self
                                    .shared_todo
                                    .data
                                    .read()
                                    .map(|s| s.queue.clone())
                                    .unwrap_or_default();
                                let queue_actions =
                                    queue_view::render_queue_view(ui, &self.theme, &queue);
                                for qa in queue_actions {
                                    match qa {
                                        queue_view::QueueViewAction::GoBack => {
                                            self.current_view = View::Timer;
                                        }
                                        queue_view::QueueViewAction::Remove(id) => {
                                            if let Some(db) = &self.database {
                                                let _ = db.remove_from_queue(id);
                                                self.refresh_todo_data();
                                            }
                                        }
                                        queue_view::QueueViewAction::ClearAll => {
                                            if let Some(db) = &self.database {
                                                let _ = db.clear_queue();
                                                self.refresh_todo_data();
                                            }
                                        }
                                        queue_view::QueueViewAction::Reorder(ids) => {
                                            if let Some(db) = &self.database {
                                                let _ = db.reorder_queue(&ids);
                                                self.refresh_todo_data();
                                            }
                                        }
                                    }
                                }
                            }
                            View::Stats => {
                                if let Some(action) = self.stats_view.show(
                                    ui,
                                    &self.session,
                                    &self.statistics,
                                    &self.theme,
                                    self.animations.pulse_value(),
                                    self.config.goals.daily_target,
                                ) {
                                    self.handle_stats_action(action);
                                }
                            }
                            View::Settings => {
                                if let Some(ref mut sv) = self.settings_view {
                                    settings_action = sv.show(ui, &self.config, &self.theme);
                                }
                            }
                        }
                    });
            });

        // Handle settings action outside closure (needs ctx for viewport commands)
        if let Some(action) = settings_action {
            self.handle_settings_action(action, ctx);
        }

        // Show close confirmation dialog
        if self.show_close_dialog {
            self.render_close_dialog(ctx);
        }

        // Force quit (from tray Quit action)
        if self.force_quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Show todo window if open
        if self.todo_window.is_open {
            self.show_todo_viewport(ctx);
        }

        // Handle keyboard shortcuts (only when no text field is focused)
        let any_text_focused = ctx.memory(|m| m.focused().is_some());
        let (space, escape, key_d, key_t, key_q, key_s) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::Space),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::D),
                i.key_pressed(egui::Key::T),
                i.key_pressed(egui::Key::Q),
                i.key_pressed(egui::Key::S),
            )
        });

        if !any_text_focused {
            if space && self.current_view == View::Timer {
                self.handle_timer_action(TimerAction::Toggle);
            }
            if key_d && self.current_view == View::Timer {
                self.current_view = View::Stats;
            }
            if key_t && self.current_view == View::Timer {
                self.todo_window.toggle();
                if !self.todo_window.is_open {
                    self.shared_todo.dwm_applied.store(false, Ordering::Relaxed);
                }
            }
            if key_q && self.current_view == View::Timer {
                self.current_view = View::Queue;
            }
            if key_s && self.current_view == View::Timer {
                self.settings_view = Some(SettingsView::new(&self.config));
                self.current_view = View::Settings;
            }
        }
        if escape {
            match self.current_view {
                View::Stats | View::Settings | View::Queue => {
                    self.current_view = View::Timer;
                    self.settings_view = None;
                }
                View::Timer => {
                    if self.config.system.minimize_to_tray && self.system_tray.is_some() {
                        self.hide_to_tray(ctx);
                    } else if self.config.system.minimize_to_tray {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                }
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Save window state to config
        if let Some(size) = self.last_window_size {
            self.config.window.width = size.x;
            self.config.window.height = size.y;
        }

        if let Some(pos) = self.last_window_pos {
            self.config.window.x = Some(pos.x);
            self.config.window.y = Some(pos.y);
        }

        self.config.window.maximized = self.last_window_maximized;

        // Save todo window position/size (stored in signals by deferred viewport)
        if let Ok(sig) = self.shared_todo.signals.lock() {
            if let Some(pos) = sig.last_window_pos {
                self.config.todo.window_x = Some(pos.x);
                self.config.todo.window_y = Some(pos.y);
            }
            if let Some(size) = sig.last_window_size {
                self.config.todo.window_width = size.x;
                self.config.todo.window_height = size.y;
            }
        }

        if let Err(e) = self.config.save() {
            tracing::error!("Failed to save window state on exit: {}", e);
        } else {
            tracing::info!("Window state saved on exit");
        }
    }
}
