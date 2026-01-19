//! IPC Server for handling CLI commands
//!
//! Runs in a separate thread and communicates with the main app via channels.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use super::protocol::{IpcCommand, IpcResponse};
use super::ipc_address;

/// IPC Server that listens for CLI commands
pub struct IpcServer {
    /// Channel to send commands to the app
    command_tx: Sender<IpcCommand>,
    /// Channel to receive commands in the app
    command_rx: Option<Receiver<IpcCommand>>,
    /// Shared state for responses
    response_state: Arc<Mutex<Option<IpcResponse>>>,
    /// Server thread handle
    _handle: Option<thread::JoinHandle<()>>,
    /// Flag to indicate if server is running
    running: Arc<Mutex<bool>>,
}

impl IpcServer {
    /// Create a new IPC server (not started yet)
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        Self {
            command_tx,
            command_rx: Some(command_rx),
            response_state: Arc::new(Mutex::new(None)),
            _handle: None,
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Take the command receiver (can only be called once)
    pub fn take_receiver(&mut self) -> Option<Receiver<IpcCommand>> {
        self.command_rx.take()
    }

    /// Set the response to send back to the client
    pub fn set_response(&self, response: IpcResponse) {
        if let Ok(mut state) = self.response_state.lock() {
            *state = Some(response);
        }
    }

    /// Start the IPC server in a background thread
    pub fn start(&mut self) {
        let command_tx = self.command_tx.clone();
        let response_state = self.response_state.clone();
        let running = self.running.clone();

        // Mark as running
        if let Ok(mut r) = running.lock() {
            *r = true;
        }

        let handle = thread::spawn(move || {
            Self::server_loop(command_tx, response_state, running);
        });

        self._handle = Some(handle);
        tracing::info!("IPC server started on {}", ipc_address());
    }

    /// Server loop that accepts connections
    fn server_loop(
        command_tx: Sender<IpcCommand>,
        response_state: Arc<Mutex<Option<IpcResponse>>>,
        running: Arc<Mutex<bool>>,
    ) {
        let listener = match TcpListener::bind(ipc_address()) {
            Ok(l) => l,
            Err(e) => {
                tracing::warn!("Failed to bind IPC server: {}. CLI will not be available.", e);
                return;
            }
        };

        // Set non-blocking to allow checking the running flag
        if let Err(e) = listener.set_nonblocking(true) {
            tracing::warn!("Failed to set non-blocking: {}", e);
        }

        loop {
            // Check if we should stop
            if let Ok(r) = running.lock() {
                if !*r {
                    break;
                }
            }

            match listener.accept() {
                Ok((stream, _addr)) => {
                    let tx = command_tx.clone();
                    let state = response_state.clone();

                    // Handle connection in a separate thread
                    thread::spawn(move || {
                        Self::handle_connection(stream, tx, state);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No connection available, sleep a bit
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    tracing::error!("IPC accept error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        tracing::info!("IPC server stopped");
    }

    /// Handle a single client connection
    fn handle_connection(
        mut stream: TcpStream,
        command_tx: Sender<IpcCommand>,
        response_state: Arc<Mutex<Option<IpcResponse>>>,
    ) {
        // Set read timeout
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));

        let mut reader = BufReader::new(stream.try_clone().unwrap_or_else(|_| {
            tracing::error!("Failed to clone stream");
            return stream.try_clone().unwrap();
        }));

        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return, // EOF
            Ok(_) => {
                let line = line.trim();
                tracing::debug!("IPC received: {}", line);

                // Parse command
                match IpcCommand::from_json(line) {
                    Ok(cmd) => {
                        // Handle ping directly
                        if matches!(cmd, IpcCommand::Ping) {
                            let response = IpcResponse::Pong;
                            let _ = writeln!(stream, "{}", response.to_json());
                            return;
                        }

                        // Clear previous response
                        if let Ok(mut state) = response_state.lock() {
                            *state = None;
                        }

                        // Send command to app
                        if command_tx.send(cmd).is_err() {
                            let response = IpcResponse::error("App not responding");
                            let _ = writeln!(stream, "{}", response.to_json());
                            return;
                        }

                        // Wait for response (with timeout)
                        let start = std::time::Instant::now();
                        let timeout = Duration::from_secs(2);

                        loop {
                            if start.elapsed() > timeout {
                                let response = IpcResponse::error("Response timeout");
                                let _ = writeln!(stream, "{}", response.to_json());
                                return;
                            }

                            if let Ok(state) = response_state.lock() {
                                if let Some(response) = state.as_ref() {
                                    let _ = writeln!(stream, "{}", response.to_json());
                                    return;
                                }
                            }

                            thread::sleep(Duration::from_millis(10));
                        }
                    }
                    Err(e) => {
                        let response = IpcResponse::error(format!("Invalid command: {}", e));
                        let _ = writeln!(stream, "{}", response.to_json());
                    }
                }
            }
            Err(e) => {
                tracing::debug!("IPC read error: {}", e);
            }
        }
    }

    /// Stop the IPC server
    pub fn stop(&self) {
        if let Ok(mut r) = self.running.lock() {
            *r = false;
        }
    }
}

impl Default for IpcServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to send a command to the running app
pub fn send_command(command: &IpcCommand) -> Result<IpcResponse, String> {
    use std::io::BufRead;

    let mut stream = TcpStream::connect(ipc_address())
        .map_err(|e| format!("Cannot connect to Pomodorust. Is it running? ({})", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    // Send command
    writeln!(stream, "{}", command.to_json())
        .map_err(|e| format!("Failed to send command: {}", e))?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    IpcResponse::from_json(line.trim())
        .map_err(|e| format!("Invalid response: {}", e))
}

/// Check if the app is running
pub fn is_app_running() -> bool {
    send_command(&IpcCommand::Ping).is_ok()
}
