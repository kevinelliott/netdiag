//! Platform provider factory.

use crate::traits::{
    AutofixProvider, CaptureProvider, NetworkProvider, PrivilegeProvider, SystemInfoProvider,
    WifiProvider,
};
use std::sync::Arc;

/// Container for all platform-specific providers.
pub struct PlatformProviders {
    /// Network provider
    pub network: Arc<dyn NetworkProvider>,
    /// WiFi provider
    pub wifi: Arc<dyn WifiProvider>,
    /// Privilege provider
    pub privilege: Arc<dyn PrivilegeProvider>,
    /// Capture provider
    pub capture: Arc<dyn CaptureProvider>,
    /// Autofix provider
    pub autofix: Arc<dyn AutofixProvider>,
    /// System info provider
    pub system: Arc<dyn SystemInfoProvider>,
}

impl PlatformProviders {
    /// Creates a new set of platform providers with stub implementations.
    ///
    /// This is a fallback. Real implementations should be provided by calling
    /// the platform-specific `create_providers()` function (e.g., from
    /// `netdiag-platform-macos`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            network: Arc::new(StubNetworkProvider),
            wifi: Arc::new(StubWifiProvider),
            privilege: Arc::new(StubPrivilegeProvider),
            capture: Arc::new(StubCaptureProvider),
            autofix: Arc::new(StubAutofixProvider),
            system: Arc::new(StubSystemInfoProvider),
        }
    }
}

impl Default for PlatformProviders {
    fn default() -> Self {
        Self::new()
    }
}

// Stub implementations for compilation - these will be replaced by real implementations

struct StubNetworkProvider;

#[async_trait::async_trait]
impl NetworkProvider for StubNetworkProvider {
    async fn list_interfaces(&self) -> netdiag_types::error::Result<Vec<netdiag_types::network::NetworkInterface>> {
        Ok(Vec::new())
    }

    async fn get_interface(&self, _name: &str) -> netdiag_types::error::Result<Option<netdiag_types::network::NetworkInterface>> {
        Ok(None)
    }

    async fn get_default_interface(&self) -> netdiag_types::error::Result<Option<netdiag_types::network::NetworkInterface>> {
        Ok(None)
    }

    async fn get_default_route(&self) -> netdiag_types::error::Result<Option<netdiag_types::network::Route>> {
        Ok(None)
    }

    async fn get_routes(&self) -> netdiag_types::error::Result<Vec<netdiag_types::network::Route>> {
        Ok(Vec::new())
    }

    async fn get_default_gateway(&self) -> netdiag_types::error::Result<Option<netdiag_types::network::Gateway>> {
        Ok(None)
    }

    async fn get_dns_servers(&self) -> netdiag_types::error::Result<Vec<netdiag_types::network::DnsServer>> {
        Ok(Vec::new())
    }

    async fn get_dhcp_info(&self, _interface: &str) -> netdiag_types::error::Result<Option<netdiag_types::network::DhcpInfo>> {
        Ok(None)
    }

    async fn detect_isp(&self) -> netdiag_types::error::Result<Option<netdiag_types::network::IspInfo>> {
        Ok(None)
    }

    fn supports_promiscuous(&self, _interface: &str) -> bool {
        false
    }

    async fn refresh(&self) -> netdiag_types::error::Result<()> {
        Ok(())
    }
}

struct StubWifiProvider;

#[async_trait::async_trait]
impl WifiProvider for StubWifiProvider {
    fn is_available(&self) -> bool {
        false
    }

    async fn list_wifi_interfaces(&self) -> netdiag_types::error::Result<Vec<crate::traits::WifiInterface>> {
        Ok(Vec::new())
    }

    async fn scan_access_points(&self, _interface: &str) -> netdiag_types::error::Result<Vec<netdiag_types::wifi::AccessPoint>> {
        Err(netdiag_types::Error::unsupported("WiFi scanning", "stub"))
    }

    async fn get_current_connection(&self, _interface: &str) -> netdiag_types::error::Result<Option<netdiag_types::wifi::WifiConnection>> {
        Ok(None)
    }

    async fn get_signal_strength(&self, _interface: &str) -> netdiag_types::error::Result<Option<i32>> {
        Ok(None)
    }

    async fn get_noise_level(&self, _interface: &str) -> netdiag_types::error::Result<Option<i32>> {
        Ok(None)
    }

    async fn get_channel_utilization(&self, _channel: netdiag_types::wifi::Channel) -> netdiag_types::error::Result<netdiag_types::wifi::ChannelUtilization> {
        Err(netdiag_types::Error::unsupported("Channel utilization", "stub"))
    }

    async fn analyze_channels(&self, _interface: &str) -> netdiag_types::error::Result<Vec<netdiag_types::wifi::ChannelUtilization>> {
        Ok(Vec::new())
    }

    fn supports_enterprise(&self) -> bool {
        false
    }

