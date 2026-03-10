//! SQLite database for session statistics
//!
//! Provides persistence for:
//! - Individual session records
//! - Daily aggregated statistics
//! - Streak tracking

mod todo_ops;
mod queue_ops;

pub use queue_ops::TaskTimeStats;

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use std::path::PathBuf;

use crate::core::SessionType;

/// Date format used in the database (ISO 8601 date only)
const DATE_FORMAT: &str = "%Y-%m-%d";

/// Seconds per hour for conversion
const SECONDS_PER_HOUR: f32 = 3600.0;

/// Number of days in a week
const DAYS_IN_WEEK: usize = 7;

/// Database connection manager
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Get the database directory path
    fn db_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "pomodorust", "PomodoRust")
            .map(|dirs| dirs.data_dir().to_path_buf())
    }

    /// Get the database file path
    fn db_path() -> Option<PathBuf> {
        Self::db_dir().map(|dir| dir.join("pomodorust.db"))
    }

    /// Open or create the database with performance optimizations
    pub fn open() -> SqliteResult<Self> {
        let path = Self::db_path().unwrap_or_else(|| PathBuf::from("pomodorust.db"));

        // Create directory if needed
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }

        let conn = Connection::open(&path)?;

        // Apply SQLite performance optimizations for faster startup
        // These are especially important on Windows 10 with slower storage
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = 2000;
            PRAGMA temp_store = MEMORY;
            PRAGMA foreign_keys = ON;
            PRAGMA busy_timeout = 5000;
            "#,
        )?;

        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Open an in-memory database (for testing)
    pub fn open_in_memory() -> SqliteResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    /// Initialize database schema
    fn initialize(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            r#"
            -- Sessions table
            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_type TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                planned_duration INTEGER NOT NULL,
                completed INTEGER NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                todo_id INTEGER,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (todo_id) REFERENCES todo_items(id) ON DELETE SET NULL
            );

            -- Daily statistics (aggregated)
            CREATE TABLE IF NOT EXISTS daily_stats (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT UNIQUE NOT NULL,
                total_work_seconds INTEGER DEFAULT 0,
                total_break_seconds INTEGER DEFAULT 0,
                completed_pomodoros INTEGER DEFAULT 0,
                interrupted_pomodoros INTEGER DEFAULT 0,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Streak tracking
            CREATE TABLE IF NOT EXISTS streaks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                current_streak INTEGER DEFAULT 0,
                longest_streak INTEGER DEFAULT 0,
                last_active_date TEXT,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            );

            -- Indexes
            CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
            CREATE INDEX IF NOT EXISTS idx_daily_stats_date ON daily_stats(date);

            -- Initialize streaks if empty
            INSERT OR IGNORE INTO streaks (id, current_streak, longest_streak)
            VALUES (1, 0, 0);

            -- Todo: Workspaces
            CREATE TABLE IF NOT EXISTS workspaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                icon TEXT,
                color TEXT,
                collapsed INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Todo: Projects
            CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                workspace_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                color TEXT,
                collapsed INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );

            -- Todo: Items
            CREATE TABLE IF NOT EXISTS todo_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id INTEGER,
                workspace_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                body TEXT,
                completed INTEGER NOT NULL DEFAULT 0,
                collapsed INTEGER NOT NULL DEFAULT 1,
                priority INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                completed_at TEXT,
                FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
            );

            -- Todo: Pomodoro queue
            CREATE TABLE IF NOT EXISTS pomodoro_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                todo_id INTEGER NOT NULL,
                planned_pomodoros INTEGER NOT NULL DEFAULT 1,
                completed_pomodoros INTEGER NOT NULL DEFAULT 0,
                position INTEGER NOT NULL DEFAULT 0,
                added_at TEXT NOT NULL DEFAULT (datetime('now')),
                FOREIGN KEY (todo_id) REFERENCES todo_items(id) ON DELETE CASCADE
            );

            -- Todo indexes
            CREATE INDEX IF NOT EXISTS idx_todo_workspace ON todo_items(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_todo_project ON todo_items(project_id);
            CREATE INDEX IF NOT EXISTS idx_projects_workspace ON projects(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_queue_position ON pomodoro_queue(position);
            "#,
        )?;

        // Create default workspace if none exist
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM workspaces",
            [],
            |row| row.get(0),
        )?;
        if count == 0 {
            self.conn.execute(
                "INSERT INTO workspaces (name, icon, position) VALUES (?1, ?2, 0)",
                params!["Задачи", Option::<String>::None],
            )?;
        }

        // Migrations for existing databases
        self.migrate_sessions_todo_id()?;
        self.migrate_todo_priority()?;

        Ok(())
    }

    /// Add todo_id column to sessions table (migration for existing databases)
    fn migrate_sessions_todo_id(&self) -> SqliteResult<()> {
        let has_column: bool = self
            .conn
            .prepare("SELECT todo_id FROM sessions LIMIT 0")
            .is_ok();
        if !has_column {
            self.conn.execute_batch(
                "ALTER TABLE sessions ADD COLUMN todo_id INTEGER REFERENCES todo_items(id) ON DELETE SET NULL;",
            )?;
            tracing::info!("Migrated sessions table: added todo_id column");
        }
        Ok(())
    }

    /// Add priority column to todo_items table (migration for existing databases)
    fn migrate_todo_priority(&self) -> SqliteResult<()> {
        let has_column: bool = self
            .conn
            .prepare("SELECT priority FROM todo_items LIMIT 0")
            .is_ok();
        if !has_column {
            self.conn.execute_batch(
                "ALTER TABLE todo_items ADD COLUMN priority INTEGER NOT NULL DEFAULT 0;",
            )?;
            tracing::info!("Migrated todo_items table: added priority column");
        }
        Ok(())
    }

    /// Record a completed session
    pub fn record_session(
        &self,
        session_type: SessionType,
        duration_secs: u64,
        planned_duration_secs: u64,
        completed: bool,
        started_at: DateTime<Utc>,
        todo_id: Option<i64>,
    ) -> SqliteResult<()> {
        let ended_at = Utc::now();
        let today = Self::today_string();

        // Insert session record
        self.insert_session_record(
            session_type,
            duration_secs,
            planned_duration_secs,
            completed,
            &started_at,
            &ended_at,
            todo_id,
        )?;

        // Ensure daily stats row exists
        self.ensure_daily_stats(&today)?;

        // Update daily stats based on session type
        self.update_daily_stats(session_type, duration_secs, completed, &today)?;

        // Update streak if completed work session
        if session_type == SessionType::Work && completed {
            self.update_streak()?;
        }

        Ok(())
    }

    /// Insert a session record into the sessions table
    fn insert_session_record(
        &self,
        session_type: SessionType,
        duration_secs: u64,
        planned_duration_secs: u64,
        completed: bool,
        started_at: &DateTime<Utc>,
        ended_at: &DateTime<Utc>,
        todo_id: Option<i64>,
    ) -> SqliteResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO sessions (session_type, duration_seconds, planned_duration, completed, started_at, ended_at, todo_id)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                session_type.as_str(),
                duration_secs as i64,
                planned_duration_secs as i64,
                completed as i32,
                started_at.to_rfc3339(),
                ended_at.to_rfc3339(),
                todo_id,
            ],
        )?;
        Ok(())
    }

    /// Ensure a daily_stats row exists for the given date
    fn ensure_daily_stats(&self, date: &str) -> SqliteResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO daily_stats (date, total_work_seconds, total_break_seconds, completed_pomodoros, interrupted_pomodoros)
            VALUES (?1, 0, 0, 0, 0)
            ON CONFLICT(date) DO NOTHING
            "#,
            params![date],
        )?;
        Ok(())
    }

    /// Update daily statistics for a session
    fn update_daily_stats(
        &self,
        session_type: SessionType,
        duration_secs: u64,
        completed: bool,
        date: &str,
    ) -> SqliteResult<()> {
        match session_type {
            SessionType::Work => {
                if completed {
                    self.conn.execute(
                        r#"
                        UPDATE daily_stats
                        SET total_work_seconds = total_work_seconds + ?1,
                            completed_pomodoros = completed_pomodoros + 1
                        WHERE date = ?2
                        "#,
                        params![duration_secs as i64, date],
                    )?;
                } else {
                    self.conn.execute(
                        r#"
                        UPDATE daily_stats
                        SET total_work_seconds = total_work_seconds + ?1,
                            interrupted_pomodoros = interrupted_pomodoros + 1
                        WHERE date = ?2
                        "#,
                        params![duration_secs as i64, date],
                    )?;
                }
            }
            SessionType::ShortBreak | SessionType::LongBreak => {
                self.conn.execute(
                    r#"
                    UPDATE daily_stats
                    SET total_break_seconds = total_break_seconds + ?1
                    WHERE date = ?2
                    "#,
                    params![duration_secs as i64, date],
                )?;
            }
        }
        Ok(())
    }

    /// Get today's date as a formatted string
    fn today_string() -> String {
        Local::now().format(DATE_FORMAT).to_string()
    }

    /// Get yesterday's date as a formatted string
    fn yesterday_string() -> String {
        (Local::now() - chrono::Duration::days(1))
            .format(DATE_FORMAT)
            .to_string()
    }

    /// Update streak tracking
    fn update_streak(&self) -> SqliteResult<()> {
        let today = Self::today_string();
        let yesterday = Self::yesterday_string();

        // Get current streak info
        let (current_streak, last_date): (i32, Option<String>) = self.conn.query_row(
            "SELECT current_streak, last_active_date FROM streaks WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let new_streak = match last_date.as_deref() {
            Some(last) if last == today => current_streak, // Already counted today
            Some(last) if last == yesterday => current_streak + 1, // Continuing streak
            _ => 1,                                        // New streak
        };

        self.conn.execute(
            r#"
            UPDATE streaks
            SET current_streak = MAX(current_streak, ?1),
                longest_streak = MAX(longest_streak, ?1),
                last_active_date = ?2,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = 1
            "#,
            params![new_streak, today],
        )?;

        Ok(())
    }

    /// Get today's statistics (total work seconds, completed pomodoros)
    pub fn get_today_stats(&self) -> SqliteResult<(i64, i32)> {
        let today = Self::today_string();

        self.conn
            .query_row(
                r#"
                SELECT COALESCE(total_work_seconds, 0), COALESCE(completed_pomodoros, 0)
                FROM daily_stats WHERE date = ?1
                "#,
                params![today],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .or(Ok((0, 0)))
    }

    /// Get this week's daily hours (Monday = index 0)
    pub fn get_week_stats(&self) -> SqliteResult<Vec<f32>> {
        let today = Local::now().date_naive();
        self.get_week_stats_for_date(today)
    }

    /// Get daily hours for the week containing the given date (Monday = index 0)
    pub fn get_week_stats_for_date(&self, reference_date: NaiveDate) -> SqliteResult<Vec<f32>> {
        let start_of_week = reference_date
            - chrono::Duration::days(reference_date.weekday().num_days_from_monday() as i64);
        let end_of_week = start_of_week + chrono::Duration::days(6);

        let mut result = vec![0.0f32; DAYS_IN_WEEK];

        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, total_work_seconds
            FROM daily_stats
            WHERE date >= ?1 AND date <= ?2
            ORDER BY date
            "#,
        )?;

        let rows = stmt.query_map(
            params![
                start_of_week.format(DATE_FORMAT).to_string(),
                end_of_week.format(DATE_FORMAT).to_string()
            ],
            |row| {
                let date_str: String = row.get(0)?;
                let seconds: i64 = row.get(1)?;
                Ok((date_str, seconds))
            },
        )?;

        for (date_str, seconds) in rows.flatten() {
            if let Ok(date) = NaiveDate::parse_from_str(&date_str, DATE_FORMAT) {
                let day_index = (date - start_of_week).num_days() as usize;
                if day_index < DAYS_IN_WEEK {
                    result[day_index] = seconds as f32 / SECONDS_PER_HOUR;
                }
            }
        }

        Ok(result)
    }

    /// Get the earliest date with recorded stats (for navigation bounds)
    pub fn get_earliest_stats_date(&self) -> SqliteResult<Option<NaiveDate>> {
        self.conn
            .query_row(
                "SELECT MIN(date) FROM daily_stats",
                [],
                |row| row.get::<_, Option<String>>(0),
            )
            .map(|opt| opt.and_then(|s| NaiveDate::parse_from_str(&s, DATE_FORMAT).ok()))
    }

    /// Get streak information
    pub fn get_streak(&self) -> SqliteResult<(i32, i32)> {
        self.conn.query_row(
            "SELECT current_streak, longest_streak FROM streaks WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    }

    /// Get total statistics
    pub fn get_total_stats(&self) -> SqliteResult<(i64, i32)> {
        self.conn
            .query_row(
                r#"
            SELECT COALESCE(SUM(total_work_seconds), 0), COALESCE(SUM(completed_pomodoros), 0)
            FROM daily_stats
            "#,
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .or(Ok((0, 0)))
    }

    /// Get all session records for export
    pub fn get_all_sessions(&self) -> SqliteResult<Vec<super::export::SessionRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT id, session_type, duration_seconds, planned_duration, completed, started_at, ended_at, todo_id
            FROM sessions
            ORDER BY started_at DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(super::export::SessionRecord {
                id: row.get(0)?,
                session_type: row.get(1)?,
                duration_seconds: row.get(2)?,
                planned_duration: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                started_at: row.get(5)?,
                ended_at: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
                todo_id: row.get(7)?,
            })
        })?;

        rows.collect()
    }

    /// Get all daily statistics for export
    pub fn get_all_daily_stats(&self) -> SqliteResult<Vec<super::export::DailyStatsRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT date, total_work_seconds, total_break_seconds, completed_pomodoros, interrupted_pomodoros
            FROM daily_stats
            ORDER BY date DESC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let total_work_seconds: i64 = row.get(1)?;
            Ok(super::export::DailyStatsRecord {
                date: row.get(0)?,
                total_work_seconds,
                total_work_hours: total_work_seconds as f32 / 3600.0,
                total_break_seconds: row.get(2)?,
                completed_pomodoros: row.get(3)?,
                interrupted_pomodoros: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    /// Get the most recent work session (for undo)
    pub fn get_last_work_session(&self) -> SqliteResult<Option<LastSession>> {
        self.conn
            .query_row(
                r#"
                SELECT id, session_type, duration_seconds, completed, started_at
                FROM sessions
                WHERE session_type = 'work'
                ORDER BY id DESC
                LIMIT 1
                "#,
                [],
                |row| {
                    Ok(LastSession {
                        id: row.get(0)?,
                        session_type: row.get(1)?,
                        duration_seconds: row.get(2)?,
                        completed: row.get::<_, i32>(3)? != 0,
                        started_at: row.get(4)?,
                    })
                },
            )
            .optional()
    }

    /// Undo the last work session
    pub fn undo_last_session(&self) -> SqliteResult<Option<LastSession>> {
        // Get the last work session
        let last_session = self.get_last_work_session()?;

        if let Some(ref session) = last_session {
            // Parse date from started_at to update correct daily_stats
            let date = session
                .started_at
                .split('T')
                .next()
                .unwrap_or(&Self::today_string())
                .to_string();

            // Update daily stats
            if session.completed {
                self.conn.execute(
                    r#"
                    UPDATE daily_stats
                    SET total_work_seconds = MAX(0, total_work_seconds - ?1),
                        completed_pomodoros = MAX(0, completed_pomodoros - 1)
                    WHERE date = ?2
                    "#,
                    params![session.duration_seconds, date],
                )?;
            } else {
                self.conn.execute(
                    r#"
                    UPDATE daily_stats
                    SET total_work_seconds = MAX(0, total_work_seconds - ?1),
                        interrupted_pomodoros = MAX(0, interrupted_pomodoros - 1)
                    WHERE date = ?2
                    "#,
                    params![session.duration_seconds, date],
                )?;
            }

            // Delete the session
            self.conn
                .execute("DELETE FROM sessions WHERE id = ?1", params![session.id])?;

            tracing::info!("Undid last session: id={}", session.id);
        }

        Ok(last_session)
    }

    /// Reset all statistics (delete all sessions, daily stats, and reset streaks)
    pub fn reset_all_stats(&self) -> SqliteResult<()> {
        self.conn.execute_batch(
            r#"
            DELETE FROM sessions;
            DELETE FROM daily_stats;
            UPDATE streaks SET current_streak = 0, longest_streak = 0, last_active_date = NULL WHERE id = 1;
            "#,
        )?;

        tracing::info!("All statistics have been reset");
        Ok(())
    }
}

/// Information about the last session (for undo functionality)
#[derive(Debug, Clone)]
pub struct LastSession {
    pub id: i64,
    pub session_type: String,
    pub duration_seconds: i64,
    pub completed: bool,
    pub started_at: String,
}
