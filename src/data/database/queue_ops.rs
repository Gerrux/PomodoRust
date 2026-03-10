use rusqlite::{params, OptionalExtension};

use crate::data::todo::QueuedTask;

use super::Database;

impl Database {
    // Pomodoro Queue

    pub fn add_to_queue(&self, todo_id: i64, planned_pomodoros: u32) -> rusqlite::Result<i64> {
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

    pub fn get_queue(&self) -> rusqlite::Result<Vec<QueuedTask>> {
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

    pub fn get_current_queue_task(&self) -> rusqlite::Result<Option<QueuedTask>> {
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

    pub fn remove_from_queue(&self, id: i64) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM pomodoro_queue WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_queue(&self) -> rusqlite::Result<()> {
        self.conn.execute("DELETE FROM pomodoro_queue", [])?;
        Ok(())
    }

    pub fn increment_queue_pomodoro(&self, id: i64) -> rusqlite::Result<bool> {
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
    pub fn complete_queue_task(&self, queue_id: i64, todo_id: i64) -> rusqlite::Result<()> {
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

    pub fn advance_queue(&self) -> rusqlite::Result<Option<QueuedTask>> {
        // Remove the first item
        if let Some(current) = self.get_current_queue_task()? {
            self.remove_from_queue(current.id)?;
        }
        self.get_current_queue_task()
    }

    pub fn reorder_queue(&self, ids: &[i64]) -> rusqlite::Result<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE pomodoro_queue SET position = ?1 WHERE id = ?2")?;
        for (i, id) in ids.iter().enumerate() {
            stmt.execute(params![i as i32, id])?;
        }
        Ok(())
    }

    pub fn update_queue_planned(&self, id: i64, planned: u32) -> rusqlite::Result<()> {
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
    pub fn get_task_time_stats(&self) -> rusqlite::Result<Vec<TaskTimeStats>> {
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
    pub fn get_task_time(&self, todo_id: i64) -> rusqlite::Result<i64> {
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
