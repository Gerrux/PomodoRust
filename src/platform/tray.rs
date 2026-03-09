//! System tray integration
//!
//! Provides a system tray icon with context menu for controlling the timer
//! when the main window is minimized or hidden.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{TrayIcon, TrayIconBuilder};

/// Actions triggered by clicking tray menu items or the tray icon itself
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Toggle,
    Skip,
    ShowWindow,
    Quit,
}

/// System tray icon with context menu
pub struct SystemTray {
    _tray_icon: TrayIcon,
    toggle_item: MenuItem,
    toggle_id: tray_icon::menu::MenuId,
    skip_id: tray_icon::menu::MenuId,
    show_id: tray_icon::menu::MenuId,
    quit_id: tray_icon::menu::MenuId,
    last_tooltip: String,
    /// Receiver for actions from the background polling thread
    action_rx: Option<mpsc::Receiver<TrayAction>>,
    /// Flag to stop the background polling thread
    polling_active: Arc<AtomicBool>,
    /// Flag to enable periodic wakeups (when hidden to tray)
    periodic_wakeup: Arc<AtomicBool>,
}

impl SystemTray {
    /// Create a new system tray icon with context menu.
    /// Must be called on the main thread before the event loop starts.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let icon = load_tray_icon()?;

        let toggle_item = MenuItem::new("Старт", true, None);
        let skip_item = MenuItem::new("Пропустить", true, None);
        let show_item = MenuItem::new("Показать окно", true, None);
        let quit_item = MenuItem::new("Выход", true, None);

        let toggle_id = toggle_item.id().clone();
        let skip_id = skip_item.id().clone();
        let show_id = show_item.id().clone();
        let quit_id = quit_item.id().clone();

        let menu = Menu::new();
        menu.append(&toggle_item)?;
        menu.append(&skip_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&show_item)?;
        menu.append(&PredefinedMenuItem::separator())?;
        menu.append(&quit_item)?;

        let tray_icon = TrayIconBuilder::new()
            .with_icon(icon)
            .with_tooltip("PomodoRust")
            .with_menu(Box::new(menu))
            .build()?;

