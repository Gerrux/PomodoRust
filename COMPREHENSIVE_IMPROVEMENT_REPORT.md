# POMODORUST: Комплексный отчёт по улучшениям
## Для бизнес-аналитиков и стратегического планирования

> **Версия:** 1.0
> **Дата:** 2026-01-19
> **Автор:** AI Business Analysis
> **Цель:** Создание топ Pomodoro-таймера, востребованного и уважаемого на GitHub

---

# СОДЕРЖАНИЕ

1. [Executive Summary](#1-executive-summary)
2. [Текущее состояние продукта](#2-текущее-состояние-продукта)
3. [Анализ конкурентного ландшафта](#3-анализ-конкурентного-ландшафта)
4. [SWOT-анализ](#4-swot-анализ)
5. [Gap Analysis: Что не хватает](#5-gap-analysis-что-не-хватает)
6. [Матрица улучшений по категориям](#6-матрица-улучшений-по-категориям)
7. [Приоритизация: Impact vs Effort](#7-приоритизация-impact-vs-effort)
8. [Детальное описание улучшений](#8-детальное-описание-улучшений)
9. [Стратегическая дорожная карта](#9-стратегическая-дорожная-карта)
10. [Метрики успеха](#10-метрики-успеха)
11. [Риски и митигация](#11-риски-и-митигация)
12. [Рекомендации](#12-рекомендации)
13. [Приложения](#13-приложения)

---

# 1. Executive Summary

## Ключевой вывод

**Pomodorust** имеет отличный технический фундамент и уникальную нишу "Developer's Pomodoro". Для достижения топ-позиции на GitHub и массового принятия необходимо:

1. **Завершить недоделанные функции** (Quick Wins) — 5-7 дней
2. **Добавить CLI интерфейс** — ключевой дифференциатор
3. **Реализовать Task List** — основной функциональный gap
4. **Усилить Open Source инфраструктуру** — для привлечения контрибьюторов

## Текущий статус

| Метрика | Значение |
|---------|----------|
| Строк кода | ~9,500 Rust |
| Платформы | Windows (полная), macOS/Linux (базовая) |
| Темы | 9 цветовых схем + 3 ретро |
| База данных | SQLite со статистикой |
| Звуки | 6 встроенных (выбор в настройках) |
| CLI | ✅ Полный (start/pause/status/stats) |
| Global Hotkeys | ✅ Ctrl+Alt+Space/S/R |
| Размер бинарника | ~15 MB (единый GUI + CLI) |

## Стратегическая позиция

```
НЕ КОНКУРИРОВАТЬ с:          ДОМИНИРОВАТЬ в:
├─ Forest (gamification)      ├─ CLI interface
├─ Pomotodo (task management) ├─ IDE integrations
└─ Be Focused (Apple native)  ├─ Developer workflows
                              └─ Retro/Terminal aesthetics
```

## ROI оценка улучшений

| Категория | Effort | Impact | ROI |
|-----------|--------|--------|-----|
| Quick Wins | Low | Medium | **Высокий** |
| CLI Interface | Medium | High | **Очень высокий** |
| Task List | Medium | High | **Высокий** |
| Achievements | Low-Medium | Medium | **Высокий** |
| Plugin System | High | High | Средний |
| Cloud Sync | Very High | Medium | Низкий |

---

# 2. Текущее состояние продукта

## 2.1 Архитектурный обзор

```
pomodorust/
├── src/
│   ├── core/           # Бизнес-логика (Timer, Session, Preset)
│   ├── data/           # Persistence (Config, Database, Statistics)
│   ├── ui/             # UI слой (egui views и components)
│   ├── platform/       # Platform-specific код
│   └── utils/          # Утилиты
├── assets/             # Звуки, иконки
└── .github/            # CI/CD workflows
```

## 2.2 Технический стек

| Компонент | Технология | Статус |
|-----------|-----------|--------|
| Язык | Rust 1.70+ | Stable |
| GUI | egui/eframe 0.29 | Modern |
| База данных | SQLite (rusqlite) | Embedded |
| Конфиг | TOML | Standard |
| Аудио | rodio | Working |
| Windows API | windows crate | Full |

## 2.3 Реализованный функционал

### Ядро таймера
- [x] Pomodoro техника (Work → Short Break → Long Break)
- [x] 4 пресета (Classic 25/5/15, Short 15/3/10, Long 50/10/30, 52/17)
- [x] Auto-start breaks и work
- [x] Session tracking с подсчётом

### UI/UX
- [x] Custom titlebar с pin/minimize/maximize/close
- [x] Always-on-top режим
- [x] Circular progress с градиентом и пульсацией
- [x] 9 цветовых тем + 3 ретро (Matrix, Amber, Synthwave)
- [x] Responsive layout (Modern/TUI в зависимости от темы)
- [x] Плавные анимации (8 easing функций)

### Звуки и уведомления
- [x] 3 встроенных звука (Soft Bell, Level Up, Digital Alert)
- [x] Volume control (0-100%)
- [x] Native Windows notifications
- [x] Window flash при завершении

### Статистика
- [x] Недельный график
- [x] Счётчики (сегодня, неделя, всего)
- [x] Стрики (текущая и максимальная серия)
- [x] Экспорт CSV/JSON

### Система
- [x] Сохранение настроек (TOML)
- [x] Сохранение позиции окна
- [x] Windows autostart (registry)

## 2.4 Недоделанные функции (Technical Debt)

| Функция | Конфиг есть | UI есть | Работает |
|---------|-------------|---------|----------|
| Window opacity | ✅ | ✅ | ✅ **DONE** |
| Compact mode | ✅ | ❌ | ❌ |
| Show in taskbar | ✅ | ❌ | ❌ |
| Tick sound | ✅ | ✅ | ✅ **DONE** |
| macOS/Linux autostart | ❌ | ❌ | ❌ |

## 2.5 Новые функции (Январь 2026)

| Функция | Описание | Статус |
|---------|----------|--------|
| CLI Interface | `pomodorust start/pause/status/stats` | ✅ **DONE** |
| IPC через TCP | Связь GUI ↔ CLI | ✅ **DONE** |
| Daily Goals | Цели на день с прогресс-баром | ✅ **DONE** |
| Global Hotkeys | Ctrl+Alt+Space (toggle), S (skip), R (reset) | ✅ **DONE** |
| Notification Sounds | 6 выбираемых звуков | ✅ **DONE** |
| Единый бинарник | GUI + CLI в одном exe | ✅ **DONE** |

---

# 3. Анализ конкурентного ландшафта

## 3.1 Карта рынка

### Сегмент: Коммерческие приложения

| Приложение | Сильные стороны | Слабости | Угроза |
|------------|-----------------|----------|--------|
| **Forest** | Gamification, iOS доминирование, реальная посадка деревьев | Не для десктопа, не классический Pomodoro | Низкая |
| **Focus To-Do** | All-in-one, кроссплатформенность, бесплатный тариф | Перегруженный UI | Средняя |
| **Pomofocus** | Простота, браузерный, бесплатный | Только веб | Низкая |
| **Session** | Apple ecosystem, блокировка приложений | Только Apple | Низкая |

### Сегмент: Open Source

| Проект | Stars | Сильные стороны | Слабости |
|--------|-------|-----------------|----------|
| **Pomotroid** | ~5,000 | Красивый UI, Electron/Vue | Тяжёлый (Electron) |
| **Pomatez** | ~4,700 | TypeScript, современный | Electron bloat |
| **Tiny Timer** | ~3,800 | Минималистичный (803KB) | Только tray, нет UI |
| **TomatoBar** | ~3,100 | Нативный macOS | Только macOS |
| **GNOME Pomodoro** | ~2,100 | GNOME интеграция | Только Linux/GNOME |

## 3.2 Конкурентное позиционирование

```
                    HIGH COMPLEXITY
                         ↑
    Pomotodo            │            Super Productivity
    Focus To-Do         │            Jira-integrated tools
                        │
GENERAL ←───────────────┼───────────────→ DEVELOPER
AUDIENCE                │                  FOCUSED
                        │
    Forest              │            Pomodorust (target)
    Pomofocus           │            TomatoBar
    Pomotroid           │            CLI tools
                        │
                         ↓
                    LOW COMPLEXITY
```

**Pomodorust Target Position**: Developer-focused, Low-Medium complexity, High customization

## 3.3 Feature Benchmark

| Фича | Forest | Focus To-Do | Pomotroid | Pomodorust |
|------|--------|-------------|-----------|------------|
| Task management | ❌ | ✅ Full | ❌ | ❌ **GAP** |
| Statistics | Basic | Full | Basic | ✅ Good |
| Themes | Limited | ✅ | ✅ | ✅ **Strong** |
| CLI interface | ❌ | ❌ | ❌ | ✅ **UNIQUE** |
| Global Hotkeys | ❌ | ❌ | ❌ | ✅ **UNIQUE** |
| IDE integration | ❌ | ❌ | ❌ | ❌ **Opportunity** |
| Gamification | ✅ Strong | ✅ | ❌ | ❌ Basic |
| Cross-platform | Mobile only | ✅ All | ✅ Desktop | ⚠️ Partial |
| Native performance | N/A | Electron | Electron | ✅ **Strong** |
| Retro themes | ❌ | ❌ | ❌ | ✅ **Unique** |
| Ambient sounds | ❌ | ✅ | ❌ | ❌ **GAP** |
| Cloud sync | ✅ | ✅ | ❌ | ❌ |

---

# 4. SWOT-анализ

## Strengths (Сильные стороны)

1. **Технический стек**
   - Rust = производительность + безопасность
   - Native binary (не Electron) = быстрый старт, малое потребление памяти
   - SQLite = надёжное хранение без сервера

2. **UI/UX**
   - 9 современных тем + 3 уникальных ретро-темы
   - Адаптивный дизайн (Modern/TUI)
   - Плавные анимации

3. **Уникальность**
   - Ретро-терминальные темы (Matrix, Amber, Synthwave) — нигде больше нет
   - "Developer's Pomodoro" позиционирование свободно

4. **Архитектура**
   - Модульная структура
   - Хорошее разделение concerns
   - Расширяемый дизайн

## Weaknesses (Слабые стороны)

1. **Функциональные gaps**
   - Нет task management
   - Нет CLI interface
   - Недоделанные фичи (opacity, compact mode)

2. **Cross-platform**
   - macOS/Linux поддержка базовая
   - Нет autostart на macOS/Linux

3. **Open Source зрелость**
   - Нет архитектурной документации
   - CI/CD требует доработки
   - Нет "good first issues"

4. **Интеграции**
   - Нет интеграций с внешними сервисами
   - Нет webhooks
   - Нет IDE плагинов

## Opportunities (Возможности)

1. **Незанятая ниша**
   - "Developer's Pomodoro" — никто не занимает
   - CLI + IDE integration = уникальное предложение

2. **Rust экосистема**
   - Растущее Rust community
   - Возможность привлечь Rust энтузиастов

3. **Тренды**
   - Рост remote work → больше потребности в фокус-инструментах
   - Privacy-first тренд → преимущество локального хранения

4. **GitHub popularity механики**
   - Ретро-темы = вирусный контент для Reddit/Twitter
   - CLI = любовь developer community

## Threats (Угрозы)

1. **Конкуренция**
   - Крупные игроки могут добавить CLI
   - Electron-based решения проще развивать

2. **Ресурсы**
   - Один/малая команда разработчиков
   - Необходимость поддержки 3 платформ

3. **Технические**
   - egui менее популярен чем Web UI frameworks
   - Сложность привлечения Rust-контрибьюторов

---

# 5. Gap Analysis: Что не хватает

## 5.1 Критические gaps (Must Have)

### Gap #1: Task Management
**Текущее:** Только таймер без задач
**Требуется:** Базовый список задач с привязкой к сессиям
**Влияние:** Главная причина ухода к конкурентам
**Сложность:** Средняя

```
User Story: Как пользователь, я хочу видеть над чем работаю
и сколько помидоров потратил на задачу
```

### ~~Gap #2: CLI Interface~~ ✅ ЗАКРЫТ
**Статус:** ✅ Реализован полностью
**Реализовано:**
- `pomodorust start [--session work|short|long]`
- `pomodorust pause/resume/toggle/stop/skip`
- `pomodorust status` — текущий статус
- `pomodorust stats [--period today|week|all]`
- `pomodorust ping` — проверка работы GUI
- IPC через TCP (localhost:47373)
- Единый бинарник (GUI + CLI)

### ~~Gap #3: Недоделанные функции~~ ⚠️ Частично закрыт
**Статус:** Частично реализован
- ✅ Window opacity — подключён к UI
- ✅ Tick sound — подключён к UI
- ❌ Compact mode — ещё не реализован
- ❌ Show in taskbar — ещё не реализован

## 5.2 Важные gaps (Should Have)

### Gap #4: macOS/Linux Parity
**Текущее:** Базовая поддержка без autostart
**Требуется:** Полная поддержка всех платформ
**Сложность:** Средняя

### Gap #5: Daily/Weekly Goals
**Текущее:** Нет целей
**Требуется:** Установка целей и отслеживание прогресса
**Сложность:** Низкая

### Gap #6: Achievement System
**Текущее:** Только стрики
**Требуется:** Достижения с badge'ами
**Сложность:** Низкая-Средняя

### Gap #7: Architecture Documentation
**Текущее:** Нет
**Требуется:** ARCHITECTURE.md для контрибьюторов
**Сложность:** Низкая

## 5.3 Желательные gaps (Nice to Have)

### Gap #8: Light Theme
**Текущее:** Только dark mode
**Требуется:** Светлая тема
**Сложность:** Средняя

### Gap #9: Ambient Sounds
**Текущее:** Только notification sounds
**Требуется:** White noise, lo-fi, природа
**Сложность:** Средняя

### Gap #10: Discord Rich Presence
**Текущее:** Нет
**Требуется:** Статус в Discord
**Сложность:** Средняя

### Gap #11: VS Code Extension
**Текущее:** Нет
**Требуется:** Status bar widget
**Сложность:** Средняя

### Gap #12: Webhooks
**Текущее:** Нет
**Требуется:** HTTP callbacks на события
**Сложность:** Средняя

---

# 6. Матрица улучшений по категориям

## 6.1 Функциональность ядра

| ID | Улучшение | Описание | Сложность | Impact | Приоритет |
|----|-----------|----------|-----------|--------|-----------|
| F01 | Task List MVP | Список задач с оценкой в помидорах | Medium | High | **P0** |
| F02 | Session Notes | Заметки к сессиям | Low | Medium | P2 |
| F03 | Task Tags | Теги для категоризации | Medium | Low | P3 |
| F04 | Multiple Timers | Несколько параллельных таймеров | High | Low | P4 |
| F05 | Strict Mode | Нельзя остановить до конца | Low | Low | P3 |
| F06 | Break Suggestions | Предложения на перерыв | Low | Medium | P3 |
| F07 | Session History | Просмотр прошлых сессий | Medium | Medium | P2 |
| F08 | Undo Session | Отмена последней сессии | Low | Medium | P2 |
| F09 | Session Recovery | Восстановление после краша | Medium | Medium | P2 |

## 6.2 Статистика и аналитика

| ID | Улучшение | Описание | Сложность | Impact | Статус |
|----|-----------|----------|-----------|--------|--------|
| S01 | Daily Goals | Цель помидоров на день | Low | High | ✅ **DONE** |
| S02 | Weekly Goals | Недельные цели | Low | Medium | ❌ Pending |
| S03 | Monthly Heatmap | GitHub-style календарь | Medium | High | P2 |
| S04 | Hourly Distribution | Когда продуктивнее | Low | Medium | P3 |
| S05 | Trend Lines | Графики трендов | Medium | Medium | P3 |
| S06 | Weekly Summary | Еженедельный отчёт | Low | Medium | P3 |
| S07 | PDF Export | Экспорт в PDF | Medium | Low | P4 |

## 6.3 Геймификация

| ID | Улучшение | Описание | Сложность | Impact | Приоритет |
|----|-----------|----------|-----------|--------|-----------|
| G01 | Achievements | 10-15 достижений | Medium | High | **P1** |
| G02 | Progress Milestones | 100, 500, 1000 помидоров | Low | Medium | P2 |
| G03 | Daily Challenges | Ежедневные испытания | Medium | Medium | P3 |
| G04 | XP/Levels | Система опыта и уровней | Medium | Medium | P3 |

## 6.4 UI/UX

| ID | Улучшение | Описание | Сложность | Impact | Приоритет |
|----|-----------|----------|-----------|--------|-----------|
| U01 | Light Mode | Светлая тема | Medium | Medium | P2 |
| U02 | Mini Mode | Только таймер, min размер | Medium | Medium | P2 |
| U03 | Fullscreen Mode | На весь экран | Low | Low | P3 |
| U04 | Confetti Celebration | При достижении цели | Low | Medium | P2 |
| U05 | Smooth Transitions | Между view'ами | Medium | Medium | P3 |
| U06 | High Contrast Mode | Accessibility | Low | Medium | **P1** |
| U07 | Reduced Motion | Отключение анимаций | Low | Medium | **P1** |
| U08 | Font Scaling | Для accessibility | Low | Medium | P2 |
| U09 | Custom Theme Editor | Создание своих тем | High | Low | P4 |

## 6.5 Звуки и уведомления

| ID | Улучшение | Описание | Сложность | Impact | Приоритет |
|----|-----------|----------|-----------|--------|-----------|
| A01 | Custom Sounds | Загрузка своих звуков | Medium | Medium | P2 |
| A02 | Ambient Sounds | White noise, природа | Medium | High | P2 |
| A03 | More Built-in Sounds | 5-10 вариантов | Low | Low | P3 |
| A04 | Volume per Sound Type | Отдельная громкость | Low | Low | P4 |

## 6.6 Интеграции

| ID | Улучшение | Описание | Сложность | Impact | Статус |
|----|-----------|----------|-----------|--------|--------|
| I01 | CLI Interface | Терминальное управление | Medium | **Very High** | ✅ **DONE** |
| I02 | Discord Rich Presence | Статус в Discord | Medium | High | ❌ Pending |
| I03 | VS Code Extension | Status bar widget | Medium | High | P2 |
| I04 | Webhooks MVP | HTTP callbacks | Medium | Medium | P2 |
| I05 | Slack Status | Автоматический статус | Medium | Medium | P3 |
| I06 | GitHub Integration | Связь с коммитами | Medium | High | P2 |
| I07 | JetBrains Plugin | IDE плагин | Medium | Medium | P3 |
| I08 | Toggl Export | Экспорт в time tracker | Low | Low | P4 |

## 6.7 Кроссплатформенность

| ID | Улучшение | Описание | Сложность | Impact | Приоритет |
|----|-----------|----------|-----------|--------|-----------|
| P01 | macOS Autostart | LaunchAgent | Medium | High | **P1** |
| P02 | Linux Autostart | systemd user service | Medium | High | **P1** |
| P03 | macOS Native Notifications | objc crate | Medium | Medium | P2 |
| P04 | Linux DBus Notifications | Улучшенные | Medium | Medium | P2 |
| P05 | Homebrew Formula | Установка на macOS | Medium | Medium | P2 |
| P06 | AUR Package | Arch Linux | Low | Medium | P2 |
| P07 | Flatpak | Linux universal | Medium | Medium | P3 |
| P08 | Menu Bar Mode | macOS menu bar app | High | Medium | P3 |

## 6.8 Архитектура и код

| ID | Улучшение | Описание | Сложность | Impact | Приоритет |
|----|-----------|----------|-----------|--------|-----------|
| C01 | Config Validation | Проверка при загрузке | Low | Medium | **P1** |
| C02 | Async Database | tokio для SQL | High | Medium | P3 |
| C03 | Plugin System MVP | Базовый API плагинов | High | High | P3 |
| C04 | Integration Tests | Тесты компонентов | Medium | Medium | P2 |
| C05 | Error Handling | Унифицировать thiserror | Low | Low | P3 |

## 6.9 Open Source инфраструктура

| ID | Улучшение | Описание | Сложность | Impact | Статус |
|----|-----------|----------|-----------|--------|--------|
| O01 | Architecture Docs | ARCHITECTURE.md | Low | High | ✅ **DONE** |
| O02 | Good First Issues | 10+ простых задач | Low | High | ✅ **DONE** |
| O03 | CI/CD All Platforms | Windows+macOS+Linux | Medium | High | ❌ Pending |
| O04 | Issue Templates | Bug/Feature templates | Low | Medium | P2 |
| O05 | PR Template | Checklist для PR | Low | Medium | P2 |
| O06 | Cargo Publish | crates.io публикация | Low | Medium | P2 |
| O07 | Release Automation | Automated releases | Medium | Medium | P2 |
| O08 | Changelog Automation | Auto CHANGELOG | Medium | Low | P3 |

## 6.10 Quick Wins (незавершённые функции)

| ID | Улучшение | Описание | Сложность | Impact | Статус |
|----|-----------|----------|-----------|--------|--------|
| Q01 | Window Opacity | Подключить к UI | Low | Medium | ✅ **DONE** |
| Q02 | Tick Sound | Подключить toggle | Low | Low | ✅ **DONE** |
| Q03 | Compact Mode | Реализовать логику | Medium | Medium | ❌ Pending |
| Q04 | Show in Taskbar | Реализовать | Low | Low | ❌ Pending |

## 6.11 Новые функции (реализовано)

| ID | Улучшение | Описание | Сложность | Impact | Статус |
|----|-----------|----------|-----------|--------|--------|
| N01 | Global Hotkeys | Ctrl+Alt+Space/S/R | Medium | High | ✅ **DONE** |
| N02 | Notification Sounds | 6 выбираемых звуков | Low | Medium | ✅ **DONE** |
| N03 | CLI PATH Setup | Кнопка копирования команды | Low | Medium | ✅ **DONE** |

---

# 7. Приоритизация: Impact vs Effort

## 7.1 Матрица приоритетов

```
                          HIGH IMPACT
                               ↑
                               │
    ┌──────────────────────────┼──────────────────────────┐
    │                          │                          │
    │  DO NOW (P0)             │  PLAN (P1)               │
    │  • Task List MVP         │  • Discord Rich Presence │
    │  • CLI Interface         │  • macOS/Linux Autostart │
    │  • Architecture Docs     │  • CI/CD All Platforms   │
    │  • Good First Issues     │  • Daily Goals           │
    │  • Window Opacity        │  • Achievements          │
    │                          │  • Tick Sound            │
LOW ←──────────────────────────┼──────────────────────────→ HIGH
EFFORT                         │                          EFFORT
    │                          │                          │
    │  FILL-INS (P2)           │  CONSIDER (P3+)          │
    │  • Monthly Heatmap       │  • Plugin System         │
    │  • Light Mode            │  • Async Database        │
    │  • VS Code Extension     │  • Custom Theme Editor   │
    │  • Confetti Animation    │  • Web WASM Version      │
    │  • Session History       │  • Cloud Sync            │
    │                          │                          │
    └──────────────────────────┼──────────────────────────┘
                               │
                               ↓
                          LOW IMPACT
```

## 7.2 Рейтинг улучшений по ROI

### Tier 1: Максимальный ROI (Do First)

| # | Улучшение | Effort | Impact | Статус |
|---|-----------|--------|--------|--------|
| 1 | **CLI Interface** | Medium | Very High | ✅ **DONE** |
| 2 | **Task List MVP** | Medium | High | ❌ Pending |
| 3 | **Architecture Docs** | Low | High | ✅ **DONE** |
| 4 | **Good First Issues** | Low | High | ✅ **DONE** |
| 5 | **Window Opacity** | Low | Medium | ✅ **DONE** |
| 6 | **Daily Goals** | Low | High | ✅ **DONE** |
| 7 | **Global Hotkeys** | Medium | High | ✅ **DONE** (новое) |

### Tier 2: Высокий ROI (Do Soon)

| # | Улучшение | Effort | Impact | ROI Score |
|---|-----------|--------|--------|-----------|
| 7 | Achievements System | Medium | High | 80/100 |
| 8 | Discord Rich Presence | Medium | High | 78/100 |
| 9 | macOS/Linux Autostart | Medium | High | 75/100 |
| 10 | CI/CD All Platforms | Medium | High | 75/100 |
| 11 | High Contrast Mode | Low | Medium | 72/100 |
| 12 | Reduced Motion | Low | Medium | 72/100 |

### Tier 3: Средний ROI (Do Later)

| # | Улучшение | Effort | Impact | ROI Score |
|---|-----------|--------|--------|-----------|
| 13 | Monthly Heatmap | Medium | Medium | 65/100 |
| 14 | Light Mode | Medium | Medium | 65/100 |
| 15 | VS Code Extension | Medium | Medium | 62/100 |
| 16 | Ambient Sounds | Medium | Medium | 60/100 |
| 17 | Webhooks MVP | Medium | Medium | 58/100 |

### Tier 4: Низкий ROI (Consider Carefully)

| # | Улучшение | Effort | Impact | ROI Score |
|---|-----------|--------|--------|-----------|
| 18 | Plugin System | High | Medium | 45/100 |
| 19 | Cloud Sync | Very High | Medium | 30/100 |
| 20 | Mobile App | Very High | Medium | 25/100 |
| 21 | Web WASM Version | High | Low | 20/100 |

---

# 8. Детальное описание улучшений

## 8.1 CLI Interface (P0 - Must Have)

### Описание
Терминальный интерфейс для управления таймером без GUI.

### User Stories
```
US-CLI-1: Как разработчик, я хочу запустить таймер из терминала
US-CLI-2: Как разработчик, я хочу видеть статус таймера в терминале
US-CLI-3: Как разработчик, я хочу интегрировать таймер в скрипты
```

### Функциональные требования

```bash
# Базовые команды
pomodorust start [work|break]    # Запустить таймер
pomodorust pause                 # Пауза
pomodorust resume                # Продолжить
pomodorust stop                  # Остановить
pomodorust skip                  # Пропустить сессию
pomodorust status                # Текущий статус (JSON)
pomodorust stats [today|week|month|all]

# Конфигурация
pomodorust config get <key>
pomodorust config set <key> <value>
pomodorust config list

# Сервис
pomodorust daemon start          # Запустить фоновый процесс
pomodorust daemon stop           # Остановить
pomodorust daemon status         # Статус демона
```

### Технический дизайн

```rust
// IPC через Named Pipe (Windows) / Unix Socket (Unix)
enum IpcCommand {
    Start { session_type: Option<SessionType> },
    Pause,
    Resume,
    Stop,
    Skip,
    Status,
    Stats { period: StatsPeriod },
}

enum IpcResponse {
    Ok,
    Status { state: TimerState, remaining: Duration, session: SessionInfo },
    Stats { data: StatsData },
    Error { message: String },
}
```

### Acceptance Criteria
- [ ] Все команды работают на Windows, macOS, Linux
- [ ] JSON output для машинной обработки
- [ ] Human-readable output по умолчанию
- [ ] Exit codes для скриптов
- [ ] Man page / --help документация

### Estimated Effort: 1-2 недели

---

## 8.2 Task List MVP (P0 - Must Have)

### Описание
Базовый список задач с привязкой к Pomodoro сессиям.

### User Stories
```
US-TL-1: Как пользователь, я хочу добавить задачу перед началом работы
US-TL-2: Как пользователь, я хочу оценить задачу в помидорах
US-TL-3: Как пользователь, я хочу видеть прогресс по задаче
US-TL-4: Как пользователь, я хочу отмечать задачи выполненными
```

### Функциональные требования

| Требование | Приоритет |
|------------|-----------|
| Добавление задачи (title only) | P0 |
| Оценка в помидорах (estimated) | P0 |
| Привязка сессии к задаче | P0 |
| Отметка completed | P0 |
| Перенос незавершённых на завтра | P1 |
| Редактирование задачи | P1 |
| Удаление задачи | P1 |
| Drag-n-drop сортировка | P2 |
| Теги/категории | P3 |

### Схема данных

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    estimated_pomodoros INTEGER DEFAULT 1,
    completed_pomodoros INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending', -- pending, in_progress, completed
    created_at TEXT NOT NULL,
    completed_at TEXT,
    position INTEGER DEFAULT 0
);

ALTER TABLE sessions ADD COLUMN task_id INTEGER REFERENCES tasks(id);
```

### UI Mockup

```
┌─────────────────────────────────┐
│ Today's Tasks                   │
├─────────────────────────────────┤
│ ○ Implement CLI interface  2/4🍅│
│ ● Fix database bug         1/1🍅│ ✓
│ ○ Write documentation      0/2🍅│
│ ○ Review PR #123           0/1🍅│
├─────────────────────────────────┤
│ [+ Add task]                    │
└─────────────────────────────────┘
```

### Acceptance Criteria
- [ ] Задачи сохраняются в SQLite
- [ ] Сессии связываются с текущей задачей
- [ ] Подсчёт помидоров на задачу корректен
- [ ] Список отображается в боковой панели или отдельном view

### Estimated Effort: 1-2 недели

---

## 8.3 Achievement System (P1 - Should Have)

### Описание
Система достижений для повышения engagement и retention.

### Список достижений (MVP - 15 штук)

| ID | Название | Условие | Иконка |
|----|----------|---------|--------|
| A01 | First Tomato | 1 помидор | 🍅 |
| A02 | Getting Started | 10 помидоров | 🌱 |
| A03 | Week Warrior | 7 дней подряд | 🔥 |
| A04 | Centurion | 100 помидоров | 💯 |
| A05 | Early Bird | 10 сессий до 9:00 | ☀️ |
| A06 | Night Owl | 10 сессий после 22:00 | 🌙 |
| A07 | Marathon | 8 помидоров в один день | 🏃 |
| A08 | Perfectionist | Неделя без skips | ✨ |
| A09 | Comeback Kid | Вернуться после 7 дней перерыва | 🔄 |
| A10 | Half K | 500 помидоров | 🎯 |
| A11 | Thousand | 1000 помидоров | 🏆 |
| A12 | Month Master | 30 дней подряд | 📅 |
| A13 | Focus Champion | 4 часа без перерыва в день | 🧘 |
| A14 | Task Slayer | Завершить 50 задач | ⚔️ |
| A15 | Theme Explorer | Попробовать все темы | 🎨 |

### Технический дизайн

```rust
struct Achievement {
    id: String,
    name: String,
    description: String,
    icon: String,
    condition: AchievementCondition,
    unlocked_at: Option<DateTime<Utc>>,
}

enum AchievementCondition {
    TotalPomodoros(u32),
    ConsecutiveDays(u32),
    SessionsBeforeHour(u32, u8),
    SessionsAfterHour(u32, u8),
    PomodorosInDay(u32),
    // ...
}
```

### UI
- Toast notification при разблокировке
- Отдельный view со всеми достижениями
- Locked achievements с подсказками

### Estimated Effort: 3-5 дней

---

## 8.4 Daily/Weekly Goals (P1 - Should Have)

### Описание
Установка целей по помидорам на день и неделю.

### Функциональные требования

```
Daily Goal: 8 🍅
Today: ████████░░ 6/8 (75%)

Weekly Goal: 40 🍅
This Week: ██████████████████████████░░░░░░░░░░░░░░ 28/40 (70%)
```

### Технический дизайн

```rust
struct Goals {
    daily_target: u32,          // default: 8
    weekly_target: u32,         // default: 40
    notify_on_goal: bool,       // default: true
}

// В config.toml
[goals]
daily = 8
weekly = 40
notify_on_complete = true
```

### Acceptance Criteria
- [ ] Настройка целей в Settings
- [ ] Прогресс-бар в Timer view
- [ ] Уведомление при достижении цели
- [ ] Streak tracking для целей

### Estimated Effort: 2-3 дня

---

## 8.5 Discord Rich Presence (P1 - Should Have)

### Описание
Отображение статуса Pomodoro в Discord.

### Функциональные требования

```
Discord Status:
┌─────────────────────────────────┐
│ 🍅 Pomodorust                   │
│ Working - 15:32 remaining       │
│ Session 3/4 | 🔥 5 day streak   │
└─────────────────────────────────┘
```

### Технический дизайн

```rust
// Использовать discord-rich-presence crate
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};

fn update_discord_presence(session: &Session) {
    let mut client = DiscordIpcClient::new("APP_ID")?;
    client.connect()?;

    client.set_activity(activity::Activity::new()
        .state(format!("{} - {} remaining", session.session_type, session.remaining))
        .details(format!("Session {}/{}", session.current, session.total))
        .assets(activity::Assets::new()
            .large_image("pomodorust-icon")
            .large_text("Pomodorust"))
    )?;
}
```

### Acceptance Criteria
- [ ] Автоматическое подключение к Discord
- [ ] Отображение типа сессии (Work/Break)
- [ ] Отображение оставшегося времени
- [ ] Отключение через настройки
- [ ] Graceful handling если Discord не запущен

### Estimated Effort: 3-5 дней

---

## 8.6 macOS/Linux Autostart (P1 - Should Have)

### macOS (LaunchAgent)

```xml
<!-- ~/Library/LaunchAgents/com.pomodorust.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" ...>
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.pomodorust</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/pomodorust</string>
        <string>--minimized</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>
```

### Linux (systemd user service)

```ini
# ~/.config/systemd/user/pomodorust.service
[Unit]
Description=Pomodorust Pomodoro Timer
After=graphical-session.target

[Service]
ExecStart=/usr/bin/pomodorust --minimized
Restart=on-failure

[Install]
WantedBy=default.target
```

### Технический дизайн

```rust
// platform/macos.rs
pub fn set_autostart(enabled: bool) -> Result<()> {
    let plist_path = dirs::home_dir()
        .unwrap()
        .join("Library/LaunchAgents/com.pomodorust.plist");

    if enabled {
        std::fs::write(&plist_path, LAUNCHAGENT_PLIST)?;
        Command::new("launchctl")
            .args(["load", plist_path.to_str().unwrap()])
            .output()?;
    } else {
        Command::new("launchctl")
            .args(["unload", plist_path.to_str().unwrap()])
            .output()?;
        std::fs::remove_file(&plist_path)?;
    }
    Ok(())
}

// platform/linux.rs
pub fn set_autostart(enabled: bool) -> Result<()> {
    let service_path = dirs::config_dir()
        .unwrap()
        .join("systemd/user/pomodorust.service");

    if enabled {
        std::fs::create_dir_all(service_path.parent().unwrap())?;
        std::fs::write(&service_path, SYSTEMD_SERVICE)?;
        Command::new("systemctl")
            .args(["--user", "enable", "pomodorust"])
            .output()?;
    } else {
        Command::new("systemctl")
            .args(["--user", "disable", "pomodorust"])
            .output()?;
        std::fs::remove_file(&service_path)?;
    }
    Ok(())
}
```

### Estimated Effort: 3-5 дней

---

## 8.7 High Contrast Mode & Accessibility (P1)

### Описание
Режим высокой контрастности и другие accessibility улучшения.

### Функциональные требования

| Фича | Описание |
|------|----------|
| High Contrast | Чёрный фон + белый текст + яркие акценты |
| Reduced Motion | Отключение всех анимаций |
| Font Scaling | 80%-150% размер шрифтов |
| Keyboard Navigation | Tab navigation по элементам |

### Технический дизайн

```rust
// В config.toml
[accessibility]
high_contrast = false
reduced_motion = false
font_scale = 1.0  // 0.8 - 1.5

// В theme.rs
impl Theme {
    pub fn high_contrast() -> Self {
        Theme {
            background: Color32::BLACK,
            text: Color32::WHITE,
            accent: Color32::from_rgb(255, 255, 0), // Bright yellow
            border: Color32::WHITE,
            // ...
        }
    }
}
```

### Estimated Effort: 2-3 дня

---

## 8.8 Monthly Heatmap Calendar (P2)

### Описание
GitHub-style календарь активности.

### UI Mockup

```
January 2026
Mo Tu We Th Fr Sa Su
      1  2  3  4  5
░░ ░░ ██ ██ ██ ░░ ░░
 6  7  8  9 10 11 12
░░ ██ ██ ██ ██ ░░ ░░
13 14 15 16 17 18 19
██ ██ ██ ██ ██ ░░ ░░
...

Legend: ░ 0  █ 1-4  ██ 5-8  ███ 9+
```

### Технический дизайн

```rust
fn render_heatmap(ui: &mut Ui, stats: &MonthlyStats) {
    for week in stats.weeks() {
        ui.horizontal(|ui| {
            for day in week {
                let intensity = day.pomodoros as f32 / 10.0;
                let color = lerp_color(LIGHT_GREEN, DARK_GREEN, intensity.min(1.0));

                let rect = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
                ui.painter().rect_filled(rect.rect, 2.0, color);

                if rect.hovered() {
                    egui::show_tooltip(ui.ctx(), |ui| {
                        ui.label(format!("{}: {} 🍅", day.date, day.pomodoros));
                    });
                }
            }
        });
    }
}
```

### Estimated Effort: 3-5 дней

---

## 8.9 Ambient Sounds (P2)

### Описание
Фоновые звуки для улучшения фокуса.

### Список звуков (MVP)

| Категория | Звуки |
|-----------|-------|
| Nature | Rain, Forest, Ocean, Fireplace |
| Urban | Coffee Shop, Library |
| White Noise | White, Pink, Brown noise |
| Music | Lo-fi beats (royalty-free) |

### Технический дизайн

```rust
// Добавить в config
[sounds]
ambient_enabled = false
ambient_sound = "none"  // rain, forest, ocean, coffee_shop, white_noise, lofi
ambient_volume = 50

// Отдельный audio stream для ambient
struct AmbientPlayer {
    stream: Option<OutputStream>,
    sink: Option<Sink>,
    current_sound: Option<AmbientSound>,
}

impl AmbientPlayer {
    pub fn play(&mut self, sound: AmbientSound, volume: f32) {
        // Looping playback
        let source = Decoder::new(sound.data())?.repeat_infinite();
        self.sink.append(source);
        self.sink.set_volume(volume);
    }
}
```

### Файлы
- Использовать royalty-free ambient sounds (freesound.org)
- Или генерировать white/pink/brown noise программно
- Размер: ~2-5 MB на звук (MP3 64kbps)

### Estimated Effort: 3-5 дней

---

## 8.10 VS Code Extension (P2)

### Описание
Status bar widget для VS Code.

### Функциональность

```
VS Code Status Bar:
┌─────────────────────────────────────────────┐
│ ...  🍅 15:32 (Work 3/4)  |  ... │
└─────────────────────────────────────────────┘

Click actions:
- Left click: Start/Pause
- Right click: Context menu (Skip, Stop, Stats)
```

### Технический дизайн

```typescript
// extension.ts
import * as vscode from 'vscode';
import * as net from 'net';

const statusBarItem = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Right,
    100
);

function connectToPomodorust() {
    // Connect via Unix socket / Named pipe
    const socket = net.connect(SOCKET_PATH);

    socket.on('data', (data) => {
        const status = JSON.parse(data.toString());
        updateStatusBar(status);
    });
}

function updateStatusBar(status: PomodoroStatus) {
    const icon = status.state === 'work' ? '🍅' : '☕';
    statusBarItem.text = `${icon} ${status.remaining} (${status.sessionType} ${status.current}/${status.total})`;
    statusBarItem.show();
}
```

### Требования
- Связь через IPC (требует CLI с daemon mode)
- Публикация в VS Code Marketplace
- Минимальные зависимости

### Estimated Effort: 1 неделя (после CLI)

---

# 9. Стратегическая дорожная карта

## 9.1 Timeline Overview

```
2026 Q1 (Jan-Mar)                    Q2 (Apr-Jun)
├── Phase 1: Foundation ────────────►├── Phase 3: Retention
│   ├── Quick Wins (Week 1-2)        │   ├── Achievement System
│   ├── Architecture Docs            │   ├── GitHub Integration
│   ├── CI/CD All Platforms          │   └── Goals & Milestones
│   └── Good First Issues            │
│                                    │
├── Phase 2: Differentiation        ─┤
│   ├── CLI Interface (Week 3-5)     │
│   ├── Task List MVP (Week 5-7)     │
│   ├── Discord Rich Presence        │
│   └── macOS/Linux Autostart        │
│                                    │
└────────────────────────────────────┘

Q3 (Jul-Sep)                         Q4 (Oct-Dec)
├── Phase 4: Scale                   └── Phase 5: Ecosystem
│   ├── VS Code Extension                ├── Plugin System
│   ├── Webhooks                         ├── JetBrains Plugin
│   ├── Monthly Heatmap                  ├── Jira/Linear Integration
│   └── Ambient Sounds                   └── Community Growth
```

## 9.2 Phase 1: Foundation (Weeks 1-3)

### Цели
- Устранить технический долг
- Подготовить инфраструктуру для контрибьюторов
- Завершить недоделанные функции

### Deliverables

| Week | Deliverable | Owner | Status |
|------|-------------|-------|--------|
| 1 | Quick Wins (opacity, tick sound) | Dev | ✅ Done |
| 1 | ARCHITECTURE.md | Dev | ✅ Done |
| 2 | CI/CD for all platforms | Dev | - |
| 2 | 10 Good First Issues | Dev | ✅ Done |
| 2 | Issue/PR templates | Dev | - |
| 3 | Daily Goals feature | Dev | ✅ Done |
| 3 | High Contrast Mode | Dev | - |

### Exit Criteria
- [x] Все Quick Wins завершены (opacity, tick sound)
- [ ] CI проходит на Windows, macOS, Linux
- [x] ARCHITECTURE.md опубликован
- [x] 10+ good first issues созданы

---

## 9.3 Phase 2: Differentiation (Weeks 4-8)

### Цели
- Реализовать ключевые дифференциаторы
- Занять нишу "Developer's Pomodoro"

### Deliverables

| Week | Deliverable | Owner | Status |
|------|-------------|-------|--------|
| 4-5 | CLI Interface MVP | Dev | ✅ Done |
| 4-5 | Global Hotkeys | Dev | ✅ Done (бонус) |
| 5-7 | Task List MVP | Dev | - |
| 6-7 | Discord Rich Presence | Dev | - |
| 7-8 | macOS Autostart | Dev | - |
| 7-8 | Linux Autostart | Dev | - |

### Exit Criteria
- [x] CLI работает: start, pause, status, stats
- [ ] Task List: add, complete, link to sessions
- [ ] Discord показывает статус
- [ ] Autostart работает на всех платформах

---

## 9.4 Phase 3: Retention (Weeks 9-14)

### Цели
- Увеличить retention пользователей
- Добавить геймификацию

### Deliverables

| Week | Deliverable | Owner | Status |
|------|-------------|-------|--------|
| 9-10 | Achievement System (15 achievements) | Dev | - |
| 10-11 | Weekly Goals | Dev | - |
| 11-12 | Progress Milestones | Dev | - |
| 12-13 | Session Notes | Dev | - |
| 13-14 | Undo Session | Dev | - |

### Exit Criteria
- [ ] 15 достижений реализованы
- [ ] Цели можно устанавливать и отслеживать
- [ ] Уведомления о достижениях работают

---

## 9.5 Phase 4: Scale (Weeks 15-22)

### Цели
- Расширить экосистему интеграций
- Улучшить аналитику

### Deliverables

| Week | Deliverable | Owner | Status |
|------|-------------|-------|--------|
| 15-16 | VS Code Extension | Dev | - |
| 16-17 | Webhooks MVP | Dev | - |
| 17-19 | Monthly Heatmap | Dev | - |
| 19-20 | Ambient Sounds | Dev | - |
| 20-22 | Light Mode | Dev | - |

---

## 9.6 Phase 5: Ecosystem (Weeks 23+)

### Цели
- Создать plugin экосистему
- Расширить интеграции

### Deliverables
- Plugin System MVP
- JetBrains Plugin
- Jira/Linear Integration
- GitHub Commit Integration

---

# 10. Метрики успеха

## 10.1 GitHub Metrics

| Metric | Current | 3 mo | 6 mo | 12 mo |
|--------|---------|------|------|-------|
| Stars | ? | 200 | 500 | 2,000 |
| Forks | ? | 20 | 50 | 200 |
| Contributors | 1 | 5 | 10 | 30 |
| Closed Issues | ? | 30 | 100 | 300 |
| Open PRs (avg) | 0 | 2 | 5 | 10 |

## 10.2 Adoption Metrics

| Metric | 3 mo | 6 mo | 12 mo |
|--------|------|------|-------|
| Downloads (total) | 500 | 2,000 | 10,000 |
| VS Code Installs | - | 200 | 1,000 |
| CLI Users (est.) | 50 | 200 | 1,000 |
| Discord Presence | 100 | 300 | 1,000 |

## 10.3 Engagement Metrics

| Metric | Target |
|--------|--------|
| Average Session Streak | 5+ days |
| Daily Active Users | 100+ |
| CLI Usage % | 30%+ |
| Achievement Unlock Rate | 50% first 5 |

## 10.4 Quality Metrics

| Metric | Target |
|--------|--------|
| Crash Rate | < 0.1% |
| Issue Response Time | < 48h |
| PR Merge Time | < 1 week |
| CI Pass Rate | > 95% |

---

# 11. Риски и митигация

## 11.1 Risk Matrix

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **Lack of contributors** | High | High | Good first issues, Architecture docs, Community engagement |
| **Feature creep** | Medium | Medium | Strict prioritization, "NOT TO DO" list |
| **Platform fragmentation** | Medium | High | CI/CD for all platforms, Feature flags |
| **Burnout (solo dev)** | Medium | High | Delegate via good first issues, pace releases |
| **Competition adds CLI** | Low | Medium | First-mover advantage, deeper integration |
| **egui limitations** | Low | Medium | Community plugins, custom rendering |
| **Rust learning curve** | Medium | Medium | Comprehensive docs, example code |

## 11.2 Mitigation Strategies

### Risk: Lack of Contributors

**Actions:**
1. Write comprehensive ARCHITECTURE.md
2. Create 20+ "good first issue" labels
3. Respond to issues within 48h
4. Write blog posts about contributing
5. Engage on Reddit/HN when relevant

### Risk: Feature Creep

**Actions:**
1. Maintain strict "NOT TO DO" list
2. Every feature requires RFC for large changes
3. Phase gates with exit criteria
4. Quarterly roadmap reviews

### Risk: Platform Fragmentation

**Actions:**
1. CI/CD on all 3 platforms from Phase 1
2. Platform abstraction layer
3. Feature flags for platform-specific features
4. Regular cross-platform testing

---

# 12. Рекомендации

## 12.1 Immediate Actions (This Week) — ОБНОВЛЕНО

| # | Action | Effort | Impact | Статус |
|---|--------|--------|--------|--------|
| 1 | Write ARCHITECTURE.md | 1 day | High | ✅ Done |
| 2 | Create 10 good first issues | 2 hours | High | ✅ Done |
| 3 | Fix window opacity | 2 hours | Medium | ✅ Done |
| 4 | Fix tick sound toggle | 2 hours | Low | ✅ Done |
| 5 | Setup GitHub Actions for macOS/Linux | 1 day | High | ❌ Pending |

## 12.2 Short-term (This Month) — ОБНОВЛЕНО

| # | Action | Effort | Impact | Статус |
|---|--------|--------|--------|--------|
| 1 | Implement CLI MVP | 1-2 weeks | Very High | ✅ Done |
| 2 | Add Daily Goals | 2-3 days | High | ✅ Done |
| 3 | Add Global Hotkeys | 3-5 days | High | ✅ Done |
| 4 | Add High Contrast Mode | 1 day | Medium | ❌ Pending |
| 5 | Post to r/rust, r/productivity | 1 hour | Medium | ❌ Pending |

## 12.3 Medium-term (This Quarter)

| # | Action | Effort | Impact |
|---|--------|--------|--------|
| 1 | Task List MVP | 1-2 weeks | High |
| 2 | Achievement System | 1 week | High |
| 3 | Discord Rich Presence | 3-5 days | High |
| 4 | macOS/Linux Autostart | 1 week | High |

## 12.4 Strategic Recommendations

### 1. Focus on Developer Niche
- **Do**: CLI, IDE integrations, webhooks, GitHub integration
- **Don't**: Mobile app, full gamification, cloud sync

### 2. Leverage Unique Assets
- **Retro themes** are viral material — promote on Twitter/Reddit
- **Rust** attracts quality contributors — engage Rust community
- **Native performance** differentiates from Electron apps

### 3. Build Community First
- Contributors → Features → Users → More Contributors
- Respond to every issue
- Merge community PRs quickly
- Credit contributors visibly

### 4. Measure and Iterate
- Track GitHub stars weekly
- Monitor download counts
- Survey users quarterly
- A/B test new features

---

# 13. Приложения

## Appendix A: Полный список улучшений (135 items)

### A.1 Core Functionality (20 items)
1. Task List MVP
2. Session Notes
3. Task Tags
4. Task Templates
5. Multiple Timers
6. Timer Presets per Project
7. Strict Mode
8. Focus Mode (block notifications)
9. Flexible Breaks
10. Break Suggestions
11. Stretch Reminders
12. Break Extension
13. Ambient Sounds
14. Session History View
15. Undo Last Session
16. Edit Past Sessions
17. Session Recovery
18. Pomodoro Estimation
19. Time Blocking
20. Calendar Integration

### A.2 Statistics & Analytics (15 items)
21. Daily Goals
22. Weekly Goals
23. Monthly Goals
24. Custom Goals
25. Monthly Heatmap
26. Yearly Heatmap
27. Hourly Distribution
28. Day of Week Analysis
29. Trend Lines
30. Comparison Charts
31. Weekly Summary
32. Monthly Report
33. PDF Export
34. Productivity Score
35. Focus Time Tracking

### A.3 Gamification (12 items)
36. Achievement System (15)
37. Progress Milestones
38. XP/Levels
39. Daily Challenges
40. Weekly Challenges
41. Badge Collection
42. Statistics Leaderboard (local)
43. Streak Bonuses
44. Focus Streaks
45. Completion Rewards
46. Virtual Currency
47. Unlockable Themes

### A.4 UI/UX (25 items)
48. Light Mode
49. Custom Theme Editor
50. Theme Import/Export
51. Font Selection
52. Icon Packs
53. Mini Mode
54. Widget Mode
55. Fullscreen Mode
56. PiP Mode
57. Particle Effects
58. Confetti Celebration
59. Smooth View Transitions
60. Progress Shake
61. Sound Visualization
62. High Contrast Mode
63. Reduced Motion
64. Font Scaling
65. Screen Reader Support
66. Keyboard Navigation
67. Focus Indicators
68. Color Blind Mode
69. Compact Mode
70. Window Opacity
71. Custom Backgrounds
72. Animated Backgrounds

### A.5 Sounds & Notifications (10 items)
73. Custom Sounds (file)
74. Ambient Sounds
75. More Built-in Sounds
76. Volume per Sound Type
77. Notification Scheduling
78. Smart Notifications
79. Sound Themes
80. Voice Announcements
81. Tick Sound Variations
82. Break Music

### A.6 Integrations (20 items)
83. CLI Interface
84. Discord Rich Presence
85. Slack Status
86. Telegram Bot
87. Microsoft Teams
88. VS Code Extension
89. JetBrains Plugin
90. Vim Plugin
91. Emacs Plugin
92. Webhooks MVP
93. Zapier Integration
94. IFTTT Integration
95. Home Assistant
96. GitHub Integration
97. GitLab Integration
98. Jira Integration
99. Linear Integration
100. Todoist Import
101. Toggl Export
102. Clockify Export

### A.7 Cross-platform (15 items)
103. macOS Autostart
104. Linux Autostart
105. macOS Native Notifications
106. Linux DBus Notifications
107. Homebrew Formula
108. Chocolatey Package
109. AUR Package
110. Flatpak
111. Snap
112. Windows Store
113. macOS Menu Bar Mode
114. Linux Tray (StatusNotifier)
115. Touch Bar Support
116. Handoff Support
117. Wayland Native

### A.8 Architecture & Code (10 items)
118. Config Validation
119. Config Migration
120. Config Profiles
121. Environment Variables
122. Async Database
123. Plugin System
124. Integration Tests
125. E2E Tests
126. Benchmarks
127. Error Handling Unification

### A.9 Open Source Infrastructure (8 items)
128. Architecture Docs
129. Good First Issues
130. CI/CD All Platforms
131. Issue Templates
132. PR Template
133. Cargo Publish
134. Release Automation
135. Changelog Automation

---

## Appendix B: Competitive Feature Matrix

| Feature | Pomodorust | Forest | Focus To-Do | Pomotroid | TomatoBar |
|---------|------------|--------|-------------|-----------|-----------|
| Basic Timer | ✅ | ✅ | ✅ | ✅ | ✅ |
| Custom Intervals | ✅ | ❌ | ✅ | ✅ | ✅ |
| Presets | ✅ | ❌ | ✅ | ✅ | ❌ |
| Auto-start | ✅ | ❌ | ✅ | ✅ | ✅ |
| Task List | ❌ | ❌ | ✅ | ❌ | ❌ |
| Statistics | ✅ | ✅ | ✅ | ✅ | Basic |
| Themes | ✅ (9+3) | Limited | ✅ | ✅ | ❌ |
| CLI | ❌ | ❌ | ❌ | ❌ | ❌ |
| IDE Plugin | ❌ | ❌ | ❌ | ❌ | ❌ |
| Discord | ❌ | ❌ | ❌ | ❌ | ❌ |
| Achievements | ❌ | ✅ | ✅ | ❌ | ❌ |
| Ambient Sounds | ❌ | ❌ | ✅ | ❌ | ❌ |
| Cloud Sync | ❌ | ✅ | ✅ | ❌ | ❌ |
| Cross-platform | ⚠️ | Mobile | ✅ | ✅ | macOS |
| Native Perf | ✅ | N/A | ❌ | ❌ | ✅ |
| Retro Themes | ✅ | ❌ | ❌ | ❌ | ❌ |
| Open Source | ✅ | ❌ | ❌ | ✅ | ✅ |
| Privacy | ✅ | ❌ | ⚠️ | ✅ | ✅ |

---

## Appendix C: Technical Specifications

### C.1 CLI IPC Protocol

```
Protocol: Named Pipe (Windows) / Unix Domain Socket (Unix)
Path:
  - Windows: \\.\pipe\pomodorust
  - Unix: /tmp/pomodorust.sock

Message Format: JSON Lines (NDJSON)

Request:
{
  "id": "uuid",
  "command": "start|pause|resume|stop|skip|status|stats",
  "params": { ... }
}

Response:
{
  "id": "uuid",
  "success": true|false,
  "data": { ... },
  "error": "message" (if success=false)
}
```

### C.2 Database Schema v2 (with Tasks)

```sql
-- Existing tables
CREATE TABLE sessions (...);
CREATE TABLE daily_stats (...);
CREATE TABLE streaks (...);

-- New tables
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    estimated_pomodoros INTEGER DEFAULT 1,
    completed_pomodoros INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending',
    tags TEXT, -- JSON array
    project_id INTEGER REFERENCES projects(id),
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    completed_at TEXT,
    due_date TEXT,
    position INTEGER DEFAULT 0
);

CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    color TEXT,
    archived INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE achievements (
    id TEXT PRIMARY KEY,
    unlocked_at TEXT,
    progress INTEGER DEFAULT 0
);

CREATE TABLE goals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    type TEXT NOT NULL, -- daily, weekly, custom
    target INTEGER NOT NULL,
    period_start TEXT NOT NULL,
    period_end TEXT NOT NULL,
    achieved INTEGER DEFAULT 0
);

-- Add task reference to sessions
ALTER TABLE sessions ADD COLUMN task_id INTEGER REFERENCES tasks(id);
```

### C.3 Configuration Schema v2

```toml
[timer]
work_duration = 25
short_break = 5
long_break = 15
sessions_before_long = 4
auto_start_breaks = true
auto_start_work = false

[appearance]
accent_color = "blue"
compact_mode = false
window_opacity = 100
theme = "dark"  # dark, light, high_contrast

[sounds]
enabled = true
volume = 80
notification_sound = "soft_bell"
tick_enabled = false
ambient_enabled = false
ambient_sound = "none"
ambient_volume = 50

[system]
start_with_windows = false
minimize_to_tray = true
show_in_taskbar = true
notifications_enabled = true

[window]
width = 360.0
height = 480.0
x = 100.0
y = 100.0
always_on_top = false
maximized = false

[accessibility]
high_contrast = false
reduced_motion = false
font_scale = 1.0

[goals]
daily = 8
weekly = 40
notify_on_complete = true

[integrations]
discord_presence = false
cli_enabled = true

[advanced]
data_path = ""  # Empty = default
log_level = "info"
```

---

## Appendix D: Glossary

| Term | Definition |
|------|------------|
| **Pomodoro** | 25-minute focused work session |
| **Streak** | Consecutive days with completed pomodoros |
| **CLI** | Command Line Interface |
| **IPC** | Inter-Process Communication |
| **Rich Presence** | Discord feature showing app activity |
| **Heatmap** | Visual representation of activity over time |
| **Good First Issue** | GitHub issue labeled for new contributors |
| **Flywheel** | Self-reinforcing cycle of growth |
| **JTBD** | Jobs To Be Done (framework) |
| **MVP** | Minimum Viable Product |
| **ROI** | Return on Investment |

---

## Appendix E: References

### Market Research
- Zapier: "6 Best Pomodoro Timer Apps"
- Reclaim.ai: "Top 11 Pomodoro Timer Apps"
- ProductHunt: Pomodoro category analysis

### Open Source Best Practices
- GitHub: "Building Welcoming Communities"
- FreeCodeCamp: "How to Get More Engagement"
- ToolJet: "12 Ways to Get More GitHub Stars"

### Technical References
- egui documentation
- rusqlite documentation
- discord-rich-presence crate
- Windows API documentation

---

*Document generated: 2026-01-19*
*Last updated: 2026-01-19*
*Version: 1.1*
*Author: AI Business Analysis*
*Next review: 2026-02-19*

---

## Changelog

### v1.1 (2026-01-19)
**Выполненные улучшения:**
- ✅ CLI Interface (I01) — полная реализация с IPC через TCP
- ✅ Window Opacity (Q01) — подключён к UI
- ✅ Tick Sound (Q02) — подключён к UI
- ✅ Daily Goals (S01) — реализованы с прогресс-баром
- ✅ Architecture Docs (O01) — ARCHITECTURE.md создан
- ✅ Good First Issues (O02) — GOOD_FIRST_ISSUES.md создан
- ✅ Global Hotkeys — Ctrl+Alt+Space/S/R (новая фича)
- ✅ Notification Sounds — 6 выбираемых звуков
- ✅ Единый бинарник — GUI + CLI в одном исполняемом файле
