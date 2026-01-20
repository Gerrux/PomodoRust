//! Global hotkeys support for Linux
//!
//! Registers system-wide hotkeys using the global-hotkey crate
//! which supports both X11 and Wayland (via xdg-desktop-portal).

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};

/// Hotkey actions that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    Toggle,
    Skip,
    Reset,
}

/// Parse a hotkey string like "Ctrl+Alt+Space" into a HotKey
fn parse_hotkey(hotkey_str: &str) -> Option<HotKey> {
    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in parts {
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "ALT" => modifiers |= Modifiers::ALT,
            "SHIFT" => modifiers |= Modifiers::SHIFT,
            "WIN" | "SUPER" | "META" => modifiers |= Modifiers::SUPER,
            "SPACE" => key_code = Some(Code::Space),
            "ENTER" | "RETURN" => key_code = Some(Code::Enter),
            "ESC" | "ESCAPE" => key_code = Some(Code::Escape),
            "TAB" => key_code = Some(Code::Tab),
            "BACKSPACE" => key_code = Some(Code::Backspace),
            // Single letter keys (A-Z)
            s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() => {
                let c = s.chars().next().unwrap().to_ascii_uppercase();
                key_code = match c {
                    'A' => Some(Code::KeyA),
                    'B' => Some(Code::KeyB),
                    'C' => Some(Code::KeyC),
                    'D' => Some(Code::KeyD),
                    'E' => Some(Code::KeyE),
                    'F' => Some(Code::KeyF),
                    'G' => Some(Code::KeyG),
                    'H' => Some(Code::KeyH),
                    'I' => Some(Code::KeyI),
                    'J' => Some(Code::KeyJ),
                    'K' => Some(Code::KeyK),
                    'L' => Some(Code::KeyL),
                    'M' => Some(Code::KeyM),
                    'N' => Some(Code::KeyN),
                    'O' => Some(Code::KeyO),
                    'P' => Some(Code::KeyP),
                    'Q' => Some(Code::KeyQ),
                    'R' => Some(Code::KeyR),
                    'S' => Some(Code::KeyS),
                    'T' => Some(Code::KeyT),
                    'U' => Some(Code::KeyU),
                    'V' => Some(Code::KeyV),
                    'W' => Some(Code::KeyW),
                    'X' => Some(Code::KeyX),
                    'Y' => Some(Code::KeyY),
                    'Z' => Some(Code::KeyZ),
                    _ => None,
                };
            }
            // Number keys (0-9)
            s if s.len() == 1 && s.chars().next().unwrap().is_ascii_digit() => {
                let c = s.chars().next().unwrap();
                key_code = match c {
                    '0' => Some(Code::Digit0),
                    '1' => Some(Code::Digit1),
                    '2' => Some(Code::Digit2),
                    '3' => Some(Code::Digit3),
                    '4' => Some(Code::Digit4),
                    '5' => Some(Code::Digit5),
                    '6' => Some(Code::Digit6),
                    '7' => Some(Code::Digit7),
                    '8' => Some(Code::Digit8),
                    '9' => Some(Code::Digit9),
                    _ => None,
                };
            }
            // Function keys (F1-F12)
            s if s.starts_with('F') && s.len() <= 3 => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    key_code = match n {
                        1 => Some(Code::F1),
                        2 => Some(Code::F2),
                        3 => Some(Code::F3),
                        4 => Some(Code::F4),
                        5 => Some(Code::F5),
                        6 => Some(Code::F6),
                        7 => Some(Code::F7),
                        8 => Some(Code::F8),
                        9 => Some(Code::F9),
                        10 => Some(Code::F10),
                        11 => Some(Code::F11),
                        12 => Some(Code::F12),
                        _ => None,
                    };
                }
            }
            _ => {
                tracing::warn!("Unknown hotkey part: {}", part);
            }
        }
    }

    key_code.map(|code| HotKey::new(Some(modifiers), code))
}

