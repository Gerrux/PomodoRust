//! Internationalization (i18n) support
//!
//! Provides zero-cost localization using static translation structs.
//! All UI strings are accessed via `tr()` which returns `&'static Tr`.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};

// ── Language enum ─────────────────────────────────────────────────

/// User-facing language preference (stored in config)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    Auto,
    English,
    Russian,
}

impl Language {
    pub fn all() -> &'static [Language] {
        &[Language::Auto, Language::English, Language::Russian]
    }

    /// Display name (always in the target language for recognition)
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Auto => "System",
            Language::English => "English",
            Language::Russian => "Русский",
        }
    }

    /// Resolve Auto to a concrete language
    pub fn resolve(&self) -> ResolvedLang {
        match self {
            Language::English => ResolvedLang::En,
            Language::Russian => ResolvedLang::Ru,
            Language::Auto => detect_system_language(),
        }
    }
}

/// Concrete resolved language (no Auto)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedLang {
    En = 0,
    Ru = 1,
}

// ── Global state ──────────────────────────────────────────────────

static CURRENT_LANG: AtomicU8 = AtomicU8::new(0);

/// Get current translations
pub fn tr() -> &'static Tr {
    match CURRENT_LANG.load(Ordering::Relaxed) {
        1 => &RU,
        _ => &EN,
    }
}

/// Set the active language
pub fn set_language(lang: Language) {
    let resolved = lang.resolve();
    CURRENT_LANG.store(resolved as u8, Ordering::Relaxed);
}

/// Detect system language from OS locale
pub fn detect_system_language() -> ResolvedLang {
    let locale = sys_locale::get_locale().unwrap_or_default();
    if locale.starts_with("ru") {
        ResolvedLang::Ru
    } else {
        ResolvedLang::En
    }
}

// ── Translation structs ───────────────────────────────────────────

pub struct Tr {
    pub nav: NavTr,
    pub timer: TimerTr,
    pub settings: SettingsTr,
    pub stats: StatsTr,
    pub todo: TodoTr,
    pub queue: QueueTr,
    pub tray: TrayTr,
    pub notif: NotifTr,
    pub common: CommonTr,
}

pub struct NavTr {
    pub settings: &'static str,
    pub statistics: &'static str,
    pub tasks: &'static str,
    pub queue: &'static str,
}

pub struct TimerTr {
    pub focus: &'static str,
    pub short_break: &'static str,
    pub long_break: &'static str,
    pub pause: &'static str,
    pub start: &'static str,
    pub skip: &'static str,
    pub session: &'static str,
}

pub struct SettingsTr {
    pub title: &'static str,
    pub timer: &'static str,
    pub focus_duration: &'static str,
    pub short_break: &'static str,
    pub long_break: &'static str,
    pub sessions_before_long: &'static str,
    pub auto_start_breaks: &'static str,
    pub auto_start_pomodoros: &'static str,
    pub sounds: &'static str,
    pub volume: &'static str,
    pub sound: &'static str,
    pub tick_sound: &'static str,
    pub appearance: &'static str,
    pub theme: &'static str,
    pub accent_color: &'static str,
    pub retro_themes: &'static str,
    pub window_opacity: &'static str,
    pub accessibility: &'static str,
    pub high_contrast: &'static str,
    pub reduced_motion: &'static str,
    pub system: &'static str,
    pub start_with_windows: &'static str,
    pub always_on_top: &'static str,
    pub goals: &'static str,
    pub daily_goal: &'static str,
    pub pomodoros: &'static str,
    pub notify_goal_reached: &'static str,
    pub global_hotkeys: &'static str,
    pub enable_hotkeys: &'static str,
    pub toggle_start_pause: &'static str,
    pub skip_session: &'static str,
    pub reset_timer: &'static str,
    pub restart_for_hotkeys: &'static str,
    pub command_line: &'static str,
    pub control_from_terminal: &'static str,
    pub copy_path_command: &'static str,
    pub copy_path_tooltip: &'static str,
    pub run_copied_command: &'static str,
    pub presets: &'static str,
    pub reset_to_defaults: &'static str,
    pub language: &'static str,
    pub language_restart_hint: &'static str,
    pub test_sound: &'static str,
    // Theme mode names
    pub theme_system: &'static str,
    pub theme_light: &'static str,
    pub theme_dark: &'static str,
    // Color names
    pub color_blue: &'static str,
    pub color_purple: &'static str,
    pub color_rose: &'static str,
    pub color_emerald: &'static str,
    pub color_amber: &'static str,
    pub color_cyan: &'static str,
    pub color_retro_amber: &'static str,
    // Preset names
    pub preset_classic: &'static str,
    pub preset_short: &'static str,
    pub preset_long: &'static str,
    pub preset_applied: &'static str, // "{} preset" / "Пресет {}"
}

