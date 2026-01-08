//! Windows platform implementation for netdiag.
//!
//! This crate provides Windows-specific implementations of the netdiag platform traits,
//! using Windows APIs for network information, WiFi management, and system diagnostics.
//!
//! # Features
//!
//! - Network interface enumeration via IP Helper API
//! - WiFi scanning and management via WLAN API
//! - System information via Windows Management
//! - Auto-fix capabilities using netsh and system commands

#![cfg(windows)]
#![warn(missing_docs)]

mod autofix;
mod network;
mod privilege;
mod system;
mod wifi;

pub use autofix::WindowsAutofixProvider;
pub use network::WindowsNetworkProvider;
pub use privilege::WindowsPrivilegeProvider;
pub use system::WindowsSystemInfoProvider;
pub use wifi::WindowsWifiProvider;

use netdiag_platform::{PlatformProviders, StubCaptureProvider};
use std::sync::Arc;

/// Creates the Windows platform providers.
pub fn create_providers() -> PlatformProviders {
    PlatformProviders {
        network: Arc::new(WindowsNetworkProvider::new()),
        wifi: Arc::new(WindowsWifiProvider::new()),
        privilege: Arc::new(WindowsPrivilegeProvider::new()),
        capture: Arc::new(StubCaptureProvider),
        autofix: Arc::new(WindowsAutofixProvider::new()),
        system: Arc::new(WindowsSystemInfoProvider::new()),
    }
}
