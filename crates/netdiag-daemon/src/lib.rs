//! NetDiag Daemon - Background service for continuous network monitoring.
//!
//! This crate provides a daemon/service that runs in the background and performs:
//! - Scheduled network diagnostics
//! - Continuous monitoring with alerting
//! - IPC communication with CLI/GUI
//!
//! # Features
//!
//! - **Scheduled Diagnostics**: Run network tests on a schedule (cron-like)
//! - **Continuous Monitoring**: Monitor network health continuously
//! - **IPC Communication**: Socket-based communication with CLI/GUI
//! - **Cross-Platform**: Works on macOS, Linux, and Windows

#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod ipc;
pub mod monitor;
pub mod scheduler;
pub mod service;

pub use config::DaemonConfig;
pub use error::{DaemonError, Result};
pub use ipc::{IpcClient, IpcServer};
pub use monitor::NetworkMonitor;
pub use scheduler::DiagnosticScheduler;
pub use service::{DaemonService, ServiceState};