pub struct StatsTr {
    pub title: &'static str,
    pub current_session: &'static str,
    pub statistics: &'static str,
    pub week_activity: &'static str,
    pub quick_start: &'static str,
    pub this_week: &'static str,
    pub today: &'static str,
    pub daily_goal: &'static str,
    pub current_streak: &'static str,
    pub total: &'static str,
    pub all_time: &'static str,
    pub best_streak: &'static str,
    pub total_sessions: &'static str,
    pub running: &'static str,
    pub completed: &'static str,
    pub paused: &'static str,
    pub done: &'static str,
    pub days: &'static str,
    pub hours: &'static str,
    pub sessions: &'static str,
    pub focus_time: &'static str,
    pub todays_focus: &'static str,
    pub goal_reached: &'static str,
    pub best: &'static str,
    pub min_break: &'static str,
    pub min_focus: &'static str,
    pub min_deep_work: &'static str,
    pub export_as: &'static str,
    pub total_label: &'static str,
    pub reset_title: &'static str,
    pub reset_confirm: &'static str,
    pub completed_label: &'static str,
    pub reset_all_hover: &'static str,
    pub undo_last_hover: &'static str,
    pub export_hover: &'static str,
    // Days of week
    pub mon: &'static str,
    pub tue: &'static str,
    pub wed: &'static str,
    pub thu: &'static str,
    pub fri: &'static str,
    pub sat: &'static str,
    pub sun: &'static str,
}

pub struct TodoTr {
    pub completed_label: &'static str,
    pub miscellaneous: &'static str,
    pub no_tasks: &'static str,
    pub create_first_task: &'static str,
    pub new_task_hint: &'static str,
    pub name_hint: &'static str,
    pub new_workspace: &'static str,
    pub project: &'static str,
    pub project_name_hint: &'static str,
    pub rename: &'static str,
    pub delete: &'static str,
    pub delete_project: &'static str,
    pub delete_project_n: &'static str, // "Delete project ({} tasks)"
    pub edit: &'static str,
    pub add_to_queue: &'static str,
    pub move_to: &'static str,
    pub no_project: &'static str,
    pub priority: &'static str,
    pub title_hint: &'static str,
    pub description_hint: &'static str,
    pub save: &'static str,
    // Priority labels
    pub priority_none: &'static str,
    pub priority_low: &'static str,
    pub priority_medium: &'static str,
    pub priority_high: &'static str,
    pub priority_urgent: &'static str,
    pub tasks_default_workspace: &'static str,
}

pub struct QueueTr {
    pub title: &'static str,
    pub empty: &'static str,
    pub empty_hint: &'static str,
    pub clear: &'static str,
}

pub struct TrayTr {
    pub focus: &'static str,
    pub short_break: &'static str,
    pub long_break: &'static str,
    pub pause: &'static str,
    pub continue_: &'static str,
    pub start: &'static str,
    pub ready: &'static str,
    pub close_app: &'static str,
    pub what_to_do: &'static str,
    pub minimize_to_tray: &'static str,
    pub quit: &'static str,
    pub show_window: &'static str,
}

pub struct NotifTr {
    pub focus_complete: &'static str,
    pub time_for_break: &'static str,
    pub break_over: &'static str,
    pub ready_to_focus: &'static str,
    pub long_break_over: &'static str,
    pub back_to_work: &'static str,
    pub daily_goal_reached: &'static str,
    pub stats_reset: &'static str,
    pub stats_cleared: &'static str,
    pub session_undone: &'static str,
    pub session_removed: &'static str,
    pub export_complete: &'static str,
    pub export_failed: &'static str,
    pub export_statistics: &'static str,
    pub defaults_restored: &'static str,
    pub settings_saved: &'static str,
}

