# PomodoRust - Project Documentation Index

## Project Overview

| Property | Value |
|----------|-------|
| **Name** | PomodoRust |
| **Type** | Lightweight Desktop Pomodoro Timer |
| **Language** | Rust 2021 Edition |
| **Platform** | Windows (primary), cross-platform potential |
| **GUI Framework** | eframe/egui (glow backend) |
| **Style** | Vercel/shadcn-inspired dark UI |
| **License** | MIT |

## Quick Start

```bash
# Build and run
cargo run

# Release build (optimized, ~3-4MB)
cargo build --release
```

---

## Architecture Overview

```
                    ┌─────────────────────────────────────────┐
                    │              UI Layer                   │
                    │  TitleBar │ TimerView │ Dashboard │ ... │
                    └────────────────────┬────────────────────┘
                                         │
                    ┌────────────────────┴────────────────────┐
                    │            App State (egui)             │
                    │         PomodoRustApp (app.rs)          │
                    └────────────────────┬────────────────────┘
                                         │
        ┌────────────────────────────────┼────────────────────────────────┐
        │                           Core Layer                            │
        │          Timer │ Session │ Preset │ PresetManager               │
        └────────────────────────────────┬────────────────────────────────┘
                                         │
        ┌────────────────────────────────┼────────────────────────────────┐
        │                          Data Layer                             │
        │              Database (SQLite) │ Config (TOML) │ Statistics     │
        └────────────────────────────────┬────────────────────────────────┘
                                         │
        ┌────────────────────────────────┴────────────────────────────────┐
        │                        Platform Layer                           │
        │       AudioPlayer │ Windows Notifications │ Autostart           │
        └─────────────────────────────────────────────────────────────────┘
```

---

## Project Structure

```
pomodorust/
├── Cargo.toml              # Dependencies and build config
├── Cargo.lock              # Dependency lock file
├── build.rs                # Windows resource embedding
├── SPECIFICATION.md        # Detailed project specification (Russian)
├── PROMPT_TEMPLATE.md      # AI prompt template
│
└── src/
    ├── main.rs             # Entry point, window setup
    ├── app.rs              # Main App struct, UI orchestration
    ├── lib.rs              # Library exports
    │
    ├── core/               # Business logic
    │   ├── mod.rs          # Module exports
    │   ├── timer.rs        # Timer state machine
    │   ├── session.rs      # Session workflow management
    │   └── preset.rs       # Timer presets (Classic, Short, Long, 52/17)
    │
    ├── ui/                 # User interface
    │   ├── mod.rs          # Module exports
    │   ├── theme.rs        # Design system (colors, spacing, fonts)
    │   ├── titlebar.rs     # Custom window titlebar
    │   ├── timer_view.rs   # Compact timer widget
    │   ├── dashboard.rs    # Statistics dashboard
    │   ├── settings.rs     # Settings panel
    │   ├── animations.rs   # Animation utilities
    │   └── components/     # Reusable UI components
    │       ├── mod.rs
    │       ├── button.rs   # GradientButton, IconButton
    │       ├── slider.rs   # CustomSlider
    │       ├── card.rs     # Card component
    │       └── circular_progress.rs
    │
    ├── data/               # Persistence layer
    │   ├── mod.rs          # Module exports
    │   ├── database.rs     # SQLite operations
    │   ├── config.rs       # TOML config management
    │   └── statistics.rs   # Statistics aggregation
    │
    ├── platform/           # Platform-specific code
    │   ├── mod.rs          # Module exports + cross-platform fallbacks
    │   ├── windows.rs      # Windows notifications, autostart, tray
    │   └── audio.rs        # Sound playback (rodio)
    │
    └── utils/              # Utility functions
        └── mod.rs          # Time formatting utilities
```

---

## Module Reference

### Core Modules (`src/core/`)

#### `timer.rs` - Timer State Machine
| Export | Type | Description |
|--------|------|-------------|
| `Timer` | struct | Core timer with start/pause/reset/update |
| `TimerState` | enum | `Idle`, `Running`, `Paused`, `Completed` |
| `TimerEvent` | enum | `Started`, `Paused`, `Resumed`, `Reset`, `Completed`, `Tick` |