    async fn trigger_scan(&self, _interface: &str) -> netdiag_types::error::Result<()> {
        Ok(())
    }

    async fn get_supported_standards(&self, _interface: &str) -> netdiag_types::error::Result<Vec<netdiag_types::wifi::WifiStandard>> {
        Ok(Vec::new())
    }
}

struct StubPrivilegeProvider;

#[async_trait::async_trait]
impl PrivilegeProvider for StubPrivilegeProvider {
    fn current_privilege_level(&self) -> netdiag_types::system::PrivilegeLevel {
        netdiag_types::system::PrivilegeLevel::User
    }

    async fn request_elevation(&self, _request: &netdiag_types::system::ElevationRequest) -> netdiag_types::error::Result<bool> {
        Ok(false)
    }

    fn has_capability(&self, _capability: crate::traits::Capability) -> bool {
        false
    }

    fn available_capabilities(&self) -> Vec<crate::traits::Capability> {
        Vec::new()
    }

    fn capabilities_requiring_elevation(&self) -> Vec<crate::traits::Capability> {
        Vec::new()
    }
}

struct StubCaptureProvider;

#[async_trait::async_trait]
impl CaptureProvider for StubCaptureProvider {
    fn is_available(&self) -> bool {
        false
    }

    async fn list_capture_interfaces(&self) -> netdiag_types::error::Result<Vec<crate::traits::CaptureInterface>> {
        Ok(Vec::new())
    }

    async fn start_capture(
        &self,
        _interface: &str,
        _filter: Option<netdiag_types::capture::CaptureFilter>,
        _packet_tx: tokio::sync::mpsc::Sender<netdiag_types::capture::CapturedPacket>,
    ) -> netdiag_types::error::Result<netdiag_types::capture::CaptureHandle> {
        Err(netdiag_types::Error::unsupported("Packet capture", "stub"))
    }

    async fn stop_capture(&self, _handle: netdiag_types::capture::CaptureHandle) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported("Packet capture", "stub"))
    }

    async fn get_capture_stats(&self, _handle: netdiag_types::capture::CaptureHandle) -> netdiag_types::error::Result<netdiag_types::capture::CaptureStats> {
        Err(netdiag_types::Error::unsupported("Packet capture", "stub"))
    }

    fn required_privilege_level(&self) -> netdiag_types::system::PrivilegeLevel {
        netdiag_types::system::PrivilegeLevel::Elevated
    }

    fn compile_filter(&self, _expression: &str) -> netdiag_types::error::Result<String> {
        Ok(String::new())
    }
}

struct StubAutofixProvider;

#[async_trait::async_trait]
impl AutofixProvider for StubAutofixProvider {
    fn is_available(&self) -> bool {
        false
    }

    async fn create_rollback_point(&self, _description: &str) -> netdiag_types::error::Result<netdiag_types::system::RollbackId> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn rollback(&self, _id: &netdiag_types::system::RollbackId) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn list_rollback_points(&self) -> netdiag_types::error::Result<Vec<crate::traits::RollbackPoint>> {
        Ok(Vec::new())
    }

    async fn flush_dns_cache(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn reset_adapter(&self, _interface: &str) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn set_dns_servers(&self, _interface: &str, _servers: &[std::net::IpAddr]) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn toggle_interface(&self, _interface: &str, _enable: bool) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn reset_tcpip_stack(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn renew_dhcp(&self, _interface: &str) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn clear_arp_cache(&self) -> netdiag_types::error::Result<()> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }

    async fn get_available_fixes(&self) -> netdiag_types::error::Result<Vec<crate::traits::AutofixAction>> {
        Ok(Vec::new())
    }

    async fn apply_fix(&self, _fix: &crate::traits::AutofixAction) -> netdiag_types::error::Result<crate::traits::FixResult> {
        Err(netdiag_types::Error::unsupported("Autofix", "stub"))
    }
}

struct StubSystemInfoProvider;

#[async_trait::async_trait]
impl SystemInfoProvider for StubSystemInfoProvider {
    async fn get_system_info(&self) -> netdiag_types::error::Result<netdiag_types::system::SystemInfo> {
        Ok(netdiag_types::system::SystemInfo {
            hostname: "unknown".to_string(),
            os_type: netdiag_types::system::OsType::current(),
            os_version: "unknown".to_string(),
            os_build: None,
            kernel_version: None,
            architecture: std::env::consts::ARCH.to_string(),
            cpu: None,
            memory: None,
            uptime: None,
        })
    }

    async fn get_hostname(&self) -> netdiag_types::error::Result<String> {
        Ok("unknown".to_string())
    }

    async fn get_uptime(&self) -> netdiag_types::error::Result<std::time::Duration> {
        Ok(std::time::Duration::ZERO)
    }

    fn get_timezone(&self) -> String {
        "UTC".to_string()
    }
}
