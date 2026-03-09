//! Main application struct and logic

use std::sync::atomic::Ordering;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

use chrono::Utc;

use crate::core::{Preset, Session, SessionType, TimerEvent};
use crate::data::todo::{Priority, Project, QueuedTask, TodoItem};
use crate::data::{Config, Database, ExportFormat, Exporter, Statistics};
use crate::ipc::{IpcCommand, IpcResponse, IpcServer, IpcStats, IpcStatus};
use crate::platform::{AudioPlayer, HotkeyAction, HotkeyManager, SystemTray, TrayAction};
use crate::ui::{
    animations::AnimationState,
    settings::{SettingsAction, SettingsView},
    stats::{StatsAction, StatsView},
    theme::Theme,
    timer_view::{TimerAction, TimerView},
    titlebar::{TitleBar, TitleBarButton},
    todo_view::TodoAction,
    todo_window::{new_shared_todo, render_todo_viewport, SharedTodo, TodoWindow},
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

    // Status message for titlebar (e.g., "Settings saved")
    status_message: Option<String>,
    status_time: Option<std::time::Instant>,

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

/// Queue view action
enum QueueViewAction {
    GoBack,
    Remove(i64),
    ClearAll,
    Reorder(Vec<i64>),
}

/// Duration to show status message in titlebar
const STATUS_MESSAGE_DURATION: std::time::Duration = std::time::Duration::from_secs(2);

impl PomodoRustApp {
    /// Load system fallback fonts for Unicode symbols and emoji
    fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Fallback fonts: symbols (box-drawing, math, etc.) + emoji
        let fallbacks: &[(&str, &[&str])] = &[
            #[cfg(windows)]
            ("symbols", &[
                "C:\\Windows\\Fonts\\seguisym.ttf",  // Segoe UI Symbol (box-drawing, math, misc)
                "C:\\Windows\\Fonts\\segoeui.ttf",   // Segoe UI (broad Unicode coverage)
            ]),
            #[cfg(windows)]
            ("emoji", &[
                "C:\\Windows\\Fonts\\seguiemj.ttf",  // Segoe UI Emoji
            ]),
            #[cfg(target_os = "linux")]
            ("symbols", &[
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                "/usr/share/fonts/TTF/DejaVuSans.ttf",
            ]),
            #[cfg(target_os = "linux")]
            ("emoji", &[
                "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
                "/usr/share/fonts/noto-emoji/NotoColorEmoji.ttf",
                "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
            ]),
            #[cfg(target_os = "macos")]
            ("symbols", &[
                "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            ]),
            #[cfg(target_os = "macos")]
            ("emoji", &[
                "/System/Library/Fonts/Apple Color Emoji.ttc",
            ]),
        ];

        for (name, paths) in fallbacks {
            for path in *paths {
                let p = std::path::Path::new(path);
                if let Ok(data) = std::fs::read(p) {
                    fonts.font_data.insert(
                        name.to_string(),
                        egui::FontData::from_owned(data),
                    );
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                        if !family.contains(&name.to_string()) {
                            family.push(name.to_string());
                        }
                    }
                    if let Some(family) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                        if !family.contains(&name.to_string()) {
                            family.push(name.to_string());
                        }
                    }
                    tracing::info!("Loaded fallback font '{name}' from {path}");
                    break; // use first found path for this name
                }
            }
        }

        ctx.set_fonts(fonts);
    }

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
            status_message: None,
            status_time: None,
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

    /// Show a status message in the titlebar
    fn show_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_time = Some(std::time::Instant::now());
    }

    /// Get current status message if still valid
    fn current_status(&mut self) -> Option<&str> {
        if let Some(time) = self.status_time {
            if time.elapsed() > STATUS_MESSAGE_DURATION {
                self.status_message = None;
                self.status_time = None;
            }
        }
        self.status_message.as_deref()
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

    /// Handle timer completion
    fn on_timer_completed(&mut self) {
        let session_type = self.session.session_type();

        // Track if goal was reached before this session
        let goal_was_reached_before = self
            .statistics
            .is_daily_goal_reached(self.config.goals.daily_target);

        // Record to database (link to current queue task if work session)
        if let (Some(db), Some(start_time)) = (&self.database, self.session_start_time) {
            let duration = self.session.timer().total_duration().as_secs();
            let todo_id = if session_type == SessionType::Work {
                db.get_current_queue_task()
                    .ok()
                    .flatten()
                    .map(|t| t.todo_id)
            } else {
                None
            };
            if let Err(e) = db.record_session(session_type, duration, duration, true, start_time, todo_id) {
                tracing::error!("Failed to record session: {e}");
            }

            // Reload statistics
            self.statistics = Statistics::load(db);
        }

        // Check if goal was just reached
        let goal_just_reached = !goal_was_reached_before
            && self
                .statistics
                .is_daily_goal_reached(self.config.goals.daily_target)
            && session_type == SessionType::Work;

        // Play sound
        if self.config.sounds.enabled {
            if let Some(ref mut audio) = self.audio {
                audio.play_notification(self.config.sounds.notification_sound);
            }
        }

        // Show notification
        if self.config.system.notifications_enabled {
            let (title, body): (&str, String) = if goal_just_reached
                && self.config.goals.notify_on_goal
            {
                (
                    "Daily Goal Reached!",
                    format!(
                        "You completed {} pomodoros today!",
                        self.config.goals.daily_target
                    ),
                )
            } else {
                match session_type {
                    SessionType::Work => ("Focus Complete!", "Time for a break.".to_string()),
                    SessionType::ShortBreak => ("Break Over", "Ready to focus again?".to_string()),
                    SessionType::LongBreak => {
                        ("Long Break Over", "Let's get back to work!".to_string())
                    }
                }
            };
            crate::platform::show_notification(title, &body);
        }

        // Flash window in taskbar to get attention
        crate::platform::flash_pomodorust_window(5);

        // Update pomodoro queue
        if session_type == SessionType::Work {
            if let Some(db) = &self.database {
                if let Ok(Some(current)) = db.get_current_queue_task() {
                    if let Ok(all_done) = db.increment_queue_pomodoro(current.id) {
                        if all_done {
                            if let Err(e) = db.complete_queue_task(current.id, current.todo_id) {
                                tracing::error!("Failed to complete queue task: {e}");
                            }
                        }
                    }
                    self.refresh_todo_data();
                }
            }
        }

        self.session_start_time = None;
    }

    /// Render the queue page inside the main pomodoro window.
    fn render_queue_view(
        ui: &mut egui::Ui,
        theme: &Theme,
        queue: &[crate::data::todo::QueuedTask],
    ) -> Vec<QueueViewAction> {
        use crate::ui::components::{draw_icon, Icon};

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

    /// Handle timer action
    fn handle_timer_action(&mut self, action: TimerAction) {
        match action {
            TimerAction::Toggle => {
                let event = self.session.toggle();
                if event == TimerEvent::Started {
                    self.session_start_time = Some(Utc::now());
                }
            }
            TimerAction::Skip => {
                self.session.skip();
                self.session_start_time = None;
            }
            TimerAction::Reset => {
                self.session.reset();
                self.session_start_time = None;
            }
            TimerAction::OpenStats => {
                self.current_view = View::Stats;
            }
            TimerAction::OpenSettings => {
                self.settings_view = Some(SettingsView::new(&self.config));
                self.current_view = View::Settings;
            }
            TimerAction::OpenTodo => {
                self.todo_window.toggle();
                // Reset DWM effects flag so they are reapplied when window reopens
                if !self.todo_window.is_open {
                    self.shared_todo.dwm_applied.store(false, Ordering::Relaxed);
                }
            }
            TimerAction::OpenQueue => {
                self.current_view = View::Queue;
            }
        }
    }

    /// Handle stats action
    fn handle_stats_action(&mut self, action: StatsAction) {
        match action {
            StatsAction::Back => {
                self.current_view = View::Timer;
            }
            StatsAction::OpenSettings => {
                self.settings_view = Some(SettingsView::new(&self.config));
                self.current_view = View::Settings;
            }
            StatsAction::QuickStart {
                session_type,
                minutes,
            } => {
                // Switch to the requested session type
                self.session.switch_to(session_type);
                // Reset timer with custom duration
                self.session
                    .timer_mut()
                    .reset_with_duration(minutes as u64 * 60);
                // Start the timer
                self.session.start();
                self.session_start_time = Some(Utc::now());
                // Go back to timer view
                self.current_view = View::Timer;
            }
            StatsAction::Export { format } => {
                self.export_statistics(format);
            }
            StatsAction::UndoLastSession => {
                self.undo_last_session();
            }
            StatsAction::ResetStats => {
                self.reset_all_stats();
            }
            StatsAction::ChangeWeek { offset } => {
                self.stats_view.week_offset = offset;
                if offset == 0 {
                    self.stats_view.selected_week_hours = None;
                } else if let Some(db) = &self.database {
                    use chrono::Local;
                    let today = Local::now().date_naive();
                    let reference =
                        today + chrono::Duration::weeks(offset as i64);
                    self.stats_view.selected_week_hours =
                        db.get_week_stats_for_date(reference).ok();
                }
            }
        }
    }

    /// Reset all statistics
    fn reset_all_stats(&mut self) {
        let Some(db) = &self.database else {
            tracing::warn!("No database available for reset");
            return;
        };

        match db.reset_all_stats() {
            Ok(()) => {
                tracing::info!("All statistics reset");
                // Reload statistics
                self.statistics = Statistics::load(db);
                // Show notification
                crate::platform::show_notification(
                    "Statistics Reset",
                    "All statistics have been cleared.",
                );
            }
            Err(e) => {
                tracing::error!("Failed to reset statistics: {}", e);
            }
        }
    }

    /// Undo the last work session
    fn undo_last_session(&mut self) {
        let Some(db) = &self.database else {
            tracing::warn!("No database available for undo");
            return;
        };

        match db.undo_last_session() {
            Ok(Some(session)) => {
                tracing::info!("Undid session: {:?}", session);
                // Reload statistics
                self.statistics = Statistics::load(db);
                // Show notification
                crate::platform::show_notification(
                    "Session Undone",
                    "Last pomodoro session has been removed from statistics.",
                );
            }
            Ok(None) => {
                tracing::info!("No session to undo");
            }
            Err(e) => {
                tracing::error!("Failed to undo session: {}", e);
            }
        }
    }

    /// Export statistics to file
    fn export_statistics(&self, format: ExportFormat) {
        let Some(db) = &self.database else {
            tracing::error!("No database available for export");
            return;
        };

        // Create file dialog
        let default_filename = Exporter::default_filename(format);
        let filter_name = format.label();
        let filter_ext = format.extension();

        let file_dialog = rfd::FileDialog::new()
            .set_title("Export Statistics")
            .set_file_name(&default_filename)
            .add_filter(filter_name, &[filter_ext]);

        // Show save dialog
        if let Some(path) = file_dialog.save_file() {
            match Exporter::export(db, &path, format) {
                Ok(()) => {
                    tracing::info!("Statistics exported to {:?}", path);
                    // Show success notification
                    crate::platform::show_notification(
                        "Export Complete",
                        &format!("Statistics saved to {}", path.display()),
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to export statistics: {}", e);
                    crate::platform::show_notification("Export Failed", &format!("Error: {}", e));
                }
            }
        }
    }

    /// Handle settings action
    fn handle_settings_action(&mut self, action: SettingsAction, ctx: &egui::Context) {
        match action {
            SettingsAction::Back => {
                self.current_view = View::Timer;
                self.settings_view = None;
            }
            SettingsAction::UpdateConfig(new_config) => {
                self.apply_config(new_config, ctx);
            }
            SettingsAction::SelectPreset(index) => {
                let presets = [Preset::classic(), Preset::short(), Preset::long()];
                let preset_names = ["Classic", "Short", "Long"];
                if let Some(preset) = presets.get(index) {
                    self.config.apply_preset(preset);
                    self.session.set_preset(preset.clone());
                    let _ = self.config.save();

                    if let Some(ref mut sv) = self.settings_view {
                        sv.reset_from_config(&self.config);
                    }
                    self.show_status(format!("{} preset applied", preset_names[index]));
                }
            }
            SettingsAction::ResetDefaults => {
                self.config.reset();
                let _ = self.config.save();

                self.session.set_preset(self.config.to_preset());
                self.theme = Theme::from_mode(
                    self.config.appearance.theme_mode,
                    self.config.appearance.accent_color,
                );
                if self.config.accessibility.high_contrast {
                    self.theme = self.theme.clone().with_high_contrast();
                }
                if self.config.accessibility.reduced_motion {
                    self.theme = self.theme.clone().with_reduced_motion();
                }
                self.todo_theme_dirty = true;

                // Reset always on top to default (false)
                self.set_always_on_top(false, ctx);

                if let Some(ref mut sv) = self.settings_view {
                    sv.reset_from_config(&self.config);
                }
                self.show_status("Defaults restored");
            }
            SettingsAction::SetAlwaysOnTop(enabled) => {
                self.set_always_on_top(enabled, ctx);
            }
            SettingsAction::TestSound(sound) => {
                if let Some(ref mut audio) = self.audio {
                    audio.play_notification(sound);
                }
            }
        }
    }

    /// Apply new configuration
    fn apply_config(&mut self, new_config: Config, ctx: &egui::Context) {
        // Check if theme changed
        if new_config.appearance.theme_mode != self.config.appearance.theme_mode
            || new_config.appearance.accent_color != self.config.appearance.accent_color
            || new_config.accessibility.high_contrast != self.config.accessibility.high_contrast
            || new_config.accessibility.reduced_motion != self.config.accessibility.reduced_motion
        {
            self.theme = Theme::from_mode(
                new_config.appearance.theme_mode,
                new_config.appearance.accent_color,
            );
            if new_config.accessibility.high_contrast {
                self.theme = self.theme.clone().with_high_contrast();
            }
            if new_config.accessibility.reduced_motion {
                self.theme = self.theme.clone().with_reduced_motion();
            }
            self.todo_theme_dirty = true;
        }

        // Check if timer settings changed
        if new_config.timer.work_duration != self.config.timer.work_duration
            || new_config.timer.short_break != self.config.timer.short_break
            || new_config.timer.long_break != self.config.timer.long_break
            || new_config.timer.sessions_before_long != self.config.timer.sessions_before_long
        {
            self.session.set_preset(new_config.to_preset());
        }

        // Update auto-start
        self.session.set_auto_start(
            new_config.timer.auto_start_breaks,
            new_config.timer.auto_start_work,
        );

        // Update audio volume
        if let Some(ref mut audio) = self.audio {
            audio.set_volume(new_config.sounds.volume as f32 / 100.0);
        }

        // Update autostart
        if new_config.system.start_with_windows != self.config.system.start_with_windows {
            let _ = crate::platform::set_autostart(new_config.system.start_with_windows);
        }

        // Update always on top
        if new_config.window.always_on_top != self.config.window.always_on_top {
            // Don't use set_always_on_top here — config is saved below with all changes
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                if new_config.window.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                },
            ));
            if let Ok(mut data) = self.shared_todo.data.write() {
                data.is_always_on_top = new_config.window.always_on_top;
            }
        }

        // Sync system tray
        if new_config.system.minimize_to_tray && self.system_tray.is_none() {
            self.system_tray = SystemTray::new().ok();
        } else if !new_config.system.minimize_to_tray && self.system_tray.is_some() {
            self.system_tray = None;
        }

        self.config = new_config;
        let _ = self.config.save();
        self.show_status("Settings saved");
    }

    /// Handle IPC commands from CLI
    fn handle_ipc_commands(&mut self) {
        // Collect all pending commands first to avoid borrow issues
        let commands: Vec<IpcCommand> = self
            .ipc_receiver
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();

        // Process collected commands
        for command in commands {
            let response = self.process_ipc_command(command);
            self.ipc_server.set_response(response);
        }
    }

    /// Process a single IPC command and return the response
    fn process_ipc_command(&mut self, command: IpcCommand) -> IpcResponse {
        match command {
            IpcCommand::Start { session_type } => {
                // Optionally switch session type
                if let Some(st) = session_type {
                    match st.to_lowercase().as_str() {
                        "work" | "focus" => self.session.switch_to(SessionType::Work),
                        "short" | "short_break" => self.session.switch_to(SessionType::ShortBreak),
                        "long" | "long_break" => self.session.switch_to(SessionType::LongBreak),
                        _ => return IpcResponse::error(format!("Unknown session type: {}", st)),
                    }
                }

                if !self.session.timer().is_running() {
                    self.session.start();
                    self.session_start_time = Some(Utc::now());
                    IpcResponse::ok_with_message("Timer started")
                } else {
                    IpcResponse::ok_with_message("Timer already running")
                }
            }

            IpcCommand::Pause => {
                if self.session.timer().is_running() {
                    self.session.pause();
                    IpcResponse::ok_with_message("Timer paused")
                } else {
                    IpcResponse::ok_with_message("Timer not running")
                }
            }

            IpcCommand::Resume => {
                if self.session.timer().is_paused() {
                    self.session.start(); // start() handles resume from paused state
                    IpcResponse::ok_with_message("Timer resumed")
                } else {
                    IpcResponse::ok_with_message("Timer not paused")
                }
            }

            IpcCommand::Toggle => {
                let event = self.session.toggle();
                match event {
                    crate::core::TimerEvent::Started => {
                        self.session_start_time = Some(Utc::now());
                        IpcResponse::ok_with_message("Timer started")
                    }
                    crate::core::TimerEvent::Resumed => {
                        IpcResponse::ok_with_message("Timer resumed")
                    }
                    crate::core::TimerEvent::Paused => IpcResponse::ok_with_message("Timer paused"),
                    _ => IpcResponse::ok(),
                }
            }

            IpcCommand::Stop => {
                self.session.reset();
                self.session_start_time = None;
                IpcResponse::ok_with_message("Timer stopped and reset")
            }

            IpcCommand::Skip => {
                self.session.skip();
                self.session_start_time = None;
                IpcResponse::ok_with_message(format!(
                    "Skipped to {}",
                    self.session.session_type().label()
                ))
            }

            IpcCommand::Status => {
                let timer = self.session.timer();
                let state = if timer.is_running() {
                    "running"
                } else if timer.is_paused() {
                    "paused"
                } else if timer.is_completed() {
                    "completed"
                } else {
                    "idle"
                };

                let session_type = match self.session.session_type() {
                    SessionType::Work => "work",
                    SessionType::ShortBreak => "short_break",
                    SessionType::LongBreak => "long_break",
                };

                IpcResponse::Status(IpcStatus {
                    state: state.to_string(),
                    session_type: session_type.to_string(),
                    remaining_secs: timer.remaining().as_secs(),
                    remaining_formatted: timer.remaining_formatted(),
                    progress: timer.progress(),
                    current_session: self.session.current_session_in_cycle(),
                    total_sessions: self.session.total_sessions_in_cycle(),
                    total_duration_secs: timer.total_duration().as_secs(),
                })
            }

            IpcCommand::Stats { period } => {
                let period = if period.is_empty() { "today" } else { &period };

                let (hours, pomodoros) = match period {
                    "today" => (
                        self.statistics.today_hours(),
                        self.statistics.today_pomodoros,
                    ),
                    "week" => (
                        self.statistics.week_hours(),
                        self.statistics.today_pomodoros,
                    ), // week doesn't have pomodoro count
                    "all" => (
                        self.statistics.total_hours() as f32,
                        self.statistics.total_pomodoros,
                    ),
                    _ => return IpcResponse::error(format!("Unknown period: {}", period)),
                };

                IpcResponse::Stats(IpcStats {
                    period: period.to_string(),
                    hours,
                    pomodoros,
                    current_streak: self.statistics.current_streak,
                    longest_streak: self.statistics.longest_streak,
                    daily_goal: self.config.goals.daily_target,
                    today_pomodoros: self.statistics.today_pomodoros,
                })
            }

            IpcCommand::Ping => IpcResponse::Pong,
        }
    }

    /// Handle global hotkey events
    fn handle_hotkey_events(&mut self) {
        // Collect all pending hotkey events
        let events: Vec<HotkeyAction> = self
            .hotkey_receiver
            .as_ref()
            .map(|rx| rx.try_iter().collect())
            .unwrap_or_default();

        // Process collected events
        for action in events {
            match action {
                HotkeyAction::Toggle => {
                    let event = self.session.toggle();
                    if event == TimerEvent::Started {
                        self.session_start_time = Some(Utc::now());
                    }
                    tracing::info!("Hotkey: Toggle timer");
                }
                HotkeyAction::Skip => {
                    self.session.skip();
                    self.session_start_time = None;
                    tracing::info!("Hotkey: Skip session");
                }
                HotkeyAction::Reset => {
                    self.session.reset();
                    self.session_start_time = None;
                    tracing::info!("Hotkey: Reset timer");
                }
            }
        }
    }

    /// Handle system tray events
    fn handle_tray_events(&mut self, ctx: &egui::Context) {
        // Start background polling thread on first call (idempotent)
        if let Some(ref mut tray) = self.system_tray {
            tray.start_polling(ctx.clone());
        }

        // Collect actions from the background thread's channel
        let actions: Vec<TrayAction> = {
            let Some(ref tray) = self.system_tray else {
                return;
            };
            let mut actions = Vec::new();
            while let Some(action) = tray.poll_action() {
                actions.push(action);
            }
            actions
        };

        for action in actions {
            match action {
                TrayAction::Toggle => {
                    self.handle_timer_action(TimerAction::Toggle);
                }
                TrayAction::Skip => {
                    self.handle_timer_action(TimerAction::Skip);
                }
                TrayAction::ShowWindow => {
                    self.show_from_tray(ctx);
                }
                TrayAction::Quit => {
                    // On Windows, the background tray thread already called
                    // force_quit_app() before this message arrives.
                    // On other platforms, use viewport command.
                    self.force_quit = true;
                    self.hidden_to_tray = false;
                    if let Some(ref tray) = self.system_tray {
                        tray.set_periodic_wakeup(false);
                    }
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    /// Update tray tooltip and toggle label based on current timer state
    fn update_tray_state(&mut self) {
        let Some(ref mut tray) = self.system_tray else {
            return;
        };

        let session_label = match self.session.session_type() {
            SessionType::Work => "Фокус",
            SessionType::ShortBreak => "Короткий перерыв",
            SessionType::LongBreak => "Длинный перерыв",
        };

        let timer = self.session.timer();
        let tooltip = if timer.is_running() {
            format!(
                "{} \u{2014} {}",
                session_label,
                timer.remaining_formatted()
            )
        } else if timer.is_paused() {
            format!(
                "Пауза \u{2014} {} {}",
                session_label,
                timer.remaining_formatted()
            )
        } else {
            "PomodoRust \u{2014} Готов".to_string()
        };

        tray.update_tooltip(&tooltip);

        let toggle_label = if timer.is_running() {
            "Пауза"
        } else if timer.is_paused() {
            "Продолжить"
        } else {
            "Старт"
        };
        tray.update_toggle_label(toggle_label);
    }

    /// Hide the main window to the system tray.
    /// Uses native Win32 API on Windows to avoid corrupting eframe's internal
    /// viewport state (ViewportCommand::Visible(false) blocks all subsequent
    /// viewport commands — known eframe bug #5229).
    fn hide_to_tray(&mut self, ctx: &egui::Context) {
        #[cfg(windows)]
        crate::platform::hide_pomodorust_window();
        #[cfg(not(windows))]
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        let _ = ctx;
        self.hidden_to_tray = true;
        if let Some(ref tray) = self.system_tray {
            tray.set_periodic_wakeup(true);
        }
    }

    /// Show the main window from the system tray.
    /// On Windows, the background tray thread already called show_pomodorust_window()
    /// via Win32 API before this runs. We just update internal state here.
    fn show_from_tray(&mut self, ctx: &egui::Context) {
        #[cfg(not(windows))]
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        let _ = ctx;
        self.hidden_to_tray = false;
        if let Some(ref tray) = self.system_tray {
            tray.set_periodic_wakeup(false);
        }
    }

    /// Render close confirmation dialog (minimize to tray or quit)
    fn render_close_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Area::new(egui::Id::new("close_dialog_overlay"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .interactable(true)
            .show(ctx, |ui| {
                let screen = ui.ctx().screen_rect();
                // Semi-transparent overlay
                ui.painter().rect_filled(
                    screen,
                    0.0,
                    egui::Color32::from_black_alpha(120),
                );
                // Consume clicks on overlay to close dialog
                let overlay_response =
                    ui.allocate_rect(screen, egui::Sense::click());
                if overlay_response.clicked() {
                    open = false;
                }
            });

        egui::Window::new("Закрыть приложение?")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label("Что вы хотите сделать?");
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button("  Свернуть в трей  ").clicked() {
                        self.hide_to_tray(ui.ctx());
                        self.show_close_dialog = false;
                    }
                    if ui.button("  Выход  ").clicked() {
                        self.force_quit = true;
                        self.show_close_dialog = false;
                    }
                });
                ui.add_space(4.0);
            });

        if !open {
            self.show_close_dialog = false;
        }
    }

    /// Refresh todo data from database into shared state
    fn refresh_todo_data(&mut self) {
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
    fn handle_todo_action(&mut self, action: TodoAction) -> bool {
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
    fn show_todo_viewport(&mut self, ctx: &egui::Context) {
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

    /// Handle window resize zones for custom decorated window
    fn handle_resize_zones(&self, ctx: &egui::Context) {
        // Skip resize handling if maximized
        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        if is_maximized {
            return;
        }

        let resize_margin = 8.0;
        let screen_rect = ctx.screen_rect();

        // Read input data first (can't call send_viewport_cmd inside input closure)
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

        // Get current status message for titlebar (only show in settings view)
        let status_for_titlebar = if self.current_view == View::Settings {
            let status = self.current_status().map(|s| s.to_string());
            if status.is_some() {
                ctx.request_repaint(); // Repaint to update/hide status
            }
            status
        } else {
            None
        };

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
                let (should_drag, button) = self.titlebar.show_with_status(
                    ui,
                    &self.theme,
                    is_maximized,
                    self.config.window.always_on_top,
                    status_for_titlebar.as_deref(),
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
                                    Self::render_queue_view(ui, &self.theme, &queue);
                                for qa in queue_actions {
                                    match qa {
                                        QueueViewAction::GoBack => {
                                            self.current_view = View::Timer;
                                        }
                                        QueueViewAction::Remove(id) => {
                                            if let Some(db) = &self.database {
                                                let _ = db.remove_from_queue(id);
                                                self.refresh_todo_data();
                                            }
                                        }
                                        QueueViewAction::ClearAll => {
                                            if let Some(db) = &self.database {
                                                let _ = db.clear_queue();
                                                self.refresh_todo_data();
                                            }
                                        }
                                        QueueViewAction::Reorder(ids) => {
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
