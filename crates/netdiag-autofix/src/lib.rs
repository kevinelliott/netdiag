//! NetDiag Auto-Fix - Automatic network remediation with rollback capability.
//!
//! This crate provides automatic fixes for common network issues with the ability
//! to rollback changes if they don't improve the situation.
//!
//! # Features
//!
//! - **DNS Cache Flush**: Clear DNS cache to resolve stale entries
//! - **Network Adapter Reset**: Reset network adapters
//! - **DNS Server Configuration**: Change DNS servers to known-good alternatives
//! - **TCP/IP Stack Reset**: Reset the TCP/IP stack
//! - **Rollback**: Automatically rollback changes if they don't help
//!
//! # Example
//!
//! ```ignore
//! use netdiag_autofix::{AutofixEngine, FixAction};
//!
//! let engine = AutofixEngine::new(providers);
//!
//! // Run in dry-run mode first
//! let plan = engine.plan(&issues).await?;
//! println!("Proposed fixes: {:?}", plan);
//!
//! // Execute with rollback on failure
//! let result = engine.execute_with_rollback(&plan).await?;
//! ```

#![warn(missing_docs)]

pub mod actions;
pub mod engine;
pub mod error;
pub mod rollback;

pub use actions::{FixAction, FixCategory, FixResult, FixSeverity};
pub use engine::AutofixEngine;
pub use error::{AutofixError, Result};
pub use rollback::{RollbackId, RollbackManager, RollbackPoint};
