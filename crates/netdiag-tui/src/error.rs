//! TUI error types.

use thiserror::Error;

/// TUI errors.
#[derive(Debug, Error)]
pub enum TuiError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal error.
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Platform error.
    #[error("Platform error: {0}")]
    Platform(String),

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),

    /// Channel error.
    #[error("Channel error: {0}")]
    Channel(String),
}

/// Result type for TUI operations.
pub type TuiResult<T> = Result<T, TuiError>;
