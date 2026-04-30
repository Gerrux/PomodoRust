use chrono::Utc;

use crate::core::{SessionType, TimerEvent};
use crate::ipc::{IpcCommand, IpcResponse, IpcStats, IpcStatus};
use crate::platform::{HotkeyAction, TrayAction};
use crate::ui::timer_view::TimerAction;

use super::PomodoRustApp;

impl PomodoRustApp {
    /// Load system fallback fonts for Unicode symbols, emoji, and Phosphor icons
    pub(super) fn setup_fonts(ctx: &egui::Context) {
        let mut fonts = egui::FontDefinitions::default();

        // Add Phosphor icons font
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);

        // Embed Unbounded fonts for modern style
        fonts.font_data.insert(
            "Unbounded-Black".to_string(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/Unbounded-Black.ttf")),
        );
        fonts.font_data.insert(
            "Unbounded-Regular".to_string(),
            egui::FontData::from_static(include_bytes!("../../assets/fonts/Unbounded-Regular.ttf")),
        );
        // Timer digits: Unbounded Black
        fonts.families.insert(
            egui::FontFamily::Name("Timer".into()),
            vec!["Unbounded-Black".to_string()],
        );
        // Modern UI text: Unbounded Regular (fallback to default proportional fonts)
        fonts.families.insert(
            egui::FontFamily::Name("Modern".into()),
            vec!["Unbounded-Regular".to_string()],
        );

        // Fallback fonts: symbols (box-drawing, math, etc.) + emoji
        let fallbacks: &[(&str, &[&str])] = &[
            #[cfg(windows)]
            (
                "symbols",
                &[
                    "C:\\Windows\\Fonts\\seguisym.ttf", // Segoe UI Symbol (box-drawing, math, misc)
                    "C:\\Windows\\Fonts\\segoeui.ttf",  // Segoe UI (broad Unicode coverage)
                ],
            ),
            #[cfg(windows)]
            (
                "emoji",
                &[
                    "C:\\Windows\\Fonts\\seguiemj.ttf", // Segoe UI Emoji
                ],
            ),
            #[cfg(target_os = "linux")]
            (
                "symbols",
                &[
                    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
                    "/usr/share/fonts/TTF/DejaVuSans.ttf",
                ],
            ),
            #[cfg(target_os = "linux")]
            (
                "emoji",
                &[
                    "/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf",
                    "/usr/share/fonts/noto-emoji/NotoColorEmoji.ttf",
                    "/usr/share/fonts/google-noto-emoji/NotoColorEmoji.ttf",
                ],
            ),
            #[cfg(target_os = "macos")]
            (
                "symbols",
                &["/System/Library/Fonts/Supplemental/Arial Unicode.ttf"],
            ),
            #[cfg(target_os = "macos")]
            ("emoji", &["/System/Library/Fonts/Apple Color Emoji.ttc"]),
        ];

        for (name, paths) in fallbacks {
            for path in *paths {
                let p = std::path::Path::new(path);
                if let Ok(data) = std::fs::read(p) {
                    fonts
                        .font_data
                        .insert(name.to_string(), egui::FontData::from_owned(data));
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

    /// Handle IPC commands from CLI
    pub(super) fn handle_ipc_commands(&mut self) {
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
    pub(super) fn handle_hotkey_events(&mut self) {
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
    pub(super) fn handle_tray_events(&mut self, ctx: &egui::Context) {
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
    pub(super) fn update_tray_state(&mut self) {
        let Some(ref mut tray) = self.system_tray else {
            return;
        };

        let t = crate::i18n::tr();
        let session_label = match self.session.session_type() {
            SessionType::Work => t.tray.focus,
            SessionType::ShortBreak => t.tray.short_break,
            SessionType::LongBreak => t.tray.long_break,
        };

        let timer = self.session.timer();
        let tooltip = if timer.is_running() {
            format!("{} \u{2014} {}", session_label, timer.remaining_formatted())
        } else if timer.is_paused() {
            format!(
                "{} \u{2014} {} {}",
                t.tray.pause,
                session_label,
                timer.remaining_formatted()
            )
        } else {
            format!("PomodoRust \u{2014} {}", t.tray.ready)
        };

        tray.update_tooltip(&tooltip);

        let toggle_label = if timer.is_running() {
            t.tray.pause
        } else if timer.is_paused() {
            t.tray.continue_
        } else {
            t.tray.start
        };
        tray.update_toggle_label(toggle_label);
    }

    /// Hide the main window to the system tray.
    /// Uses native Win32 API on Windows to avoid corrupting eframe's internal
    /// viewport state (ViewportCommand::Visible(false) blocks all subsequent
    /// viewport commands — known eframe bug #5229).
    pub(super) fn hide_to_tray(&mut self, ctx: &egui::Context) {
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
    pub(super) fn show_from_tray(&mut self, ctx: &egui::Context) {
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
    pub(super) fn render_close_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;
        egui::Area::new(egui::Id::new("close_dialog_overlay"))
            .fixed_pos(egui::pos2(0.0, 0.0))
            .order(egui::Order::Middle)
            .interactable(true)
            .show(ctx, |ui| {
                let screen = ui.ctx().screen_rect();
                // Semi-transparent overlay
                ui.painter()
                    .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(120));
                // Consume clicks on overlay to close dialog
                let overlay_response = ui.allocate_rect(screen, egui::Sense::click());
                if overlay_response.clicked() {
                    open = false;
                }
            });

        let t = crate::i18n::tr();
        egui::Window::new(t.tray.close_app)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(t.tray.what_to_do);
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui.button(t.tray.minimize_to_tray).clicked() {
                        self.hide_to_tray(ui.ctx());
                        self.show_close_dialog = false;
                    }
                    if ui.button(t.tray.quit).clicked() {
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

    /// Handle window resize zones for custom decorated window
    pub(super) fn handle_resize_zones(&self, ctx: &egui::Context) {
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
