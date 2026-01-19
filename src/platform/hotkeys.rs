//! Global hotkeys support for Windows
//!
//! Registers system-wide hotkeys and sends events to the app via a channel.

use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT,
    MOD_SHIFT, MOD_WIN, VIRTUAL_KEY, VK_ESCAPE, VK_RETURN, VK_SPACE,
};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, TranslateMessage, MSG, WM_HOTKEY};

/// Hotkey actions that can be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotkeyAction {
    Toggle,
    Skip,
    Reset,
}

impl HotkeyAction {
    /// Get the hotkey ID for Windows API
    fn id(&self) -> i32 {
        match self {
            HotkeyAction::Toggle => 1,
            HotkeyAction::Skip => 2,
            HotkeyAction::Reset => 3,
        }
    }

    /// Get from hotkey ID
    fn from_id(id: i32) -> Option<Self> {
        match id {
            1 => Some(HotkeyAction::Toggle),
            2 => Some(HotkeyAction::Skip),
            3 => Some(HotkeyAction::Reset),
            _ => None,
        }
    }
}

/// Parse a hotkey string like "Ctrl+Alt+Space" into modifiers and key
fn parse_hotkey(hotkey: &str) -> Option<(HOT_KEY_MODIFIERS, VIRTUAL_KEY)> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = MOD_NOREPEAT; // Prevent repeated events when key is held
    let mut key: Option<VIRTUAL_KEY> = None;

    for part in parts {
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= MOD_CONTROL,
            "ALT" => modifiers |= MOD_ALT,
            "SHIFT" => modifiers |= MOD_SHIFT,
            "WIN" | "SUPER" | "META" => modifiers |= MOD_WIN,
            "SPACE" => key = Some(VK_SPACE),
            "ENTER" | "RETURN" => key = Some(VK_RETURN),
            "ESC" | "ESCAPE" => key = Some(VK_ESCAPE),
            // Single letter keys (A-Z)
            s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphabetic() => {
                let c = s.chars().next().unwrap().to_ascii_uppercase();
                key = Some(VIRTUAL_KEY(c as u16));
            }
            // Number keys (0-9)
            s if s.len() == 1 && s.chars().next().unwrap().is_ascii_digit() => {
                let c = s.chars().next().unwrap();
                key = Some(VIRTUAL_KEY(c as u16));
            }
            // Function keys (F1-F12)
            s if s.starts_with('F') && s.len() <= 3 => {
                if let Ok(n) = s[1..].parse::<u16>() {
                    if (1..=12).contains(&n) {
                        // VK_F1 = 0x70
                        key = Some(VIRTUAL_KEY(0x70 + n - 1));
                    }
                }
            }
            _ => {
                tracing::warn!("Unknown hotkey part: {}", part);
            }
        }
    }

    key.map(|k| (modifiers, k))
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
    /// Registered hotkeys
    registered: Arc<Mutex<HashMap<HotkeyAction, (HOT_KEY_MODIFIERS, VIRTUAL_KEY)>>>,
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
            registered: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Take the event receiver (can only be called once)
    pub fn take_receiver(&mut self) -> Option<Receiver<HotkeyAction>> {
        self.event_rx.take()
    }

    /// Register hotkeys and start listening
    pub fn start(&mut self, toggle: &str, skip: &str, reset: &str) {
        // Parse hotkeys
        let hotkeys: Vec<(HotkeyAction, Option<(HOT_KEY_MODIFIERS, VIRTUAL_KEY)>)> = vec![
            (HotkeyAction::Toggle, parse_hotkey(toggle)),
            (HotkeyAction::Skip, parse_hotkey(skip)),
            (HotkeyAction::Reset, parse_hotkey(reset)),
        ];

        // Store valid hotkeys
        {
            let mut registered = self.registered.lock().unwrap();
            for (action, parsed) in &hotkeys {
                if let Some((mods, key)) = parsed {
                    registered.insert(*action, (*mods, *key));
                }
            }
        }

        let event_tx = self.event_tx.clone();
        let running = self.running.clone();
        let registered = self.registered.clone();

        // Mark as running
        {
            let mut r = running.lock().unwrap();
            *r = true;
        }

        // Start hotkey listener thread
        let handle = thread::spawn(move || {
            Self::hotkey_loop(event_tx, running, registered);
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

        // Post a quit message to break the GetMessage loop
        // The thread will exit on its own when running becomes false
        if let Some(handle) = self.thread_handle.take() {
            // We can't easily post WM_QUIT to another thread's message queue,
            // so we just let it timeout on the next GetMessage check
            let _ = handle.join();
        }

        tracing::info!("Hotkey manager stopped");
    }

    /// The main hotkey listening loop (runs in a separate thread)
    fn hotkey_loop(
        event_tx: Sender<HotkeyAction>,
        running: Arc<Mutex<bool>>,
        registered: Arc<Mutex<HashMap<HotkeyAction, (HOT_KEY_MODIFIERS, VIRTUAL_KEY)>>>,
    ) {
        // Register all hotkeys
        {
            let reg = registered.lock().unwrap();
            for (action, (mods, key)) in reg.iter() {
                unsafe {
                    let result = RegisterHotKey(HWND::default(), action.id(), *mods, key.0 as u32);
                    if result.is_err() {
                        tracing::warn!(
                            "Failed to register hotkey for {:?}: {:?}",
                            action,
                            result.err()
                        );
                    } else {
                        tracing::info!("Registered hotkey for {:?}", action);
                    }
                }
            }
        }

        // Message loop
        let mut msg = MSG::default();
        loop {
            // Check if we should stop
            {
                if let Ok(r) = running.lock() {
                    if !*r {
                        break;
                    }
                }
            }

            unsafe {
                // GetMessageW blocks until a message is available
                // We use a timeout approach by checking running flag periodically
                // Since GetMessageW blocks, we'll use PeekMessageW with PM_REMOVE instead
                use windows::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_REMOVE};

                if PeekMessageW(&mut msg, HWND::default(), 0, 0, PM_REMOVE).as_bool() {
                    if msg.message == WM_HOTKEY {
                        let hotkey_id = msg.wParam.0 as i32;
                        if let Some(action) = HotkeyAction::from_id(hotkey_id) {
                            tracing::debug!("Hotkey triggered: {:?}", action);
                            let _ = event_tx.send(action);
                        }
                    }

                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                } else {
                    // No message available, sleep a bit
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }

        // Unregister all hotkeys
        {
            let reg = registered.lock().unwrap();
            for (action, _) in reg.iter() {
                unsafe {
                    let _ = UnregisterHotKey(HWND::default(), action.id());
                }
            }
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
