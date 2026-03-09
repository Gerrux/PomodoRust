//! SQLite database for session statistics
//!
//! Provides persistence for:
//! - Individual session records
//! - Daily aggregated statistics
//! - Streak tracking

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection, OptionalExtension, Result as SqliteResult};
use std::path::PathBuf;

use crate::core::SessionType;
use crate::data::todo::{Priority, Project, QueuedTask, TodoItem, Workspace};

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

// ── Todo CRUD ──────────────────────────────────────────────────────

impl Database {
    // Workspaces

    pub fn create_workspace(
        &self,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
    ) -> SqliteResult<i64> {
        let name = name.trim();
        if name.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("empty name".into()));
        }
        // Check duplicate name
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM workspaces WHERE name = ?1",
            params![name],
            |row| row.get(0),
        )?;
        if exists {
            return Err(rusqlite::Error::InvalidParameterName("duplicate workspace name".into()));
        }
        let pos: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM workspaces",
            [],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO workspaces (name, icon, color, position) VALUES (?1, ?2, ?3, ?4)",
            params![name, icon, color, pos],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_workspaces(&self) -> SqliteResult<Vec<Workspace>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon, color, collapsed, position FROM workspaces ORDER BY position",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                name: row.get(1)?,
                icon: row.get(2)?,
                color: row.get(3)?,
                collapsed: row.get::<_, i32>(4)? != 0,
                position: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_workspace(&self, workspace: &Workspace) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE workspaces SET name = ?1, icon = ?2, color = ?3, collapsed = ?4, position = ?5 WHERE id = ?6",
            params![
                workspace.name,
                workspace.icon,
                workspace.color,
                workspace.collapsed as i32,
                workspace.position,
                workspace.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_workspace(&self, id: i64) -> SqliteResult<()> {
        self.conn
            .execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_workspaces(&self, ids: &[i64]) -> SqliteResult<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE workspaces SET position = ?1 WHERE id = ?2")?;
        for (i, id) in ids.iter().enumerate() {
            stmt.execute(params![i as i32, id])?;
        }
        Ok(())
    }

    // Projects

    pub fn create_project(
        &self,
        workspace_id: i64,
        name: &str,
        color: Option<&str>,
    ) -> SqliteResult<i64> {
        let name = name.trim();
        if name.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("empty name".into()));
        }
        // Check duplicate name within workspace
        let exists: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM projects WHERE workspace_id = ?1 AND name = ?2",
            params![workspace_id, name],
            |row| row.get(0),
        )?;
        if exists {
            return Err(rusqlite::Error::InvalidParameterName("duplicate project name".into()));
        }
        let pos: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM projects WHERE workspace_id = ?1",
            params![workspace_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO projects (workspace_id, name, color, position) VALUES (?1, ?2, ?3, ?4)",
            params![workspace_id, name, color, pos],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_projects(&self, workspace_id: i64) -> SqliteResult<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, name, color, collapsed, position FROM projects WHERE workspace_id = ?1 ORDER BY position",
        )?;
        let rows = stmt.query_map(params![workspace_id], |row| {
            Ok(Project {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                name: row.get(2)?,
                color: row.get(3)?,
                collapsed: row.get::<_, i32>(4)? != 0,
                position: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_project(&self, project: &Project) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE projects SET name = ?1, color = ?2, collapsed = ?3, position = ?4 WHERE id = ?5",
            params![
                project.name,
                project.color,
                project.collapsed as i32,
                project.position,
                project.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_project(&self, id: i64) -> SqliteResult<()> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_projects(&self, ids: &[i64]) -> SqliteResult<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE projects SET position = ?1 WHERE id = ?2")?;
        for (i, id) in ids.iter().enumerate() {
            stmt.execute(params![i as i32, id])?;
        }
        Ok(())
    }

    // Todo Items

    pub fn create_todo(
        &self,
        workspace_id: i64,
        project_id: Option<i64>,
        title: &str,
    ) -> SqliteResult<i64> {
        let title = title.trim();
        if title.is_empty() {
            return Err(rusqlite::Error::InvalidParameterName("empty title".into()));
        }
        let pos: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM todo_items WHERE workspace_id = ?1",
            params![workspace_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO todo_items (workspace_id, project_id, title, position) VALUES (?1, ?2, ?3, ?4)",
            params![workspace_id, project_id, title, pos],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn create_todo_with_body(
        &self,
        workspace_id: i64,
        project_id: Option<i64>,
        title: &str,
        body: &str,
    ) -> SqliteResult<i64> {
        let pos: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM todo_items WHERE workspace_id = ?1",
            params![workspace_id],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO todo_items (workspace_id, project_id, title, body, position) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![workspace_id, project_id, title, body, pos],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_todos(&self, workspace_id: i64) -> SqliteResult<Vec<TodoItem>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, project_id, workspace_id, title, body, completed, collapsed, priority, position, created_at, completed_at
               FROM todo_items WHERE workspace_id = ?1 ORDER BY completed ASC, priority DESC, position ASC"#,
        )?;
        let rows = stmt.query_map(params![workspace_id], |row| Self::row_to_todo(row))?;
        rows.collect()
    }

    pub fn get_todos_by_project(&self, project_id: i64) -> SqliteResult<Vec<TodoItem>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, project_id, workspace_id, title, body, completed, collapsed, priority, position, created_at, completed_at
               FROM todo_items WHERE project_id = ?1 ORDER BY completed ASC, priority DESC, position ASC"#,
        )?;
        let rows = stmt.query_map(params![project_id], |row| Self::row_to_todo(row))?;
        rows.collect()
    }

    pub fn get_unassigned_todos(&self, workspace_id: i64) -> SqliteResult<Vec<TodoItem>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, project_id, workspace_id, title, body, completed, collapsed, priority, position, created_at, completed_at
               FROM todo_items WHERE workspace_id = ?1 AND project_id IS NULL ORDER BY completed ASC, priority DESC, position ASC"#,
        )?;
        let rows = stmt.query_map(params![workspace_id], |row| Self::row_to_todo(row))?;
        rows.collect()
    }

    fn row_to_todo(row: &rusqlite::Row) -> rusqlite::Result<TodoItem> {
        let created_str: String = row.get(9)?;
        let completed_str: Option<String> = row.get(10)?;
        Ok(TodoItem {
            id: row.get(0)?,
            project_id: row.get(1)?,
            workspace_id: row.get(2)?,
            title: row.get(3)?,
            body: row.get(4)?,
            completed: row.get::<_, i32>(5)? != 0,
            collapsed: row.get::<_, i32>(6)? != 0,
            priority: Priority::from_i32(row.get::<_, i32>(7)?),
            position: row.get(8)?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            completed_at: completed_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|d| d.with_timezone(&Utc))
                    .ok()
            }),
        })
    }

    pub fn update_todo(&self, todo: &TodoItem) -> SqliteResult<()> {
        self.conn.execute(
            r#"UPDATE todo_items SET project_id = ?1, title = ?2, body = ?3, completed = ?4,
               collapsed = ?5, priority = ?6, position = ?7, completed_at = ?8 WHERE id = ?9"#,
            params![
                todo.project_id,
                todo.title,
                todo.body,
                todo.completed as i32,
                todo.collapsed as i32,
                todo.priority as i32,
                todo.position,
                todo.completed_at.map(|d| d.to_rfc3339()),
                todo.id,
            ],
        )?;
        Ok(())
    }

    pub fn toggle_todo(&self, id: i64) -> SqliteResult<bool> {
        self.conn.execute(
            r#"UPDATE todo_items SET completed = NOT completed,
               completed_at = CASE WHEN completed = 0 THEN datetime('now') ELSE NULL END
               WHERE id = ?1"#,
            params![id],
        )?;
        let completed: bool = self.conn.query_row(
            "SELECT completed FROM todo_items WHERE id = ?1",
            params![id],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        )?;
        Ok(completed)
    }

    pub fn toggle_todo_collapsed(&self, id: i64) -> SqliteResult<bool> {
        self.conn.execute(
            "UPDATE todo_items SET collapsed = NOT collapsed WHERE id = ?1",
            params![id],
        )?;
        let collapsed: bool = self.conn.query_row(
            "SELECT collapsed FROM todo_items WHERE id = ?1",
            params![id],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        )?;
        Ok(collapsed)
    }

    pub fn delete_todo(&self, id: i64) -> SqliteResult<()> {
        self.conn
            .execute("DELETE FROM todo_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_todos(&self, ids: &[i64]) -> SqliteResult<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE todo_items SET position = ?1 WHERE id = ?2")?;
        for (i, id) in ids.iter().enumerate() {
            stmt.execute(params![i as i32, id])?;
        }
        Ok(())
    }

    pub fn move_todo(&self, id: i64, project_id: Option<i64>) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE todo_items SET project_id = ?1 WHERE id = ?2",
            params![project_id, id],
        )?;
        Ok(())
    }

    /// Move todo to a project and insert at a specific position, shifting others down
    pub fn reorder_todo_to(&self, id: i64, project_id: Option<i64>, new_position: i32) -> SqliteResult<()> {
        let tx = self.conn.unchecked_transaction()?;
        // Update the project assignment
        tx.execute(
            "UPDATE todo_items SET project_id = ?1 WHERE id = ?2",
            params![project_id, id],
        )?;
        // Shift items at or after new_position down
        if let Some(pid) = project_id {
            tx.execute(
                "UPDATE todo_items SET position = position + 1 WHERE project_id = ?1 AND position >= ?2 AND id != ?3",
                params![pid, new_position, id],
            )?;
        } else {
            // For unassigned: workspace scope — get workspace_id from the todo
            let ws_id: i64 = tx.query_row(
                "SELECT workspace_id FROM todo_items WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )?;
            tx.execute(
                "UPDATE todo_items SET position = position + 1 WHERE workspace_id = ?1 AND project_id IS NULL AND position >= ?2 AND id != ?3",
                params![ws_id, new_position, id],
            )?;
        }
        // Set the todo's position
        tx.execute(
            "UPDATE todo_items SET position = ?1 WHERE id = ?2",
            params![new_position, id],
        )?;
        tx.commit()
    }

    pub fn set_todo_priority(&self, id: i64, priority: Priority) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE todo_items SET priority = ?1 WHERE id = ?2",
            params![priority as i32, id],
        )?;
        Ok(())
    }

    // Pomodoro Queue

    pub fn add_to_queue(&self, todo_id: i64, planned_pomodoros: u32) -> SqliteResult<i64> {
        let already: bool = self.conn.query_row(
            "SELECT COUNT(*) > 0 FROM pomodoro_queue WHERE todo_id = ?1",
            params![todo_id],
            |row| row.get(0),
        )?;
        if already {
            return Ok(-1);
        }
        let pos: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(position), -1) + 1 FROM pomodoro_queue",
            [],
            |row| row.get(0),
        )?;
        self.conn.execute(
            "INSERT INTO pomodoro_queue (todo_id, planned_pomodoros, position) VALUES (?1, ?2, ?3)",
            params![todo_id, planned_pomodoros, pos],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_queue(&self) -> SqliteResult<Vec<QueuedTask>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT q.id, q.todo_id, t.title, q.planned_pomodoros, q.completed_pomodoros, q.position
               FROM pomodoro_queue q JOIN todo_items t ON q.todo_id = t.id
               ORDER BY q.position ASC"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(QueuedTask {
                id: row.get(0)?,
                todo_id: row.get(1)?,
                title: row.get(2)?,
                planned_pomodoros: row.get::<_, u32>(3)?,
                completed_pomodoros: row.get::<_, u32>(4)?,
                position: row.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_current_queue_task(&self) -> SqliteResult<Option<QueuedTask>> {
        self.conn
            .query_row(
                r#"SELECT q.id, q.todo_id, t.title, q.planned_pomodoros, q.completed_pomodoros, q.position
                   FROM pomodoro_queue q JOIN todo_items t ON q.todo_id = t.id
                   ORDER BY q.position ASC LIMIT 1"#,
                [],
                |row| {
                    Ok(QueuedTask {
                        id: row.get(0)?,
                        todo_id: row.get(1)?,
                        title: row.get(2)?,
                        planned_pomodoros: row.get::<_, u32>(3)?,
                        completed_pomodoros: row.get::<_, u32>(4)?,
                        position: row.get(5)?,
                    })
                },
            )
            .optional()
    }

    pub fn remove_from_queue(&self, id: i64) -> SqliteResult<()> {
        self.conn
            .execute("DELETE FROM pomodoro_queue WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_queue(&self) -> SqliteResult<()> {
        self.conn.execute("DELETE FROM pomodoro_queue", [])?;
        Ok(())
    }

    pub fn increment_queue_pomodoro(&self, id: i64) -> SqliteResult<bool> {
        self.conn.execute(
            "UPDATE pomodoro_queue SET completed_pomodoros = completed_pomodoros + 1 WHERE id = ?1",
            params![id],
        )?;
        let done: bool = self.conn.query_row(
            "SELECT completed_pomodoros >= planned_pomodoros FROM pomodoro_queue WHERE id = ?1",
            params![id],
            |row| Ok(row.get::<_, i32>(0)? != 0),
        )?;
        Ok(done)
    }

    /// Complete current queue task atomically: toggle todo + advance queue in one transaction
    pub fn complete_queue_task(&self, queue_id: i64, todo_id: i64) -> SqliteResult<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            r#"UPDATE todo_items SET completed = NOT completed,
               completed_at = CASE WHEN completed = 0 THEN datetime('now') ELSE NULL END
               WHERE id = ?1"#,
            params![todo_id],
        )?;
        tx.execute(
            "DELETE FROM pomodoro_queue WHERE id = ?1",
            params![queue_id],
        )?;
        tx.commit()
    }

    pub fn advance_queue(&self) -> SqliteResult<Option<QueuedTask>> {
        // Remove the first item
        if let Some(current) = self.get_current_queue_task()? {
            self.remove_from_queue(current.id)?;
        }
        self.get_current_queue_task()
    }

    pub fn reorder_queue(&self, ids: &[i64]) -> SqliteResult<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE pomodoro_queue SET position = ?1 WHERE id = ?2")?;
        for (i, id) in ids.iter().enumerate() {
            stmt.execute(params![i as i32, id])?;
        }
        Ok(())
    }

    pub fn update_queue_planned(&self, id: i64, planned: u32) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE pomodoro_queue SET planned_pomodoros = ?1 WHERE id = ?2",
            params![planned, id],
        )?;
        Ok(())
    }
}

