//! Speed test error types.

use thiserror::Error;

/// Speed test error type.
#[derive(Debug, Error)]
pub enum SpeedError {
    /// No provider available.
    #[error("no speed test provider available")]
    NoProvider,

    /// Provider not found.
    #[error("provider not found: {0}")]
    ProviderNotFound(String),

    /// Server not found.
    #[error("no speed test server found")]
    ServerNotFound,

    /// Connection failed.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Timeout during test.
    #[error("test timed out after {0} seconds")]
    Timeout(u64),

    /// HTTP error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Parse error.
    #[error("parse error: {0}")]
    Parse(String),

    /// iPerf3 error.
    #[error("iPerf3 error: {0}")]
    Iperf(String),

    /// Test cancelled.
    #[error("test cancelled")]
    Cancelled,

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

/// Result type for speed test operations.
pub type SpeedResult<T> = Result<T, SpeedError>;
