# Todo Feature — Implementation Order

## Overview

Todo виджет для PomodoRust — отдельное окно-виджет с workspace табами, проектами, задачами с markdown, и интеграцией с pomodoro очередью.

## Phases

### Phase 1: Data Layer (SQLite + Config)
**Файл плана:** `PHASE-1-DATA-LAYER.md`
**Что делаем:**
- SQLite таблицы: workspaces, projects, todo_items, pomodoro_queue
- Rust модели: Workspace, Project, TodoItem, QueuedTask
- CRUD методы в Database
- TodoConfig в Config
- Модуль `src/data/todo.rs`

**Файлы для изменения:**
- `src/data/database.rs` — добавить таблицы и CRUD
- `src/data/todo.rs` — NEW: модели данных
- `src/data/config.rs` — добавить TodoConfig
- `src/data/mod.rs` — добавить pub mod todo

**Зависимости:** нет

---

### Phase 2: Todo Viewport (Отдельное окно)
**Файл плана:** `PHASE-2-TODO-VIEWPORT.md`
**Что делаем:**
- TodoWindow struct для управления viewport
- SharedTodoState через Arc<Mutex<>>
- Кастомный titlebar для todo-окна
- Интеграция с app.rs (открытие/закрытие)
- Сохранение позиции окна
- DWM эффекты (Windows)

**Файлы для изменения:**
- `src/ui/todo_window.rs` — NEW: viewport manager
- `src/app.rs` — добавить todo_window, shared state, viewport rendering
- `src/ui/mod.rs` — добавить pub mod todo_window

**Зависимости:** Phase 1 (нужны модели данных)

---

### Phase 3: Todo UI (Workspace Tabs + Projects + Tasks)
**Файл плана:** `PHASE-3-TODO-UI.md`
**Что делаем:**
- TodoView struct с полным UI
- Workspace табы (переключение, создание, переименование)
- Список проектов с CollapsingHeader
- Задачи с чекбоксами, markdown body, сворачивание
- Inline-редактирование (двойной клик)
- Поле "Новая задача" внизу
- Контекстные меню (right-click)
- TodoAction enum

**Файлы для изменения:**
- `src/ui/todo_view.rs` — NEW: основной UI todo
- `src/ui/mod.rs` — добавить pub mod todo_view
- `Cargo.toml` — добавить egui_commonmark (или fallback)

**Зависимости:** Phase 1 + Phase 2

---

### Phase 4: Pomodoro Queue Integration
**Файл плана:** `PHASE-4-POMODORO-INTEGRATION.md`
**Что делаем:**
- Hover-кнопки на таймере (скрыть Stats/Settings, показывать на hover)
- Кнопки Queue и Todo на hover
- Отображение текущей задачи в таймере (одна строка)
- Queue popup
- Логика очереди: добавление, завершение, автопереход
- Новые иконки (ListTodo, ClipboardList)

**Файлы для изменения:**
- `src/ui/timer_view.rs` — hover кнопки, текущая задача, queue popup
- `src/app.rs` — handle_pomodoro_completed с очередью
- `src/ui/components/icons.rs` — новые иконки

**Зависимости:** Phase 1 + Phase 2 + Phase 3

---

### Phase 5: Polish & Clipboard
**Файл плана:** `PHASE-5-POLISH.md`
**Что делаем:**
- Clipboard paste (Ctrl+V) — создание задач из буфера
- Горячие клавиши (Ctrl+N, Delete, F2, Ctrl+1..9)
- Анимации hover
- Фильтр завершённых задач
- Edge cases (кириллица, длинные тексты, пустые состояния)
- Drag & drop переупорядочивание (опционально)

**Файлы для изменения:**
- `src/ui/todo_view.rs` — clipboard, hotkeys, animations
- `src/data/database.rs` — toggle_todo_collapsed

**Зависимости:** Phase 1-4

---

## File Map (все новые и изменяемые файлы)

### Новые файлы:
```
src/data/todo.rs          — модели данных (Workspace, Project, TodoItem, QueuedTask)
src/ui/todo_view.rs       — UI todo (TodoView, TodoAction, рендер)
src/ui/todo_window.rs     — viewport manager (TodoWindow, SharedTodoState)
```

### Изменяемые файлы:
```
Cargo.toml                — egui_commonmark dependency
src/data/database.rs      — таблицы + CRUD методы
src/data/config.rs        — TodoConfig
src/data/mod.rs           — pub mod todo, pub use exports
src/ui/mod.rs             — pub mod todo_view, todo_window
src/app.rs                — TodoWindow, SharedTodoState, handle_todo_action, viewport
src/ui/timer_view.rs      — hover кнопки, текущая задача, queue popup, TimerAction
src/ui/components/icons.rs — новые иконки
```

## Key Technical Decisions

1. **Кириллица:** Rust String = UTF-8, egui FontFamily::Proportional поддерживает кириллицу нативно. Все строковые операции через `.chars()`, не byte indexing.

2. **Shared state:** `Arc<Mutex<SharedTodoState>>` для обмена данными между главным окном и todo viewport, т.к. `show_viewport_deferred` не даёт `&mut self`.

3. **Markdown:** Попробовать `egui_commonmark`. Если версия несовместима с egui 0.29 — использовать ручной fallback-рендер (render_markdown_simple).

4. **Длинные задачи:** `Label::new(text).wrap()` + `ui.horizontal_top()` для корректного переноса. В таймере — `truncate_text()` с обрезкой по `.chars()`.

5. **Viewport vs View:** Todo — отдельный viewport (окно OS), не переключение view внутри таймера. Это позволяет держать оба окна рядом.

6. **Default workspace:** При пустой БД автоматически создаётся workspace "Задачи" с иконкой "📋".
