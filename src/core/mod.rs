//! Core business logic for the Pomodoro timer
//!
//! This module contains the core domain logic, independent of UI:
//!
//! - [`Timer`]: Countdown timer with precise timing using `Instant`
//! - [`Session`]: Manages the Pomodoro workflow (work/break cycles)
//! - [`Preset`]: Timer configuration presets (work/break durations)
//!
//! ## Architecture
//!
//! The core module follows a state machine pattern:
//!
//! ```text
//! Timer States: Idle -> Running <-> Paused -> Completed
//! Session Flow: Work -> ShortBreak -> Work -> ... -> LongBreak
//! ```
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use pomodorust::core::{Session, Preset};
//!
//! let mut session = Session::with_preset(Preset::classic());
//! session.start();
//!
//! // In your event loop:
//! let (event, should_auto_start) = session.update();
//! if let Some(TimerEvent::Completed) = event {
//!     // Session completed, session auto-transitions to next type
//! }
//! ```

mod preset;
mod session;
mod timer;

pub use preset::{Preset, PresetManager};
pub use session::{Session, SessionState, SessionType};
pub use timer::{Timer, TimerEvent, TimerState};
