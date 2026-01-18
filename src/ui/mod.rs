//! UI components and views
//!
//! This module provides all UI-related functionality:
//! - `animations`: Animation state and easing functions
//! - `components`: Reusable UI components (buttons, progress, etc.)
//! - `stats`: Statistics view
//! - `layout`: Layout constants and responsive sizing utilities
//! - `settings`: Settings panel view
//! - `theme`: Design system with colors, spacing, and fonts
//! - `timer_view`: Main timer view
//! - `titlebar`: Custom window title bar

pub mod animations;
pub mod components;
pub mod layout;
pub mod settings;
pub mod stats;
pub mod theme;
pub mod timer_view;
pub mod titlebar;

pub use animations::AnimationState;
pub use layout::{responsive, ResponsiveSize};
pub use settings::SettingsView;
pub use stats::StatsView;
pub use theme::Theme;
pub use timer_view::TimerView;
pub use titlebar::TitleBar;
