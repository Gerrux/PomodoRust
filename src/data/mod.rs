//! Data layer - configuration and persistence
//!
//! This module handles all data storage and retrieval:
//!
//! - [`Config`]: Application configuration stored in TOML format
//! - [`Database`]: SQLite database for session history and statistics
//! - [`Statistics`]: Aggregated statistics loaded from the database
//!
//! ## Storage Locations
//!
//! Configuration and data are stored in platform-specific directories:
//!
//! - **Windows**: `%APPDATA%/PomodoRust/`
//! - **macOS**: `~/Library/Application Support/com.pomodorust.PomodoRust/`
//! - **Linux**: `~/.config/pomodorust/`
//!
//! ## Database Schema
//!
//! The SQLite database contains three tables:
//!
//! - `sessions`: Individual session records with timing data
//! - `daily_stats`: Aggregated daily statistics
//! - `streaks`: Current and longest streak tracking

mod config;
mod database;
pub mod export;
mod statistics;

pub use config::{Config, GoalsConfig, NotificationSound};
pub use database::{Database, LastSession};
pub use export::{ExportFormat, Exporter};
pub use statistics::Statistics;
