//! Android WiFi provider implementation.

use async_trait::async_trait;
use netdiag_platform::{WifiInterface, WifiProvider};
use netdiag_types::{
    error::Result,
    wifi::{AccessPoint, Channel, ChannelUtilization, WifiConnection, WifiStandard},
    Error,
};

/// Android WiFi provider.
///
/// Uses Android's WifiManager through JNI to access WiFi information.
/// Note: Starting with Android 10, WiFi scanning has restrictions
/// and requires location permission.
pub struct AndroidWifiProvider {
    // JNI references would be stored here
}

impl AndroidWifiProvider {
    /// Creates a new Android WiFi provider.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AndroidWifiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WifiProvider for AndroidWifiProvider {
    fn is_available(&self) -> bool {
        // WiFi is available on most Android devices
        true
    }

    async fn list_wifi_interfaces(&self) -> Result<Vec<WifiInterface>> {
        // Android typically has one WiFi interface (wlan0)
        Ok(vec![WifiInterface {
            name: "wlan0".to_string(),
            display_name: Some("Wi-Fi".to_string()),
            hardware_address: None,
            is_powered_on: true,
        }])
    }

    async fn scan_access_points(&self, _interface: &str) -> Result<Vec<AccessPoint>> {
        // In a real implementation, this would:
        // 1. Check for ACCESS_FINE_LOCATION permission (required since Android 10)
        // 2. Use WifiManager.startScan() and WifiManager.getScanResults()
        //
        // For now, return empty since we don't have JNI set up
        Err(Error::UnsupportedOnPlatform {
            feature: "WiFi scanning".to_string(),
            platform: "Android".to_string(),
            alternative: Some("Grant location permission and use WifiManager".to_string()),
        })
    }

    async fn get_current_connection(&self, _interface: &str) -> Result<Option<WifiConnection>> {
        // Would use WifiManager.getConnectionInfo()
        // Requires ACCESS_FINE_LOCATION on Android 8.0+
        Ok(None)
    }

    async fn get_signal_strength(&self, _interface: &str) -> Result<Option<i32>> {
        // Would use WifiManager.getConnectionInfo().getRssi()
        Ok(None)
    }

    async fn get_noise_level(&self, _interface: &str) -> Result<Option<i32>> {
        // Noise level is not available through standard Android APIs
        Ok(None)
    }

    async fn get_channel_utilization(
        &self,
        _channel: Channel,
    ) -> Result<ChannelUtilization> {
        Err(Error::unsupported("Channel utilization", "Android"))
    }

    async fn analyze_channels(&self, _interface: &str) -> Result<Vec<ChannelUtilization>> {
        // Would need scan results to analyze channels
        Ok(Vec::new())
    }

    fn supports_enterprise(&self) -> bool {
        // Android supports WPA2/WPA3 Enterprise
        true
    }

    async fn trigger_scan(&self, _interface: &str) -> Result<()> {
        // Would use WifiManager.startScan()
        // Note: Throttled on Android 8+ (4 scans in 2 minutes for foreground)
        Err(Error::unsupported("WiFi scan trigger without permission", "Android"))
    }

    async fn get_supported_standards(&self, _interface: &str) -> Result<Vec<WifiStandard>> {
        // Modern Android devices support up to WiFi 6E/7
        Ok(vec![
            WifiStandard::B,
            WifiStandard::G,
            WifiStandard::N,
            WifiStandard::Ac,
            WifiStandard::Ax,
        ])
    }
}
