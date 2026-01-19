# Good First Issues

Welcome contributors! These issues are designed to help you get started with the Pomodorust codebase. Each issue includes context, files to modify, and implementation hints.

---

## Issue #1: Add "Undo Last Session" Feature
**Difficulty:** Easy | **Area:** Data/UI

**Description:**
Add ability to undo/delete the last recorded session if user accidentally completed it.

**Files to modify:**
- `src/data/database.rs` - Add `delete_last_session()` method
- `src/ui/stats.rs` - Add "Undo" button in stats view
- `src/app.rs` - Handle the action

**Implementation hints:**
```rust
// In database.rs
pub fn delete_last_session(&self) -> Result<(), Error> {
    self.conn.execute(
        "DELETE FROM sessions WHERE id = (SELECT MAX(id) FROM sessions)",
        [],
    )?;
    Ok(())
}
```

---

## Issue #2: Add Session Notes
**Difficulty:** Easy-Medium | **Area:** Data/UI

**Description:**
Allow users to add optional notes to their Pomodoro sessions (e.g., "Worked on bug fix").

**Files to modify:**
- `src/data/database.rs` - Add `notes` column to sessions table
- `src/ui/timer_view.rs` - Add optional notes input
- `src/core/session.rs` - Store current notes

**Schema change:**
```sql
ALTER TABLE sessions ADD COLUMN notes TEXT;
```

---

## Issue #3: Add Config Validation
**Difficulty:** Easy | **Area:** Data

**Description:**
Validate config values when loading to prevent invalid states (e.g., work_duration = 0).

**Files to modify:**
- `src/data/config.rs` - Add `validate()` method

**Implementation hints:**
```rust
impl Config {
    pub fn validate(&mut self) {
        // Ensure reasonable bounds
        self.timer.work_duration = self.timer.work_duration.clamp(1, 120);
        self.timer.short_break = self.timer.short_break.clamp(1, 60);
        self.timer.long_break = self.timer.long_break.clamp(1, 60);
        self.sounds.volume = self.sounds.volume.clamp(0, 100);
        self.appearance.window_opacity = self.appearance.window_opacity.clamp(30, 100);
    }
}
```

---

## Issue #4: Add Keyboard Shortcut Hints
**Difficulty:** Easy | **Area:** UI

**Description:**
Show keyboard shortcuts on hover/tooltip for main buttons (Space = Toggle, S = Skip, etc.).

**Files to modify:**
- `src/ui/timer_view.rs` - Add `.on_hover_text()` to buttons

**Example:**
```rust
if GradientButton::new(icon, text)
    .show(ui, theme)
    .on_hover_text("Space to toggle")
    .clicked()
{
    // ...
}
```

---

## Issue #5: Add "Compact Mode" Layout
**Difficulty:** Medium | **Area:** UI

**Description:**
Implement compact mode that shows only the timer (no statistics, minimal UI). Config field already exists: `config.appearance.compact_mode`.

**Files to modify:**
- `src/ui/timer_view.rs` - Create compact layout variant
- `src/ui/settings.rs` - Add compact mode toggle
- `src/app.rs` - Pass compact_mode to timer view

---

## Issue #6: Add More Notification Sounds
**Difficulty:** Easy | **Area:** Assets/Audio

**Description:**
Add 2-3 more notification sound options. Sounds must be royalty-free.

**Files to modify:**
- `assets/` - Add new MP3 files (keep under 100KB each)
- `src/data/config.rs` - Add variants to `NotificationSound` enum
- `src/platform/audio.rs` - Add sound data constants

**Resources for free sounds:**
- freesound.org
- mixkit.co
- pixabay.com/sound-effects

---

## Issue #7: Add Hourly Statistics Distribution
**Difficulty:** Medium | **Area:** Data/UI

**Description:**
Show what hours of the day user is most productive (bar chart showing pomodoros by hour).

**Files to modify:**
- `src/data/statistics.rs` - Add `hourly_distribution()` method
- `src/ui/stats.rs` - Add hourly chart visualization

