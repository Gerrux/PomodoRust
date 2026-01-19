# Pomodorust Architecture

This document provides a technical overview of Pomodorust's architecture for contributors and maintainers.

## Overview

Pomodorust is a native desktop Pomodoro timer built in Rust using the egui/eframe GUI framework. It follows a modular architecture with clear separation of concerns.

```
┌─────────────────────────────────────────────────────────────┐
│                       main.rs                               │
│                    (Entry point)                            │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    PomodoRustApp                            │
│                      (app.rs)                               │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ Session  │ │  Config  │ │  Theme   │ │   Database   │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │
└─────────────────────────┬───────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          ▼               ▼               ▼
    ┌──────────┐    ┌──────────┐    ┌──────────┐
    │TimerView │    │StatsView │    │ Settings │
    └──────────┘    └──────────┘    └──────────┘
```

## Directory Structure

```
pomodorust/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports
│   ├── app.rs               # Main application struct (PomodoRustApp)
│   ├── error.rs             # Error types and handling
│   │
│   ├── core/                # Business logic
│   │   ├── mod.rs           # Module exports
│   │   ├── timer.rs         # Timer state machine
│   │   ├── session.rs       # Pomodoro session management
│   │   └── preset.rs        # Timer presets (Classic, Short, Long)
│   │
│   ├── data/                # Persistence layer
│   │   ├── mod.rs           # Module exports
│   │   ├── config.rs        # TOML configuration
│   │   ├── database.rs      # SQLite database
│   │   ├── statistics.rs    # Statistics aggregation
│   │   └── export.rs        # CSV/JSON export
│   │
│   ├── ui/                  # User interface
│   │   ├── mod.rs           # Module exports
│   │   ├── theme.rs         # Design system (colors, fonts, spacing)
│   │   ├── timer_view.rs    # Main timer screen
│   │   ├── stats.rs         # Statistics dashboard
│   │   ├── settings.rs      # Settings panel
│   │   ├── titlebar.rs      # Custom window titlebar
│   │   ├── animations.rs    # Animation utilities
│   │   ├── layout.rs        # Layout helpers
│   │   └── components/      # Reusable UI components
│   │       ├── mod.rs
│   │       ├── ascii_art.rs     # ASCII spinners for retro themes
│   │       ├── button.rs        # GradientButton, IconButton
│   │       ├── card.rs          # Card container
│   │       ├── circular_progress.rs  # Circular progress bar
│   │       ├── icons.rs         # Icon definitions
│   │       └── slider.rs        # Custom slider
│   │
│   ├── platform/            # Platform-specific code
│   │   ├── mod.rs           # Cross-platform abstractions
│   │   ├── audio.rs         # Audio playback (rodio)
│   │   └── windows.rs       # Windows-specific APIs (DWM, registry)
│   │
│   └── utils/
│       └── mod.rs           # Utility functions
│
├── assets/                  # Static assets
│   ├── icon.png             # Application icon
│   ├── soft_bell.mp3        # Notification sounds
│   ├── level_up.mp3
│   └── digital_alert.mp3
│
├── .github/
│   └── workflows/           # CI/CD pipelines
│
├── Cargo.toml               # Dependencies
├── build.rs                 # Windows resource compilation
├── README.md
├── CHANGELOG.md
└── CONTRIBUTING.md
```

## Core Modules

### Timer (`core/timer.rs`)

The `Timer` struct is a state machine with the following states:

```rust
pub enum TimerState {
    Idle,      // Timer not started
    Running,   // Timer counting down
    Paused,    // Timer paused
    Completed, // Timer finished
}

pub enum TimerEvent {
    Started,   // Timer started
    Resumed,   // Timer resumed from pause
    Paused,    // Timer paused
    Ticked,    // Second elapsed
    Completed, // Timer finished
}
```

