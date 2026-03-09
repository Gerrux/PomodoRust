# Phase 3: Todo UI — Workspace Tabs + Projects + Tasks

## Цель
Реализовать полный UI todo-виджета: табы workspace'ов, проекты, задачи с markdown, сворачивание/разворачивание.

---

## Step 3.1: Workspace Tabs (верхняя панель)

**Файл:** `src/ui/todo_view.rs`

### Визуал:

```
┌────────────────────────────────────┐
│ [Учёба] [Работа] [Дом]  [+]       │
└────────────────────────────────────┘
```

### Реализация:

```rust
fn render_workspace_tabs(
    &mut self,
    ui: &mut egui::Ui,
    theme: &Theme,
    workspaces: &[Workspace],
    current_id: i64,
) -> Option<TodoAction>
```

**Детали:**
- Горизонтальный `ui.horizontal()` со `ScrollArea::horizontal()` (если много табов)
- Каждый таб — `ui.selectable_label()` или кастомная кнопка
- Активный таб: `theme.accent_solid()` цвет подчёркивания (2px линия снизу)
- Неактивный: `theme.text_secondary`
- Hover: `theme.bg_hover`
- Кнопка "+" справа: `Icon::Plus`, при нажатии — inline input для имени нового workspace
- Right-click на таб → контекстное меню: Переименовать, Удалить, Изменить иконку
- Текст табов: `theme.font_small()` (14px) — чтобы помещалось много

**Кириллица в табах:**
- egui рендерит UTF-8 нативно, кириллица работает "из коробки"
- `FontFamily::Proportional` включает шрифты с поддержкой кириллицы
- Для длинных названий: `ui.label()` с `text.truncate()` или `egui::Label::new().truncate()`

**Inline редактирование названия workspace:**
```rust
if self.renaming_workspace_id == Some(workspace.id) {
    let response = ui.text_edit_singleline(&mut self.rename_buffer);
    if response.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
        // Сохранить переименование
        actions.push(TodoAction::RenameWorkspace {
            id: workspace.id,
            name: self.rename_buffer.clone(),
        });
        self.renaming_workspace_id = None;
    }
}
```

---

## Step 3.2: Список проектов и задач

**Файл:** `src/ui/todo_view.rs`

### Визуал:

```
┌────────────────────────────────────┐
│ ▸ Курсовая                    [+] │  ← проект (свёрнут)
│ ▾ Экзамены                    [+] │  ← проект (развёрнут)
│   ☐ Матан — глава 5          🍅  │  ← задача (hover = 🍅)
│   ▾ ☐ Физика: подготовка     🍅  │  ← задача с body (развёрнута)
│     │ **Темы для повторения:**    │
│     │ - Механика                  │
│     │ - Термодинамика             │
│     │ - Волновая оптика           │
│   ☑ Английский — эссе            │  ← завершённая задача
│                                    │
│ ── Разное ──                       │  ← задачи без проекта
│   ☐ Сдать книгу в библиотеку     │
│   ☐ Очень длинное название задачи │
│     которое переносится на        │
│     несколько строк               │
└────────────────────────────────────┘
```

### Структура рендера:

```rust
fn render_task_list(
    &mut self,
    ui: &mut egui::Ui,
    theme: &Theme,
    state: &SharedTodoState,
) -> Vec<TodoAction> {
    let mut actions = vec![];

    // ScrollArea для прокрутки
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // 1. Проекты с задачами
            for project in &state.projects {
                actions.extend(self.render_project(ui, theme, project, &state.todos));
            }

            // 2. Разделитель "Разное" (если есть задачи без проекта)
            let unassigned: Vec<_> = state.todos.iter()
                .filter(|t| t.project_id.is_none())
                .collect();
            if !unassigned.is_empty() {
                self.render_separator(ui, theme, "Разное");
                for todo in &unassigned {
                    actions.extend(self.render_todo_item(ui, theme, todo));
                }
            }

            // 3. Поле добавления новой задачи внизу
            actions.extend(self.render_new_task_input(ui, theme, state));
        });

    actions
}
```

---

## Step 3.3: Рендер проекта (CollapsingHeader)

