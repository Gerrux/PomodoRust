//! UI components and views

pub mod animations;
pub mod components;
pub mod dashboard;
pub mod settings;
pub mod theme;
pub mod timer_view;
pub mod titlebar;

pub use animations::AnimationState;
pub use dashboard::DashboardView;
pub use settings::SettingsView;
pub use theme::Theme;
pub use timer_view::TimerView;
pub use titlebar::TitleBar;
