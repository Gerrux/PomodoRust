#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([360.0, 480.0])
            .with_min_inner_size([320.0, 400.0])
            .with_decorations(false)
            .with_transparent(true)
            .with_resizable(true)
            .with_icon(load_icon()),
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
    // Generate a simple programmatic icon if file doesn't exist
    let size = 64u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    // Create a tomato-colored circle
    let center = size as f32 / 2.0;
    let radius = center - 4.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let idx = ((y * size + x) * 4) as usize;

            if dist <= radius {
                // Gradient from rose-500 (#F43F5E) to rose-600 (#E11D48)
                let t = dist / radius;
                rgba[idx] = (244.0 - t * 19.0) as u8; // R
                rgba[idx + 1] = (63.0 - t * 34.0) as u8; // G
                rgba[idx + 2] = (94.0 - t * 22.0) as u8; // B
                rgba[idx + 3] = 255; // A
            } else if dist <= radius + 2.0 {
                // Anti-aliased edge
                let alpha = ((radius + 2.0 - dist) / 2.0 * 255.0) as u8;
                rgba[idx] = 244;
                rgba[idx + 1] = 63;
                rgba[idx + 2] = 94;
                rgba[idx + 3] = alpha;
            }
        }
    }

    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}
