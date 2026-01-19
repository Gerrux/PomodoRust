//! PomodoRust - A modern, lightweight Pomodoro timer
//!
//! This library provides all the core functionality for the PomodoRust application.

pub mod app;
pub mod core;
pub mod data;
pub mod error;
pub mod ipc;
pub mod platform;
pub mod ui;
pub mod utils;

pub use app::PomodoRustApp;
pub use error::{Error, Result};
pub use ipc::{is_app_running, send_command, IpcCommand, IpcResponse, IpcServer};
