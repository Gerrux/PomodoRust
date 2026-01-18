# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-01-18

### Added
- Initial release of PomodoRust
- Pomodoro timer with work, short break, and long break modes
- Customizable timer durations via settings panel
- Modern Vercel-style dark UI with smooth animations
- Statistics dashboard with productivity metrics
- Session tracking and history
- SQLite database for persistent data storage
- Audio notifications when timer completes
- Windows-specific features: native notifications, DWM effects
- Custom frameless window with draggable titlebar
- Keyboard shortcuts for timer control
- Timer presets for quick configuration
- Cross-platform support (Windows, macOS, Linux)

### Technical
- Built with Rust and egui/eframe
- SQLite for data persistence
- rodio for audio playback
- Optimized release profile with LTO and stripping

[Unreleased]: https://github.com/gerrux/pomodorust/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/gerrux/pomodorust/releases/tag/v0.1.0
