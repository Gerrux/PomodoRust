#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use pomodorust::data::Config;
use pomodorust::PomodoRustApp;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
#[cfg(windows)]
use windows::core::PCWSTR;

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "pomodorust=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting PomodoRust...");

    // Load config early to apply window settings
    let config = Config::load();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([360.0, 480.0])
        .with_min_inner_size([320.0, 400.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(true)
        .with_icon(load_icon());

    if config.window.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    // Spawn thread to apply Windows DWM effects after window creation
    #[cfg(windows)]
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(100));
        unsafe {
            let title: Vec<u16> = "PomodoRust\0".encode_utf16().collect();
            if let Ok(hwnd) = FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) {
                if !hwnd.is_invalid() {
                    pomodorust::platform::apply_window_effects(hwnd.0 as isize);
                }
            }
        }
    });

    eframe::run_native(
        "PomodoRust",
        options,
        Box::new(|cc| Ok(Box::new(PomodoRustApp::new(cc)))),
    )
}

fn load_icon() -> egui::IconData {
    // Load icon from assets/icon.png
    let icon_bytes = include_bytes!("../assets/icon.png");

    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();

    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    egui::IconData {
        rgba,
        width,
        height,
    }
}