pub struct CommonTr {
    pub cancel: &'static str,
    pub reset: &'static str,
    pub min: &'static str,
    pub pin_window: &'static str,
    pub unpin_window: &'static str,
}

// ── English translations ──────────────────────────────────────────

static EN: Tr = Tr {
    nav: NavTr {
        settings: "Settings",
        statistics: "Statistics",
        tasks: "Tasks",
        queue: "Queue",
    },
    timer: TimerTr {
        focus: "FOCUS",
        short_break: "SHORT BREAK",
        long_break: "LONG BREAK",
        pause: "PAUSE",
        start: "START",
        skip: "SKIP",
        session: "Session",
    },
    settings: SettingsTr {
        title: "Settings",
        timer: "Timer",
        focus_duration: "Focus Duration",
        short_break: "Short Break",
        long_break: "Long Break",
        sessions_before_long: "Sessions before long break",
        auto_start_breaks: "Auto-start breaks",
        auto_start_pomodoros: "Auto-start pomodoros",
        sounds: "Sounds",
        volume: "Volume",
        sound: "Sound",
        tick_sound: "Tick sound",
        appearance: "Appearance",
        theme: "Theme",
        accent_color: "Accent Color",
        retro_themes: "Retro Themes",
        window_opacity: "Window Opacity",
        accessibility: "Accessibility",
        high_contrast: "High contrast mode",
        reduced_motion: "Reduced motion",
        system: "System",
        start_with_windows: "Start with Windows",
        always_on_top: "Always on top",
        goals: "Goals",
        daily_goal: "Daily goal",
        pomodoros: "pomodoros",
        notify_goal_reached: "Notify when goal reached",
        global_hotkeys: "Global Hotkeys",
        enable_hotkeys: "Enable global hotkeys",
        toggle_start_pause: "Toggle (start/pause)",
        skip_session: "Skip session",
        reset_timer: "Reset timer",
        restart_for_hotkeys: "Restart app to apply hotkey changes",
        command_line: "Command Line",
        control_from_terminal: "Control timer from terminal:",
        copy_path_command: "Copy PATH command",
        copy_path_tooltip: "Copy PowerShell command to add pomodorust to PATH",
        run_copied_command: "Run copied command in PowerShell, then restart terminal",
        presets: "Presets",
        reset_to_defaults: "Reset to Defaults",
        language: "Language",
        language_restart_hint: "",
        test_sound: "Test sound",
        theme_system: "System",
        theme_light: "Light",
        theme_dark: "Dark",
        color_blue: "Blue",
        color_purple: "Purple",
        color_rose: "Rose",
        color_emerald: "Emerald",
        color_amber: "Amber",
        color_cyan: "Cyan",
        color_retro_amber: "Retro Amber",
        preset_classic: "Classic",
        preset_short: "Short",
        preset_long: "Long",
        preset_applied: "preset",
    },
    stats: StatsTr {
        title: "Statistics",
        current_session: "Current Session",
        statistics: "Statistics",
        week_activity: "Week Activity",
        quick_start: "Quick Start",
        this_week: "This Week",
        today: "Today",
        daily_goal: "Daily Goal",
        current_streak: "Current Streak",
        total: "Total",
        all_time: "All Time",
        best_streak: "Best Streak",
        total_sessions: "Total Sessions",
        running: "Running",
        completed: "Completed",
        paused: "Paused",
        done: "Done!",
        days: "days",
        hours: "hours",
        sessions: "sessions",
        focus_time: "focus time",
        todays_focus: "Today's Focus",
        goal_reached: "Goal reached!",
        best: "Best",
        min_break: "5 min break",
        min_focus: "25 min focus",
        min_deep_work: "50 min deep work",
        export_as: "Export as",
        total_label: "total",
        reset_title: "Reset Statistics?",
        reset_confirm: "This will permanently delete all\nsession history and statistics.",
        completed_label: "completed",
        reset_all_hover: "Reset all statistics",
        undo_last_hover: "Undo last session",
        export_hover: "Export statistics",
        mon: "Mon",
        tue: "Tue",
        wed: "Wed",
        thu: "Thu",
        fri: "Fri",
        sat: "Sat",
        sun: "Sun",
    },
    todo: TodoTr {
        completed_label: "Completed",
        miscellaneous: "Miscellaneous",
        no_tasks: "No tasks",
        create_first_task: "Create your first task below",
        new_task_hint: "New task...",
        name_hint: "Name...",
        new_workspace: "New workspace",
        project: "Project",
        project_name_hint: "Project name...",
        rename: "Rename",
        delete: "Delete",
        delete_project: "Delete project",
        delete_project_n: "Delete project ({} tasks)",
        edit: "Edit",
        add_to_queue: "Add to queue",
        move_to: "Move to...",
        no_project: "No project",
        priority: "Priority",
        title_hint: "Title...",
        description_hint: "Description (markdown)...",
        save: "Save",
        priority_none: "\u{2014}",
        priority_low: "Low",
        priority_medium: "Medium",
        priority_high: "High",
        priority_urgent: "Urgent",
        tasks_default_workspace: "Tasks",
    },
    queue: QueueTr {
        title: "Queue",
        empty: "Queue is empty",
        empty_hint: "Add tasks via \u{22EE} menu in the task list",
        clear: "Clear queue",
    },
    tray: TrayTr {
        focus: "Focus",
        short_break: "Short Break",
        long_break: "Long Break",
        pause: "Pause",
        continue_: "Continue",
        start: "Start",
        ready: "Ready",
        close_app: "Close application?",
        what_to_do: "What would you like to do?",
        minimize_to_tray: "  Minimize to tray  ",
        quit: "  Quit  ",
        show_window: "Show",
    },
    notif: NotifTr {
        focus_complete: "Focus Complete!",
        time_for_break: "Time for a break.",
        break_over: "Break Over",
        ready_to_focus: "Ready to focus again?",
        long_break_over: "Long Break Over",
        back_to_work: "Let's get back to work!",
        daily_goal_reached: "Daily Goal Reached!",
        stats_reset: "Statistics Reset",
        stats_cleared: "All statistics have been cleared.",
        session_undone: "Session Undone",
        session_removed: "Last pomodoro session has been removed from statistics.",
        export_complete: "Export Complete",
        export_failed: "Export Failed",
        export_statistics: "Export Statistics",
        defaults_restored: "Defaults restored",
        settings_saved: "Settings saved",
    },
    common: CommonTr {
        cancel: "Cancel",
        reset: "Reset",
        min: "min",
        pin_window: "Pin window (always on top)",
        unpin_window: "Unpin window (disable always on top)",
    },
};