**SQL query:**
```sql
SELECT strftime('%H', started_at) as hour, COUNT(*) as count
FROM sessions
WHERE session_type = 'work' AND completed = 1
GROUP BY hour
ORDER BY hour;
```

---

## Issue #8: Add Session Recovery After Crash
**Difficulty:** Medium | **Area:** Core/Data

**Description:**
If app crashes during a session, offer to resume or record partial session on next start.

**Files to modify:**
- `src/data/database.rs` - Track "in progress" sessions
- `src/app.rs` - Check for incomplete sessions on startup

**Implementation idea:**
Save session start time to a temp file. On startup, check if file exists and offer recovery.

---

## Issue #9: Add macOS LaunchAgent Autostart
**Difficulty:** Easy-Medium | **Area:** Platform

**Description:**
Implement autostart for macOS using LaunchAgents.

**Files to modify:**
- `src/platform/mod.rs` - Add macOS implementation
- Create new file: `src/platform/macos.rs`

**LaunchAgent template:**
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.pomodorust</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/pomodorust</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
```

---

## Issue #10: Add Linux systemd Autostart
**Difficulty:** Easy-Medium | **Area:** Platform

**Description:**
Implement autostart for Linux using systemd user services.

**Files to modify:**
- `src/platform/mod.rs` - Add Linux implementation
- Create new file: `src/platform/linux.rs`

**Service file template:**
```ini
[Unit]
Description=Pomodorust Pomodoro Timer
After=graphical-session.target

[Service]
ExecStart=/usr/bin/pomodorust
Restart=on-failure

[Install]
WantedBy=default.target
```

---

## Issue #11: Add Confetti Animation on Goal Reached
**Difficulty:** Easy | **Area:** UI

**Description:**
Show a simple celebration animation when user completes their daily goal.

**Files to modify:**
- `src/ui/animations.rs` - Add confetti particle system
- `src/app.rs` - Trigger on goal completion

---

## Issue #12: Add "52/17" Preset
**Difficulty:** Easy | **Area:** Core/UI

**Description:**
Add the popular 52/17 productivity technique as a preset (52 min work, 17 min break).

**Files to modify:**
- `src/core/preset.rs` - Add new preset
- `src/ui/settings.rs` - Add preset button

```rust
pub fn fifty_two_seventeen() -> Self {
    Self {
        name: "52/17",
        work_duration: 52,
        short_break: 17,
        long_break: 30,
        sessions_before_long_break: 2,
    }
}
```

---

## Issue #13: Add Export to PDF
**Difficulty:** Medium | **Area:** Data

**Description:**
Add ability to export statistics report as PDF.

**Files to modify:**
- `Cargo.toml` - Add PDF crate (e.g., `printpdf`)
- `src/data/export.rs` - Add PDF export format

---

## Issue #14: Improve Error Messages
**Difficulty:** Easy | **Area:** Error Handling

**Description:**
Make error messages more user-friendly with actionable suggestions.

**Files to modify:**
- `src/error.rs` - Add `user_message()` method to errors

**Example:**
```rust
impl Error {
    pub fn user_message(&self) -> String {
        match self {
            Error::Config(ConfigError::DirectoryNotFound) =>
                "Could not find config directory. Please check your permissions.".into(),
            Error::Database(DatabaseError::Open { .. }) =>
                "Could not open database. Try deleting the data folder and restarting.".into(),
            // ...
        }
    }
}
```

---

## How to Contribute

1. **Comment on the issue** you want to work on
2. **Fork the repository**
3. **Create a branch:** `git checkout -b feature/issue-name`
4. **Make your changes**
5. **Run tests:** `cargo test`
6. **Run clippy:** `cargo clippy`
7. **Format code:** `cargo fmt`
8. **Submit a PR** referencing the issue

### Questions?

- Open a Discussion on GitHub
- Check `ARCHITECTURE.md` for codebase overview
- Look at existing code for patterns

---

*We welcome all contributions, big or small!*
