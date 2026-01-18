# PomodoRust

[![CI](https://github.com/gerrux/pomodorust/actions/workflows/ci.yml/badge.svg)](https://github.com/gerrux/pomodorust/actions/workflows/ci.yml)
[![Release](https://github.com/gerrux/pomodorust/actions/workflows/release.yml/badge.svg)](https://github.com/gerrux/pomodorust/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

A modern, lightweight Pomodoro timer with a sleek Vercel-style dark UI built in Rust.

<!--
![Screenshot](assets/screenshot.png)
-->

## Features

- **Pomodoro Technique**: Work sessions with short and long breaks
- **Customizable Timers**: Adjust work, short break, and long break durations
- **Session Tracking**: Track completed pomodoros and work sessions
- **Statistics Dashboard**: View your productivity metrics
- **Modern UI**: Beautiful dark theme with smooth animations
- **Audio Notifications**: Sound alerts when timer completes
- **Windows Integration**: Native notifications and system tray (Windows)
- **Persistent Data**: SQLite database for session history
- **Cross-platform**: Windows, macOS, Linux support
- **Standalone**: Single executable, no runtime dependencies

## Installation

### Download Binary

Download the latest release for your platform from the [Releases](https://github.com/gerrux/pomodorust/releases) page:

| Platform | Download |
|----------|----------|
| Windows (x64) | [pomodorust-windows-x64.zip](https://github.com/gerrux/pomodorust/releases/latest) |
| macOS (Intel) | [pomodorust-macos-x64.tar.gz](https://github.com/gerrux/pomodorust/releases/latest) |
| macOS (Apple Silicon) | [pomodorust-macos-arm64.tar.gz](https://github.com/gerrux/pomodorust/releases/latest) |
| Linux (x64) | [pomodorust-linux-x64.tar.gz](https://github.com/gerrux/pomodorust/releases/latest) |

### Build from Source

#### Prerequisites

- [Rust](https://rustup.rs/) 1.70+
- (Optional) [UPX](https://upx.github.io/) for compression

#### Quick Build

```bash
git clone https://github.com/gerrux/pomodorust.git
cd pomodorust
cargo build --release
```

The binary will be at `target/release/pomodorust` (or `pomodorust.exe` on Windows).

## Usage

### Timer Modes

| Mode | Default Duration | Description |
|------|------------------|-------------|
| Work | 25 minutes | Focus time for productive work |
| Short Break | 5 minutes | Quick rest between work sessions |
| Long Break | 15 minutes | Extended rest after 4 pomodoros |

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Space` | Start/Pause timer |
| `R` | Reset current timer |
| `S` | Skip to next session |
| `Escape` | Close settings/dialogs |

### Configuration

Settings are stored in your system's config directory:
- **Windows**: `%APPDATA%\pomodorust\config.toml`
- **macOS**: `~/Library/Application Support/pomodorust/config.toml`
- **Linux**: `~/.config/pomodorust/config.toml`

## Project Structure

```
pomodorust/
├── src/
│   ├── main.rs           # Entry point
│   ├── app.rs            # Application state
│   ├── lib.rs            # Library exports
│   ├── core/
│   │   ├── timer.rs      # Timer logic
│   │   ├── session.rs    # Session management
│   │   └── preset.rs     # Timer presets
│   ├── data/
│   │   ├── config.rs     # Configuration handling
│   │   ├── database.rs   # SQLite persistence
│   │   └── statistics.rs # Stats calculations
│   ├── ui/
│   │   ├── theme.rs      # Dark theme styling
│   │   ├── timer_view.rs # Timer display
│   │   ├── dashboard.rs  # Statistics view
│   │   ├── settings.rs   # Settings panel
│   │   ├── titlebar.rs   # Custom titlebar
│   │   ├── animations.rs # UI animations
│   │   └── components/   # Reusable UI components
│   ├── platform/
│   │   ├── audio.rs      # Sound playback
│   │   └── windows.rs    # Windows-specific features
│   └── utils/
│       └── mod.rs        # Utility functions
├── assets/               # Icons and resources
├── .github/
│   └── workflows/        # CI/CD pipelines
├── build.rs              # Build script
├── Cargo.toml
├── CHANGELOG.md
├── CONTRIBUTING.md
└── LICENSE
```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [egui](https://github.com/emilk/egui) - Immediate mode GUI library
- [rodio](https://github.com/RustAudio/rodio) - Audio playback
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite bindings
