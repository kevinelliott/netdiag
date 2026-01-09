//! # netdiag-platform-ios
//!
//! iOS-specific platform implementation for netdiag.
//!
//! This crate provides implementations of the platform traits for iOS.
//! Note that iOS has more limited network APIs compared to desktop platforms,
//! and some features require special entitlements.

#![cfg(target_os = "ios")]
#![warn(missing_docs)]
#![warn(clippy::all)]

mod network;
mod privilege;
mod system;
mod wifi;

pub use network::IosNetworkProvider;
pub use privilege::IosPrivilegeProvider;
pub use system::IosSystemInfoProvider;
pub use wifi::IosWifiProvider;

use netdiag_platform::{
    AutofixAction, AutofixProvider, CaptureInterface, CaptureProvider, FixResult,
    PlatformProviders, RollbackPoint,
};
use netdiag_types::system::PrivilegeLevel;
use std::sync::Arc;

/// Creates platform providers for iOS.
pub fn create_providers() -> PlatformProviders {
    PlatformProviders {
        network: Arc::new(IosNetworkProvider::new()),
        wifi: Arc::new(IosWifiProvider::new()),
        privilege: Arc::new(IosPrivilegeProvider::new()),
        capture: Arc::new(StubCaptureProvider),
        autofix: Arc::new(StubAutofixProvider),
        system: Arc::new(IosSystemInfoProvider::new()),
    }
}

// Stub capture provider - packet capture is not available on iOS
struct StubCaptureProvider;

#[async_trait::async_trait]
impl CaptureProvider for StubCaptureProvider {
    fn is_available(&self) -> bool {
        false
    }

    async fn list_capture_interfaces(&self) -> netdiag_types::error::Result<Vec<CaptureInterface>> {
        Ok(Vec::new())
    }

    async fn start_capture(
        &self,
        _interface: &str,
        _filter: Option<netdiag_types::capture::CaptureFilter>,
        _packet_tx: tokio::sync::mpsc::Sender<netdiag_types::capture::CapturedPacket>,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureHandle> {
        Err(netdiag_types::Error::unsupported("Packet capture", "iOS"))
    }

    async fn stop_capture(
        &self,
        _handle: netdiag_types::capture::CaptureHandle,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported("Packet capture", "iOS"))
    }

    async fn get_capture_stats(
        &self,
        _handle: netdiag_types::capture::CaptureHandle,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported("Packet capture", "iOS"))
    }

    fn required_privilege_level(&self) -> PrivilegeLevel {
        PrivilegeLevel::Elevated
    }

    fn compile_filter(&self, _expression: &str) -> netdiag_types::error::Result<String> {
        Err(netdiag_types::Error::unsupported("Packet capture", "iOS"))
    }
}

// Stub autofix provider - system modifications are very limited on iOS
struct StubAutofixProvider;

#[async_trait::async_trait]
impl AutofixProvider for StubAutofixProvider {
    fn is_available(&self) -> bool {
        false
    }

    async fn create_rollback_point(
        &self,
        _description: &str,
    ) -> netdiag_types::error::Result<netdiag_types::system::RollbackId> {
        Err(netdiag_types::Error::unsupported("Autofix", "iOS"))
    }

    async fn rollback(
        &self,
        _id: &netdiag_types::system::RollbackId,
    ) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "iOS"))
    }

    async fn list_rollback_points(&self) -> netdiag_types::error::Result<Vec<RollbackPoint>> {
        Ok(Vec::new())
    }

    async fn flush_dns_cache(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("DNS cache flush", "iOS"))
    }

    async fn reset_adapter(&self, _interface: &str) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Adapter reset", "iOS"))
    }

    async fn set_dns_servers(
        &self,
        _interface: &str,
        _servers: &[std::net::IpAddr],
    ) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Set DNS servers", "iOS"))
    }

    async fn toggle_interface(
        &self,
        _interface: &str,
        _enable: bool,
    ) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Toggle interface", "iOS"))
    }

    async fn reset_tcpip_stack(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported(
            "Reset TCP/IP stack",
            "iOS",
        ))
    }

    async fn renew_dhcp(&self, _interface: &str) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Renew DHCP", "iOS"))
    }

    async fn clear_arp_cache(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Clear ARP cache", "iOS"))
    }

    async fn get_available_fixes(&self) -> netdiag_types::error::Result<Vec<AutofixAction>> {
        Ok(Vec::new())
    }

    async fn apply_fix(&self, _fix: &AutofixAction) -> netdiag_types::error::Result<FixResult> {
        Err(netdiag_types::Error::unsupported("Autofix", "iOS"))
    }
}
