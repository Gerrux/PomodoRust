# Phase 2: Todo Viewport (Отдельное окно)

## Цель
Создать отдельное окно-виджет для Todo с кастомным titlebar, используя egui viewport API.

---

## Step 2.1: Создать Todo viewport manager

**Новый файл:** `src/ui/todo_window.rs`

Этот модуль управляет открытием/закрытием отдельного viewport для Todo.

```rust
use eframe::egui;

/// Состояние todo-окна
pub struct TodoWindow {
    pub is_open: bool,
    pub viewport_id: egui::ViewportId,
}

impl TodoWindow {
    pub fn new() -> Self {
        Self {
            is_open: false,
            viewport_id: egui::ViewportId::from_hash_of("todo_viewport"),
        }
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}
```

### Как отображать viewport

В `app.rs` в методе `update()`, после рендера основного UI, если `todo_window.is_open`:

```rust
if self.todo_window.is_open {
    let todo_config = &self.config.todo;

    let mut viewport_builder = egui::ViewportBuilder::default()
        .with_title("PomodoRust Todo")
        .with_inner_size([todo_config.window_width, todo_config.window_height])
        .with_min_inner_size([280.0, 350.0])
        .with_decorations(false)    // кастомный titlebar как у таймера
        .with_transparent(true)
        .with_resizable(true);

    if let (Some(x), Some(y)) = (todo_config.window_x, todo_config.window_y) {
        viewport_builder = viewport_builder.with_position([x, y]);
    }

    if self.config.window.always_on_top {
        viewport_builder = viewport_builder.with_always_on_top();
    }

    // Показать дочерний viewport
    ctx.show_viewport_deferred(
        self.todo_window.viewport_id,
        viewport_builder,
        |ctx, class| {
            // Рендер todo UI внутри viewport
        },
    );
}
```

### Важно: Shared State

`show_viewport_deferred` принимает `FnOnce` без доступа к `&mut self`. Для обмена данными между таймером и todo нужен **shared state** через `Arc<Mutex<TodoState>>`:

```rust
use std::sync::{Arc, Mutex};

/// Shared state между главным окном и todo viewport
pub struct SharedTodoState {
    // Данные для отображения
    pub workspaces: Vec<Workspace>,
    pub current_workspace_id: i64,
    pub projects: Vec<Project>,
    pub todos: Vec<TodoItem>,

    // Действия из todo → таймер
    pub pending_actions: Vec<TodoAction>,

    // Флаги
    pub needs_refresh: bool,
    pub should_close: bool,

    // Настройки отображения
    pub show_completed: bool,
    pub theme_mode: ThemeMode,
    pub accent_color: AccentColor,
}
```

В `PomodoRustApp` добавить:
```rust
pub struct PomodoRustApp {
    // ... существующие поля ...
    todo_window: TodoWindow,
    shared_todo: Arc<Mutex<SharedTodoState>>,
}
```

---

## Step 2.2: Кастомный titlebar для Todo окна

**В файле:** `src/ui/todo_window.rs` или переиспользовать `titlebar.rs`

Todo titlebar проще чем у таймера — нужны только:
- Заголовок "Todo" (или название текущего workspace)
- Кнопка закрытия (X)
- Drag area для перемещения

Можно переиспользовать существующий `TitleBar` из `titlebar.rs`, но с минимальным набором кнопок. Существующий `TitleBar` поддерживает кнопки: AlwaysOnTop, Minimize, Maximize, Close. Для Todo достаточно Close.

**Вариант:** Создать упрощённую версию:

```rust
fn render_todo_titlebar(ui: &mut egui::Ui, theme: &Theme, title: &str) -> (bool, bool) {
    // Возвращает (should_drag, should_close)
    // Высота: 32px, как у основного titlebar
    // Фон: theme.bg_primary с opacity
    // Текст заголовка слева
    // Кнопка X справа
}
```

Или расширить существующий TitleBar параметром для набора кнопок.

---

## Step 2.3: Рендер Todo UI внутри viewport

**Новый файл:** `src/ui/todo_view.rs`

Это основной файл UI для Todo. Структура рендера:

```rust
pub struct TodoView {
    // Состояние UI (hover, анимации)
    titlebar: TitleBar,  // или упрощённый вариант
    // Состояния ховера для workspace табов
    workspace_tab_states: Vec<InteractionState>,
    // Состояния ховера для задач (для 🍅 кнопки)
    task_hover_states: HashMap<i64, InteractionState>,
    // Текущий режим редактирования
    editing_task_id: Option<i64>,
    editing_title: String,
    editing_body: String,
    // Новая задача
    new_task_title: String,
    // Новый проект
    adding_project: bool,
    new_project_name: String,
}
```

Основной метод:
```rust
impl TodoView {
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        theme: &Theme,
        state: &mut SharedTodoState,
    ) -> Vec<TodoAction> {
        // 1. Применить тему
        // 2. Рендер titlebar
        // 3. Рендер workspace tabs
        // 4. Рендер списка проектов и задач
        // 5. Собрать actions
    }
}
```

---

## Step 2.4: Интеграция с app.rs

**Файл:** `src/app.rs`

### Добавить в PomodoRustApp:
```rust
todo_window: TodoWindow,
shared_todo: Arc<Mutex<SharedTodoState>>,
todo_view: TodoView,  // живёт в главном app, передаётся в viewport
```

### В `update()`:
```rust
// После основного UI рендера:

// 1. Обработать действия из todo (если есть)
if let Ok(mut state) = self.shared_todo.lock() {
    for action in state.pending_actions.drain(..) {
        self.handle_todo_action(action);
    }
    if state.should_close {
        self.todo_window.close();
        state.should_close = false;
    }
}

// 2. Показать todo viewport если открыт
if self.todo_window.is_open {
    self.show_todo_viewport(ctx);
}
```

### Кнопка открытия todo в таймере:
В titlebar или в hover-панели таймера добавить иконку для открытия Todo.
Использовать иконку `Icon::LayoutDashboard` или добавить новую `Icon::ListTodo`.

---

## Step 2.5: Сохранение позиции todo-окна

При закрытии или перемещении todo-окна сохранять позицию и размер в `TodoConfig`.

В viewport callback можно получить позицию:
```rust
ctx.input(|i| {
    if let Some(pos) = i.viewport().inner_rect {
        // Сохранить pos в shared state
    }
});
```

При `on_exit()` приложения (в `app.rs`) — сохранить todo config.

---

## Step 2.6: DWM-эффекты для Todo окна (Windows)

**Файл:** `src/platform/windows.rs`

Существующий `apply_window_effects()` применяет Mica/Acrylic эффекты.
Нужно применить те же эффекты к Todo окну.

После создания viewport — найти HWND по заголовку "PomodoRust Todo" и вызвать:
```rust
apply_window_effects(hwnd);
```

Это можно сделать с задержкой через `std::thread::spawn` как сделано для основного окна в `main.rs`.

---

## Критерии готовности Phase 2:
- [ ] Todo открывается как отдельное окно по кнопке в таймере
- [ ] Окно имеет кастомный titlebar (drag + close)
- [ ] Окно перемещается, ресайзится
- [ ] Позиция/размер сохраняются между запусками
- [ ] Тема синхронизирована с основным таймером
- [ ] Windows DWM эффекты применяются
- [ ] Окно закрывается без закрытия основного таймера
- [ ] `todo_autoopen: true` — окно открывается при старте приложения
