//! iOS WiFi provider implementation.

use async_trait::async_trait;
use netdiag_platform::{WifiInterface, WifiProvider};
use netdiag_types::{
    error::Result,
    wifi::{AccessPoint, Channel, ChannelUtilization, WifiConnection, WifiStandard},
    Error,
};

/// iOS WiFi provider.
///
/// Note: On iOS, WiFi scanning requires special entitlements (NEHotspotHelper)
/// that are only available to specific app categories. This implementation
/// provides limited WiFi information that's available to standard apps.
pub struct IosWifiProvider {
    // No persistent state needed
}

impl IosWifiProvider {
    /// Creates a new iOS WiFi provider.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for IosWifiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WifiProvider for IosWifiProvider {
    fn is_available(&self) -> bool {
        // WiFi exists on iOS but scanning is restricted
        true
    }

    async fn list_wifi_interfaces(&self) -> Result<Vec<WifiInterface>> {
        // iOS typically has one WiFi interface (en0)
        Ok(vec![WifiInterface {
            name: "en0".to_string(),
            display_name: Some("Wi-Fi".to_string()),
            hardware_address: None,
            is_powered_on: true,
        }])
    }

    async fn scan_access_points(&self, _interface: &str) -> Result<Vec<AccessPoint>> {
        // WiFi scanning requires NEHotspotHelper entitlement
        // which is restricted to certain app categories
        Err(Error::UnsupportedOnPlatform {
            feature: "WiFi scanning".to_string(),
            platform: "iOS".to_string(),
            alternative: Some("Use NEHotspotHelper with proper entitlements".to_string()),
        })
    }

    async fn get_current_connection(&self, _interface: &str) -> Result<Option<WifiConnection>> {
        // Getting current connection info is possible with CNCopyCurrentNetworkInfo
        // but requires the Access WiFi Information capability
        // For now, return None to indicate unknown
        Ok(None)
    }

    async fn get_signal_strength(&self, _interface: &str) -> Result<Option<i32>> {
        // Signal strength is not directly available to iOS apps
        Ok(None)
    }

    async fn get_noise_level(&self, _interface: &str) -> Result<Option<i32>> {
        // Noise level is not available on iOS
        Ok(None)
    }

    async fn get_channel_utilization(
        &self,
        _channel: Channel,
    ) -> Result<ChannelUtilization> {
        Err(Error::unsupported("Channel utilization", "iOS"))
    }

    async fn analyze_channels(&self, _interface: &str) -> Result<Vec<ChannelUtilization>> {
        // Channel analysis requires WiFi scanning which is restricted
        Ok(Vec::new())
    }

    fn supports_enterprise(&self) -> bool {
        // iOS supports enterprise WiFi but we can't query it
        false
    }

    async fn trigger_scan(&self, _interface: &str) -> Result<()> {
        Err(Error::unsupported("WiFi scan trigger", "iOS"))
    }

    async fn get_supported_standards(&self, _interface: &str) -> Result<Vec<WifiStandard>> {
        // Modern iOS devices support up to WiFi 6E
        // but we can't query this programmatically
        Ok(vec![
            WifiStandard::B,
            WifiStandard::G,
            WifiStandard::N,
            WifiStandard::Ac,
            WifiStandard::Ax,
        ])
    }
}
