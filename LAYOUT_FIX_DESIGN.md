# PomodoRust UI Layout Fix Design

## Problem Analysis

The UI "floats" (elements misalign) because of several anti-patterns:

### 1. **Manual Centering with Calculations**
```rust
// WRONG: Manual spacing calculations that break on resize
ui.add_space((ui.available_width() - 120.0) / 2.0);
```
This breaks because:
- `available_width()` changes during layout
- Hardcoded pixel values don't adapt

### 2. **Inconsistent Layout Containers**
- Mixing `ui.horizontal()` with manual `add_space()` centering
- No use of egui's built-in layout alignment

### 3. **Arbitrary Pixel Values**
```rust
ui.add_space(ui.available_width() - 100.0);  // Titlebar
ui.add_space(ui.available_width() - 120.0);  // Settings
ui.add_space(ui.available_width() - 180.0);  // Volume slider
```
These magic numbers cause layout drift.

### 4. **No Fixed Layout Constraints**
- Cards and components don't use consistent sizing
- No grid or container system

---

## Best Practices Solution

### Pattern 1: Use egui Layouts Properly
```rust
// CORRECT: Use layout alignment
ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    // Content is auto-centered
});

// CORRECT: Horizontal with spacing
ui.horizontal(|ui| {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center).with_main_justify(true), |ui| {
        // Elements spread evenly
    });
});
```

### Pattern 2: Use `ui.centered_and_justified()` for Centering
```rust
// CORRECT: Center content properly
ui.vertical_centered_justified(|ui| {
    // All content centered with full width
});
```

### Pattern 3: Use Columns for Grid Layouts
```rust
// CORRECT: Use columns for multi-column layouts
ui.columns(2, |columns| {
    columns[0].vertical(|ui| { /* Left column */ });
    columns[1].vertical(|ui| { /* Right column */ });
});
```

### Pattern 4: Fixed Size Containers
```rust
// CORRECT: Use allocate_ui_with_layout for fixed areas
ui.allocate_ui_with_layout(
    vec2(300.0, 400.0),
    egui::Layout::top_down(egui::Align::Center),
    |ui| { /* Content */ }
);
```

### Pattern 5: Use `ui.add_sized()` for Consistent Sizing
```rust
// CORRECT: Explicit sizing for widgets
ui.add_sized(vec2(100.0, 40.0), egui::Button::new("Click"));
```

---

## Files to Fix

### 1. `src/ui/timer_view.rs`

**Current Issues:**
- Line 75: `ui.add_space((ui.available_width() - 120.0) / 2.0);` - manual centering
- Line 144: `ui.add_space((ui.available_width() - (total as f32 * 16.0)) / 2.0);` - dots centering

**Fix Strategy:**
```rust
// Replace manual centering with:
ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    ui.horizontal(|ui| {
        // Control buttons - auto-centered
    });
});
```

### 2. `src/ui/titlebar.rs`

**Current Issues:**
- Uses hardcoded button positions
- Manual rect calculations

**Fix Strategy:**
- Keep manual positioning (necessary for custom titlebar)
- But use consistent spacing constants

### 3. `src/ui/dashboard.rs`

**Current Issues:**
- Line 45: `ui.add_space(ui.available_width() - 100.0);` - settings button push
- Complex nested horizontals without proper constraints

**Fix Strategy:**
```rust
// Use layout with main_wrap or justify
ui.with_layout(egui::Layout::left_to_right(egui::Align::Center).with_main_justify(true), |ui| {
    ui.label("← Back");
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.button("⚙");
    });
});
```

### 4. `src/ui/settings.rs`

**Current Issues:**
- Line 105, 290, 323: `ui.add_space(ui.available_width() - XXX)` - push elements right
- No consistent row layout

**Fix Strategy:**
Use a helper for settings rows:
```rust
fn setting_row(ui: &mut Ui, label: &str, content: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            content(ui);
        });
    });
}
```

### 5. `src/app.rs`