// ── Task time tracking ─────────────────────────────────────────────

/// Time spent on a specific task
#[derive(Debug, Clone)]
pub struct TaskTimeStats {
    pub todo_id: i64,
    pub title: String,
    pub total_seconds: i64,
    pub completed_pomodoros: i32,
}

impl Database {
    /// Get total time spent per task (only tasks that have recorded sessions)
    pub fn get_task_time_stats(&self) -> SqliteResult<Vec<TaskTimeStats>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT s.todo_id, t.title,
                   SUM(s.duration_seconds) as total_seconds,
                   COUNT(*) as completed_pomodoros
            FROM sessions s
            JOIN todo_items t ON s.todo_id = t.id
            WHERE s.todo_id IS NOT NULL
              AND s.session_type = 'work'
              AND s.completed = 1
            GROUP BY s.todo_id
            ORDER BY total_seconds DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(TaskTimeStats {
                todo_id: row.get(0)?,
                title: row.get(1)?,
                total_seconds: row.get(2)?,
                completed_pomodoros: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    /// Get time spent on a specific task
    pub fn get_task_time(&self, todo_id: i64) -> SqliteResult<i64> {
        self.conn
            .query_row(
                r#"
                SELECT COALESCE(SUM(duration_seconds), 0)
                FROM sessions
                WHERE todo_id = ?1 AND session_type = 'work' AND completed = 1
                "#,
                params![todo_id],
                |row| row.get(0),
            )
            .or(Ok(0))
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
