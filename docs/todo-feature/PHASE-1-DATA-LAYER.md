# Phase 1: Data Layer (SQLite + Config)

## Цель
Создать модели данных, SQLite таблицы и конфигурацию для Todo-системы.

---

## Step 1.1: Расширить SQLite схему

**Файл:** `src/data/database.rs`

Добавить 4 новые таблицы в метод `open()` рядом с существующими CREATE TABLE:

```sql
CREATE TABLE IF NOT EXISTS workspaces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    icon TEXT,              -- emoji или код иконки
    color TEXT,             -- hex цвет, например "#7C3AED"
    collapsed INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

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

CREATE TABLE IF NOT EXISTS todo_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER,         -- NULL = задача "Без проекта"
    workspace_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    body TEXT,                   -- markdown содержимое
    completed INTEGER NOT NULL DEFAULT 0,
    collapsed INTEGER NOT NULL DEFAULT 1,  -- body свёрнуто по умолчанию
    position INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS pomodoro_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    todo_id INTEGER NOT NULL,
    planned_pomodoros INTEGER NOT NULL DEFAULT 1,
    completed_pomodoros INTEGER NOT NULL DEFAULT 0,
    position INTEGER NOT NULL DEFAULT 0,
    added_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (todo_id) REFERENCES todo_items(id) ON DELETE CASCADE
);
```

Также создать индексы:
```sql
CREATE INDEX IF NOT EXISTS idx_todo_workspace ON todo_items(workspace_id);
CREATE INDEX IF NOT EXISTS idx_todo_project ON todo_items(project_id);
CREATE INDEX IF NOT EXISTS idx_projects_workspace ON projects(workspace_id);
CREATE INDEX IF NOT EXISTS idx_queue_position ON pomodoro_queue(position);
```

---

## Step 1.2: Создать модели данных

**Новый файл:** `src/data/todo.rs`

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Workspace = "сфера жизни" (Работа, Учёба, Дом...)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: i64,
    pub name: String,           // Кириллица поддерживается — String это UTF-8
    pub icon: Option<String>,   // emoji: "📚", "💼", "🏠"
    pub color: Option<String>,  // hex: "#7C3AED"
    pub collapsed: bool,
    pub position: i32,
}

/// Проект внутри Workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub workspace_id: i64,
    pub name: String,
    pub color: Option<String>,
    pub collapsed: bool,
    pub position: i32,
}