Key methods:
- `Timer::new(duration_secs)` / `Timer::from_minutes(minutes)`
- `start()`, `pause()`, `toggle()`, `reset()`
- `update()` - Call every frame, returns `Option<TimerEvent>`
- `progress()` - Returns 0.0-1.0
- `remaining_formatted()` - Returns "MM:SS"

#### `session.rs` - Pomodoro Session Workflow
| Export | Type | Description |
|--------|------|-------------|
| `Session` | struct | Manages work/break cycle |
| `SessionType` | enum | `Work`, `ShortBreak`, `LongBreak` |
| `SessionState` | enum | `Ready`, `Active`, `Paused`, `Completed` |

Key methods:
- `Session::new()` / `Session::with_preset(preset)`
- `toggle()`, `skip()`, `reset()`
- `update()` - Returns `(Option<TimerEvent>, should_auto_start)`
- `current_session_in_cycle()` - Returns 1-4

#### `preset.rs` - Timer Presets
| Export | Type | Description |
|--------|------|-------------|
| `Preset` | struct | Timer configuration (work/break durations) |
| `PresetManager` | struct | Manages multiple presets |

Built-in presets:
- `Preset::classic()` - 25/5/15 (4 sessions)
- `Preset::short()` - 15/3/10 (4 sessions)
- `Preset::long()` - 50/10/30 (2 sessions)
- `Preset::fifty_two_seventeen()` - 52/17/30 (2 sessions)

---

### UI Modules (`src/ui/`)

#### `theme.rs` - Design System
| Export | Type | Description |
|--------|------|-------------|
| `Theme` | struct | Complete design tokens |
| `AccentColor` | enum | `Blue`, `Purple`, `Rose`, `Emerald`, `Amber`, `Cyan` |

