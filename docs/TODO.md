# PomodoRust — Roadmap / TODO

## 1. System Tray [HIGH PRIORITY]
- [ ] Добавить system tray иконку (crate: `tray-icon`)
- [ ] Показывать текущий статус таймера в тултипе трея
- [ ] Контекстное меню трея: Start/Pause, Skip, Show Window, Quit
- [ ] Реализовать `minimize_to_tray` (конфиг уже есть, реализации нет)
- [ ] Обновление иконки трея при смене сессии (work/break)

## 2. Привязка помодоро к задачам [HIGH PRIORITY]
- [x] Добавить `todo_id INTEGER REFERENCES todo_items(id)` в таблицу `sessions`
- [x] При завершении work-сессии записывать `todo_id` текущей задачи из очереди
- [x] Статистика по задачам: сколько времени потрачено на каждую задачу
- [x] Миграция для существующих БД (ALTER TABLE)
- [x] Обновление экспорта (CSV/JSON) с полем `todo_id`
- [ ] Отображение потраченного времени в todo view рядом с задачей
- [ ] Фильтр статистики по задаче/проекту

## 3. Локализация (i18n) [HIGH PRIORITY]
- [ ] Вынести все UI-строки в систему локализации
- [ ] Устранить микс русского и английского в интерфейсе
- [ ] Поддержка русского и английского языков
- [ ] Настройка языка в Settings
- [ ] Добавить `language` в `Config`

## 4. Графики за произвольный период [MEDIUM]
- [x] Навигация по неделям (кнопки < >) в stats view
- [x] Метод `get_week_stats_for_date(date: NaiveDate)` в Database
- [x] Метод `get_earliest_stats_date()` для границ навигации
- [x] Отображение диапазона дат для прошлых недель
- [ ] Навигация по месяцам
- [ ] Выбор произвольного диапазона дат
- [ ] Месячная агрегация данных

## 5. Приоритеты задач [MEDIUM]
- [x] Добавить `priority INTEGER DEFAULT 0` в `todo_items`
- [x] Enum `Priority` (None, Low, Medium, High, Urgent)
- [x] Цветовая индикация приоритета в todo view (цветная точка)
- [x] Сортировка задач по приоритету (DESC)
- [x] Выбор приоритета через контекстное меню задачи
- [x] Миграция для существующих БД
- [ ] Выбор приоритета при создании задачи (inline)

## 6. Drag & Drop [MEDIUM]
- [ ] Drag & Drop для задач внутри проекта (изменение порядка)
- [ ] Drag & Drop задач между проектами
- [ ] Drag & Drop для элементов очереди (изменение порядка)
- [ ] Визуальная индикация drop-зоны
- [ ] Использовать egui `dnd` API

## 7. Тесты [MEDIUM]
- [ ] Unit-тесты для Database (используя `open_in_memory()`)
- [ ] Тесты для Timer/Session логики (core)
- [ ] Тесты для Config (load/save/reset/preset)
- [ ] Тесты для IPC протокола (serialize/deserialize)
- [ ] Тесты для экспорта (CSV/JSON)
- [ ] CI pipeline (GitHub Actions)

## 8. Heatmap активности [MEDIUM]
- [ ] Компонент heatmap (стиль GitHub contributions)
- [ ] Данные из `daily_stats` за последние 365 дней
- [ ] Цветовая градация по количеству помодоро
- [ ] Тултипы с деталями дня при наведении
- [ ] Размещение в stats view

## 9. Теги/метки для задач [LOW]
- [ ] Таблица `tags (id, name, color)`
- [ ] Таблица `todo_tags (todo_id, tag_id)` (many-to-many)
- [ ] UI для создания/удаления тегов
- [ ] Фильтрация задач по тегам
- [ ] Отображение тегов в todo view (цветные badges)

## 10. Дедлайны задач [LOW]
- [ ] Добавить `due_date TEXT` в `todo_items`
- [ ] Выбор даты в todo editing UI
- [ ] Визуальная индикация: просроченные — красным, скоро — жёлтым
- [ ] Сортировка по дедлайну
- [ ] Уведомление о приближающемся дедлайне

## 11. Пользовательские звуки [LOW]
- [ ] Добавить вариант `Custom(PathBuf)` в `NotificationSound`
- [ ] Кнопка выбора файла через `rfd::FileDialog`
- [ ] Поддержка `.mp3` и `.wav`
- [ ] Сохранение пути в конфиг
- [ ] Валидация файла при загрузке

## 12. Настраиваемый tick sound [LOW]
- [ ] Заменить `tick_enabled: bool` на `tick_mode: TickMode`
- [ ] Варианты: Off, EverySecond, Every5Seconds, Last10Seconds, LastMinute
- [ ] UI для выбора режима в Settings

## 13. Фокус-режим (Do Not Disturb) [LOW]
- [ ] Опция "Блокировать уведомления ОС во время фокуса"
- [ ] Windows: `SetNotificationMode` / Focus Assist API
- [ ] Linux: `dunstctl set-paused true/false`
- [ ] Автоматическое включение/выключение при start/end work-сессии

## 14. macOS поддержка [LOW]
- [ ] Платформенные хуки для macOS (уведомления, window effects)
- [ ] Global hotkeys на macOS
- [ ] System tray (menu bar) на macOS
- [ ] Тестирование и CI для macOS

## 15. Кнопка Reset на главном экране [LOW]
- [ ] Добавить Reset через long-press на кнопке Stop
- [ ] Или: третья кнопка Reset рядом с Play/Skip
- [ ] Или: контекстное меню на таймере (right-click)
