# PomodoRust Session Context

## Project Overview
**Type:** Desktop Pomodoro Timer Application
**Stack:** Rust + egui/eframe
**Status:** MVP Complete, Layout Fixes Applied

## Architecture

### Core Modules
- `src/core/` - Timer logic, session management, presets
  - `timer.rs` - State machine for countdown timer
  - `session.rs` - Work/Break cycle management
  - `preset.rs` - Timer presets (Classic 25/5/15, Short, Long)

### UI Modules
- `src/ui/` - egui-based interface
  - `theme.rs` - Vercel-style design system with accent colors
  - `timer_view.rs` - Main compact timer widget
  - `dashboard.rs` - Statistics view with weekly chart
  - `settings.rs` - Configuration panel
  - `titlebar.rs` - Custom frameless window controls
  - `components/` - Reusable UI components (Card, CircularProgress, GradientButton, IconButton)

### Data Layer
- `src/data/` - Persistence
  - `config.rs` - TOML configuration
  - `database.rs` - SQLite for session history
  - `statistics.rs` - Stats aggregation

### Platform Layer
- `src/platform/` - Windows-specific
  - `audio.rs` - Sound notifications (programmatic WAV)
  - `windows.rs` - Registry autostart, notifications

## Key Technical Decisions

### Layout Best Practices (egui)
1. **Use `egui::Frame`** for containers with padding/borders - NOT `new_child`
2. **Use `vertical_centered`** for centering content
3. **Use `horizontal`** inside centered layouts for button groups
4. **Avoid mixing layout directions** in nested closures
5. **Use `allocate_ui_at_rect`** for positioned content instead of `new_child`
6. **Use `Layout::right_to_left`** for right-aligned elements (settings rows)

### What NOT to do (caused layout drift):
- `ui.add_space((available - X) / 2)` - manual centering
- `ui.add_space(available - X)` - pushing elements
- Nested `with_layout` calls inside `vertical_centered`
- Using `new_child` / `child_ui` for content areas

### Gradient Rendering
egui doesn't support native gradients. Workaround:
- Draw base color with `rect_filled`
- Overlay 4 semi-transparent stepped rects for gradient effect

### Custom Titlebar
- Frameless window with `with_decorations(false)`
- Manual drag handling via `ViewportCommand::StartDrag`
- Custom minimize/maximize/close buttons

## Current State

### Completed
- Full timer functionality
- Work/Break cycle management
- SQLite statistics storage
- TOML configuration
- Custom UI with animations
- Sound notifications
- Windows autostart support

### Recent Fixes (Layout)
- Simplified `app.rs` - using `egui::Frame` with `inner_margin`
- Simplified `timer_view.rs` - pure `vertical_centered` + `horizontal`
- Simplified `card.rs` - replaced custom painting with `egui::Frame`
- Fixed `circular_progress.rs` - using `allocate_ui_at_rect`

### Known Issues
- Some deprecated warnings (cosmetic)
- Unused code warnings (planned features)

## Build Info
- Binary size: ~4.9 MB (release)
- Platform: Windows x64
- Dependencies: egui 0.29, rusqlite, rodio, chrono

## File Structure
```
pomodorust/
├── Cargo.toml
├── build.rs (Windows manifest)
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── app.rs (main app struct)
│   ├── core/
│   │   ├── mod.rs
│   │   ├── timer.rs
│   │   ├── session.rs
│   │   └── preset.rs
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── theme.rs
│   │   ├── animations.rs
│   │   ├── titlebar.rs
│   │   ├── timer_view.rs
│   │   ├── dashboard.rs
│   │   ├── settings.rs
│   │   └── components/
│   │       ├── mod.rs
│   │       ├── button.rs
│   │       ├── card.rs
│   │       ├── circular_progress.rs
│   │       └── slider.rs
│   ├── data/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── database.rs
│   │   └── statistics.rs
│   ├── platform/
│   │   ├── mod.rs
│   │   ├── audio.rs
│   │   └── windows.rs
│   └── utils/
│       └── mod.rs
└── .gitignore
```

## Session Summary
Created complete Pomodoro timer app with modern UI. Fixed layout issues by switching from manual spacing calculations to proper egui layout primitives (Frame, vertical_centered, horizontal).