**Current Issues:**
- Line 346-355: Content rect calculation using cursor position
- Uses deprecated `child_ui`

**Fix Strategy:**
```rust
// Use allocate_ui_at_rect or proper layout nesting
ui.allocate_ui_at_rect(content_rect, |ui| {
    // Content
});
```

---

## Implementation Checklist

### Phase 1: Core Layout Fixes
- [ ] Replace all manual `add_space((available - X) / 2)` with proper centering
- [ ] Use `Layout::right_to_left` for right-aligned elements
- [ ] Use `with_layout` instead of manual spacing

### Phase 2: Component Fixes
- [ ] Timer view: Use `vertical_centered` for controls
- [ ] Dashboard: Use grid/columns for stat cards
- [ ] Settings: Create reusable `setting_row` helper

### Phase 3: Consistency
- [ ] Replace deprecated `child_ui` with `new_child` or `allocate_ui_at_rect`
- [ ] Add consistent spacing using theme constants only
- [ ] Remove all magic numbers

---

## Specific Code Changes

### timer_view.rs - Control Buttons
```rust
// BEFORE (line 74-101):
ui.horizontal(|ui| {
    ui.add_space((ui.available_width() - 120.0) / 2.0);
    // buttons...
});

// AFTER:
ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    ui.horizontal(|ui| {
        if IconButton::new(play_icon)...
        ui.add_space(theme.spacing_sm);
        if IconButton::new('⏭')...
    });
});
```

### timer_view.rs - Session Dots
```rust
// BEFORE (line 143-144):
ui.horizontal(|ui| {
    ui.add_space((ui.available_width() - (total as f32 * 16.0)) / 2.0);

// AFTER:
ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
    ui.horizontal(|ui| {
        for i in 0..total {
            // dots...
        }
    });
});
```

### dashboard.rs - Header
```rust
// BEFORE (line 37-53):
ui.horizontal(|ui| {
    if ui.add(egui::Button::new("← Back")...
    ui.add_space(ui.available_width() - 100.0);
    if ui.add(egui::Button::new("⚙")...
});

// AFTER:
ui.horizontal(|ui| {
    if ui.add(egui::Button::new("← Back").frame(false)).clicked() {
        action = Some(DashboardAction::Back);
    }
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        if ui.add(egui::Button::new("⚙").frame(false)).clicked() {
            action = Some(DashboardAction::OpenSettings);
        }
    });
});
```

### settings.rs - Row Layout Helper
```rust
// Add helper function:
fn settings_row(ui: &mut Ui, theme: &Theme, label: &str, add_control: impl FnOnce(&mut Ui)) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).color(theme.text_secondary));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            add_control(ui);
        });
    });
    ui.add_space(theme.spacing_xs);
}

// Usage:
settings_row(ui, theme, "Volume", |ui| {
    ui.label(egui::RichText::new(format!("{}%", self.volume as u32)).color(theme.text_muted));
    ui.add(egui::Slider::new(&mut self.volume, 0.0..=100.0).show_value(false));
});
```

---

## Constants to Define (theme.rs)

```rust
// Layout constants
pub const TIMER_RADIUS: f32 = 100.0;
pub const BUTTON_HEIGHT: f32 = 40.0;
pub const ICON_BUTTON_SIZE: f32 = 56.0;
pub const ICON_BUTTON_SMALL: f32 = 48.0;
pub const CARD_MIN_WIDTH: f32 = 140.0;
pub const STAT_CARD_SIZE: f32 = 80.0;
pub const CHART_HEIGHT: f32 = 120.0;
```

---

## Summary

The main fixes are:
1. **Never use `ui.add_space((available - X) / 2)`** - use `Layout::top_down(Align::Center)`
2. **Never push with `add_space(available - X)`** - use `Layout::right_to_left`
3. **Use egui's layout system** instead of manual positioning
4. **Define size constants** in theme for consistency
5. **Replace deprecated methods** (`child_ui` → `new_child`)
