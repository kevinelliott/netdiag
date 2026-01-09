//! # netdiag-platform-linux
//!
//! Linux-specific platform implementation for netdiag.
//!
//! This crate provides real implementations of the platform traits
//! for Linux, using native APIs and system commands.

#![cfg(target_os = "linux")]
#![warn(missing_docs)]
#![warn(clippy::all)]

mod network;
mod privilege;
mod system;
mod wifi;

pub use network::LinuxNetworkProvider;
pub use privilege::LinuxPrivilegeProvider;
pub use system::LinuxSystemInfoProvider;
pub use wifi::LinuxWifiProvider;

use netdiag_platform::{CaptureProvider, PlatformProviders};
use std::sync::Arc;

/// Creates platform providers for Linux.
pub fn create_providers() -> PlatformProviders {
    PlatformProviders {
        network: Arc::new(LinuxNetworkProvider::new()),
        wifi: Arc::new(LinuxWifiProvider::new()),
        privilege: Arc::new(LinuxPrivilegeProvider::new()),
        capture: Arc::new(StubCaptureProvider),
        autofix: Arc::new(LinuxAutofixProvider::new()),
        system: Arc::new(LinuxSystemInfoProvider::new()),
    }
}

// Autofix provider for Linux
mod autofix;
pub use autofix::LinuxAutofixProvider;

// Stub capture provider - will be implemented in Phase 3
struct StubCaptureProvider;

#[async_trait::async_trait]
impl CaptureProvider for StubCaptureProvider {
    fn is_available(&self) -> bool {
        false
    }

    async fn list_capture_interfaces(
        &self,
    ) -> netdiag_types::error::Result<Vec<netdiag_platform::CaptureInterface>> {
        Ok(Vec::new())
    }

    async fn start_capture(
        &self,
        _interface: &str,
        _filter: Option<netdiag_types::capture::CaptureFilter>,
        _packet_tx: tokio::sync::mpsc::Sender<netdiag_types::capture::CapturedPacket>,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureHandle> {
        Err(netdiag_types::Error::unsupported(
            "Packet capture",
            "not yet implemented",
        ))
    }

    async fn stop_capture(
        &self,
        _handle: netdiag_types::capture::CaptureHandle,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported(
            "Packet capture",
            "not yet implemented",
        ))
    }

    async fn get_capture_stats(
        &self,
        _handle: netdiag_types::capture::CaptureHandle,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported(
            "Packet capture",
            "not yet implemented",
        ))
    }

    fn required_privilege_level(&self) -> netdiag_types::system::PrivilegeLevel {
        netdiag_types::system::PrivilegeLevel::Elevated
    }

    fn compile_filter(&self, _expression: &str) -> netdiag_types::error::Result<String> {
        Ok(String::new())
    }
}