Color palette (Vercel-inspired):
- Backgrounds: `bg_base` (#000), `bg_primary` (#0A0A0B), `bg_secondary` (#111113)
- Text: `text_primary` (#FAFAFA), `text_secondary` (#A1A1AA), `text_muted` (#71717A)
- Session: `work_start/end` (rose), `break_start/end` (emerald), `long_break_start/end` (indigo)

#### `animations.rs` - Animation System
| Export | Type | Description |
|--------|------|-------------|
| `AnimationState` | struct | Global animation state (pulse, glow, transitions) |
| `AnimatedValue` | struct | Single animated float value |
| `InteractionState` | struct | Hover/press animations |
| `Easing` | enum | `Linear`, `EaseIn`, `EaseOut`, `EaseInOut`, `Spring` |
| `CounterAnimation` | struct | Spring-based number counter |

#### `components/` - Reusable Components
| Component | Description |
|-----------|-------------|
| `GradientButton` | Primary action button with gradient |
| `IconButton` | Icon-only button |
| `Card` | Container with border and background |
| `CircularProgress` | Timer progress ring |
| `CustomSlider` | Styled slider for settings |

---

### Data Modules (`src/data/`)

#### `database.rs` - SQLite Persistence
Tables:
- `sessions` - Individual pomodoro records
- `daily_stats` - Aggregated daily statistics
- `streaks` - Current/longest streak tracking
- `presets` - Custom timer presets

#### `config.rs` - TOML Configuration
Config sections:
- `[timer]` - Durations, auto-start settings
- `[sounds]` - Volume, sound selections
- `[appearance]` - Accent color, compact mode
- `[system]` - Autostart, tray, notifications
- `[window]` - Size, position, always-on-top

#### `statistics.rs` - Statistics Aggregation
| Field | Description |
|-------|-------------|
| `today_work_seconds` | Today's focus time |
| `today_pomodoros` | Completed sessions today |
| `week_daily_hours` | Array of 7 daily hours |
| `current_streak` | Current day streak |
| `total_hours()` | All-time hours |

---

### Platform Modules (`src/platform/`)

#### `windows.rs` - Windows Integration
| Function | Description |
|----------|-------------|
| `show_notification(title, body)` | Windows Toast notification |
| `set_autostart(enabled)` | Add/remove from startup |
| `remove_autostart()` | Remove from startup |

#### `audio.rs` - Sound Playback
| Export | Type | Description |
|--------|------|-------------|
| `AudioPlayer` | struct | rodio-based audio player |
| `SoundType` | enum | `WorkEnd`, `BreakEnd`, `Notification`, `Tick` |

---

### App Module (`src/app.rs`)

#### `PomodoRustApp` - Main Application
| Field | Type | Description |
|-------|------|-------------|
| `session` | Session | Current pomodoro session |
| `config` | Config | User configuration |
| `theme` | Theme | Active theme |
| `database` | Option<Database> | SQLite connection |
| `statistics` | Statistics | Cached stats |
| `current_view` | View | `Timer`, `Dashboard`, `Settings` |

Implements `eframe::App` with:
- Custom decorated window with resize handling
- Keyboard shortcuts (Space, Escape, D)
- Timer update loop with auto-start support
- View switching (Timer/Dashboard/Settings)

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| eframe | 0.29 | GUI framework |
| egui | 0.29 | Immediate mode GUI |
| rusqlite | 0.32 | SQLite database |
| serde | 1.0 | Serialization |
| toml | 0.8 | Config file format |
| rodio | 0.19 | Audio playback |
| chrono | 0.4 | Date/time handling |
| directories | 5.0 | Platform directories |
| tracing | 0.1 | Logging |

Windows-specific:
| Crate | Version | Purpose |
|-------|---------|---------|
| windows | 0.58 | Win32 API bindings |
| winreg | 0.52 | Registry access |
| notify-rust | 4 | Toast notifications |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Space` | Start/Pause timer (Timer view) |
| `D` | Open Dashboard (Timer view) |
| `Escape` | Back / Minimize to tray |

---

## Configuration File

Location: `%APPDATA%/pomodorust/config.toml`

```toml
[timer]
work_duration = 25
short_break = 5
long_break = 15
sessions_before_long = 4
auto_start_breaks = false
auto_start_work = false

[sounds]
enabled = true
volume = 80

[appearance]
accent_color = "blue"

[system]
start_with_windows = false
minimize_to_tray = true
notifications_enabled = true
```

---

## Database Schema

Location: `%APPDATA%/pomodorust/pomodorust.db`

```sql
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    session_type TEXT NOT NULL,
    duration_seconds INTEGER NOT NULL,
    planned_duration INTEGER NOT NULL,
    completed BOOLEAN NOT NULL,
    started_at DATETIME NOT NULL,
    ended_at DATETIME
);

CREATE TABLE daily_stats (
    id INTEGER PRIMARY KEY,
    date DATE UNIQUE NOT NULL,
    total_work_seconds INTEGER DEFAULT 0,
    completed_pomodoros INTEGER DEFAULT 0
);

CREATE TABLE streaks (
    id INTEGER PRIMARY KEY,
    current_streak INTEGER DEFAULT 0,
    longest_streak INTEGER DEFAULT 0,
    last_active_date DATE
);
```

---

## Build Profiles

### Debug
```toml
[profile.dev]
opt-level = 1
```

### Release (Optimized)
```toml
[profile.release]
opt-level = "z"      # Size optimization
lto = true           # Link-time optimization
codegen-units = 1    # Single codegen unit
panic = "abort"      # Abort on panic
strip = true         # Strip symbols
```

---

## Cross-References

| Feature | Primary Files |
|---------|---------------|
| Timer Logic | `core/timer.rs`, `core/session.rs` |
| Presets | `core/preset.rs`, `data/config.rs` |
| Theme/Styling | `ui/theme.rs`, `ui/animations.rs` |
| Statistics | `data/statistics.rs`, `data/database.rs` |
| Windows Integration | `platform/windows.rs` |
| Settings UI | `ui/settings.rs`, `data/config.rs` |
| Main App Loop | `app.rs`, `main.rs` |

---

*Generated by /sc:index*