/// Задача Todo — может содержать длинный markdown body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: i64,
    pub project_id: Option<i64>,  // None = "Без проекта"
    pub workspace_id: i64,
    pub title: String,
    pub body: Option<String>,     // markdown, может быть очень длинным
    pub completed: bool,
    pub collapsed: bool,          // свёрнут ли body
    pub position: i32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Задача в очереди помодоро
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    pub id: i64,
    pub todo_id: i64,
    pub title: String,            // кэшированный заголовок для отображения
    pub planned_pomodoros: u32,
    pub completed_pomodoros: u32,
    pub position: i32,
}
```

---

## Step 1.3: CRUD-методы для Database

**Файл:** `src/data/database.rs`

Добавить блок `impl Database` с методами:

### Workspaces:
```rust
pub fn create_workspace(&self, name: &str, icon: Option<&str>, color: Option<&str>) -> Result<i64>
pub fn get_workspaces(&self) -> Result<Vec<Workspace>>
pub fn update_workspace(&self, workspace: &Workspace) -> Result<()>
pub fn delete_workspace(&self, id: i64) -> Result<()>
pub fn reorder_workspaces(&self, ids: &[i64]) -> Result<()>
```

### Projects:
```rust
pub fn create_project(&self, workspace_id: i64, name: &str, color: Option<&str>) -> Result<i64>
pub fn get_projects(&self, workspace_id: i64) -> Result<Vec<Project>>
pub fn update_project(&self, project: &Project) -> Result<()>
pub fn delete_project(&self, id: i64) -> Result<()>
pub fn reorder_projects(&self, ids: &[i64]) -> Result<()>
```

### TodoItems:
```rust
pub fn create_todo(&self, workspace_id: i64, project_id: Option<i64>, title: &str) -> Result<i64>
pub fn get_todos(&self, workspace_id: i64) -> Result<Vec<TodoItem>>
pub fn get_todos_by_project(&self, project_id: i64) -> Result<Vec<TodoItem>>
pub fn get_unassigned_todos(&self, workspace_id: i64) -> Result<Vec<TodoItem>>
pub fn update_todo(&self, todo: &TodoItem) -> Result<()>
pub fn toggle_todo(&self, id: i64) -> Result<bool>  // возвращает новое состояние
pub fn delete_todo(&self, id: i64) -> Result<()>
pub fn reorder_todos(&self, ids: &[i64]) -> Result<()>
```

### Pomodoro Queue:
```rust
pub fn add_to_queue(&self, todo_id: i64, planned_pomodoros: u32) -> Result<i64>
pub fn get_queue(&self) -> Result<Vec<QueuedTask>>
pub fn remove_from_queue(&self, id: i64) -> Result<()>
pub fn clear_queue(&self) -> Result<()>
pub fn increment_queue_pomodoro(&self, id: i64) -> Result<bool>  // true если все помодоро выполнены
pub fn get_current_queue_task(&self) -> Result<Option<QueuedTask>>  // первый по position
pub fn advance_queue(&self) -> Result<Option<QueuedTask>>  // удаляет текущий, возвращает следующий
pub fn reorder_queue(&self, ids: &[i64]) -> Result<()>
```

### Важные детали реализации:

1. **Кириллица:** Rust String — UTF-8, SQLite TEXT — UTF-8. Никаких особых действий не требуется. Просто использовать `&str` параметры как обычно.

2. **Длинные тексты:** SQLite TEXT не имеет ограничения длины. Body может быть любой длины.

3. **Позиционирование:** При создании новой записи `position` = `SELECT COALESCE(MAX(position), -1) + 1`. При `reorder` — обновлять position по порядку в массиве ids.

4. **Каскадное удаление:** Включить `PRAGMA foreign_keys = ON` при открытии БД (добавить в `open()`).

5. **Default workspace:** При первом запуске (пустая таблица workspaces) создать workspace "Задачи" с иконкой "📋".

---

## Step 1.4: Расширить Config

**Файл:** `src/data/config.rs`

Добавить секцию:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoConfig {
    /// Автоматически открывать todo-окно при запуске
    pub auto_open: bool,
    /// Ширина окна todo
    pub window_width: f32,
    /// Высота окна todo
    pub window_height: f32,
    /// Позиция X
    pub window_x: Option<f32>,
    /// Позиция Y
    pub window_y: Option<f32>,
    /// Показывать завершённые задачи
    pub show_completed: bool,
    /// ID последнего активного workspace
    pub last_workspace_id: Option<i64>,
}

impl Default for TodoConfig {
    fn default() -> Self {
        Self {
            auto_open: false,
            window_width: 340.0,
            window_height: 500.0,
            window_x: None,
            window_y: None,
            show_completed: true,
            last_workspace_id: None,
        }
    }
}
```

Добавить поле в `Config`:
```rust
pub struct Config {
    // ... существующие поля ...
    pub todo: TodoConfig,
}
```

---

## Step 1.5: Обновить модуль data

**Файл:** `src/data/mod.rs`

Добавить:
```rust
pub mod todo;
pub use todo::{Workspace, Project, TodoItem, QueuedTask};
```

---

## Критерии готовности Phase 1:
- [ ] Таблицы создаются при первом запуске без ошибок
- [ ] CRUD операции для Workspace, Project, TodoItem, QueuedTask работают
- [ ] Кириллические названия корректно сохраняются и читаются
- [ ] Foreign keys с каскадным удалением работают
- [ ] Config с TodoConfig сериализуется/десериализуется в TOML
- [ ] Default workspace создаётся при пустой БД
- [ ] Проект компилируется без warnings
