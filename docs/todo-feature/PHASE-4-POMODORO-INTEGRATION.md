# Phase 4: Pomodoro Queue Integration

## Цель
Связать todo-задачи с помодоро-таймером через очередь задач. Реализовать hover-кнопки на таймере.

---

## Step 4.1: Hover-кнопки на таймере

**Файл:** `src/ui/timer_view.rs`

Сейчас кнопки Stats и Settings всегда видны внизу таймера. Нужно:
1. Скрыть их по умолчанию
2. Показывать при наведении на окно таймера
3. Добавить кнопки: Queue (🗂), Open Todo (📋)

### Определение hover на окне:

```rust
// В update() перед рендером timer_view:
let window_hovered = ctx.input(|i| {
    i.pointer.hover_pos().map_or(false, |pos| {
        // Проверяем что курсор внутри окна
        ctx.screen_rect().contains(pos)
    })
});
```

Или проще — отслеживать hover на CentralPanel:

```rust
let panel_response = egui::CentralPanel::default()
    .frame(/* ... */)
    .show(ctx, |ui| {
        // ... render timer ...
    });
let window_hovered = panel_response.response.hovered();
```

### Передать в TimerView:

Расширить `TimerView::show()`:
```rust
pub fn show(
    &mut self,
    ui: &mut egui::Ui,
    session: &Session,
    theme: &Theme,
    config: &Config,
    animations: &AnimationState,
    window_hovered: bool,           // NEW
    current_task: Option<&QueuedTask>,  // NEW
) -> Option<TimerAction>
```

### Рендер hover-кнопок:

```rust
// Внизу timer_view, вместо постоянных кнопок Stats/Settings:

if window_hovered {
    // Анимация появления (fade in)
    let opacity = self.hover_buttons_opacity.animate_to(1.0, theme.anim_fast);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = theme.spacing_sm;

        // Settings
        let settings_btn = IconButton::new(Icon::Settings)
            .size(28.0)
            .opacity(opacity);
        if settings_btn.show(ui, theme).clicked() {
            return Some(TimerAction::OpenSettings);
        }

        // Stats
        let stats_btn = IconButton::new(Icon::BarChart3)
            .size(28.0)
            .opacity(opacity);
        if stats_btn.show(ui, theme).clicked() {
            return Some(TimerAction::OpenStats);
        }

        // Queue (🗂) — показывает popup с очередью
        let queue_btn = IconButton::new(Icon::ListTodo) // новая иконка
            .size(28.0)
            .opacity(opacity);
        let queue_response = queue_btn.show(ui, theme);
        if queue_response.clicked() {
            self.show_queue_popup = !self.show_queue_popup;
        }

        // Open Todo (📋)
        let todo_btn = IconButton::new(Icon::LayoutDashboard)
            .size(28.0)
            .opacity(opacity);
        if todo_btn.show(ui, theme).clicked() {
            return Some(TimerAction::OpenTodo);
        }
    });
} else {
    self.hover_buttons_opacity.animate_to(0.0, theme.anim_fast);
    self.show_queue_popup = false;  // скрыть popup при уходе курсора
}
```

### Новые TimerAction:

```rust
pub enum TimerAction {
    Toggle,
    Skip,
    Reset,
    OpenStats,
    OpenSettings,
    OpenTodo,          // NEW — открыть todo viewport
    OpenQueue,         // NEW — показать queue popup
}
```

---

## Step 4.2: Отображение текущей задачи в таймере

Одна строка внизу таймера (перед hover-кнопками):

```
┌──────────────────────┐
│      Focus 25:00     │
│         ●━━━━        │
│                      │
│  📌 Написать API 1/2 │  ← эта строка
│                      │
│  (hover buttons)     │
└──────────────────────┘
```

```rust
// В timer_view после рендера прогресса и перед кнопками:
if let Some(task) = current_task {
    ui.add_space(theme.spacing_xs);

    ui.horizontal(|ui| {
        // Иконка пина
        ui.label(
            egui::RichText::new("📌")
                .size(12.0)
        );

        // Название задачи (обрезать если длинное)
        let max_title_width = ui.available_width() - 50.0;
        let title = truncate_text(&task.title, max_title_width, 12.0);
        ui.label(
            egui::RichText::new(&title)
                .size(12.0)
                .color(theme.text_secondary)
        );

        // Счётчик помодорок
        ui.label(
            egui::RichText::new(format!(
                "{}/{}",
                task.completed_pomodoros,
                task.planned_pomodoros
            ))
            .size(11.0)
            .color(theme.text_muted)
        );
    });
}
```

