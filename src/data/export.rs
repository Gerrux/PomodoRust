//! Statistics export functionality
//!
//! Provides export capabilities for statistics data in CSV and JSON formats.

use chrono::Local;
use serde::Serialize;
use std::path::Path;

use super::Database;

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
}

impl ExportFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "csv",
            ExportFormat::Json => "json",
        }
    }

    /// Get the display name for this format
    pub fn label(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "CSV",
            ExportFormat::Json => "JSON",
        }
    }
}

/// Session record for export
#[derive(Debug, Clone, Serialize)]
pub struct SessionRecord {
    pub id: i64,
    pub session_type: String,
    pub duration_seconds: i64,
    pub planned_duration: i64,
    pub completed: bool,
    pub started_at: String,
    pub ended_at: String,
}

/// Daily statistics record for export
#[derive(Debug, Clone, Serialize)]
pub struct DailyStatsRecord {
    pub date: String,
    pub total_work_seconds: i64,
    pub total_work_hours: f32,
    pub total_break_seconds: i64,
    pub completed_pomodoros: i32,
    pub interrupted_pomodoros: i32,
}

/// Summary statistics for export
#[derive(Debug, Clone, Serialize)]
pub struct SummaryStats {
    pub export_date: String,
    pub total_work_hours: f32,
    pub total_pomodoros: i32,
    pub total_days_tracked: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub average_daily_hours: f32,
    pub average_daily_pomodoros: f32,
}

/// Complete export data structure
#[derive(Debug, Clone, Serialize)]
pub struct ExportData {
    pub summary: SummaryStats,
    pub daily_stats: Vec<DailyStatsRecord>,
    pub sessions: Vec<SessionRecord>,
}

/// Statistics exporter
pub struct Exporter;

impl Exporter {
    /// Export all statistics to the specified path
    pub fn export(db: &Database, path: &Path, format: ExportFormat) -> Result<(), ExportError> {
        let data = Self::gather_data(db)?;

        match format {
            ExportFormat::Json => Self::export_json(&data, path),
            ExportFormat::Csv => Self::export_csv(&data, path),
        }
    }

    /// Gather all export data from the database
    fn gather_data(db: &Database) -> Result<ExportData, ExportError> {
        let sessions = db.get_all_sessions().map_err(ExportError::Database)?;
        let daily_stats = db.get_all_daily_stats().map_err(ExportError::Database)?;
        let (current_streak, longest_streak) = db.get_streak().unwrap_or((0, 0));

        let total_work_seconds: i64 = daily_stats.iter().map(|d| d.total_work_seconds).sum();
        let total_pomodoros: i32 = daily_stats.iter().map(|d| d.completed_pomodoros).sum();
        let total_days = daily_stats.len() as i32;

        let summary = SummaryStats {
            export_date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            total_work_hours: total_work_seconds as f32 / 3600.0,
            total_pomodoros,
            total_days_tracked: total_days,
            current_streak,
            longest_streak,
            average_daily_hours: if total_days > 0 {
                (total_work_seconds as f32 / 3600.0) / total_days as f32
            } else {
                0.0
            },
            average_daily_pomodoros: if total_days > 0 {
                total_pomodoros as f32 / total_days as f32
            } else {
                0.0
            },
        };

        Ok(ExportData {
            summary,
            daily_stats,
            sessions,
        })
    }

    /// Export data as JSON
    fn export_json(data: &ExportData, path: &Path) -> Result<(), ExportError> {
        let json = serde_json::to_string_pretty(data).map_err(ExportError::Serialization)?;
        std::fs::write(path, json).map_err(ExportError::Io)
    }

    /// Export data as CSV (multiple files in a directory or combined file)
    fn export_csv(data: &ExportData, path: &Path) -> Result<(), ExportError> {
        let mut content = String::new();

        // Summary section
        content.push_str("# Summary\n");
        content.push_str("Metric,Value\n");
        content.push_str(&format!("Export Date,{}\n", data.summary.export_date));
        content.push_str(&format!("Total Work Hours,{:.2}\n", data.summary.total_work_hours));
        content.push_str(&format!("Total Pomodoros,{}\n", data.summary.total_pomodoros));
        content.push_str(&format!("Days Tracked,{}\n", data.summary.total_days_tracked));
        content.push_str(&format!("Current Streak,{}\n", data.summary.current_streak));
        content.push_str(&format!("Longest Streak,{}\n", data.summary.longest_streak));
        content.push_str(&format!(
            "Average Daily Hours,{:.2}\n",
            data.summary.average_daily_hours
        ));
        content.push_str(&format!(
            "Average Daily Pomodoros,{:.2}\n",
            data.summary.average_daily_pomodoros
        ));
        content.push('\n');

        // Daily stats section
        content.push_str("# Daily Statistics\n");
        content.push_str(
            "Date,Work Seconds,Work Hours,Break Seconds,Completed Pomodoros,Interrupted Pomodoros\n",
        );
        for daily in &data.daily_stats {
            content.push_str(&format!(
                "{},{},{:.2},{},{},{}\n",
                daily.date,
                daily.total_work_seconds,
                daily.total_work_hours,
                daily.total_break_seconds,
                daily.completed_pomodoros,
                daily.interrupted_pomodoros
            ));
        }
        content.push('\n');

        // Sessions section
        content.push_str("# Sessions\n");
        content.push_str("ID,Type,Duration (s),Planned Duration (s),Completed,Started At,Ended At\n");
        for session in &data.sessions {
            content.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                session.id,
                session.session_type,
                session.duration_seconds,
                session.planned_duration,
                session.completed,
                session.started_at,
                session.ended_at
            ));
        }

        std::fs::write(path, content).map_err(ExportError::Io)
    }

    /// Generate a default filename for export
    pub fn default_filename(format: ExportFormat) -> String {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        format!("pomodorust_stats_{}.{}", timestamp, format.extension())
    }
}

/// Export error types
#[derive(Debug)]
pub enum ExportError {
    Database(rusqlite::Error),
    Io(std::io::Error),
    Serialization(serde_json::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportError::Database(e) => write!(f, "Database error: {}", e),
            ExportError::Io(e) => write!(f, "IO error: {}", e),
            ExportError::Serialization(e) => write!(f, "Serialization error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}
