//! Storage error types.

use thiserror::Error;

/// Storage error types.
#[derive(Error, Debug)]
pub enum StorageError {
    /// Database connection error
    #[error("Database connection error: {0}")]
    Connection(String),

    /// Query execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// SQLx error
    #[error("Database error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

/// Storage result type.
pub type StorageResult<T> = Result<T, StorageError>;
