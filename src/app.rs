//! Main application struct and logic

use std::sync::mpsc::Receiver;

use chrono::Utc;

use crate::core::{Preset, Session, SessionType, TimerEvent};
use crate::data::{Config, Database, ExportFormat, Exporter, Statistics};
use crate::ipc::{IpcCommand, IpcResponse, IpcServer, IpcStats, IpcStatus};
use crate::platform::{AudioPlayer, HotkeyAction, HotkeyManager};
use crate::ui::{
    animations::AnimationState,
    settings::{SettingsAction, SettingsView},
    stats::{StatsAction, StatsView},
    theme::Theme,
    timer_view::{TimerAction, TimerView},
    titlebar::{TitleBar, TitleBarButton},
};

/// Application view state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Timer,
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
}

/// Duration to show status message in titlebar
const STATUS_MESSAGE_DURATION: std::time::Duration = std::time::Duration::from_secs(2);

impl PomodoRustApp {
    /// Create a new application instance with the given config
    pub fn with_config(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        Self::init(cc, config)
    }

    /// Create a new application instance (loads config from disk)
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let config = Config::load();
        Self::init(cc, config)
    }

    /// Internal initialization with a config
    fn init(cc: &eframe::CreationContext<'_>, config: Config) -> Self {

        // Create theme from config
        let mut theme = Theme::new(config.appearance.accent_color);
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

        Self {
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
        }
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

    /// Handle timer completion
    fn on_timer_completed(&mut self) {
        let session_type = self.session.session_type();

        // Track if goal was reached before this session
        let goal_was_reached_before = self
            .statistics
            .is_daily_goal_reached(self.config.goals.daily_target);

        // Record to database
        if let (Some(db), Some(start_time)) = (&self.database, self.session_start_time) {
            let duration = self.session.timer().total_duration().as_secs();
            let _ = db.record_session(session_type, duration, duration, true, start_time);

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

        self.session_start_time = None;
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
                self.theme = Theme::new(self.config.appearance.accent_color);
                if self.config.accessibility.high_contrast {
                    self.theme = self.theme.clone().with_high_contrast();
                }
                if self.config.accessibility.reduced_motion {
                    self.theme = self.theme.clone().with_reduced_motion();
                }

                // Reset always on top to default (false)
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::Normal,
                ));

                if let Some(ref mut sv) = self.settings_view {
                    sv.reset_from_config(&self.config);
                }
                self.show_status("Defaults restored");
            }
            SettingsAction::SetAlwaysOnTop(enabled) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(if enabled {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                }));
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
        if new_config.appearance.accent_color != self.config.appearance.accent_color
            || new_config.accessibility.high_contrast != self.config.accessibility.high_contrast
            || new_config.accessibility.reduced_motion != self.config.accessibility.reduced_motion
        {
            self.theme = Theme::new(new_config.appearance.accent_color);
            if new_config.accessibility.high_contrast {
                self.theme = self.theme.clone().with_high_contrast();
            }
            if new_config.accessibility.reduced_motion {
                self.theme = self.theme.clone().with_reduced_motion();
            }
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
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                if new_config.window.always_on_top {
                    egui::WindowLevel::AlwaysOnTop
                } else {
                    egui::WindowLevel::Normal
                },
            ));
        }

        self.config = new_config;
        let _ = self.config.save();
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
                            self.config.window.always_on_top = !self.config.window.always_on_top;
                            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                                if self.config.window.always_on_top {
                                    egui::WindowLevel::AlwaysOnTop
                                } else {
                                    egui::WindowLevel::Normal
                                },
                            ));
                            let _ = self.config.save();
                        }
                        TitleBarButton::Minimize => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                        TitleBarButton::Maximize => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(
                                !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
                            ));
                        }
                        TitleBarButton::Close => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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
                                if let Some(action) = self.timer_view.show(
                                    ui,
                                    &self.session,
                                    &self.theme,
                                    self.animations.pulse_value(),
                                ) {
                                    self.handle_timer_action(action);
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

        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) && self.current_view == View::Timer {
                self.handle_timer_action(TimerAction::Toggle);
            }
            if i.key_pressed(egui::Key::Escape) {
                match self.current_view {
                    View::Stats | View::Settings => {
                        self.current_view = View::Timer;
                        self.settings_view = None;
                    }
                    View::Timer => {
                        if self.config.system.minimize_to_tray {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                        }
                    }
                }
            }
            if i.key_pressed(egui::Key::D) && self.current_view == View::Timer {
                self.current_view = View::Stats;
            }
        });
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

        if let Err(e) = self.config.save() {
            tracing::error!("Failed to save window state on exit: {}", e);
        } else {
            tracing::info!("Window state saved on exit");
        }
    }
}
