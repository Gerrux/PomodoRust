# Prompt Template: Lightweight Desktop App with Modern Dark UI

## Для чего этот шаблон

Шаблон промпта для создания lightweight desktop приложений на Rust с современным темным дизайном. Основан на опыте создания rust-calc.

## Рекомендуемые технологии

| Задача | GUI | TUI |
|--------|-----|-----|
| Framework | egui/eframe | ratatui + crossterm |
| Размер бинарника | ~2-4 MB | ~500KB-1MB |
| OpenGL | Требуется (glow) | Не требуется |
| Кроссплатформенность | Win/Linux/macOS | Win/Linux/macOS |

## Промпт-шаблон

```
Создай [ТИП ПРИЛОЖЕНИЯ] на Rust со следующими требованиями:

## Технический стек
- Rust 2021 edition
- GUI: eframe/egui (с glow backend, без wgpu для меньшего размера)
- Опционально TUI: ratatui + crossterm (для терминального режима)
- Feature flags для переключения между GUI/TUI

## Дизайн UI
- Современный темный интерфейс (dark mode only)
- Цветовая схема:
  - Фон окна: #16161E (RGB: 22, 22, 30)
  - Панели: #1C1C26 (RGB: 28, 28, 38)
  - Кнопки: #30303E (RGB: 48, 48, 62)
  - Акцент: #417BC3 (RGB: 65, 125, 195)
  - Текст: #FFFFFF (белый)
  - Текст вторичный: #A0A0B4 (RGB: 160, 160, 180)
- Скругленные углы (12px для окна, 8-12px для элементов)
- Кастомный title bar (без системных декораций)
- Прозрачный фон окна для rounded corners

## Кастомный Title Bar
- Высота: 32px
- Кнопки: minimize, maximize/restore, close
- Drag для перемещения окна
- Double-click для maximize/restore

## Структура проекта
src/
├── main.rs          # Entry point, window setup
├── app.rs           # Main app struct, UI rendering
├── [domain].rs      # Business logic
└── icon.rs          # Embedded icon (опционально)

## Cargo.toml оптимизации
[profile.release]
opt-level = "z"      # Размер
lto = true           # Link-time optimization
codegen-units = 1    # Один codegen unit
panic = "abort"      # Без unwinding
strip = true         # Strip symbols

## Фичи
- Keyboard shortcuts
- Responsive layout (адаптация к размеру окна)
- Плавные hover эффекты на кнопках
- Встроенный шрифт (subset для минимизации размера)

## Constraints
- Минимальный размер бинарника
- Быстрый cold start
- Нативный look and feel
- Без внешних зависимостей runtime (статическая линковка)
```

## Альтернативные шаблоны для начала

### 1. Минимальный egui starter
```bash
cargo new my-app
cd my-app
cargo add eframe
```

### 2. С готовым template (eframe)
```bash
cargo generate --git https://github.com/emilk/eframe_template
```

### 3. TUI с ratatui
```bash
cargo new my-app
cd my-app
cargo add ratatui crossterm
```

### 4. Tauri (для web-like UI)
```bash
npm create tauri-app@latest
```

## Чек-лист для проекта

- [ ] Feature flags для GUI/TUI режимов
- [ ] Кастомный title bar с drag & window controls
- [ ] Прозрачный фон для rounded corners
- [ ] Keyboard shortcuts
- [ ] Embedded font (subset)
- [ ] Release profile с оптимизациями
- [ ] Cross-platform build (CI/CD)
- [ ] Icon embedding (Windows resource)

## Пример minimal main.rs для egui

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_decorations(false)  // Кастомный title bar
            .with_transparent(true),   // Для rounded corners
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "My App",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

struct MyApp;

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self
    }
}

impl eframe::App for MyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.0, 0.0, 0.0, 0.0] // Transparent
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Draw rounded window background
            let rect = ui.max_rect();
            ui.painter().rect_filled(
                rect,
                12.0,  // Rounding
                egui::Color32::from_rgb(22, 22, 30),
            );

            ui.heading("Hello!");
        });
    }
}
```

## Lessons Learned из rust-calc

1. **glow vs wgpu**: glow даёт меньший бинарник и быстрее компилируется
2. **Font subsetting**: Уменьшает размер шрифта с ~200KB до ~30KB
3. **Decorations false**: Обязательно для кастомного title bar
4. **Transparent true**: Обязательно для rounded corners на Windows
5. **windows_subsystem = "windows"**: Скрывает консоль в release
6. **Feature flags**: Позволяют собирать разные версии (GUI/TUI/both)

## Полезные ресурсы

- [egui demo](https://www.egui.rs/)
- [eframe template](https://github.com/emilk/eframe_template)
- [ratatui examples](https://github.com/ratatui/ratatui/tree/main/examples)
- [Tauri](https://tauri.app/)
