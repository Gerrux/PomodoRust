# Phase 5: Polish & Clipboard Integration

## Цель
Финальная полировка UX, clipboard-импорт, сохранение состояний, edge cases.

---

## Step 5.1: Clipboard-импорт задач

**Файл:** `src/ui/todo_view.rs`

Кнопка "Вставить из буфера" или горячая клавиша Ctrl+V в todo-окне.

### Логика:
```rust
fn handle_clipboard_paste(
    &mut self,
    ctx: &egui::Context,
    workspace_id: i64,
    project_id: Option<i64>,
) -> Vec<TodoAction> {
    let mut actions = vec![];

    // Проверить Ctrl+V (только когда не в режиме редактирования)
    if self.editing_task_id.is_none()
        && ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V))
    {
        if let Some(clipboard_text) = get_clipboard_text(ctx) {
            let text = clipboard_text.trim().to_string();
            if !text.is_empty() {
                // Парсинг: если несколько строк — каждая строка = отдельная задача
                // Если одна строка — одна задача
                let lines: Vec<&str> = text.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();

                if lines.len() == 1 {
                    actions.push(TodoAction::CreateTodo {
                        workspace_id,
                        project_id,
                        title: lines[0].to_string(),
                    });
                } else {
                    // Первая строка — заголовок, остальные — body
                    // ИЛИ каждая строка — отдельная задача
                    // Эвристика: если строки начинаются с "- " или "* " — это чеклист в body
                    let looks_like_list = lines.iter().skip(1)
                        .all(|l| l.starts_with("- ") || l.starts_with("* ") || l.starts_with("- [ ]"));

                    if looks_like_list {
                        // Одна задача с body
                        actions.push(TodoAction::CreateTodoWithBody {
                            workspace_id,
                            project_id,
                            title: lines[0].to_string(),
                            body: lines[1..].join("\n"),
                        });
                    } else {
                        // Каждая строка — отдельная задача
                        for line in &lines {
                            // Убрать markdown-маркеры если есть
                            let title = line
                                .trim_start_matches("- [ ] ")
                                .trim_start_matches("- [x] ")
                                .trim_start_matches("- ")
                                .trim_start_matches("* ")
                                .to_string();
                            actions.push(TodoAction::CreateTodo {
                                workspace_id,
                                project_id,
                                title,
                            });
                        }
                    }
                }
            }
        }
    }

    actions
}

fn get_clipboard_text(ctx: &egui::Context) -> Option<String> {
    ctx.input(|i| i.events.iter().find_map(|e| {
        if let egui::Event::Paste(text) = e {
            Some(text.clone())
        } else {
            None
        }
    }))
}
```

---

## Step 5.2: Сохранение состояний сворачивания

Состояния collapsed для проектов и задач должны сохраняться в SQLite.

При каждом `ToggleCollapse` или `ToggleProjectCollapse`:
```rust
TodoAction::ToggleCollapse { id } => {
    if let Some(db) = &self.database {
        // Прочитать текущее состояние и инвертировать
        let _ = db.toggle_todo_collapsed(id);
    }
}
```

**Метод в database.rs:**
```rust
pub fn toggle_todo_collapsed(&self, id: i64) -> Result<bool> {
    self.conn.execute(
        "UPDATE todo_items SET collapsed = NOT collapsed WHERE id = ?1",
        [id],
    )?;
    let collapsed: bool = self.conn.query_row(
        "SELECT collapsed FROM todo_items WHERE id = ?1",
        [id],
        |row| row.get(0),
    )?;
    Ok(collapsed)
}
```

---

## Step 5.3: Фильтр завершённых задач

В todo-окне — toggle для показа/скрытия завершённых:

```rust
// Внизу списка или в header рядом с табами:
ui.horizontal(|ui| {
    let label = if state.show_completed {
        "Скрыть завершённые"
    } else {
        "Показать завершённые"
    };
    if ui.small_button(label).clicked() {
        state.show_completed = !state.show_completed;
        // Сохранить в config
    }
});
```

Завершённые задачи рендерятся внизу списка, с приглушённым цветом и зачёркнутым текстом.

---

## Step 5.4: Горячие клавиши в todo-окне

```rust
// В todo viewport update():
if ctx.input(|i| i.key_pressed(egui::Key::N) && i.modifiers.ctrl) {
    // Фокус на поле "Новая задача"
    self.focus_new_task = true;
}

if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
    // Удалить выделенную задачу (если есть)
}

if ctx.input(|i| i.key_pressed(egui::Key::F2)) {
    // Переименовать выделенную задачу
}

// Ctrl+1..9 — переключение workspace табов
for i in 1..=9 {
    let key = match i {
        1 => egui::Key::Num1,
        2 => egui::Key::Num2,
        // ...
    };
    if ctx.input(|i| i.key_pressed(key) && i.modifiers.ctrl) {
        // Переключить на workspace #i
    }
}
```

---

## Step 5.5: Анимации

### Появление/исчезновение todo-окна:
- Нет — viewport не поддерживает анимацию появления

### Сворачивание/разворачивание body:
- egui `CollapsingState` имеет встроенную анимацию
- Для кастомного: `egui::lerp` по высоте контента

### Hover-эффекты на задачах:
```rust
// InteractionState для каждой задачи
let hover_bg = egui::Color32::from_rgba_unmultiplied(
    theme.bg_hover.r(),
    theme.bg_hover.g(),
    theme.bg_hover.b(),
    (hover_progress * 255.0) as u8,  // fade in/out
);
```

