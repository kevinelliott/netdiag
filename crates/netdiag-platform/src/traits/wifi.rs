//! WiFi provider trait.

use async_trait::async_trait;
use netdiag_types::{
    error::Result,
    wifi::{AccessPoint, Channel, ChannelUtilization, WifiConnection},
};

/// Provider for WiFi operations.
#[async_trait]
pub trait WifiProvider: Send + Sync {
    /// Checks if WiFi is available on this system.
    fn is_available(&self) -> bool;

    /// Lists available WiFi interfaces.
    async fn list_wifi_interfaces(&self) -> Result<Vec<WifiInterface>>;

    /// Scans for available access points.
    async fn scan_access_points(&self, interface: &str) -> Result<Vec<AccessPoint>>;

    /// Gets the current WiFi connection info.
    async fn get_current_connection(&self, interface: &str) -> Result<Option<WifiConnection>>;

    /// Gets the current signal strength (RSSI) in dBm.
    async fn get_signal_strength(&self, interface: &str) -> Result<Option<i32>>;

    /// Gets the current noise level in dBm.
    async fn get_noise_level(&self, interface: &str) -> Result<Option<i32>>;

    /// Gets channel utilization information.
    async fn get_channel_utilization(&self, channel: Channel) -> Result<ChannelUtilization>;

    /// Analyzes channels and returns recommendations.
    async fn analyze_channels(&self, interface: &str) -> Result<Vec<ChannelUtilization>>;

    /// Checks if enterprise (802.1X) authentication is supported.
    fn supports_enterprise(&self) -> bool;

    /// Triggers an active WiFi scan (may require privileges).
    async fn trigger_scan(&self, interface: &str) -> Result<()>;

    /// Gets the list of supported WiFi standards for an interface.
    async fn get_supported_standards(
        &self,
        interface: &str,
    ) -> Result<Vec<netdiag_types::wifi::WifiStandard>>;
}

/// WiFi interface information.
#[derive(Debug, Clone)]
pub struct WifiInterface {
    /// Interface name (e.g., "en0", "wlan0")
    pub name: String,
    /// Hardware address (MAC)
    pub mac_address: Option<netdiag_types::network::MacAddress>,
    /// Whether the interface is powered on
    pub powered_on: bool,
    /// Whether the interface is connected to a network
    pub connected: bool,
    /// Current country code
    pub country_code: Option<String>,
}

/// Extension trait for WiFi operations.
#[async_trait]
pub trait WifiProviderExt: WifiProvider {
    /// Gets the primary WiFi interface.
    async fn get_primary_interface(&self) -> Result<Option<WifiInterface>> {
        let interfaces = self.list_wifi_interfaces().await?;
        Ok(interfaces.into_iter().find(|i| i.powered_on))
    }

    /// Finds the best channel in a given band.
    async fn find_best_channel(
        &self,
        interface: &str,
        band: netdiag_types::wifi::WifiBand,
    ) -> Result<Option<Channel>> {
        let utilizations = self.analyze_channels(interface).await?;
        Ok(utilizations
            .into_iter()
            .filter(|u| u.channel.band == band && u.recommended)
            .min_by(|a, b| {
                a.utilization_percent
                    .partial_cmp(&b.utilization_percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|u| u.channel))
    }

    /// Checks if there's interference on the current channel.
    async fn has_interference(&self, interface: &str) -> Result<bool> {
        if let Some(connection) = self.get_current_connection(interface).await? {
            let utilization = self
                .get_channel_utilization(connection.access_point.channel)
                .await?;
            Ok(utilization.interference_level != netdiag_types::wifi::InterferenceLevel::Low)
        } else {
            Ok(false)
        }
    }
}

// Blanket implementation
impl<T: WifiProvider + ?Sized> WifiProviderExt for T {}
