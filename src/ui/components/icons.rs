//! Phosphor-based icon system for egui
//!
//! Uses egui-phosphor font glyphs instead of hand-drawn painter geometry.
//! All icons are rendered as text with configurable size and color.

use egui::{vec2, Color32, Pos2, Rect, Ui};

/// Icon identifier enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    // Playback controls
    Play,
    Pause,
    Stop,
    SkipForward,
    SkipBack,
    RotateCcw,

    // Navigation
    ChevronLeft,
    ChevronRight,
    ChevronUp,
    ChevronDown,
    ArrowLeft,

    // UI controls
    X,
    Plus,
    Minus,
    Check,

    // Window controls
    Minimize,
    Maximize,
    Restore,

    // App-specific
    Settings,
    LayoutDashboard,
    Timer,
    Clock,
    Bell,
    BellOff,
    Volume2,
    VolumeX,

    // Status
    Coffee,
    Target,
    Flame,
    Calendar,
    BarChart3,
    TrendingUp,

    // General
    Sun,
    Moon,
    Palette,
    Zap,
    Download,
    Pin,
    PinOff,
    Trash,

    // Todo
    ListTodo,
    ClipboardList,
    MoreVertical,
    Square,
    CheckSquare,
    CirclePlus,
    GripVertical,
}

impl Icon {
    /// Get the phosphor icon glyph string for this icon
    fn glyph(self) -> &'static str {
        use egui_phosphor::regular::*;
        match self {
            // Playback
            Icon::Play => PLAY,
            Icon::Pause => PAUSE,
            Icon::Stop => STOP,
            Icon::SkipForward => SKIP_FORWARD,
            Icon::SkipBack => SKIP_BACK,
            Icon::RotateCcw => ARROW_COUNTER_CLOCKWISE,

            // Navigation
            Icon::ChevronLeft => CARET_LEFT,
            Icon::ChevronRight => CARET_RIGHT,
            Icon::ChevronUp => CARET_UP,
            Icon::ChevronDown => CARET_DOWN,
            Icon::ArrowLeft => ARROW_LEFT,

            // UI controls
            Icon::X => X,
            Icon::Plus => PLUS,
            Icon::Minus => MINUS,
            Icon::Check => CHECK,

            // Window controls
            Icon::Minimize => MINUS,
            Icon::Maximize => SQUARE,
            Icon::Restore => SQUARES_FOUR,

            // App-specific
            Icon::Settings => GEAR,
            Icon::LayoutDashboard => LAYOUT,
            Icon::Timer => TIMER,
            Icon::Clock => CLOCK,
            Icon::Bell => BELL,
            Icon::BellOff => BELL_SLASH,
            Icon::Volume2 => SPEAKER_HIGH,
            Icon::VolumeX => SPEAKER_X,

            // Status
            Icon::Coffee => COFFEE,
            Icon::Target => TARGET,
            Icon::Flame => FIRE,
            Icon::Calendar => CALENDAR,
            Icon::BarChart3 => CHART_BAR,
            Icon::TrendingUp => TREND_UP,

            // General
            Icon::Sun => SUN,
            Icon::Moon => MOON,
            Icon::Palette => PALETTE,
            Icon::Zap => LIGHTNING,
            Icon::Download => DOWNLOAD,
            Icon::Pin => PUSH_PIN,
            Icon::PinOff => PUSH_PIN_SLASH,
            Icon::Trash => TRASH,

            // Todo
            Icon::ListTodo => LIST_CHECKS,
            Icon::ClipboardList => CLIPBOARD_TEXT,
            Icon::MoreVertical => DOTS_THREE_VERTICAL,
            Icon::Square => SQUARE,
            Icon::CheckSquare => CHECK_SQUARE,
            Icon::CirclePlus => PLUS_CIRCLE,
            Icon::GripVertical => DOTS_SIX_VERTICAL,
        }
    }
}

/// Draw an icon at the specified rectangle
pub fn draw_icon(ui: &mut Ui, icon: Icon, rect: Rect, color: Color32) {
    let size = rect.width().min(rect.height());
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        icon.glyph(),
        egui::FontId::proportional(size),
        color,
    );
}

/// Draw an icon centered at a position with given size
pub fn draw_icon_at(ui: &mut Ui, icon: Icon, center: Pos2, size: f32, color: Color32) {
    let rect = Rect::from_center_size(center, vec2(size, size));
    draw_icon(ui, icon, rect, color);
}

/// Helper struct for drawing icons with builder pattern
pub struct IconPainter {
    icon: Icon,
    size: f32,
    color: Color32,
}

impl IconPainter {
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: 24.0,
            color: Color32::WHITE,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn paint(self, ui: &mut Ui, center: Pos2) {
        draw_icon_at(ui, self.icon, center, self.size, self.color);
    }

    pub fn paint_rect(self, ui: &mut Ui, rect: Rect) {
        draw_icon(ui, self.icon, rect, self.color);
    }
}