        Ok(Self {
            _tray_icon: tray_icon,
            toggle_item,
            toggle_id,
            skip_id,
            show_id,
            quit_id,
            last_tooltip: String::new(),
            action_rx: None,
            polling_active: Arc::new(AtomicBool::new(false)),
            periodic_wakeup: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Start a background thread that polls tray events and wakes the main
    /// event loop via `ctx.request_repaint()`.  Idempotent — safe to call
    /// every frame; only the first call spawns a thread.
    pub fn start_polling(&mut self, ctx: egui::Context) {
        if self.polling_active.load(Ordering::Relaxed) {
            return;
        }

        let (tx, rx) = mpsc::channel();
        self.action_rx = Some(rx);
        self.polling_active.store(true, Ordering::Relaxed);

        let active = self.polling_active.clone();
        let periodic = self.periodic_wakeup.clone();
        let toggle_id = self.toggle_id.clone();
        let skip_id = self.skip_id.clone();
        let show_id = self.show_id.clone();
        let quit_id = self.quit_id.clone();

        std::thread::Builder::new()
            .name("tray-poll".into())
            .spawn(move || {
                let mut tick: u32 = 0;
                while active.load(Ordering::Relaxed) {
                    let mut got_event = false;

                    // Drain menu events
                    while let Ok(event) = MenuEvent::receiver().try_recv() {
                        let action = if event.id == toggle_id {
                            Some(TrayAction::Toggle)
                        } else if event.id == skip_id {
                            Some(TrayAction::Skip)
                        } else if event.id == show_id {
                            Some(TrayAction::ShowWindow)
                        } else if event.id == quit_id {
                            Some(TrayAction::Quit)
                        } else {
                            None
                        };
                        if let Some(a) = action {
                            // ShowWindow and Quit must be handled here directly
                            // because eframe's Visible(false) blocks viewport commands,
                            // making it impossible to show or close the window from
                            // the main thread's update() loop.
                            #[cfg(windows)]
                            match a {
                                TrayAction::ShowWindow => {
                                    crate::platform::show_pomodorust_window();
                                }
                                TrayAction::Quit => {
                                    crate::platform::force_quit_app();
                                }
                                _ => {}
                            }
                            let _ = tx.send(a);
                            got_event = true;
                        }
                    }

                    // Drain tray icon click events
                    while let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
                        match event {
                            tray_icon::TrayIconEvent::Click {
                                button: tray_icon::MouseButton::Left,
                                button_state: tray_icon::MouseButtonState::Up,
                                ..
                            }
                            | tray_icon::TrayIconEvent::DoubleClick {
                                button: tray_icon::MouseButton::Left,
                                ..
                            } => {
                                #[cfg(windows)]
                                crate::platform::show_pomodorust_window();
                                let _ = tx.send(TrayAction::ShowWindow);
                                got_event = true;
                            }
                            _ => {}
                        }
                    }

                    if got_event {
                        ctx.request_repaint();
                    }

                    // Periodic wakeup when hidden (for timer/tooltip updates)
                    tick += 1;
                    if tick >= 50 && periodic.load(Ordering::Relaxed) {
                        tick = 0;
                        ctx.request_repaint();
                    }

                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
            })
            .ok();
    }

    /// Enable or disable periodic ~500 ms wakeups (used when hidden to tray).
    pub fn set_periodic_wakeup(&self, enabled: bool) {
        self.periodic_wakeup.store(enabled, Ordering::Relaxed);
    }

    /// Poll for tray actions.  After `start_polling` this reads from the
    /// background thread's channel; before that it falls back to direct polling.
    pub fn poll_action(&self) -> Option<TrayAction> {
        if let Some(ref rx) = self.action_rx {
            rx.try_recv().ok()
        } else {
            self.poll_action_direct()
        }
    }

    /// Direct (synchronous) polling — used before `start_polling` is called.
    fn poll_action_direct(&self) -> Option<TrayAction> {
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.toggle_id {
                return Some(TrayAction::Toggle);
            } else if event.id == self.skip_id {
                return Some(TrayAction::Skip);
            } else if event.id == self.show_id {
                return Some(TrayAction::ShowWindow);
            } else if event.id == self.quit_id {
                return Some(TrayAction::Quit);
            }
        }
        if let Ok(event) = tray_icon::TrayIconEvent::receiver().try_recv() {
            match event {
                tray_icon::TrayIconEvent::Click {
                    button: tray_icon::MouseButton::Left,
                    button_state: tray_icon::MouseButtonState::Up,
                    ..
                }
                | tray_icon::TrayIconEvent::DoubleClick {
                    button: tray_icon::MouseButton::Left,
                    ..
                } => {
                    return Some(TrayAction::ShowWindow);
                }
                _ => {}
            }
        }
        None
    }

    /// Update the tray tooltip text (only sends OS call if changed).
    pub fn update_tooltip(&mut self, tooltip: &str) {
        if self.last_tooltip != tooltip {
            self.last_tooltip = tooltip.to_string();
            let _ = self._tray_icon.set_tooltip(Some(tooltip));
        }
    }

    /// Update the toggle menu item label (Start/Pause/Resume).
    pub fn update_toggle_label(&self, label: &str) {
        self.toggle_item.set_text(label);
    }
}

impl Drop for SystemTray {
    fn drop(&mut self) {
        self.polling_active.store(false, Ordering::Relaxed);
    }
}

fn load_tray_icon() -> Result<tray_icon::Icon, Box<dyn std::error::Error>> {
    let icon_bytes = include_bytes!("../../assets/icon.png");
    let image = image::load_from_memory(icon_bytes)?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    let icon = tray_icon::Icon::from_rgba(rgba, width, height)?;
    Ok(icon)
}
