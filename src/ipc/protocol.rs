//! IPC Protocol definitions
//!
//! JSON-based protocol for communication between GUI and CLI.

use serde::{Deserialize, Serialize};

/// Commands that can be sent to the GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum IpcCommand {
    /// Start the timer (optionally specify session type)
    Start {
        #[serde(skip_serializing_if = "Option::is_none")]
        session_type: Option<String>,
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
    /// Get current status
    Status,
    /// Get statistics
    Stats {
        #[serde(default)]
        period: String, // today, week, all
    },
    /// Ping to check if server is running
    Ping,
}

/// Response from the GUI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IpcResponse {
    /// Command executed successfully
    Ok {
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
    },
    /// Current timer status
    Status(IpcStatus),
    /// Statistics data
    Stats(IpcStats),
    /// Pong response
    Pong,
    /// Error occurred
    Error { message: String },
}

/// Timer status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcStatus {
    /// Current state: idle, running, paused, completed
    pub state: String,
    /// Session type: work, short_break, long_break
    pub session_type: String,
    /// Remaining time in seconds
    pub remaining_secs: u64,
    /// Remaining time formatted (MM:SS)
    pub remaining_formatted: String,
    /// Progress (0.0 to 1.0)
    pub progress: f32,
    /// Current session number in cycle
    pub current_session: u32,
    /// Total sessions in cycle
    pub total_sessions: u32,
    /// Total duration of current session in seconds
    pub total_duration_secs: u64,
}

/// Statistics information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcStats {
    /// Period: today, week, all
    pub period: String,
    /// Work hours
    pub hours: f32,
    /// Completed pomodoros
    pub pomodoros: i32,
    /// Current streak (days)
    pub current_streak: i32,
    /// Longest streak (days)
    pub longest_streak: i32,
    /// Daily goal target
    pub daily_goal: u32,
    /// Today's pomodoros (for goal progress)
    pub today_pomodoros: i32,
}

impl IpcCommand {
    /// Parse command from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Convert command to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

impl IpcResponse {
    /// Create an OK response
    pub fn ok() -> Self {
        Self::Ok { message: None }
    }

    /// Create an OK response with message
    pub fn ok_with_message(msg: impl Into<String>) -> Self {
        Self::Ok {
            message: Some(msg.into()),
        }
    }

    /// Create an error response
    pub fn error(msg: impl Into<String>) -> Self {
        Self::Error {
            message: msg.into(),
        }
    }

    /// Convert response to JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }

    /// Parse response from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let cmd = IpcCommand::Start { session_type: None };
        let json = cmd.to_json();
        assert!(json.contains("start"));

        let cmd = IpcCommand::Status;
        let json = cmd.to_json();
        assert!(json.contains("status"));
    }

    #[test]
    fn test_response_serialization() {
        let resp = IpcResponse::ok();
        let json = resp.to_json();
        assert!(json.contains("ok"));

        let resp = IpcResponse::error("test error");
        let json = resp.to_json();
        assert!(json.contains("error"));
        assert!(json.contains("test error"));
    }
}
