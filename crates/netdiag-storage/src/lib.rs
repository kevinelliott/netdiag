//! # netdiag-storage
//!
//! Storage and persistence layer for netdiag.
//!
//! Provides SQLite-based storage for diagnostic results, configuration,
//! and historical data analysis.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod database;
mod error;
mod models;
mod repository;

pub use database::{Database, DatabaseConfig};
pub use error::{StorageError, StorageResult};
pub use models::*;
pub use repository::*;

/// Default database file name.
pub const DEFAULT_DB_NAME: &str = "netdiag.db";

/// Get the default database path.
pub fn default_database_path() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("netdiag")
        .join(DEFAULT_DB_NAME)
}
