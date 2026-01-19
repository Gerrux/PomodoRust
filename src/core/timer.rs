//! Timer logic with state management

use std::time::{Duration, Instant};

/// Timer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Completed,
}

/// Events emitted by the timer
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerEvent {
    Started,
    Paused,
    Resumed,
    Reset,
    Completed,
    Tick,
}

/// Core timer implementation
#[derive(Debug)]
pub struct Timer {
    /// Total duration for this timer
    total_duration: Duration,
    /// Remaining time
    remaining: Duration,
    /// Current state
    state: TimerState,
    /// When the timer was last started/resumed
    last_tick: Option<Instant>,
    /// Accumulated elapsed time (for pause/resume)
    elapsed_while_running: Duration,
}

impl Timer {
    /// Create a new timer with the given duration in seconds
    pub fn new(duration_secs: u64) -> Self {
        let duration = Duration::from_secs(duration_secs);
        Self {
            total_duration: duration,
            remaining: duration,
            state: TimerState::Idle,
            last_tick: None,
            elapsed_while_running: Duration::ZERO,
        }
    }

    /// Create a timer from minutes
    pub fn from_minutes(minutes: u32) -> Self {
        Self::new(minutes as u64 * 60)
    }

    /// Start or resume the timer
    pub fn start(&mut self) -> TimerEvent {
        match self.state {
            TimerState::Idle | TimerState::Completed => {
                self.state = TimerState::Running;
                self.last_tick = Some(Instant::now());
                self.elapsed_while_running = Duration::ZERO;
                TimerEvent::Started
            }
            TimerState::Paused => {
                self.state = TimerState::Running;
                self.last_tick = Some(Instant::now());
                TimerEvent::Resumed
            }
            TimerState::Running => TimerEvent::Tick,
        }
    }

    /// Pause the timer
    pub fn pause(&mut self) -> TimerEvent {
        if self.state == TimerState::Running {
            if let Some(last) = self.last_tick {
                self.elapsed_while_running += last.elapsed();
            }
            self.state = TimerState::Paused;
            self.last_tick = None;
            TimerEvent::Paused
        } else {
            TimerEvent::Tick
        }
    }

    /// Toggle between running and paused states
    pub fn toggle(&mut self) -> TimerEvent {
        match self.state {
            TimerState::Running => self.pause(),
            TimerState::Idle | TimerState::Paused | TimerState::Completed => self.start(),
        }
    }

    /// Reset the timer to its initial state
    pub fn reset(&mut self) -> TimerEvent {
        self.remaining = self.total_duration;
        self.state = TimerState::Idle;
        self.last_tick = None;
        self.elapsed_while_running = Duration::ZERO;
        TimerEvent::Reset
    }

    /// Reset with a new duration
    pub fn reset_with_duration(&mut self, duration_secs: u64) {
        self.total_duration = Duration::from_secs(duration_secs);
        self.reset();
    }

    /// Update the timer (call this every frame)
    /// Returns Some(event) if an event occurred
    pub fn update(&mut self) -> Option<TimerEvent> {
        if self.state != TimerState::Running {
            return None;
        }

        let now = Instant::now();
        let elapsed = if let Some(last) = self.last_tick {
            self.elapsed_while_running + now.duration_since(last)
        } else {
            self.elapsed_while_running
        };

        if elapsed >= self.total_duration {
            self.remaining = Duration::ZERO;
            self.state = TimerState::Completed;
            self.last_tick = None;
            Some(TimerEvent::Completed)
        } else {
            self.remaining = self.total_duration - elapsed;
            Some(TimerEvent::Tick)
        }
    }

    /// Get the current state
    pub fn state(&self) -> TimerState {
        self.state
    }

    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.state == TimerState::Running
    }

    /// Check if timer is paused
    pub fn is_paused(&self) -> bool {
        self.state == TimerState::Paused
    }

    /// Check if timer is completed
    pub fn is_completed(&self) -> bool {
        self.state == TimerState::Completed
    }

    /// Get remaining time
    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    /// Get remaining seconds
    pub fn remaining_secs(&self) -> u64 {
        self.remaining.as_secs()
    }

    /// Get total duration
    pub fn total_duration(&self) -> Duration {
        self.total_duration
    }

    /// Get progress as a value between 0.0 and 1.0
    pub fn progress(&self) -> f32 {
        if self.total_duration.as_secs() == 0 {
            return 1.0;
        }
        let elapsed = self.total_duration.as_secs_f32() - self.remaining.as_secs_f32();
        (elapsed / self.total_duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    /// Get remaining time in milliseconds (for precise animations)
    pub fn remaining_millis(&self) -> u64 {
        self.remaining.as_millis() as u64
    }

    /// Get precise progress with sub-second accuracy (for smooth animations)
    pub fn progress_precise(&self) -> f32 {
        if self.total_duration.as_millis() == 0 {
            return 1.0;
        }
        let elapsed_ms = self.total_duration.as_millis() as f32 - self.remaining.as_millis() as f32;
        (elapsed_ms / self.total_duration.as_millis() as f32).clamp(0.0, 1.0)
    }

    /// Get remaining time formatted as MM:SS
    pub fn remaining_formatted(&self) -> String {
        // Round up to show accurate countdown (e.g., 59.1 seconds shows as 01:00)
        let total_secs = (self.remaining.as_millis() as f64 / 1000.0).ceil() as u64;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Get remaining time formatted with hours if needed (HH:MM:SS or MM:SS)
    pub fn remaining_formatted_full(&self) -> String {
        let total_secs = self.remaining.as_secs();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::from_minutes(25)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_creation() {
        let timer = Timer::from_minutes(25);
        assert_eq!(timer.remaining_secs(), 25 * 60);
        assert_eq!(timer.state(), TimerState::Idle);
    }

    #[test]
    fn test_timer_format() {
        let timer = Timer::new(90);
        assert_eq!(timer.remaining_formatted(), "01:30");
    }

    #[test]
    fn test_timer_progress() {
        let timer = Timer::from_minutes(25);
        assert_eq!(timer.progress(), 0.0);
    }
}
