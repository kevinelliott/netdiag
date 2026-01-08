//! WiFi error types.

use thiserror::Error;

/// WiFi analysis error type.
#[derive(Debug, Error)]
pub enum WifiError {
    /// WiFi not available.
    #[error("WiFi not available on this system")]
    NotAvailable,

    /// Interface not found.
    #[error("WiFi interface not found: {0}")]
    InterfaceNotFound(String),

    /// Not connected.
    #[error("not connected to a WiFi network")]
    NotConnected,

    /// Scan failed.
    #[error("WiFi scan failed: {0}")]
    ScanFailed(String),

    /// Analysis failed.
    #[error("analysis failed: {0}")]
    AnalysisFailed(String),

    /// Platform error.
    #[error("platform error: {0}")]
    Platform(#[from] netdiag_types::error::Error),

    /// Privilege error.
    #[error("insufficient privileges: {0}")]
    InsufficientPrivileges(String),

    /// Timeout.
    #[error("operation timed out")]
    Timeout,
}

/// Result type for WiFi operations.
pub type WifiResult<T> = Result<T, WifiError>;
