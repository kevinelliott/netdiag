//! # netdiag-platform-android
//!
//! Android-specific platform implementation for netdiag.
//!
//! This crate provides implementations of the platform traits for Android.
//! It uses JNI to access Android system APIs for network information.

#![cfg(target_os = "android")]
#![warn(missing_docs)]
#![warn(clippy::all)]

mod network;
mod privilege;
mod system;
mod wifi;

pub use network::AndroidNetworkProvider;
pub use privilege::AndroidPrivilegeProvider;
pub use system::AndroidSystemInfoProvider;
pub use wifi::AndroidWifiProvider;

use netdiag_platform::{
    AutofixAction, AutofixProvider, CaptureInterface, CaptureProvider, FixResult,
    PlatformProviders, RollbackPoint,
};
use netdiag_types::system::PrivilegeLevel;
use std::sync::Arc;

/// Creates platform providers for Android.
pub fn create_providers() -> PlatformProviders {
    PlatformProviders {
        network: Arc::new(AndroidNetworkProvider::new()),
        wifi: Arc::new(AndroidWifiProvider::new()),
        privilege: Arc::new(AndroidPrivilegeProvider::new()),
        capture: Arc::new(StubCaptureProvider),
        autofix: Arc::new(StubAutofixProvider),
        system: Arc::new(AndroidSystemInfoProvider::new()),
    }
}

// Stub capture provider - packet capture requires root on Android
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
        Err(netdiag_types::Error::unsupported(
            "Packet capture",
            "Android",
        ))
    }

    async fn stop_capture(
        &self,
        _handle: netdiag_types::capture::CaptureHandle,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported(
            "Packet capture",
            "Android",
        ))
    }

    async fn get_capture_stats(
        &self,
        _handle: netdiag_types::capture::CaptureHandle,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported(
            "Packet capture",
            "Android",
        ))
    }

    fn required_privilege_level(&self) -> PrivilegeLevel {
        PrivilegeLevel::Root
    }

    fn compile_filter(&self, _expression: &str) -> netdiag_types::error::Result<String> {
        Err(netdiag_types::Error::unsupported(
            "Packet capture",
            "Android",
        ))
    }
}

// Stub autofix provider - system modifications require root on Android
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
        Err(netdiag_types::Error::unsupported("Autofix", "Android"))
    }

    async fn rollback(
        &self,
        _id: &netdiag_types::system::RollbackId,
    ) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "Android"))
    }

    async fn list_rollback_points(&self) -> netdiag_types::error::Result<Vec<RollbackPoint>> {
        Ok(Vec::new())
    }

    async fn flush_dns_cache(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported(
            "DNS cache flush",
            "Android",
        ))
    }

    async fn reset_adapter(&self, _interface: &str) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported(
            "Adapter reset",
            "Android",
        ))
    }

    async fn set_dns_servers(
        &self,
        _interface: &str,
        _servers: &[std::net::IpAddr],
    ) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported(
            "Set DNS servers",
            "Android",
        ))
    }

    async fn toggle_interface(
        &self,
        _interface: &str,
        _enable: bool,
    ) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported(
            "Toggle interface",
            "Android",
        ))
    }

    async fn reset_tcpip_stack(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported(
            "Reset TCP/IP stack",
            "Android",
        ))
    }

    async fn renew_dhcp(&self, _interface: &str) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Renew DHCP", "Android"))
    }

    async fn clear_arp_cache(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported(
            "Clear ARP cache",
            "Android",
        ))
    }

    async fn get_available_fixes(&self) -> netdiag_types::error::Result<Vec<AutofixAction>> {
        Ok(Vec::new())
    }

    async fn apply_fix(&self, _fix: &AutofixAction) -> netdiag_types::error::Result<FixResult> {
        Err(netdiag_types::Error::unsupported("Autofix", "Android"))
    }
}