// ── Russian translations ──────────────────────────────────────────

static RU: Tr = Tr {
    nav: NavTr {
        settings: "Настройки",
        statistics: "Статистика",
        tasks: "Задачи",
        queue: "Очередь",
    },
    timer: TimerTr {
        focus: "ФОКУС",
        short_break: "ПЕРЕРЫВ",
        long_break: "ДЛИННЫЙ ПЕРЕРЫВ",
        pause: "ПАУЗА",
        start: "СТАРТ",
        skip: "ДАЛЕЕ",
        session: "Сессия",
    },
    settings: SettingsTr {
        title: "Настройки",
        timer: "Таймер",
        focus_duration: "Длительность фокуса",
        short_break: "Короткий перерыв",
        long_break: "Длинный перерыв",
        sessions_before_long: "Сессий до длинного перерыва",
        auto_start_breaks: "Автозапуск перерывов",
        auto_start_pomodoros: "Автозапуск помодоро",
        sounds: "Звуки",
        volume: "Громкость",
        sound: "Звук",
        tick_sound: "Звук тиканья",
        appearance: "Внешний вид",
        theme: "Тема",
        accent_color: "Акцентный цвет",
        retro_themes: "Ретро темы",
        window_opacity: "Прозрачность окна",
        accessibility: "Доступность",
        high_contrast: "Высокий контраст",
        reduced_motion: "Уменьшить анимации",
        system: "Система",
        start_with_windows: "Запуск с Windows",
        always_on_top: "Поверх всех окон",
        goals: "Цели",
        daily_goal: "Дневная цель",
        pomodoros: "помодоро",
        notify_goal_reached: "Уведомлять о достижении цели",
        global_hotkeys: "Горячие клавиши",
        enable_hotkeys: "Включить горячие клавиши",
        toggle_start_pause: "Старт/пауза",
        skip_session: "Пропустить сессию",
        reset_timer: "Сбросить таймер",
        restart_for_hotkeys: "Перезапустите приложение для применения",
        command_line: "Командная строка",
        control_from_terminal: "Управление таймером из терминала:",
        copy_path_command: "Копировать команду PATH",
        copy_path_tooltip: "Копировать команду для добавления в PATH",
        run_copied_command: "Выполните команду в PowerShell, затем перезапустите терминал",
        presets: "Пресеты",
        reset_to_defaults: "Сбросить по умолчанию",
        language: "Язык",
        language_restart_hint: "",
        test_sound: "Тест звука",
        theme_system: "Системная",
        theme_light: "Светлая",
        theme_dark: "Тёмная",
        color_blue: "Синий",
        color_purple: "Фиолетовый",
        color_rose: "Розовый",
        color_emerald: "Изумрудный",
        color_amber: "Янтарный",
        color_cyan: "Голубой",
        color_retro_amber: "Ретро янтарный",
        preset_classic: "Классический",
        preset_short: "Короткий",
        preset_long: "Длинный",
        preset_applied: "пресет",
    },
    stats: StatsTr {
        title: "Статистика",
        current_session: "Текущая сессия",
        statistics: "Статистика",
        week_activity: "Активность за неделю",
        quick_start: "Быстрый старт",
        this_week: "Эта неделя",
        today: "Сегодня",
        daily_goal: "Дневная цель",
        current_streak: "Текущая серия",
        total: "Всего",
        all_time: "За всё время",
        best_streak: "Лучшая серия",
        total_sessions: "Всего сессий",
        running: "Активно",
        completed: "Завершено",
        paused: "Пауза",
        done: "Готово!",
        days: "дн.",
        hours: "часов",
        sessions: "сессий",
        focus_time: "время фокуса",
        todays_focus: "Фокус сегодня",
        goal_reached: "Цель достигнута!",
        best: "Лучший",
        min_break: "5 мин перерыв",
        min_focus: "25 мин фокус",
        min_deep_work: "50 мин глубокая работа",
        export_as: "Экспорт в",
        total_label: "всего",
        reset_title: "Сбросить статистику?",
        reset_confirm: "Это безвозвратно удалит всю\nисторию сессий и статистику.",
        completed_label: "завершено",
        reset_all_hover: "Сбросить всю статистику",
        undo_last_hover: "Отменить последнюю сессию",
        export_hover: "Экспорт статистики",
        mon: "Пн",
        tue: "Вт",
        wed: "Ср",
        thu: "Чт",
        fri: "Пт",
        sat: "Сб",
        sun: "Вс",
    },
    todo: TodoTr {
        completed_label: "Завершённые",
        miscellaneous: "Разное",
        no_tasks: "Нет задач",
        create_first_task: "Создайте первую задачу ниже",
        new_task_hint: "Новая задача...",
        name_hint: "Название...",
        new_workspace: "Новый workspace",
        project: "Проект",
        project_name_hint: "Название проекта...",
        rename: "Переименовать",
        delete: "Удалить",
        delete_project: "Удалить проект",
        delete_project_n: "Удалить проект ({} задач)",
        edit: "Редактировать",
        add_to_queue: "В очередь",
        move_to: "Переместить в...",
        no_project: "Без проекта",
        priority: "Приоритет",
        title_hint: "Заголовок...",
        description_hint: "Описание (markdown)...",
        save: "Сохранить",
        priority_none: "\u{2014}",
        priority_low: "Низкий",
        priority_medium: "Средний",
        priority_high: "Высокий",
        priority_urgent: "Срочный",
        tasks_default_workspace: "Задачи",
    },
    queue: QueueTr {
        title: "Очередь",
        empty: "Очередь пуста",
        empty_hint: "Добавляйте задачи через меню \u{22EE} в списке задач",
        clear: "Очистить очередь",
    },
    tray: TrayTr {
        focus: "Фокус",
        short_break: "Короткий перерыв",
        long_break: "Длинный перерыв",
        pause: "Пауза",
        continue_: "Продолжить",
        start: "Старт",
        ready: "Готов",
        close_app: "Закрыть приложение?",
        what_to_do: "Что вы хотите сделать?",
        minimize_to_tray: "  Свернуть в трей  ",
        quit: "  Выход  ",
        show_window: "Показать окно",
    },
    notif: NotifTr {
        focus_complete: "Фокус завершён!",
        time_for_break: "Время для перерыва.",
        break_over: "Перерыв окончен",
        ready_to_focus: "Готовы снова сфокусироваться?",
        long_break_over: "Длинный перерыв окончен",
        back_to_work: "Пора вернуться к работе!",
        daily_goal_reached: "Дневная цель достигнута!",
        stats_reset: "Статистика сброшена",
        stats_cleared: "Вся статистика была очищена.",
        session_undone: "Сессия отменена",
        session_removed: "Последняя сессия удалена из статистики.",
        export_complete: "Экспорт завершён",
        export_failed: "Ошибка экспорта",
        export_statistics: "Экспорт статистики",
        defaults_restored: "Настройки по умолчанию восстановлены",
        settings_saved: "Настройки сохранены",
    },
    common: CommonTr {
        cancel: "Отмена",
        reset: "Сбросить",
        min: "мин",
        pin_window: "Закрепить окно (поверх всех)",
        unpin_window: "Открепить окно (снять поверх всех)",
    },
};

