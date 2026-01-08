//! Capture error types.

use thiserror::Error;

/// Capture error type.
#[derive(Debug, Error)]
pub enum CaptureError {
    /// No capture device found.
    #[error("no capture device found")]
    NoDeviceFound,

    /// Device not found.
    #[error("device not found: {0}")]
    DeviceNotFound(String),

    /// Permission denied.
    #[error("permission denied: capture requires elevated privileges")]
    PermissionDenied,

    /// Invalid filter.
    #[error("invalid BPF filter: {0}")]
    InvalidFilter(String),

    /// Capture already running.
    #[error("capture already running")]
    AlreadyRunning,

    /// Capture not started.
    #[error("capture not started")]
    NotStarted,

    /// PCAP error.
    #[error("pcap error: {0}")]
    PcapError(String),

    /// Decode error.
    #[error("decode error: {0}")]
    DecodeError(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Timeout.
    #[error("capture timed out")]
    Timeout,

    /// Capture stopped.
    #[error("capture stopped")]
    Stopped,
}

/// Result type for capture operations.
pub type CaptureResult<T> = Result<T, CaptureError>;
