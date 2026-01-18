# Contributing to PomodoRust

Thank you for your interest in contributing! This document provides guidelines and information for contributors.

## How to Contribute

### Reporting Bugs

1. Check if the bug has already been reported in [Issues](https://github.com/gerrux/pomodorust/issues)
2. If not, create a new issue with:
   - Steps to reproduce
   - Expected vs actual behavior
   - OS and version
   - Screenshots if applicable

### Suggesting Features

1. Check existing issues for similar ideas
2. Create a new issue describing:
   - The feature and its use case
   - Why it would benefit users
   - Possible implementation approach

### Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature-name`
3. Make your changes
4. Run tests and linting (see below)
5. Commit with clear messages
6. Push and create a Pull Request

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later
- Git
- (Windows) Visual Studio Build Tools

### Clone and Build

```bash
git clone https://github.com/gerrux/pomodorust.git
cd pomodorust
cargo build
```

### Running

```bash
cargo run
```

### Code Quality

Before submitting a PR, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Full check (same as CI)
cargo fmt -- --check && cargo clippy -- -D warnings && cargo test
```

## Code Style

- Follow standard Rust conventions and idioms
- Use `cargo fmt` for formatting
- Keep functions focused and small
- Add doc comments for public APIs
- Prefer descriptive variable names

### Commit Messages

- Use present tense: "Add feature" not "Added feature"
- Use imperative mood: "Fix bug" not "Fixes bug"
- Keep the first line under 72 characters
- Reference issues when applicable: "Fix timer reset issue (#123)"

Examples:
```
Add keyboard shortcut for timer pause
Fix audio notification not playing on Windows
Update egui to latest version
Refactor timer module for clarity
```

## Project Structure

```
src/
├── main.rs           # Application entry point
├── app.rs            # Main application state
├── lib.rs            # Library exports
├── core/             # Core business logic
│   ├── timer.rs      # Timer state machine
│   ├── session.rs    # Session management
│   └── preset.rs     # Timer presets
├── data/             # Data layer
│   ├── config.rs     # Configuration handling
│   ├── database.rs   # SQLite operations
│   └── statistics.rs # Stats calculations
├── ui/               # User interface
│   ├── theme.rs      # Styling and colors
│   ├── timer_view.rs # Main timer display
│   ├── dashboard.rs  # Statistics view
│   ├── settings.rs   # Settings panel
│   ├── titlebar.rs   # Custom window titlebar
│   └── animations.rs # Animation utilities
├── platform/         # Platform-specific code
│   ├── audio.rs      # Sound playback
│   └── windows.rs    # Windows APIs
└── utils/            # Utility functions
```

### Key Components

- **Timer**: State machine for countdown logic
- **Session**: Tracks work/break cycles
- **Database**: SQLite for history persistence
- **Theme**: Vercel-style dark mode styling

## Testing Guidelines

- Write tests for core logic (timer, session)
- Test edge cases (timer at 0, invalid config, etc.)
- Keep tests focused and fast

Example test:
```rust
#[test]
fn test_timer_countdown() {
    let mut timer = Timer::new(Duration::from_secs(60));
    timer.start();
    timer.tick(Duration::from_secs(10));
    assert_eq!(timer.remaining(), Duration::from_secs(50));
}
```

## Questions?

Feel free to open an issue or start a discussion if you have questions about contributing.

Thank you for helping improve PomodoRust!