### Чекбокс анимация:
```rust
// При toggle — кратковременная анимация (scale + color)
// Хранить timestamp завершения для анимации "галочки"
```

---

## Step 5.6: Drag & Drop для переупорядочивания

**Опционально, если время позволяет.**

egui поддерживает drag & drop через `ui.dnd_drag_source()` и `ui.dnd_drop_zone()`.

```rust
// Каждая задача:
let item_id = egui::Id::new(("todo_drag", todo.id));

let response = ui.dnd_drag_source(item_id, |ui| {
    // Рендер задачи
    self.render_todo_item_inner(ui, theme, todo);
});

// Drop zone между задачами:
let drop_response = ui.dnd_drop_zone::<i64, _>(egui::Frame::none(), |ui| {
    // Тонкая линия-индикатор
});

if let Some(dragged_id) = drop_response.payload {
    // Переместить задачу
    actions.push(TodoAction::ReorderTodo {
        id: *dragged_id,
        new_position: position,
    });
}
```

Это сложная фича, можно отложить. Простая альтернатива — контекстное меню "Переместить вверх/вниз".

---

## Step 5.7: Edge Cases

### Кириллица:
- [x] Все строковые операции через `.chars()` а не `.len()` / byte indexing
- [x] `truncate_text()` режет по символам
- [x] Поиск/фильтрация: `.to_lowercase()` работает с кириллицей
- [x] SQLite COLLATE — для сортировки по алфавиту можно использовать COLLATE NOCASE (работает с ASCII), для кириллицы сортировка будет по Unicode codepoints (приемлемо)

### Длинные задачи:
- [x] Заголовки: `Label::new().wrap()` — автоперенос
- [x] Body: ScrollArea не нужен для отдельной задачи, общий scroll списка
- [x] В таймере: `truncate_text()` с "…"
- [x] В queue popup: `truncate_text()` с "…"

### Пустые состояния:
- Пустой workspace → "Нет задач. Создайте первую!" + большая кнопка "+"
- Пустая очередь → "Очередь пуста" в popup
- Нет workspaces → невозможно (default создаётся автоматически)

### Удаление с каскадом:
- Удалить workspace → все проекты и задачи удаляются (CASCADE)
- Удалить проект → задачи становятся "без проекта" (SET NULL)
- Удалить задачу в очереди → удаляется из очереди (CASCADE)
- Подтверждение: для удаления workspace/проекта — popup "Вы уверены?"

### Одновременное редактирование:
- Shared state через `Arc<Mutex<>>` — потокобезопасно
- UI обновляется каждый кадр — изменения видны мгновенно
- `needs_refresh` флаг — перечитать данные из БД при изменениях

---

## Step 5.8: Обновить модули

**Файл:** `src/ui/mod.rs`
```rust
pub mod todo_view;
pub mod todo_window;
```

**Файл:** `src/lib.rs`
Экспорты уже покрыты через `pub mod ui`.

---

## Step 5.9: Обновить Cargo.toml

```toml
[dependencies]
# Добавить:
egui_commonmark = "0.18"   # markdown рендер (проверить совместимость с egui 0.29)
```

Проверить совместимость версий:
- egui 0.29 → egui_commonmark версия должна поддерживать egui 0.29
- Если нет совместимой версии — использовать `egui_commonmark` из git или рендерить markdown вручную (bold/italic/lists через RichText)

**Fallback без egui_commonmark:**
```rust
fn render_markdown_simple(ui: &mut egui::Ui, theme: &Theme, text: &str) {
    for line in text.lines() {
        if line.starts_with("# ") {
            ui.label(egui::RichText::new(&line[2..]).size(18.0).strong());
        } else if line.starts_with("## ") {
            ui.label(egui::RichText::new(&line[3..]).size(16.0).strong());
        } else if line.starts_with("- [x] ") {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("☑").color(theme.success));
                ui.label(egui::RichText::new(&line[6..]).strikethrough().color(theme.text_muted));
            });
        } else if line.starts_with("- [ ] ") {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("☐").color(theme.text_secondary));
                ui.label(&line[6..]);
            });
        } else if line.starts_with("- ") || line.starts_with("* ") {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("•").color(theme.text_muted));
                ui.label(&line[2..]);
            });
        } else if line.starts_with("> ") {
            // Blockquote
            let frame = egui::Frame::none()
                .inner_margin(egui::Margin { left: 8.0, ..Default::default() })
                .stroke(egui::Stroke::new(2.0, theme.border_default));
            frame.show(ui, |ui| {
                ui.label(
                    egui::RichText::new(&line[2..]).color(theme.text_secondary).italics()
                );
            });
        } else if line.starts_with("**") && line.ends_with("**") {
            ui.label(egui::RichText::new(&line[2..line.len()-2]).strong());
        } else if line.is_empty() {
            ui.add_space(4.0);
        } else {
            ui.label(line);
        }
    }
}
```

---

## Критерии готовности Phase 5:
- [ ] Ctrl+V вставляет задачи из буфера обмена
- [ ] Многострочный paste парсится интеллигентно (список vs body)
- [ ] Состояния сворачивания сохраняются в SQLite
- [ ] Фильтр завершённых задач работает
- [ ] Горячие клавиши: Ctrl+N, Delete, F2, Ctrl+1..9
- [ ] Hover-анимации на задачах
- [ ] Пустые состояния отображаются корректно
- [ ] Удаление workspace/проекта с подтверждением
- [ ] Каскадное удаление работает корректно
- [ ] Markdown рендерится (egui_commonmark или fallback)
- [ ] Все edge cases с кириллицей обработаны
- [ ] Проект компилируется и работает без ошибок
