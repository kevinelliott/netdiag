//! Report error types.

use thiserror::Error;

/// Report generation error types.
#[derive(Error, Debug)]
pub enum ReportError {
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Formatting error
    #[error("Formatting error: {0}")]
    Formatting(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Template error
    #[error("Template error: {0}")]
    Template(String),

    /// Invalid report data
    #[error("Invalid report data: {0}")]
    InvalidData(String),
}

impl From<serde_json::Error> for ReportError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

/// Report result type.
pub type ReportResult<T> = Result<T, ReportError>;
