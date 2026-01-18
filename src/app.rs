//! Main application struct and logic

use chrono::Utc;

use crate::core::{Preset, Session, SessionType, TimerEvent};
use crate::data::{Config, Database, Statistics};
use crate::platform::{AudioPlayer, SoundType};
use crate::ui::{
    animations::AnimationState,
    dashboard::{DashboardAction, DashboardView},
    settings::{SettingsAction, SettingsView},
    theme::Theme,
    timer_view::{TimerAction, TimerView},
    titlebar::{TitleBar, TitleBarButton},
};

/// Application view state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Timer,
    Dashboard,
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
    dashboard_view: DashboardView,
    settings_view: Option<SettingsView>,

    // Animation state
    animations: AnimationState,

    // Current view
    current_view: View,

    // Audio
    audio: Option<AudioPlayer>,

    // Session tracking
    session_start_time: Option<chrono::DateTime<Utc>>,
}

impl PomodoRustApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load configuration
        let config = Config::load();

        // Create theme from config
        let theme = Theme::new(config.appearance.accent_color);
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

        Self {
            session,
            config,
            theme,
            database,
            statistics,
            titlebar: TitleBar::new(),
            timer_view: TimerView::new(),
            dashboard_view: DashboardView::new(),
            settings_view: None,
            animations: AnimationState::new(),
            current_view: View::Timer,
            audio,
            session_start_time: None,
        }
    }

    /// Handle timer completion
    fn on_timer_completed(&mut self) {
        let session_type = self.session.session_type();

        // Record to database
        if let (Some(db), Some(start_time)) = (&self.database, self.session_start_time) {
            let duration = self.session.timer().total_duration().as_secs();
            let _ = db.record_session(session_type, duration, duration, true, start_time);

            // Reload statistics
            self.statistics = Statistics::load(db);
        }

        // Play sound
        if self.config.sounds.enabled {
            if let Some(ref audio) = self.audio {
                match session_type {
                    SessionType::Work => audio.play(SoundType::WorkEnd),
                    SessionType::ShortBreak | SessionType::LongBreak => {
                        audio.play(SoundType::BreakEnd)
                    }
                }
            }
        }

        // Show notification
        if self.config.system.notifications_enabled {
            let (title, body) = match session_type {
                SessionType::Work => ("Focus Complete!", "Time for a break."),
                SessionType::ShortBreak => ("Break Over", "Ready to focus again?"),
                SessionType::LongBreak => ("Long Break Over", "Let's get back to work!"),
            };
            crate::platform::show_notification(title, body);
        }

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
            TimerAction::OpenDashboard => {
                self.current_view = View::Dashboard;
            }
            TimerAction::OpenSettings => {
                self.settings_view = Some(SettingsView::new(&self.config));
                self.current_view = View::Settings;
            }
        }
    }

    /// Handle dashboard action
    fn handle_dashboard_action(&mut self, action: DashboardAction) {
        match action {
            DashboardAction::Back => {
                self.current_view = View::Timer;
            }
            DashboardAction::OpenSettings => {
                self.settings_view = Some(SettingsView::new(&self.config));
                self.current_view = View::Settings;
            }
        }
    }

    /// Handle settings action
    fn handle_settings_action(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::Back => {
                self.current_view = View::Timer;
                self.settings_view = None;
            }
            SettingsAction::UpdateConfig(new_config) => {
                self.apply_config(new_config);
            }
            SettingsAction::SelectPreset(index) => {
                let presets = [Preset::classic(), Preset::short(), Preset::long()];
                if let Some(preset) = presets.get(index) {
                    self.config.apply_preset(preset);
                    self.session.set_preset(preset.clone());
                    let _ = self.config.save();

                    if let Some(ref mut sv) = self.settings_view {
                        sv.reset_from_config(&self.config);
                    }
                }
            }
            SettingsAction::ResetDefaults => {
                self.config.reset();
                let _ = self.config.save();

                self.session.set_preset(self.config.to_preset());
                self.theme = Theme::new(self.config.appearance.accent_color);

                if let Some(ref mut sv) = self.settings_view {
                    sv.reset_from_config(&self.config);
                }
            }
        }
    }

    /// Apply new configuration
    fn apply_config(&mut self, new_config: Config) {
        // Check if theme changed
        if new_config.appearance.accent_color != self.config.appearance.accent_color {
            self.theme = Theme::new(new_config.appearance.accent_color);
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

        self.config = new_config;
        let _ = self.config.save();
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
        let (hover_pos, primary_pressed) = ctx.input(|input| {
            (input.pointer.hover_pos(), input.pointer.primary_pressed())
        });

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

        // Update animations
        self.animations.update(self.session.timer().is_running());

        // Request continuous repaint when timer is running or animating
        if self.session.timer().is_running() || self.animations.needs_repaint() {
            ctx.request_repaint();
        }

        // Check if maximized for rounding
        let is_maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));

        // Main panel with custom frame - no rounding or border when maximized
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(self.theme.bg_primary)
                    .rounding(if is_maximized { egui::Rounding::ZERO } else { self.theme.window_rounding() })
            )
            .show(ctx, |ui| {
                // Title bar
                let (should_drag, button) = self.titlebar.show(ui, &self.theme, is_maximized);

                if should_drag {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                if let Some(button) = button {
                    match button {
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
                    .inner_margin(egui::Margin::symmetric(self.theme.spacing_md, self.theme.spacing_sm))
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
                            View::Dashboard => {
                                if let Some(action) = self.dashboard_view.show(
                                    ui,
                                    &self.session,
                                    &self.statistics,
                                    &self.theme,
                                    self.animations.pulse_value(),
                                ) {
                                    self.handle_dashboard_action(action);
                                }
                            }
                            View::Settings => {
                                if let Some(ref mut sv) = self.settings_view {
                                    if let Some(action) = sv.show(ui, &self.config, &self.theme) {
                                        self.handle_settings_action(action);
                                    }
                                }
                            }
                        }
                    });
            });

        // Handle keyboard shortcuts
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) && self.current_view == View::Timer {
                self.handle_timer_action(TimerAction::Toggle);
            }
            if i.key_pressed(egui::Key::Escape) {
                match self.current_view {
                    View::Dashboard | View::Settings => {
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
                self.current_view = View::Dashboard;
            }
        });
    }
}
