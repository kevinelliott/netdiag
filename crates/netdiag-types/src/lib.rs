//! # netdiag-types
//!
//! Shared types, errors, and data structures for the netdiag network diagnostics tool.
//!
//! This crate provides the foundational types used across all netdiag components,
//! ensuring consistent data representation and serialization.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod config;
pub mod error;

pub mod capture;
pub mod diagnostics;
pub mod network;
pub mod report;
pub mod sync;
pub mod system;
pub mod wifi;

pub use error::{Error, Result};