### Усечение длинного текста:
```rust
fn truncate_text(text: &str, max_width: f32, font_size: f32) -> String {
    // Примерная ширина символа (для proportional font ~ 0.5 * size)
    let approx_char_width = font_size * 0.5;
    let max_chars = (max_width / approx_char_width) as usize;

    if text.chars().count() <= max_chars {
        text.to_string()
    } else {
        // Обрезать по символам (не байтам!) — важно для кириллицы
        let truncated: String = text.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{}…", truncated)
    }
}
```

**ВАЖНО для кириллицы:**
- `.chars().count()` вместо `.len()` — кириллический символ = 2 байта в UTF-8
- `.chars().take(n)` вместо `&text[..n]` — иначе может обрезать посреди символа и паника
- Кириллические символы примерно такой же ширины как латинские в proportional шрифте

---

## Step 4.3: Queue popup

При нажатии на кнопку 🗂 — popup над/под кнопкой:

```
┌──────────────────────┐
│ Очередь         3 🍅│
├──────────────────────┤
│ ▸ Написать API    ×2 │  ← текущая (выделена)
│   Ревью PR        ×1 │
│                      │
│ [Очистить]           │
└──────────────────────┘
```

```rust
fn render_queue_popup(
    &mut self,
    ui: &mut egui::Ui,
    theme: &Theme,
    queue: &[QueuedTask],
) -> Vec<TimerAction> {
    // egui::popup_below_widget или Area::new()
    egui::Area::new(egui::Id::new("queue_popup"))
        .order(egui::Order::Foreground)
        .fixed_pos(/* позиция под кнопкой */)
        .show(ui.ctx(), |ui| {
            egui::Frame::popup(ui.style())
                .fill(theme.bg_elevated)
                .rounding(theme.rounding_md)
                .stroke(egui::Stroke::new(1.0, theme.border_default))
                .shadow(egui::epaint::Shadow {
                    offset: egui::vec2(0.0, 4.0),
                    blur: 12.0,
                    spread: 0.0,
                    color: egui::Color32::from_black_alpha(40),
                })
                .show(ui, |ui| {
                    ui.set_min_width(200.0);
                    ui.set_max_width(280.0);

                    // Заголовок
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Очередь")
                                .size(14.0)
                                .strong()
                                .color(theme.text_primary)
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let total: u32 = queue.iter()
                                .map(|t| t.planned_pomodoros - t.completed_pomodoros)
                                .sum();
                            ui.label(
                                egui::RichText::new(format!("{} 🍅", total))
                                    .size(12.0)
                                    .color(theme.text_muted)
                            );
                        });
                    });

                    ui.separator();

                    if queue.is_empty() {
                        ui.label(
                            egui::RichText::new("Очередь пуста")
                                .size(13.0)
                                .color(theme.text_muted)
                                .italics()
                        );
                    } else {
                        for (i, task) in queue.iter().enumerate() {
                            ui.horizontal(|ui| {
                                // Индикатор текущей
                                if i == 0 {
                                    ui.label(
                                        egui::RichText::new("▸")
                                            .size(12.0)
                                            .color(theme.accent_solid())
                                    );
                                } else {
                                    ui.allocate_exact_size(
                                        egui::vec2(12.0, 12.0),
                                        egui::Sense::hover(),
                                    );
                                }

                                // Название
                                let title = truncate_text(&task.title, 160.0, 13.0);
                                let color = if i == 0 {
                                    theme.text_primary
                                } else {
                                    theme.text_secondary
                                };
                                ui.label(
                                    egui::RichText::new(&title).size(13.0).color(color)
                                );

                                // Помодорки
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            egui::RichText::new(format!(
                                                "×{}",
                                                task.planned_pomodoros
                                            ))
                                            .size(12.0)
                                            .color(theme.text_muted)
                                        );

                                        // Кнопка удаления из очереди (на hover)
                                        if ui.small_button("✕").clicked() {
                                            // RemoveFromQueue action
                                        }
                                    },
                                );
                            });
                        }

                        ui.separator();
                        if ui.small_button("Очистить").clicked() {
                            // ClearQueue action
                        }
                    }
                });
        });
}
```

---

## Step 4.4: Логика очереди в app.rs

**Файл:** `src/app.rs`

### При завершении помодоро (work session completed):

