use std::sync::atomic::Ordering;

use chrono::Utc;

use crate::core::{Preset, SessionType, TimerEvent};
use crate::data::{Config, ExportFormat, Exporter, Statistics};
use crate::ui::settings::{SettingsAction, SettingsView};
use crate::ui::stats::StatsAction;
use crate::ui::theme::Theme;
use crate::ui::timer_view::TimerAction;
use crate::platform::SystemTray;

use super::PomodoRustApp;
use super::View;

impl PomodoRustApp {
    /// Handle timer completion
    pub(super) fn on_timer_completed(&mut self) {
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
            let t = crate::i18n::tr();
            let (title, body): (&str, String) = if goal_just_reached
                && self.config.goals.notify_on_goal
            {
                (
                    t.notif.daily_goal_reached,
                    format!("{} {}", self.config.goals.daily_target, t.settings.pomodoros),
                )
            } else {
                match session_type {
                    SessionType::Work => (t.notif.focus_complete, t.notif.time_for_break.to_string()),
                    SessionType::ShortBreak => (t.notif.break_over, t.notif.ready_to_focus.to_string()),
                    SessionType::LongBreak => {
                        (t.notif.long_break_over, t.notif.back_to_work.to_string())
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

    /// Handle timer action
    pub(super) fn handle_timer_action(&mut self, action: TimerAction) {
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
    pub(super) fn handle_stats_action(&mut self, action: StatsAction) {
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
                    crate::i18n::tr().notif.stats_reset,
                    crate::i18n::tr().notif.stats_cleared,
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
                    crate::i18n::tr().notif.session_undone,
                    crate::i18n::tr().notif.session_removed,
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
            .set_title(crate::i18n::tr().notif.export_statistics)
            .set_file_name(&default_filename)
            .add_filter(filter_name, &[filter_ext]);

        // Show save dialog
        if let Some(path) = file_dialog.save_file() {
            match Exporter::export(db, &path, format) {
                Ok(()) => {
                    tracing::info!("Statistics exported to {:?}", path);
                    // Show success notification
                    crate::platform::show_notification(
                        crate::i18n::tr().notif.export_complete,
                        &format!("Statistics saved to {}", path.display()),
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to export statistics: {}", e);
                    crate::platform::show_notification(crate::i18n::tr().notif.export_failed, &format!("Error: {}", e));
                }
            }
        }
    }

    /// Handle settings action
    pub(super) fn handle_settings_action(&mut self, action: SettingsAction, ctx: &egui::Context) {
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
                let t = crate::i18n::tr();
                let preset_names = [t.settings.preset_classic, t.settings.preset_short, t.settings.preset_long];
                if let Some(preset) = presets.get(index) {
                    self.config.apply_preset(preset);
                    self.session.set_preset(preset.clone());
                    let _ = self.config.save();

                    if let Some(ref mut sv) = self.settings_view {
                        sv.reset_from_config(&self.config);
                    }
                    self.show_status(format!("{} {}", preset_names[index], t.settings.preset_applied));
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

                // Reset language to auto
                crate::i18n::set_language(self.config.appearance.language);

                // Reset always on top to default (false)
                self.set_always_on_top(false, ctx);

                if let Some(ref mut sv) = self.settings_view {
                    sv.reset_from_config(&self.config);
                }
                self.show_status(crate::i18n::tr().notif.defaults_restored);
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
        // Check if language changed
        if new_config.appearance.language != self.config.appearance.language {
            crate::i18n::set_language(new_config.appearance.language);
        }

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
        self.show_status(crate::i18n::tr().notif.settings_saved);
    }
}
