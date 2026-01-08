//! Error types for the GUI backend.

use serde::{Deserialize, Serialize};

/// GUI-specific error type.
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum GuiError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("DNS resolution failed: {0}")]
    DnsResolution(String),

    #[error("Operation timed out: {0}")]
    Timeout(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<netdiag_types::Error> for GuiError {
    fn from(err: netdiag_types::Error) -> Self {
        match &err {
            netdiag_types::Error::NetworkInterface { message, .. } => {
                GuiError::Network(message.clone())
            }
            netdiag_types::Error::DnsResolution { message, .. } => {
                GuiError::DnsResolution(message.clone())
            }
            netdiag_types::Error::Timeout { operation, duration_ms } => {
                GuiError::Timeout(format!("{} timed out after {}ms", operation, duration_ms))
            }
            netdiag_types::Error::PermissionDenied { operation, required_privilege } => {
                GuiError::PermissionDenied(format!(
                    "{} requires {} privilege",
                    operation, required_privilege
                ))
            }
            netdiag_types::Error::UnsupportedOnPlatform { feature, platform, .. } => {
                GuiError::NotSupported(format!("{} not supported on {}", feature, platform))
            }
            _ => GuiError::Internal(err.to_string()),
        }
    }
}

/// Result type for GUI operations.
pub type GuiResult<T> = Result<T, GuiError>;
