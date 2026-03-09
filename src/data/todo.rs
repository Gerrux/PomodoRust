use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum Priority {
    #[default]
    None = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Urgent = 4,
}

impl Priority {
    pub fn all() -> &'static [Priority] {
        &[
            Priority::None,
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Urgent,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            Priority::None => "—",
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Urgent => "Urgent",
        }
    }

    pub fn from_i32(v: i32) -> Self {
        match v {
            1 => Priority::Low,
            2 => Priority::Medium,
            3 => Priority::High,
            4 => Priority::Urgent,
            _ => Priority::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub collapsed: bool,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
    pub color: Option<String>,
    pub collapsed: bool,
    pub position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: i64,
    pub project_id: Option<i64>,
    pub workspace_id: i64,
    pub title: String,
    pub body: Option<String>,
    pub completed: bool,
    pub collapsed: bool,
    pub priority: Priority,
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    pub id: i64,
    pub todo_id: i64,
    pub title: String,
    pub planned_pomodoros: u32,
    pub completed_pomodoros: u32,
    pub position: i32,
}