```rust
fn handle_pomodoro_completed(&mut self) {
    // Существующая логика сохранения сессии...

    // Новое: обновить очередь
    if let Some(db) = &self.database {
        if let Ok(Some(current)) = db.get_current_queue_task() {
            // Увеличить счётчик помодорок
            if let Ok(all_done) = db.increment_queue_pomodoro(current.id) {
                if all_done {
                    // Задача выполнена — отметить todo как completed
                    let _ = db.toggle_todo(current.todo_id);
                    // Перейти к следующей задаче
                    let _ = db.advance_queue();
                }
            }

            // Обновить shared state для todo viewport
            if let Ok(mut state) = self.shared_todo.lock() {
                state.needs_refresh = true;
            }
        }
    }
}
```

### Добавление в очередь из todo:

```rust
fn handle_todo_action(&mut self, action: TodoAction) {
    match action {
        TodoAction::AddToQueue { todo_id, planned_pomodoros } => {
            if let Some(db) = &self.database {
                let _ = db.add_to_queue(todo_id, planned_pomodoros);
            }
        }
        // ... остальные действия
    }
}
```

---

## Step 4.5: Выбор количества помодорок

При нажатии 🍅 кнопки в todo — сразу добавляет 1 помодорку.
Но нужна возможность указать количество. Варианты:

### Вариант A: Popup при нажатии 🍅

```
🍅 clicked →
┌─────────────┐
│  1  2  3  4 │  ← кнопки количества
│    [+] [-]  │  ← или stepper
└─────────────┘
```

### Вариант B: Модификатор

- Клик = 1 помодорка
- Shift+клик = показать popup для выбора количества
- В popup: кнопки 1-4, или +/- stepper

### Рекомендация: Вариант B

Быстрое действие (1 клик = 1 помидорка) для 80% случаев.
Shift+клик для кастомного количества — power user feature.

```rust
if tomato_button.clicked() {
    if ui.input(|i| i.modifiers.shift) {
        // Показать popup выбора количества
        self.pomodoro_count_popup = Some(todo.id);
    } else {
        actions.push(TodoAction::AddToQueue {
            todo_id: todo.id,
            planned_pomodoros: 1,
        });
    }
}
```

---

## Step 4.6: Изменение количества помодорок в очереди

В Queue popup — возможность увеличить/уменьшить planned_pomodoros:

```rust
// В render_queue_popup, рядом с "×2":
if ui.small_button("+").clicked() {
    actions.push(QueueAction::IncrementPlanned { id: task.id });
}
if task.planned_pomodoros > 1 {
    if ui.small_button("-").clicked() {
        actions.push(QueueAction::DecrementPlanned { id: task.id });
    }
}
```

---

## Step 4.7: Новые иконки

**Файл:** `src/ui/components/icons.rs`

Добавить в `Icon` enum:
```rust
// Todo
ListTodo,       // иконка списка задач (для кнопки очереди)
ClipboardList,  // иконка todo-листа (для кнопки открытия todo)
GripVertical,   // иконка drag handle (для переупорядочивания, Phase 5)
```

Реализовать отрисовку в `draw_icon()`:

```rust
Icon::ListTodo => {
    // Три горизонтальные линии с чекбоксами
    // Линия 1
    painter.rect_stroke(
        Rect::from_min_size(scale(6.0, 7.0), vec2(s(3.0), s(3.0))),
        0.0, stroke
    );
    painter.line_segment([scale(11.0, 8.5), scale(18.0, 8.5)], stroke);
    // Линия 2
    painter.rect_stroke(
        Rect::from_min_size(scale(6.0, 12.0), vec2(s(3.0), s(3.0))),
        0.0, stroke
    );
    painter.line_segment([scale(11.0, 13.5), scale(18.0, 13.5)], stroke);
    // Линия 3
    painter.rect_stroke(
        Rect::from_min_size(scale(6.0, 17.0), vec2(s(3.0), s(3.0))),
        0.0, stroke
    );
    painter.line_segment([scale(11.0, 18.5), scale(18.0, 18.5)], stroke);
}
```

---

## Критерии готовности Phase 4:
- [ ] Кнопки Stats/Settings скрыты по умолчанию, появляются на hover окна
- [ ] Кнопки Queue и Todo появляются на hover
- [ ] Текущая задача отображается одной строкой в таймере
- [ ] Длинные названия задач корректно обрезаются (с учётом кириллицы)
- [ ] Queue popup показывает очередь задач
- [ ] 🍅 клик добавляет задачу в очередь (1 помидорка)
- [ ] Shift+🍅 — выбор количества помидорок
- [ ] При завершении помодоро — счётчик в очереди обновляется
- [ ] При выполнении всех помидорок задачи — автопереход к следующей
- [ ] Задачу можно удалить из очереди
- [ ] Новые иконки ListTodo, ClipboardList добавлены