State transitions:
```
Idle ──[start]──► Running ──[complete]──► Completed
                     │ ▲                      │
                     │ │                      │
               [pause] [resume]          [reset]
                     │ │                      │
                     ▼ │                      │
                   Paused ◄────────────────────┘
```

### Session (`core/session.rs`)

`Session` manages the Pomodoro cycle:

```
Work → Short Break → Work → Short Break → Work → Short Break → Work → Long Break
 1         1          2         2          3         3          4        4
```

Key methods:
- `toggle()` - Start/pause the timer
- `skip()` - Skip current session
- `reset()` - Reset to beginning
- `update()` - Called every frame, returns events

### Configuration (`data/config.rs`)

Configuration is stored in TOML format:

```toml
[timer]
work_duration = 25
short_break = 5
long_break = 15
sessions_before_long = 4

[sounds]
enabled = true
volume = 80
notification_sound = "SoftBell"

[appearance]
accent_color = "Blue"
window_opacity = 100

[system]
start_with_windows = false
notifications_enabled = true

[window]
always_on_top = false
```

Location:
- Windows: `%APPDATA%\pomodorust\config.toml`
- macOS: `~/Library/Application Support/pomodorust/config.toml`
- Linux: `~/.config/pomodorust/config.toml`

### Database (`data/database.rs`)

SQLite database with three tables:

```sql
-- Individual sessions
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    session_type TEXT NOT NULL,      -- 'work', 'short_break', 'long_break'
    planned_duration INTEGER,
    actual_duration INTEGER,
    completed INTEGER,
    started_at TEXT,
    ended_at TEXT
);

-- Daily aggregates
CREATE TABLE daily_stats (
    date TEXT PRIMARY KEY,
    total_work_seconds INTEGER,
    total_pomodoros INTEGER,
    total_breaks INTEGER
);

-- Streak tracking
CREATE TABLE streaks (
    id INTEGER PRIMARY KEY,
    type TEXT,                       -- 'current', 'longest'
    count INTEGER,
    last_date TEXT
);
```

## UI Architecture

### Theme System (`ui/theme.rs`)

The `Theme` struct contains all design tokens:

```rust
pub struct Theme {
    // Colors
    pub bg_primary: Color32,
    pub bg_secondary: Color32,
    pub text_primary: Color32,
    pub accent: AccentColor,

    // Spacing
    pub spacing_sm: f32,
    pub spacing_md: f32,

    // Rounding
    pub rounding_md: f32,

    // ...
}
```

Nine accent colors available:
- Modern: Blue, Purple, Rose, Emerald, Amber, Cyan
- Retro: Matrix (green), RetroAmber, Synthwave (pink/cyan)

### View Pattern

Each view returns an `Option<Action>`:

```rust
pub enum TimerAction {
    Toggle,
    Skip,
    Reset,
    OpenStats,
    OpenSettings,
}

impl TimerView {
    pub fn show(&mut self, ui: &mut Ui, ...) -> Option<TimerAction> {
        // Render UI
        // Return action if user clicked something
    }
}
```

The main app handles actions in `handle_*_action()` methods.

### Component Composition

UI components are composable:

```rust
// Card component wraps content
Card::new().show(ui, theme, |ui| {
    // Content goes here
    duration_row(ui, theme, "Focus", &mut value, 1.0, 90.0);
});

// Buttons with gradients
GradientButton::new(icon, text)
    .with_gradient(start_color, end_color)
    .show(ui, theme);
```

## Platform Abstraction

### Cross-platform Interface (`platform/mod.rs`)

```rust
// Notification abstraction
pub fn show_notification(title: &str, body: &str);

// Autostart abstraction
pub fn set_autostart(enabled: bool) -> Result<(), Error>;

// Window effects
pub fn apply_window_effects(hwnd: isize);
```

### Windows-specific (`platform/windows.rs`)

- DWM effects (dark mode frame, rounded corners)
- Registry autostart
- Window flashing on timer completion
- Native toast notifications

### Audio (`platform/audio.rs`)

