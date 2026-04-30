use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::data::todo::{Priority, Project, TodoItem, Workspace};

use super::Database;

impl Database {
    // Workspaces

    pub fn create_workspace(
        &self,
        name: &str,
        icon: Option<&str>,
        color: Option<&str>,
    ) -> rusqlite::Result<i64> {
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
            return Err(rusqlite::Error::InvalidParameterName(
                "duplicate workspace name".into(),
            ));
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

    pub fn get_workspaces(&self) -> rusqlite::Result<Vec<Workspace>> {
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

    pub fn update_workspace(&self, workspace: &Workspace) -> rusqlite::Result<()> {
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

    pub fn delete_workspace(&self, id: i64) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM workspaces WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_workspaces(&self, ids: &[i64]) -> rusqlite::Result<()> {
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
    ) -> rusqlite::Result<i64> {
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
            return Err(rusqlite::Error::InvalidParameterName(
                "duplicate project name".into(),
            ));
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

    pub fn get_projects(&self, workspace_id: i64) -> rusqlite::Result<Vec<Project>> {
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

    pub fn update_project(&self, project: &Project) -> rusqlite::Result<()> {
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

    pub fn delete_project(&self, id: i64) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_projects(&self, ids: &[i64]) -> rusqlite::Result<()> {
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
    ) -> rusqlite::Result<i64> {
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
    ) -> rusqlite::Result<i64> {
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

    pub fn get_todos(&self, workspace_id: i64) -> rusqlite::Result<Vec<TodoItem>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, project_id, workspace_id, title, body, completed, collapsed, priority, position, created_at, completed_at
               FROM todo_items WHERE workspace_id = ?1 ORDER BY completed ASC, priority DESC, position ASC"#,
        )?;
        let rows = stmt.query_map(params![workspace_id], Self::row_to_todo)?;
        rows.collect()
    }

    pub fn get_todos_by_project(&self, project_id: i64) -> rusqlite::Result<Vec<TodoItem>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, project_id, workspace_id, title, body, completed, collapsed, priority, position, created_at, completed_at
               FROM todo_items WHERE project_id = ?1 ORDER BY completed ASC, priority DESC, position ASC"#,
        )?;
        let rows = stmt.query_map(params![project_id], Self::row_to_todo)?;
        rows.collect()
    }

    pub fn get_unassigned_todos(&self, workspace_id: i64) -> rusqlite::Result<Vec<TodoItem>> {
        let mut stmt = self.conn.prepare(
            r#"SELECT id, project_id, workspace_id, title, body, completed, collapsed, priority, position, created_at, completed_at
               FROM todo_items WHERE workspace_id = ?1 AND project_id IS NULL ORDER BY completed ASC, priority DESC, position ASC"#,
        )?;
        let rows = stmt.query_map(params![workspace_id], Self::row_to_todo)?;
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

    pub fn update_todo(&self, todo: &TodoItem) -> rusqlite::Result<()> {
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

    pub fn toggle_todo(&self, id: i64) -> rusqlite::Result<bool> {
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

    pub fn toggle_todo_collapsed(&self, id: i64) -> rusqlite::Result<bool> {
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

    pub fn delete_todo(&self, id: i64) -> rusqlite::Result<()> {
        self.conn
            .execute("DELETE FROM todo_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_todos(&self, ids: &[i64]) -> rusqlite::Result<()> {
        let mut stmt = self
            .conn
            .prepare("UPDATE todo_items SET position = ?1 WHERE id = ?2")?;
        for (i, id) in ids.iter().enumerate() {
            stmt.execute(params![i as i32, id])?;
        }
        Ok(())
    }

    pub fn move_todo(&self, id: i64, project_id: Option<i64>) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE todo_items SET project_id = ?1 WHERE id = ?2",
            params![project_id, id],
        )?;
        Ok(())
    }

    /// Move todo to a project and insert at a specific position, shifting others down
    pub fn reorder_todo_to(
        &self,
        id: i64,
        project_id: Option<i64>,
        new_position: i32,
    ) -> rusqlite::Result<()> {
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

    pub fn set_todo_priority(&self, id: i64, priority: Priority) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE todo_items SET priority = ?1 WHERE id = ?2",
            params![priority as i32, id],
        )?;
        Ok(())
    }
}
