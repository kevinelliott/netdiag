//! Daemon error types.

use thiserror::Error;

/// Result type for daemon operations.
pub type Result<T> = std::result::Result<T, DaemonError>;

/// Errors that can occur in daemon operations.
#[derive(Debug, Error)]
pub enum DaemonError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Configuration error.
    #[error("Configuration error: {message}")]
    Config {
        /// Error message.
        message: String,
    },

    /// IPC error.
    #[error("IPC error: {message}")]
    Ipc {
        /// Error message.
        message: String,
    },

    /// Service error.
    #[error("Service error: {message}")]
    Service {
        /// Error message.
        message: String,
    },

    /// Scheduler error.
    #[error("Scheduler error: {message}")]
    Scheduler {
        /// Error message.
        message: String,
    },

    /// The daemon is already running.
    #[error("Daemon is already running (PID: {pid})")]
    AlreadyRunning {
        /// Process ID of the running daemon.
        pid: u32,
    },

    /// The daemon is not running.
    #[error("Daemon is not running")]
    NotRunning,

    /// Permission denied.
    #[error("Permission denied: {message}")]
    PermissionDenied {
        /// Error message.
        message: String,
    },

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// TOML parsing error.
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Platform-specific error.
    #[error("Platform error: {message}")]
    Platform {
        /// Error message.
        message: String,
    },

    /// Netdiag error.
    #[error("Netdiag error: {0}")]
    Netdiag(#[from] netdiag_types::Error),
}

impl DaemonError {
    /// Creates a configuration error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Creates an IPC error.
    pub fn ipc(message: impl Into<String>) -> Self {
        Self::Ipc {
            message: message.into(),
        }
    }

    /// Creates a service error.
    pub fn service(message: impl Into<String>) -> Self {
        Self::Service {
            message: message.into(),
        }
    }

    /// Creates a scheduler error.
    pub fn scheduler(message: impl Into<String>) -> Self {
        Self::Scheduler {
            message: message.into(),
        }
    }

    /// Creates a platform error.
    pub fn platform(message: impl Into<String>) -> Self {
        Self::Platform {
            message: message.into(),
        }
    }
}
