//! Error types for integrations.

use thiserror::Error;

/// Integration errors.
#[derive(Debug, Error)]
pub enum IntegrationError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error.
    #[error("API error: {0}")]
    Api(String),

    /// Rate limit exceeded.
    #[error("Rate limit exceeded for {0}")]
    RateLimited(&'static str),

    /// API key required but not provided.
    #[error("API key required for {0}")]
    ApiKeyRequired(&'static str),

    /// Invalid API key.
    #[error("Invalid API key for {0}")]
    InvalidApiKey(&'static str),

    /// Resource not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// JSON parsing error.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Service unavailable.
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Timeout.
    #[error("Request timeout")]
    Timeout,

    /// Analysis in progress.
    #[error("Analysis in progress, check back later")]
    AnalysisInProgress,
}

/// Result type for integration operations.
pub type IntegrationResult<T> = Result<T, IntegrationError>;