```rust
fn render_project(
    &mut self,
    ui: &mut egui::Ui,
    theme: &Theme,
    project: &Project,
    todos: &[TodoItem],
) -> Vec<TodoAction> {
    let project_todos: Vec<_> = todos.iter()
        .filter(|t| t.project_id == Some(project.id))
        .collect();

    let header_id = ui.make_persistent_id(format!("project_{}", project.id));

    // Используем egui::CollapsingState для контроля
    let mut collapsing = egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(), header_id, !project.collapsed
    );

    let header_response = ui.horizontal(|ui| {
        // Стрелка ▸/▾
        let arrow = if collapsing.is_open() { "▾" } else { "▸" };
        if ui.add(egui::Label::new(
            egui::RichText::new(arrow).color(theme.text_secondary).size(14.0)
        ).sense(egui::Sense::click())).clicked() {
            collapsing.toggle(ui);
        }

        // Цветная точка проекта (если есть color)
        if let Some(color) = &project.color {
            let (rect, _) = ui.allocate_exact_size(
                egui::vec2(8.0, 8.0), egui::Sense::hover()
            );
            ui.painter().circle_filled(rect.center(), 4.0, parse_hex_color(color));
        }

        // Название проекта
        let name_text = egui::RichText::new(&project.name)
            .size(15.0)
            .color(theme.text_primary)
            .strong();
        ui.label(name_text);

        // Счётчик задач
        let active_count = project_todos.iter().filter(|t| !t.completed).count();
        if active_count > 0 {
            ui.label(
                egui::RichText::new(format!("{}", active_count))
                    .size(12.0)
                    .color(theme.text_muted)
            );
        }

        // Кнопка "+" для добавления задачи в проект (появляется на hover)
        // ... (render on hover)
    });

    // Right-click контекстное меню
    header_response.response.context_menu(|ui| {
        if ui.button("Переименовать").clicked() { /* ... */ }
        if ui.button("Удалить проект").clicked() { /* ... */ }
        ui.separator();
        if ui.button("Добавить задачу").clicked() { /* ... */ }
    });

    // Тело проекта (задачи)
    collapsing.show_body_unindented(ui, |ui| {
        ui.indent(format!("project_indent_{}", project.id), |ui| {
            for todo in &project_todos {
                self.render_todo_item(ui, theme, todo);
            }
        });
    });
}
```

---

## Step 3.4: Рендер задачи (TodoItem)

Самая важная часть. Задача может быть:
- Короткой: "Купить молоко"
- Длинной: "Подготовить презентацию по архитектуре микросервисов для ретро"
- С markdown body: развёрнутое описание с чеклистами

```rust
fn render_todo_item(
    &mut self,
    ui: &mut egui::Ui,
    theme: &Theme,
    todo: &TodoItem,
) -> Vec<TodoAction> {
    let mut actions = vec![];
    let item_id = ui.make_persistent_id(format!("todo_{}", todo.id));

    // Полная строка задачи — кликабельная область
    let frame = egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(4.0, 4.0))
        .rounding(theme.rounding_sm)
        .fill(if is_hovered { theme.bg_hover } else { egui::Color32::TRANSPARENT });

    frame.show(ui, |ui| {
        ui.horizontal_top(|ui| {
            // 1. Стрелка сворачивания (только если есть body)
            if todo.body.is_some() {
                let arrow = if todo.collapsed { "▸" } else { "▾" };
                if ui.add(egui::Label::new(
                    egui::RichText::new(arrow).size(12.0).color(theme.text_muted)
                ).sense(egui::Sense::click())).clicked() {
                    actions.push(TodoAction::ToggleCollapse { id: todo.id });
                }
            } else {
                // Пустое место вместо стрелки для выравнивания
                ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
            }

            // 2. Чекбокс
            let checkbox_icon = if todo.completed { "☑" } else { "☐" };
            let checkbox_color = if todo.completed {
                theme.success
            } else {
                theme.text_secondary
            };
            if ui.add(egui::Label::new(
                egui::RichText::new(checkbox_icon).size(16.0).color(checkbox_color)
            ).sense(egui::Sense::click())).clicked() {
                actions.push(TodoAction::ToggleComplete { id: todo.id });
            }

            // 3. Заголовок задачи
            // ВАЖНО: для длинных заголовков — wrap_mode(TextWrapMode::Wrap)
            let title_color = if todo.completed {
                theme.text_muted
            } else {
                theme.text_primary
            };
            let mut title_text = egui::RichText::new(&todo.title)
                .size(14.0)
                .color(title_color);
            if todo.completed {
                title_text = title_text.strikethrough();
            }

            // Wrap длинного текста, а не обрезать
            let title_label = egui::Label::new(title_text).wrap();
            let title_response = ui.add(title_label);

            // 4. 🍅 кнопка (на hover всей строки задачи)
            // Появляется справа при наведении на строку
            if is_row_hovered && !todo.completed {
                let tomato = egui::RichText::new("🍅").size(14.0);
                if ui.add(egui::Label::new(tomato).sense(egui::Sense::click()))
                    .on_hover_text("Добавить в очередь помодоро")
                    .clicked()
                {
                    actions.push(TodoAction::AddToQueue {
                        todo_id: todo.id,
                        planned_pomodoros: 1,
                    });
                }
            }
        });

        // 5. Markdown body (если развёрнут)
        if !todo.collapsed {
            if let Some(body) = &todo.body {
                ui.indent(format!("todo_body_{}", todo.id), |ui| {
                    // egui_commonmark для рендера markdown
                    render_markdown(ui, theme, body);
                });
            }
        }
    });

    // Right-click контекстное меню
    // context_menu: Редактировать, Переместить в проект..., Удалить

    actions
}
```

