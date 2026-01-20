#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! PomodoRust - A modern, lightweight Pomodoro timer
//!
//! Run without arguments to start the GUI.
//! Run with a command (e.g., `pomodorust status`) to use CLI mode.

use clap::{Parser, Subcommand};
use eframe::egui;
use pomodorust::data::Config;
use pomodorust::ipc::{IpcCommand, IpcResponse, IpcStats, IpcStatus};
use pomodorust::{is_app_running, send_command, PomodoRustApp};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

/// CLI argument parser
#[derive(Parser)]
#[command(name = "pomodorust")]
#[command(author = "gerrux")]
#[command(version)]
#[command(about = "PomodoRust - A modern Pomodoro timer", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the timer (optionally specify session type)
    Start {
        /// Session type: work, short, long
        #[arg(short, long)]
        session: Option<String>,
    },
    /// Pause the timer
    Pause,
    /// Resume the timer
    Resume,
    /// Toggle start/pause
    Toggle,
    /// Stop and reset the timer
    Stop,
    /// Skip to next session
    Skip,
    /// Get current timer status
    Status,
    /// Get statistics
    Stats {
        /// Period: today, week, all (default: today)
        #[arg(short, long, default_value = "today")]
        period: String,
    },
    /// Check if PomodoRust GUI is running
    Ping,
}

fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // If a command is provided, run in CLI mode
    if let Some(command) = cli.command {
        run_cli(command);
        return;
    }

    // Otherwise, run GUI
    run_gui();
}

/// Run the CLI mode
fn run_cli(command: Commands) {
    // Attach to parent console on Windows (needed because of windows_subsystem = "windows")
    #[cfg(windows)]
    unsafe {
        use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        if AttachConsole(ATTACH_PARENT_PROCESS).is_ok() {
            // Move to new line since cursor is at end of command line
            println!();
        }
    }

    // Check if app is running for non-ping commands
    if !matches!(command, Commands::Ping) && !is_app_running() {
        eprintln!("Error: PomodoRust GUI is not running. Start the app first.");
        std::process::exit(1);
    }

    let ipc_command = match command {
        Commands::Start { session } => IpcCommand::Start {
            session_type: session,
        },
        Commands::Pause => IpcCommand::Pause,
        Commands::Resume => IpcCommand::Resume,
        Commands::Toggle => IpcCommand::Toggle,
        Commands::Stop => IpcCommand::Stop,
        Commands::Skip => IpcCommand::Skip,
        Commands::Status => IpcCommand::Status,
        Commands::Stats { period } => IpcCommand::Stats { period },
        Commands::Ping => IpcCommand::Ping,
    };

    match send_command(&ipc_command) {
        Ok(response) => handle_cli_response(response),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn handle_cli_response(response: IpcResponse) {
    match response {
        IpcResponse::Ok { message } => {
            if let Some(msg) = message {
                println!("{}", msg);
            } else {
                println!("OK");
            }
        }
        IpcResponse::Status(status) => {
            print_status(&status);
        }
        IpcResponse::Stats(stats) => {
            print_stats(&stats);
        }
        IpcResponse::Pong => {
            println!("PomodoRust is running");
        }
        IpcResponse::Error { message } => {
            eprintln!("Error: {}", message);
            std::process::exit(1);
        }
    }
}

fn print_status(status: &IpcStatus) {
    let state_icon = match status.state.as_str() {
        "running" => ">>",
        "paused" => "||",
        "completed" => "**",
        _ => "--",
    };

    let session_label = match status.session_type.as_str() {
        "work" => "Focus",
        "short_break" => "Short Break",
        "long_break" => "Long Break",
        _ => &status.session_type,
    };

    println!(
        "{} {} - {}",
        state_icon, session_label, status.remaining_formatted
    );
    println!(
        "   Session {}/{} | Progress: {:.0}%",
        status.current_session,
        status.total_sessions,
        status.progress * 100.0
    );
}

fn print_stats(stats: &IpcStats) {
    let period_label = match stats.period.as_str() {
        "today" => "Today",
        "week" => "This Week",
        "all" => "All Time",
        _ => &stats.period,
    };

    println!("=== {} ===", period_label);
    println!("Focus Time: {:.1}h", stats.hours);
    println!("Pomodoros:  {}", stats.pomodoros);

    if stats.period == "today" {
        let progress = if stats.daily_goal > 0 {
            (stats.today_pomodoros as f32 / stats.daily_goal as f32 * 100.0).min(100.0)
        } else {
            100.0
        };
        println!(
            "Daily Goal: {}/{} ({:.0}%)",
            stats.today_pomodoros, stats.daily_goal, progress
        );
    }

    println!(
        "Streak:     {} days (best: {})",
        stats.current_streak, stats.longest_streak
    );
}

/// Run the GUI mode
fn run_gui() {
    // Check if another instance is already running
    if is_app_running() {
        // Try to show existing window and exit
        #[cfg(windows)]
        {
            pomodorust::platform::show_pomodorust_window();
        }
        return;
    }

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
        .with_inner_size([config.window.width, config.window.height])
        .with_min_inner_size([320.0, 375.0])
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(true)
        .with_icon(load_icon());

    // Restore window position if saved
    if let (Some(x), Some(y)) = (config.window.x, config.window.y) {
        viewport = viewport.with_position([x, y]);
    }

    if config.window.always_on_top {
        viewport = viewport.with_always_on_top();
    }

    if config.window.maximized {
        viewport = viewport.with_maximized(true);
    }

    // Only center if no position was saved
    let centered = config.window.x.is_none() || config.window.y.is_none();

    let options = eframe::NativeOptions {
        viewport,
        centered,
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

    let _ = eframe::run_native(
        "PomodoRust",
        options,
        Box::new(|cc| Ok(Box::new(PomodoRustApp::new(cc)))),
    );
}

fn load_icon() -> egui::IconData {
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