/// Global hotkey manager
pub struct HotkeyManager {
    /// Channel to receive hotkey events
    event_rx: Option<Receiver<HotkeyAction>>,
    /// Sender for the hotkey thread
    event_tx: Sender<HotkeyAction>,
    /// Thread handle
    thread_handle: Option<thread::JoinHandle<()>>,
    /// Running flag
    running: Arc<Mutex<bool>>,
    /// Hotkey configuration to register
    hotkey_config: Arc<Mutex<Vec<(HotkeyAction, String)>>>,
}

impl HotkeyManager {
    /// Create a new hotkey manager
    pub fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            event_rx: Some(event_rx),
            event_tx,
            thread_handle: None,
            running: Arc::new(Mutex::new(false)),
            hotkey_config: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Take the event receiver (can only be called once)
    pub fn take_receiver(&mut self) -> Option<Receiver<HotkeyAction>> {
        self.event_rx.take()
    }

    /// Register hotkeys and start listening
    pub fn start(&mut self, toggle: &str, skip: &str, reset: &str) {
        // Store hotkey configuration
        {
            let mut config = self.hotkey_config.lock().unwrap();
            config.clear();
            config.push((HotkeyAction::Toggle, toggle.to_string()));
            config.push((HotkeyAction::Skip, skip.to_string()));
            config.push((HotkeyAction::Reset, reset.to_string()));
        }

        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let hotkey_config = self.hotkey_config.clone();

        // Mark as running
        {
            let mut r = running.lock().unwrap();
            *r = true;
        }

        // Start hotkey listener thread
        let handle = thread::spawn(move || {
            Self::hotkey_loop(event_tx, running, hotkey_config);
        });

        self.thread_handle = Some(handle);
        tracing::info!("Hotkey manager started");
    }

    /// Stop the hotkey manager
    pub fn stop(&mut self) {
        {
            let mut r = self.running.lock().unwrap();
            *r = false;
        }

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        tracing::info!("Hotkey manager stopped");
    }

    /// The main hotkey listening loop (runs in a separate thread)
    fn hotkey_loop(
        event_tx: Sender<HotkeyAction>,
        running: Arc<Mutex<bool>>,
        hotkey_config: Arc<Mutex<Vec<(HotkeyAction, String)>>>,
    ) {
        // Create the hotkey manager (must be done in the thread that will process events)
        let manager = match GlobalHotKeyManager::new() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to create GlobalHotKeyManager: {}", e);
                return;
            }
        };

        // Parse and register hotkeys
        let mut hotkey_map: HashMap<u32, HotkeyAction> = HashMap::new();
        {
            let config = hotkey_config.lock().unwrap();
            for (action, hotkey_str) in config.iter() {
                if let Some(hotkey) = parse_hotkey(hotkey_str) {
                    match manager.register(hotkey) {
                        Ok(()) => {
                            hotkey_map.insert(hotkey.id(), *action);
                            tracing::info!("Registered hotkey for {:?}: {}", action, hotkey_str);
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to register hotkey for {:?} ({}): {}",
                                action,
                                hotkey_str,
                                e
                            );
                        }
                    }
                } else {
                    tracing::warn!("Failed to parse hotkey: {}", hotkey_str);
                }
            }
        }

        // Event receiver from global-hotkey crate
        let receiver = GlobalHotKeyEvent::receiver();

        // Event loop
        loop {
            // Check if we should stop
            {
                if let Ok(r) = running.lock() {
                    if !*r {
                        break;
                    }
                }
            }

            // Check for hotkey events with timeout
            if let Ok(event) = receiver.try_recv() {
                if let Some(action) = hotkey_map.get(&event.id) {
                    tracing::debug!("Hotkey triggered: {:?}", action);
                    let _ = event_tx.send(*action);
                }
            }

            // Small sleep to avoid busy-waiting
            thread::sleep(std::time::Duration::from_millis(50));
        }

        // Unregister hotkeys
        for (id, _) in hotkey_map.iter() {
            // We need to reconstruct the HotKey to unregister, but since we don't store them,
            // just let them be cleaned up when the manager is dropped
            let _ = id;
        }

        tracing::info!("Hotkey loop ended");
    }

    /// Check if manager is running
    pub fn is_running(&self) -> bool {
        self.running.lock().map(|r| *r).unwrap_or(false)
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.stop();
    }
}