### Обработка длинных заголовков:

- `egui::Label::new(text).wrap()` — переносит текст на следующую строку
- НЕ использовать `.truncate()` — пользователь должен видеть полный текст
- `ui.horizontal_top()` вместо `ui.horizontal()` — чтобы чекбокс оставался вверху при переносе текста
- Максимальная ширина текста = `ui.available_width() - 50.0` (место для кнопок)

### Обработка подпунктов в markdown body:

Markdown чеклисты (`- [ ] подпункт`, `- [x] подпункт`) рендерятся через `egui_commonmark`. Это стандартный GitHub-flavored markdown.

Пример body:
```markdown
**Темы для повторения:**
- [ ] Механика
- [x] Термодинамика
- [ ] Волновая оптика

> Экзамен 15 марта
```

---

## Step 3.5: Markdown рендеринг

**Добавить в `Cargo.toml`:**
```toml
egui_commonmark = "0.18"  # версию подобрать под egui 0.29
```

**Рендер функция:**

```rust
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};

fn render_markdown(ui: &mut egui::Ui, theme: &Theme, text: &str, cache: &mut CommonMarkCache) {
    // CommonMarkViewer рендерит markdown в egui
    CommonMarkViewer::new()
        .max_image_width(Some(200))
        .show(ui, cache, text);
}
```

**CommonMarkCache** хранить в `TodoView` — он кэширует парсинг markdown для производительности.

### Кириллица в markdown:
- `egui_commonmark` использует те же шрифты что и egui
- Кириллица рендерится нативно, без дополнительных настроек
- Для **жирного** и *курсивного* текста egui автоматически применяет соответствующие начертания

---

## Step 3.6: Поле ввода новой задачи

Внизу списка — постоянно видимое поле ввода:

```
┌────────────────────────────────────┐
│  + Новая задача...                 │
└────────────────────────────────────┘
```

```rust
fn render_new_task_input(
    &mut self,
    ui: &mut egui::Ui,
    theme: &Theme,
    current_workspace_id: i64,
    current_project_id: Option<i64>,  // если фокус внутри проекта
) -> Vec<TodoAction> {
    let mut actions = vec![];

    ui.separator();

    let response = ui.horizontal(|ui| {
        // Иконка "+"
        ui.label(egui::RichText::new("+").size(16.0).color(theme.text_muted));

        // Текстовое поле
        let text_edit = egui::TextEdit::singleline(&mut self.new_task_title)
            .hint_text("Новая задача...")         // placeholder (кириллица!)
            .desired_width(ui.available_width())
            .font(egui::FontId::proportional(14.0))
            .frame(false);                         // без рамки, минималистично

        let response = ui.add(text_edit);

        // Enter → создать задачу
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let title = self.new_task_title.trim().to_string();
            if !title.is_empty() {
                actions.push(TodoAction::CreateTodo {
                    workspace_id: current_workspace_id,
                    project_id: current_project_id,
                    title,
                });
                self.new_task_title.clear();
                // Вернуть фокус на поле ввода
                response.request_focus();
            }
        }
    });

    actions
}
```

---

## Step 3.7: Редактирование задачи

При двойном клике на заголовок задачи — включается inline-редактирование:

