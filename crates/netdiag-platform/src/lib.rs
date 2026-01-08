//! # netdiag-platform
//!
//! Platform abstraction layer for netdiag network diagnostics tool.
//!
//! This crate defines the traits that platform-specific implementations must provide,
//! enabling cross-platform network diagnostics functionality.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod traits;

mod detection;
mod factory;

pub use detection::*;
pub use factory::*;
pub use traits::*;
