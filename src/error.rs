//! Custom error types for PomodoRust
//!
//! This module provides strongly-typed errors for better error handling
//! across the application, replacing generic String errors.

use std::fmt;
use std::io;
use std::path::PathBuf;

/// Result type alias using PomodoRust errors
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for PomodoRust
#[derive(Debug)]
pub enum Error {
    /// Configuration-related errors
    Config(ConfigError),
    /// Database-related errors
    Database(DatabaseError),
    /// Platform-specific errors (Windows, notifications, etc.)
    Platform(PlatformError),
    /// Audio-related errors
    Audio(AudioError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(e) => write!(f, "Configuration error: {}", e),
            Error::Database(e) => write!(f, "Database error: {}", e),
            Error::Platform(e) => write!(f, "Platform error: {}", e),
            Error::Audio(e) => write!(f, "Audio error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Config(e) => Some(e),
            Error::Database(e) => Some(e),
            Error::Platform(e) => Some(e),
            Error::Audio(e) => Some(e),
        }
    }
}

// Conversion implementations
impl From<ConfigError> for Error {
    fn from(e: ConfigError) -> Self {
        Error::Config(e)
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Error::Database(e)
    }
}

impl From<PlatformError> for Error {
    fn from(e: PlatformError) -> Self {
        Error::Platform(e)
    }
}

impl From<AudioError> for Error {
    fn from(e: AudioError) -> Self {
        Error::Audio(e)
    }
}

/// Configuration-related errors
#[derive(Debug)]
pub enum ConfigError {
    /// Failed to determine config directory
    DirectoryNotFound,
    /// Failed to create config directory
    DirectoryCreation { path: PathBuf, source: io::Error },
    /// Failed to read config file
    ReadFile { path: PathBuf, source: io::Error },
    /// Failed to write config file
    WriteFile { path: PathBuf, source: io::Error },
    /// Failed to parse config file
    Parse { path: PathBuf, message: String },
    /// Failed to serialize config
    Serialize { message: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::DirectoryNotFound => {
                write!(f, "could not determine configuration directory")
            }
            ConfigError::DirectoryCreation { path, source } => {
                write!(f, "failed to create directory {:?}: {}", path, source)
            }
            ConfigError::ReadFile { path, source } => {
                write!(f, "failed to read {:?}: {}", path, source)
            }
            ConfigError::WriteFile { path, source } => {
                write!(f, "failed to write {:?}: {}", path, source)
            }
            ConfigError::Parse { path, message } => {
                write!(f, "failed to parse {:?}: {}", path, message)
            }
            ConfigError::Serialize { message } => {
                write!(f, "failed to serialize config: {}", message)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::DirectoryCreation { source, .. } => Some(source),
            ConfigError::ReadFile { source, .. } => Some(source),
            ConfigError::WriteFile { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// Database-related errors
#[derive(Debug)]
pub enum DatabaseError {
    /// Failed to determine database path
    PathNotFound,
    /// Failed to create database directory
    DirectoryCreation { path: PathBuf, source: io::Error },
    /// SQLite error
    Sqlite(rusqlite::Error),
    /// Query returned no rows when one was expected
    NotFound { table: &'static str },
    /// Data integrity error
    Integrity { message: String },
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::PathNotFound => {
                write!(f, "could not determine database path")
            }
            DatabaseError::DirectoryCreation { path, source } => {
                write!(
                    f,
                    "failed to create database directory {:?}: {}",
                    path, source
                )
            }
            DatabaseError::Sqlite(e) => write!(f, "SQLite error: {}", e),
            DatabaseError::NotFound { table } => {
                write!(f, "no record found in table '{}'", table)
            }
            DatabaseError::Integrity { message } => {
                write!(f, "data integrity error: {}", message)
            }
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DatabaseError::DirectoryCreation { source, .. } => Some(source),
            DatabaseError::Sqlite(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(e: rusqlite::Error) -> Self {
        DatabaseError::Sqlite(e)
    }
}

/// Platform-specific errors
#[derive(Debug)]
pub enum PlatformError {
    /// Windows registry error
    Registry {
        operation: &'static str,
        message: String,
    },
    /// DWM (Desktop Window Manager) error
    Dwm {
        operation: &'static str,
        message: String,
    },
    /// Notification error
    Notification { message: String },
    /// Executable path not found
    ExecutablePath { source: io::Error },
    /// Feature not supported on this platform
    Unsupported { feature: &'static str },
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::Registry { operation, message } => {
                write!(f, "registry {} failed: {}", operation, message)
            }
            PlatformError::Dwm { operation, message } => {
                write!(f, "DWM {} failed: {}", operation, message)
            }
            PlatformError::Notification { message } => {
                write!(f, "notification failed: {}", message)
            }
            PlatformError::ExecutablePath { source } => {
                write!(f, "failed to get executable path: {}", source)
            }
            PlatformError::Unsupported { feature } => {
                write!(f, "'{}' is not supported on this platform", feature)
            }
        }
    }
}

impl std::error::Error for PlatformError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PlatformError::ExecutablePath { source } => Some(source),
            _ => None,
        }
    }
}

/// Audio-related errors
#[derive(Debug)]
pub enum AudioError {
    /// Failed to create audio output stream
    StreamCreation { message: String },
    /// Failed to play sound
    Playback {
        sound: &'static str,
        message: String,
    },
    /// Audio device not available
    DeviceNotAvailable,
}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioError::StreamCreation { message } => {
                write!(f, "failed to create audio stream: {}", message)
            }
            AudioError::Playback { sound, message } => {
                write!(f, "failed to play '{}': {}", sound, message)
            }
            AudioError::DeviceNotAvailable => {
                write!(f, "no audio device available")
            }
        }
    }
}

impl std::error::Error for AudioError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Config(ConfigError::DirectoryNotFound);
        assert!(err.to_string().contains("configuration directory"));
    }

    #[test]
    fn test_database_error_from_sqlite() {
        let sqlite_err = rusqlite::Error::QueryReturnedNoRows;
        let db_err: DatabaseError = sqlite_err.into();
        assert!(matches!(db_err, DatabaseError::Sqlite(_)));
    }

    #[test]
    fn test_platform_error_display() {
        let err = PlatformError::Unsupported {
            feature: "autostart",
        };
        assert!(err.to_string().contains("not supported"));
    }
}