// ── Helper methods ────────────────────────────────────────────────

impl Tr {
    /// Get session type label
    pub fn session_label(&self, st: crate::core::SessionType) -> &'static str {
        match st {
            crate::core::SessionType::Work => self.timer.focus,
            crate::core::SessionType::ShortBreak => self.timer.short_break,
            crate::core::SessionType::LongBreak => self.timer.long_break,
        }
    }

    /// Get priority label
    pub fn priority_label(&self, p: crate::data::todo::Priority) -> &'static str {
        match p {
            crate::data::todo::Priority::None => self.todo.priority_none,
            crate::data::todo::Priority::Low => self.todo.priority_low,
            crate::data::todo::Priority::Medium => self.todo.priority_medium,
            crate::data::todo::Priority::High => self.todo.priority_high,
            crate::data::todo::Priority::Urgent => self.todo.priority_urgent,
        }
    }

    /// Get localized theme mode name
    pub fn theme_name(&self, mode: crate::ui::theme::ThemeMode) -> &'static str {
        use crate::ui::theme::ThemeMode;
        match mode {
            ThemeMode::System => self.settings.theme_system,
            ThemeMode::Light => self.settings.theme_light,
            ThemeMode::Dark => self.settings.theme_dark,
            // Brand names stay the same in all languages
            ThemeMode::CatppuccinLatte => "Catppuccin Latte",
            ThemeMode::CatppuccinFrappe => "Catppuccin Frappé",
            ThemeMode::CatppuccinMacchiato => "Catppuccin Macchiato",
            ThemeMode::CatppuccinMocha => "Catppuccin Mocha",
        }
    }

    /// Get localized accent color name
    pub fn accent_name(&self, color: crate::ui::theme::AccentColor) -> &'static str {
        use crate::ui::theme::AccentColor;
        match color {
            AccentColor::Blue => self.settings.color_blue,
            AccentColor::Purple => self.settings.color_purple,
            AccentColor::Rose => self.settings.color_rose,
            AccentColor::Emerald => self.settings.color_emerald,
            AccentColor::Amber => self.settings.color_amber,
            AccentColor::Cyan => self.settings.color_cyan,
            // Brand/proper names
            AccentColor::Matrix => "Matrix",
            AccentColor::RetroAmber => self.settings.color_retro_amber,
            AccentColor::Synthwave => "Synthwave",
        }
    }

    /// Get days of week array
    pub fn days_of_week(&self) -> [&'static str; 7] {
        [
            self.stats.mon,
            self.stats.tue,
            self.stats.wed,
            self.stats.thu,
            self.stats.fri,
            self.stats.sat,
            self.stats.sun,
        ]
    }
}
