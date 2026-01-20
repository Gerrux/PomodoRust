#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! PomodoRust - A modern, lightweight Pomodoro timer
//!
//! Run without arguments to start the GUI.
//! Run with a command (e.g., `pomodorust status`) to use CLI mode.

use eframe::egui;
use pomodorust::data::Config;
use pomodorust::ipc::{IpcCommand, IpcResponse, IpcStats, IpcStatus};
use pomodorust::{is_app_running, send_command, PomodoRustApp};
use std::env;

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;

const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Command {
    Start { session: Option<String> },
    Pause,
    Resume,
    Toggle,
    Stop,
    Skip,
    Status,
    Stats { period: String },
    Ping,
}

fn print_help() {
    println!("PomodoRust v{} - A modern Pomodoro timer", VERSION);
    println!();
    println!("USAGE: pomodorust [COMMAND]");
    println!();
    println!("COMMANDS:");
    println!("  start [-s <type>]   Start timer (type: work, short, long)");
    println!("  pause               Pause the timer");
    println!("  resume              Resume the timer");
    println!("  toggle              Toggle start/pause");
    println!("  stop                Stop and reset the timer");
    println!("  skip                Skip to next session");
    println!("  status              Get current timer status");
    println!("  stats [-p <period>] Get statistics (period: today, week, all)");
    println!("  ping                Check if GUI is running");
    println!();
    println!("Run without arguments to start the GUI.");
}

fn parse_args() -> Option<Command> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return None;
    }

    let cmd = args[1].to_lowercase();

    match cmd.as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            std::process::exit(0);
        }
        "-v" | "--version" | "version" => {
            println!("pomodorust {}", VERSION);
            std::process::exit(0);
        }
        "start" => {
            let session = parse_option(&args[2..], &["-s", "--session"]);
            Some(Command::Start { session })
        }
        "pause" => Some(Command::Pause),
        "resume" => Some(Command::Resume),
        "toggle" => Some(Command::Toggle),
        "stop" => Some(Command::Stop),
        "skip" => Some(Command::Skip),
        "status" => Some(Command::Status),
        "stats" => {
            let period = parse_option(&args[2..], &["-p", "--period"])
                .unwrap_or_else(|| "today".to_string());
            Some(Command::Stats { period })
        }
        "ping" => Some(Command::Ping),
        _ => {
            eprintln!("Unknown command: {}", cmd);
            eprintln!("Run 'pomodorust --help' for usage.");
            std::process::exit(1);
        }
    }
}

fn parse_option(args: &[String], flags: &[&str]) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if flags.contains(&arg.as_str()) {
            return args.get(i + 1).cloned();
        }
    }
    None
}

fn main() {
    // Parse CLI arguments
    if let Some(command) = parse_args() {
        run_cli(command);
        return;
    }

    // Otherwise, run GUI
    run_gui();
}

/// Run the CLI mode
fn run_cli(command: Command) {
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
    if !matches!(command, Command::Ping) && !is_app_running() {
        eprintln!("Error: PomodoRust GUI is not running. Start the app first.");
        std::process::exit(1);
    }

    let ipc_command = match command {
        Command::Start { session } => IpcCommand::Start {
            session_type: session,
        },
        Command::Pause => IpcCommand::Pause,
        Command::Resume => IpcCommand::Resume,
        Command::Toggle => IpcCommand::Toggle,
        Command::Stop => IpcCommand::Stop,
        Command::Skip => IpcCommand::Skip,
        Command::Status => IpcCommand::Status,
        Command::Stats { period } => IpcCommand::Stats { period },
        Command::Ping => IpcCommand::Ping,
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
        Box::new(move |cc| Ok(Box::new(PomodoRustApp::with_config(cc, config)))),
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
