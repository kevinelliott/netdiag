//! Error types for netdiag.

use std::net::IpAddr;
use thiserror::Error;

/// Result type alias using `netdiag` Error.
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for netdiag operations.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Network interface error
    #[error("Network interface error: {message}")]
    NetworkInterface {
        /// The interface name, if known
        interface: Option<String>,
        /// Error message
        message: String,
    },

    /// DNS resolution error
    #[error("DNS resolution error for {host}: {message}")]
    DnsResolution {
        /// The host that failed to resolve
        host: String,
        /// Error message
        message: String,
    },

    /// Ping/ICMP error
    #[error("Ping error to {target}: {message}")]
    Ping {
        /// The target address
        target: IpAddr,
        /// Error message
        message: String,
    },

    /// Traceroute error
    #[error("Traceroute error to {target}: {message}")]
    Traceroute {
        /// The target address
        target: IpAddr,
        /// Error message
        message: String,
    },

    /// Speed test error
    #[error("Speed test error: {message}")]
    SpeedTest {
        /// The server, if known
        server: Option<String>,
        /// Error message
        message: String,
    },

    /// WiFi error
    #[error("WiFi error: {message}")]
    Wifi {
        /// The interface, if known
        interface: Option<String>,
        /// Error message
        message: String,
    },

    /// Packet capture error
    #[error("Packet capture error: {message}")]
    PacketCapture {
        /// The interface, if known
        interface: Option<String>,
        /// Error message
        message: String,
    },

    /// Permission denied error
    #[error("Permission denied: {operation} requires {required_privilege}")]
    PermissionDenied {
        /// The operation that was denied
        operation: String,
        /// The privilege level required
        required_privilege: String,
    },

    /// Feature not supported on platform
    #[error("{feature} is not supported on {platform}")]
    UnsupportedOnPlatform {
        /// The feature name
        feature: String,
        /// The platform
        platform: String,
        /// Alternative suggestion, if any
        alternative: Option<String>,
    },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration {
        /// Error message
        message: String,
    },

    /// Database/storage error
    #[error("Storage error: {message}")]
    Storage {
        /// Error message
        message: String,
    },

    /// External API error
    #[error("External API error ({service}): {message}")]
    ExternalApi {
        /// The service name
        service: String,
        /// Error message
        message: String,
    },

    /// Report generation error
    #[error("Report generation error: {message}")]
    ReportGeneration {
        /// Error message
        message: String,
    },

    /// Timeout error
    #[error("Operation timed out after {duration_ms}ms: {operation}")]
    Timeout {
        /// The operation that timed out
        operation: String,
        /// Duration in milliseconds
        duration_ms: u64,
    },

    /// Parse error
    #[error("Parse error: {message}")]
    Parse {
        /// What was being parsed
        what: String,
        /// Error message
        message: String,
    },

    /// Invalid argument
    #[error("Invalid argument: {message}")]
    InvalidArgument {
        /// The argument name
        argument: String,
        /// Error message
        message: String,
    },

    /// Autofix error
    #[error("Autofix error: {message}")]
    Autofix {
        /// The fix that failed
        fix: String,
        /// Error message
        message: String,
        /// Whether rollback is possible
        rollback_available: bool,
    },

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Generic error with context
    #[error("{context}: {message}")]
    Other {
        /// Context for the error
        context: String,
        /// Error message
        message: String,
    },
}

impl Error {
    /// Returns true if this is a permission denied error.
    #[must_use]
    pub fn is_permission_denied(&self) -> bool {
        matches!(self, Self::PermissionDenied { .. })
    }

    /// Returns true if this is a timeout error.
    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(self, Self::Timeout { .. })
    }

    /// Returns true if this is an unsupported platform error.
    #[must_use]
    pub fn is_unsupported_platform(&self) -> bool {
        matches!(self, Self::UnsupportedOnPlatform { .. })
    }

    /// Creates a network interface error.
    #[must_use]
    pub fn network_interface(interface: impl Into<Option<String>>, message: impl Into<String>) -> Self {
        Self::NetworkInterface {
            interface: interface.into(),
            message: message.into(),
        }
    }

    /// Creates a permission denied error.
    #[must_use]
    pub fn permission_denied(operation: impl Into<String>, required: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            required_privilege: required.into(),
        }
    }

    /// Creates an unsupported platform error.
    #[must_use]
    pub fn unsupported(feature: impl Into<String>, platform: impl Into<String>) -> Self {
        Self::UnsupportedOnPlatform {
            feature: feature.into(),
            platform: platform.into(),
            alternative: None,
        }
    }

    /// Creates an unsupported platform error with an alternative.
    #[must_use]
    pub fn unsupported_with_alternative(
        feature: impl Into<String>,
        platform: impl Into<String>,
        alternative: impl Into<String>,
    ) -> Self {
        Self::UnsupportedOnPlatform {
            feature: feature.into(),
            platform: platform.into(),
            alternative: Some(alternative.into()),
        }
    }

    /// Creates a timeout error.
    #[must_use]
    pub fn timeout(operation: impl Into<String>, duration_ms: u64) -> Self {
        Self::Timeout {
            operation: operation.into(),
            duration_ms,
        }
    }
}
