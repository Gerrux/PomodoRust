//! IPC (Inter-Process Communication) for CLI integration
//!
//! Uses a simple TCP localhost socket for cross-platform compatibility.
//! The GUI app runs a server, CLI sends commands.

mod protocol;
mod server;

pub use protocol::{IpcCommand, IpcResponse, IpcStats, IpcStatus};
pub use server::{is_app_running, send_command, IpcServer};

/// Default IPC port
pub const IPC_PORT: u16 = 19847;

/// Get the IPC address
pub fn ipc_address() -> String {
    format!("127.0.0.1:{}", IPC_PORT)
}
