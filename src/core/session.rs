//! Session management for Pomodoro workflow

use super::{Preset, Timer, TimerEvent};
use serde::{Deserialize, Serialize};

/// Type of Pomodoro session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    Work,
    ShortBreak,
    LongBreak,
}

impl SessionType {
    pub fn label(&self) -> &'static str {
        match self {
            SessionType::Work => "FOCUS",
            SessionType::ShortBreak => "SHORT BREAK",
            SessionType::LongBreak => "LONG BREAK",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            SessionType::Work => "🍅",
            SessionType::ShortBreak => "☕",
            SessionType::LongBreak => "🌴",
        }
    }
}

/// Overall session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Ready,
    Active,
    Paused,
    Completed,
}

/// Manages the Pomodoro session workflow
#[derive(Debug)]
pub struct Session {
    /// Current timer
    timer: Timer,
    /// Current session type
    session_type: SessionType,
    /// Number of completed work sessions
    completed_work_sessions: u32,
    /// Current preset settings
    preset: Preset,
    /// Whether to auto-start next session
    auto_start_breaks: bool,
    auto_start_work: bool,
}

impl Session {
    /// Create a new session with default preset
    pub fn new() -> Self {
        let preset = Preset::default();
        Self {
            timer: Timer::from_minutes(preset.work_duration),
            session_type: SessionType::Work,
            completed_work_sessions: 0,
            preset,
            auto_start_breaks: false,
            auto_start_work: false,
        }
    }

    /// Create a session with a specific preset
    pub fn with_preset(preset: Preset) -> Self {
        Self {
            timer: Timer::from_minutes(preset.work_duration),
            session_type: SessionType::Work,
            completed_work_sessions: 0,
            preset,
            auto_start_breaks: false,
            auto_start_work: false,
        }
    }

    /// Update preset
    pub fn set_preset(&mut self, preset: Preset) {
        self.preset = preset;
        // Reset to work session with new duration
        self.session_type = SessionType::Work;
        self.timer = Timer::from_minutes(self.preset.work_duration);
    }

    /// Set auto-start preferences
    pub fn set_auto_start(&mut self, breaks: bool, work: bool) {
        self.auto_start_breaks = breaks;
        self.auto_start_work = work;
    }

    /// Get the current timer
    pub fn timer(&self) -> &Timer {
        &self.timer
    }

    /// Get mutable timer
    pub fn timer_mut(&mut self) -> &mut Timer {
        &mut self.timer
    }

    /// Get current session type
    pub fn session_type(&self) -> SessionType {
        self.session_type
    }

    /// Get state
    pub fn state(&self) -> SessionState {
        match self.timer.state() {
            super::TimerState::Idle => SessionState::Ready,
            super::TimerState::Running => SessionState::Active,
            super::TimerState::Paused => SessionState::Paused,
            super::TimerState::Completed => SessionState::Completed,
        }
    }

    /// Get completed work sessions count
    pub fn completed_work_sessions(&self) -> u32 {
        self.completed_work_sessions
    }

    /// Get sessions until long break
    pub fn sessions_until_long_break(&self) -> u32 {
        let remaining = self.preset.sessions_before_long_break
            - (self.completed_work_sessions % self.preset.sessions_before_long_break);
        if remaining == self.preset.sessions_before_long_break && self.completed_work_sessions > 0 {
            0
        } else {
            remaining
        }
    }

    /// Get total sessions in a cycle
    pub fn total_sessions_in_cycle(&self) -> u32 {
        self.preset.sessions_before_long_break
    }

    /// Get current session in cycle (1-indexed)
    pub fn current_session_in_cycle(&self) -> u32 {
        (self.completed_work_sessions % self.preset.sessions_before_long_break) + 1
    }

    /// Start/resume the timer
    pub fn start(&mut self) -> TimerEvent {
        self.timer.start()
    }

    /// Pause the timer
    pub fn pause(&mut self) -> TimerEvent {
        self.timer.pause()
    }

    /// Toggle timer
    pub fn toggle(&mut self) -> TimerEvent {
        self.timer.toggle()
    }

    /// Reset current session
    pub fn reset(&mut self) -> TimerEvent {
        self.timer.reset()
    }

    /// Update timer and handle session transitions
    /// Returns (timer_event, should_auto_start)
    pub fn update(&mut self) -> (Option<TimerEvent>, bool) {
        let event = self.timer.update();

        if let Some(TimerEvent::Completed) = event {
            let should_auto_start = self.handle_completion();
            return (event, should_auto_start);
        }

        (event, false)
    }

    /// Handle session completion and transition to next
    /// Returns whether to auto-start the next session
    fn handle_completion(&mut self) -> bool {
        match self.session_type {
            SessionType::Work => {
                self.completed_work_sessions += 1;

                // Determine next break type
                if self.completed_work_sessions % self.preset.sessions_before_long_break == 0 {
                    self.transition_to(SessionType::LongBreak);
                } else {
                    self.transition_to(SessionType::ShortBreak);
                }

                self.auto_start_breaks
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                self.transition_to(SessionType::Work);
                self.auto_start_work
            }
        }
    }

    /// Transition to a specific session type
    fn transition_to(&mut self, session_type: SessionType) {
        self.session_type = session_type;
        let duration = match session_type {
            SessionType::Work => self.preset.work_duration,
            SessionType::ShortBreak => self.preset.short_break,
            SessionType::LongBreak => self.preset.long_break,
        };
        self.timer = Timer::from_minutes(duration);
    }

    /// Skip to next session
    pub fn skip(&mut self) {
        // If currently working, don't count as completed
        match self.session_type {
            SessionType::Work => {
                // Skip to break (short break by default when skipping)
                self.transition_to(SessionType::ShortBreak);
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                self.transition_to(SessionType::Work);
            }
        }
    }

    /// Force transition to a specific session type
    pub fn switch_to(&mut self, session_type: SessionType) {
        self.transition_to(session_type);
    }

    /// Get session duration for a type (in minutes)
    pub fn duration_for(&self, session_type: SessionType) -> u32 {
        match session_type {
            SessionType::Work => self.preset.work_duration,
            SessionType::ShortBreak => self.preset.short_break,
            SessionType::LongBreak => self.preset.long_break,
        }
    }

    /// Get the current preset
    pub fn preset(&self) -> &Preset {
        &self.preset
    }

    /// Reset session count (for new day)
    pub fn reset_session_count(&mut self) {
        self.completed_work_sessions = 0;
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}
