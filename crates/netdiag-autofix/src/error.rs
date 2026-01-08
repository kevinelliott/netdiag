//! Auto-fix error types.

use thiserror::Error;

/// Result type for autofix operations.
pub type Result<T> = std::result::Result<T, AutofixError>;

/// Errors that can occur during auto-fix operations.
#[derive(Debug, Error)]
pub enum AutofixError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The fix failed to apply.
    #[error("Fix failed: {message}")]
    FixFailed {
        /// Error message.
        message: String,
    },

    /// Rollback failed.
    #[error("Rollback failed: {message}")]
    RollbackFailed {
        /// Error message.
        message: String,
    },

    /// Rollback point not found.
    #[error("Rollback point not found: {id}")]
    RollbackNotFound {
        /// Rollback point ID.
        id: String,
    },

    /// Permission denied.
    #[error("Permission denied: {message}")]
    PermissionDenied {
        /// Error message.
        message: String,
    },

    /// Feature not supported on this platform.
    #[error("Not supported on this platform: {feature}")]
    NotSupported {
        /// Feature name.
        feature: String,
    },

    /// Verification failed after applying fix.
    #[error("Verification failed: {message}")]
    VerificationFailed {
        /// Error message.
        message: String,
    },

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Platform error.
    #[error("Platform error: {0}")]
    Platform(#[from] netdiag_types::Error),
}

impl AutofixError {
    /// Creates a fix failed error.
    pub fn fix_failed(message: impl Into<String>) -> Self {
        Self::FixFailed {
            message: message.into(),
        }
    }

    /// Creates a rollback failed error.
    pub fn rollback_failed(message: impl Into<String>) -> Self {
        Self::RollbackFailed {
            message: message.into(),
        }
    }

    /// Creates a permission denied error.
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied {
            message: message.into(),
        }
    }

    /// Creates a not supported error.
    pub fn not_supported(feature: impl Into<String>) -> Self {
        Self::NotSupported {
            feature: feature.into(),
        }
    }

    /// Creates a verification failed error.
    pub fn verification_failed(message: impl Into<String>) -> Self {
        Self::VerificationFailed {
            message: message.into(),
        }
    }
}