```rust
if title_response.double_clicked() {
    self.editing_task_id = Some(todo.id);
    self.editing_title = todo.title.clone();
    self.editing_body = todo.body.clone().unwrap_or_default();
}

// Режим редактирования
if self.editing_task_id == Some(todo.id) {
    // Заголовок как TextEdit singleline
    let title_edit = egui::TextEdit::singleline(&mut self.editing_title)
        .font(egui::FontId::proportional(14.0))
        .desired_width(ui.available_width() - 30.0);
    ui.add(title_edit);

    // Body как TextEdit multiline
    let body_edit = egui::TextEdit::multiline(&mut self.editing_body)
        .font(egui::FontId::proportional(13.0))
        .desired_width(ui.available_width() - 20.0)
        .hint_text("Описание (markdown)...")
        .desired_rows(4);
    ui.add(body_edit);

    // Кнопки: Сохранить / Отмена
    ui.horizontal(|ui| {
        if ui.small_button("Сохранить").clicked()
            || ui.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::Enter))
        {
            actions.push(TodoAction::UpdateTodo {
                id: todo.id,
                title: self.editing_title.clone(),
                body: if self.editing_body.is_empty() {
                    None
                } else {
                    Some(self.editing_body.clone())
                },
            });
            self.editing_task_id = None;
        }
        if ui.small_button("Отмена").clicked()
            || ui.input(|i| i.key_pressed(egui::Key::Escape))
        {
            self.editing_task_id = None;
        }
    });
}
```

---

## Step 3.8: Контекстные меню

Right-click на задачу:
```rust
response.context_menu(|ui| {
    if ui.button("Редактировать").clicked() {
        self.editing_task_id = Some(todo.id);
        // ...
        ui.close_menu();
    }
    if ui.button("Переместить в проект...").clicked() {
        // Подменю с проектами
        ui.close_menu();
    }
    ui.separator();
    if !todo.completed {
        if ui.button("🍅 Добавить в очередь").clicked() {
            actions.push(TodoAction::AddToQueue { todo_id: todo.id, planned_pomodoros: 1 });
            ui.close_menu();
        }
    }
    ui.separator();
    if ui.button("Удалить").clicked() {
        actions.push(TodoAction::DeleteTodo { id: todo.id });
        ui.close_menu();
    }
});
```

---

## Step 3.9: TodoAction enum

**Файл:** `src/ui/todo_view.rs` или `src/data/todo.rs`

```rust
pub enum TodoAction {
    // Workspaces
    CreateWorkspace { name: String },
    RenameWorkspace { id: i64, name: String },
    DeleteWorkspace { id: i64 },
    SwitchWorkspace { id: i64 },

    // Projects
    CreateProject { workspace_id: i64, name: String },
    RenameProject { id: i64, name: String },
    DeleteProject { id: i64 },
    ToggleProjectCollapse { id: i64 },

    // Todos
    CreateTodo { workspace_id: i64, project_id: Option<i64>, title: String },
    UpdateTodo { id: i64, title: String, body: Option<String> },
    ToggleComplete { id: i64 },
    ToggleCollapse { id: i64 },
    DeleteTodo { id: i64 },
    MoveTodo { id: i64, project_id: Option<i64> },

    // Queue
    AddToQueue { todo_id: i64, planned_pomodoros: u32 },

    // Window
    Close,
}
```

---

## Особенности UX

### Кириллица:
- Все `TextEdit`, `Label`, `RichText` работают с UTF-8 нативно
- `hint_text("Новая задача...")` — кириллица в placeholder
- Контекстные меню с кириллическими надписями
- Поиск/фильтрация по кириллице — стандартное сравнение строк

### Длинные задачи:
- `Label::new(text).wrap()` — автоперенос заголовков
- `ui.horizontal_top()` — элементы выровнены по верху при переносе
- Body может быть любой длины — ScrollArea внутри задачи не нужен, общий scroll у всего списка

### Подпункты:
- Поддерживаются через markdown чеклисты в body: `- [ ] пункт`, `- [x] пункт`
- `egui_commonmark` рендерит их как интерактивные чекбоксы
- Альтернатива: иерархия задач через parent_id (усложняет, пока не нужно)

### Современный UX:
- Минимальные рамки, чистые поверхности
- Hover-эффекты на задачах (subtle bg highlight)
- Анимированные переходы для сворачивания
- Контекстные меню вместо множества кнопок
- Inline-редактирование (без модальных окон)

---

## Критерии готовности Phase 3:
- [ ] Workspace табы отображаются, переключаются, создаются
- [ ] Проекты отображаются с CollapsingHeader
- [ ] Задачи рендерятся с чекбоксами и markdown body
- [ ] Длинные заголовки корректно переносятся
- [ ] Markdown body рендерится через egui_commonmark
- [ ] Кириллица отображается корректно везде (табы, задачи, placeholder)
- [ ] Inline-редактирование задач работает (двойной клик)
- [ ] Новая задача создаётся через поле ввода внизу
- [ ] Контекстные меню работают (right-click)
- [ ] 🍅 кнопка появляется на hover задачи
- [ ] Сворачивание/разворачивание body запоминается
- [ ] ScrollArea корректно прокручивает длинный список
