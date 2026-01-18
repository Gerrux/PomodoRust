//! SQLite database for statistics

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::path::PathBuf;

use crate::core::SessionType;

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

    /// Open or create the database
    pub fn open() -> SqliteResult<Self> {
        let path = Self::db_path().unwrap_or_else(|| PathBuf::from("pomodorust.db"));

        // Create directory if needed
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }

        let conn = Connection::open(&path)?;
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
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
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
            "#,
        )?;

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
    ) -> SqliteResult<()> {
        let session_type_str = match session_type {
            SessionType::Work => "work",
            SessionType::ShortBreak => "short_break",
            SessionType::LongBreak => "long_break",
        };

        let ended_at = Utc::now();

        // Insert session record
        self.conn.execute(
            r#"
            INSERT INTO sessions (session_type, duration_seconds, planned_duration, completed, started_at, ended_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                session_type_str,
                duration_secs as i64,
                planned_duration_secs as i64,
                completed as i32,
                started_at.to_rfc3339(),
                ended_at.to_rfc3339(),
            ],
        )?;

        // Update daily stats
        let today = Local::now().format("%Y-%m-%d").to_string();

        self.conn.execute(
            r#"
            INSERT INTO daily_stats (date, total_work_seconds, total_break_seconds, completed_pomodoros, interrupted_pomodoros)
            VALUES (?1, 0, 0, 0, 0)
            ON CONFLICT(date) DO NOTHING
            "#,
            params![today],
        )?;

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
                        params![duration_secs as i64, today],
                    )?;
                } else {
                    self.conn.execute(
                        r#"
                        UPDATE daily_stats
                        SET total_work_seconds = total_work_seconds + ?1,
                            interrupted_pomodoros = interrupted_pomodoros + 1
                        WHERE date = ?2
                        "#,
                        params![duration_secs as i64, today],
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
                    params![duration_secs as i64, today],
                )?;
            }
        }

        // Update streak if completed work session
        if session_type == SessionType::Work && completed {
            self.update_streak()?;
        }

        Ok(())
    }

    /// Update streak tracking
    fn update_streak(&self) -> SqliteResult<()> {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let yesterday = (Local::now() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();

        // Get current streak info
        let (current_streak, last_date): (i32, Option<String>) = self.conn.query_row(
            "SELECT current_streak, last_active_date FROM streaks WHERE id = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let new_streak = match last_date.as_deref() {
            Some(last) if last == today => current_streak, // Already counted today
            Some(last) if last == yesterday => current_streak + 1, // Continuing streak
            _ => 1,                                         // New streak
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

    /// Get today's statistics
    pub fn get_today_stats(&self) -> SqliteResult<(i64, i32)> {
        let today = Local::now().format("%Y-%m-%d").to_string();

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

    /// Get this week's daily hours
    pub fn get_week_stats(&self) -> SqliteResult<Vec<f32>> {
        let today = Local::now().date_naive();
        let start_of_week = today - chrono::Duration::days(today.weekday().num_days_from_monday() as i64);

        let mut result = vec![0.0f32; 7];

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
                start_of_week.format("%Y-%m-%d").to_string(),
                today.format("%Y-%m-%d").to_string()
            ],
            |row| {
                let date_str: String = row.get(0)?;
                let seconds: i64 = row.get(1)?;
                Ok((date_str, seconds))
            },
        )?;

        for row_result in rows {
            if let Ok((date_str, seconds)) = row_result {
                if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                    let day_index = (date - start_of_week).num_days() as usize;
                    if day_index < 7 {
                        result[day_index] = seconds as f32 / 3600.0;
                    }
                }
            }
        }

        Ok(result)
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
}