Uses `rodio` crate for cross-platform audio:

```rust
pub struct AudioPlayer {
    stream: OutputStream,
    handle: OutputStreamHandle,
    volume: f32,
}

impl AudioPlayer {
    pub fn play_notification(&self, sound: NotificationSound);
    pub fn set_volume(&mut self, volume: f32);
}
```

Sounds are embedded in binary via `include_bytes!()`.

## Data Flow

### Timer Update Loop

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   eframe    │────►│ app.update()│────►│session.update│
│  (60 fps)   │     │             │     │             │
└─────────────┘     └─────────────┘     └──────┬──────┘
                                               │
                           ┌───────────────────┴───────────────────┐
                           │                                       │
                           ▼                                       ▼
                    TimerEvent::Ticked                    TimerEvent::Completed
                           │                                       │
                           ▼                                       ▼
                    Request repaint                        on_timer_completed()
                                                                   │
                                                    ┌──────────────┴──────────────┐
                                                    │              │              │
                                                    ▼              ▼              ▼
                                              Record to DB   Play sound    Show notification
```

### Config Update Flow

```
Settings UI  ──►  SettingsState  ──►  SettingsAction::UpdateConfig
                       │                        │
                       │                        ▼
                       │               app.apply_config()
                       │                        │
                       │         ┌──────────────┴──────────────┐
                       │         │              │              │
                       │         ▼              ▼              ▼
                       │   Update theme   Update timer   Update audio
                       │                                       │
                       └───────────────────────────────────────┘
                                           │
                                           ▼
                                    config.save()
```

## Error Handling

All errors use the `Error` enum from `error.rs`:

```rust
#[derive(Debug)]
pub enum Error {
    Config(ConfigError),
    Database(DatabaseError),
    Audio(AudioError),
    // ...
}
```

Errors are logged via `tracing` and shown to user when appropriate.

## Testing

Current test coverage:
- `core/timer.rs` - Unit tests for state machine
- `error.rs` - Error type tests
- `ui/animations.rs` - Easing function tests
- `ui/layout.rs` - Layout calculation tests

Run tests:
```bash
cargo test
```

## Building

### Debug Build
```bash
cargo build
```

### Release Build (Optimized)
```bash
cargo build --release
```

Release profile settings (Cargo.toml):
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization
panic = "abort"     # Smaller binary
strip = true        # Strip symbols
```

## Contributing

### Adding a New Feature

1. **Core logic** - Add to `core/` if it's business logic
2. **Persistence** - Add to `data/` if it needs storage
3. **UI** - Add to `ui/` for visual components
4. **Platform** - Add to `platform/` for OS-specific features

### Adding a New Setting

1. Add field to appropriate struct in `config.rs`
2. Add to `SettingsState` in `settings.rs`
3. Update `from_config()`, `differs_from()`, `apply_to()`
4. Add UI element in `SettingsView::show()`
5. Handle in `app.rs` `apply_config()` if needed

### Code Style

- Use `rustfmt` for formatting
- Use `clippy` for linting
- Document public APIs with `///` comments
- Keep functions small and focused

## Dependencies

| Crate | Purpose |
|-------|---------|
| egui/eframe | GUI framework |
| rusqlite | SQLite database |
| serde | Serialization |
| toml | Config file format |
| rodio | Audio playback |
| chrono | Date/time handling |
| directories | Platform paths |
| tracing | Logging |
| windows | Windows API (Windows only) |

## Performance Considerations

1. **Animation frame rate** - Only request repaint when needed
2. **Database** - Sync writes (may block briefly on completion)
3. **Audio** - Separate thread via rodio
4. **Binary size** - ~15MB with all optimizations

## Future Architecture Plans

Planned improvements:
- [ ] Async database operations (tokio)
- [ ] Plugin system for extensibility
- [ ] IPC for CLI integration
- [ ] Trait-based platform abstraction

---

*Last updated: 2026-01-19*
