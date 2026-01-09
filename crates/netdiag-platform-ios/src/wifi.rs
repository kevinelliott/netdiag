//! iOS WiFi provider implementation.
//!
//! Uses CNCopyCurrentNetworkInfo from the SystemConfiguration framework
//! to get current WiFi connection info. This requires the "Access WiFi Information"
//! entitlement in your app's capabilities.

use async_trait::async_trait;
use netdiag_platform::{WifiInterface, WifiProvider};
use netdiag_types::{
    error::Result,
    wifi::{
        AccessPoint, AccessPointCapabilities, Bssid, Channel, ChannelUtilization, SecurityType,
        Ssid, WifiBand, WifiAuthState, WifiConnection, WifiConnectionState, WifiStandard,
    },
    Error,
};

#[cfg(target_os = "ios")]
use core_foundation::base::{CFType, TCFType};
#[cfg(target_os = "ios")]
use core_foundation::dictionary::CFDictionary;
#[cfg(target_os = "ios")]
use core_foundation::string::CFString;

/// iOS WiFi provider.
///
/// Note: On iOS, WiFi scanning requires the NEHotspotHelper entitlement which
/// is restricted to specific app categories. This implementation provides
/// WiFi information available to standard apps via CNCopyCurrentNetworkInfo.
pub struct IosWifiProvider {
    // No persistent state needed
}

impl IosWifiProvider {
    /// Creates a new iOS WiFi provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the current WiFi connection info using CNCopyCurrentNetworkInfo.
    ///
    /// Requires the "Access WiFi Information" entitlement.
    #[cfg(target_os = "ios")]
    fn get_current_wifi_info(&self) -> Option<WifiConnection> {
        // Import the C function from SystemConfiguration framework
        #[link(name = "SystemConfiguration", kind = "framework")]
        extern "C" {
            fn CNCopyCurrentNetworkInfo(
                interfaceName: core_foundation_sys::base::CFTypeRef,
            ) -> core_foundation_sys::base::CFTypeRef;

            fn CNCopySupportedInterfaces() -> core_foundation_sys::base::CFTypeRef;
        }

        unsafe {
            // Get supported interfaces
            let interfaces_ref = CNCopySupportedInterfaces();
            if interfaces_ref.is_null() {
                return None;
            }

            // The first interface is typically the WiFi interface
            let interfaces: core_foundation::array::CFArray<CFString> =
                core_foundation::array::CFArray::wrap_under_create_rule(
                    interfaces_ref as core_foundation_sys::array::CFArrayRef,
                );

            if interfaces.is_empty() {
                return None;
            }

            let wifi_interface = interfaces.get(0)?;

            // Get network info for the interface
            let info_ref = CNCopyCurrentNetworkInfo(wifi_interface.as_CFTypeRef());
            if info_ref.is_null() {
                return None;
            }

            let info: CFDictionary<CFString, CFType> = CFDictionary::wrap_under_create_rule(
                info_ref as core_foundation_sys::dictionary::CFDictionaryRef,
            );

            // Extract SSID
            let ssid_key = CFString::new("SSID");
            let ssid = info
                .find(&ssid_key)
                .and_then(|v| {
                    // SSID can be CFString or CFData
                    if let Some(s) = v.downcast::<CFString>() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            // Extract BSSID
            let bssid_key = CFString::new("BSSID");
            let bssid_str = info
                .find(&bssid_key)
                .and_then(|v| v.downcast::<CFString>())
                .map(|s| s.to_string());

            let bssid = bssid_str.and_then(|s| parse_bssid(&s));

            // Create access point info
            let access_point = AccessPoint {
                ssid: Ssid::new(&ssid),
                bssid: bssid.unwrap_or_else(|| Bssid::new([0; 6])),
                rssi: -50, // Not available from CNCopyCurrentNetworkInfo
                signal_quality: 75, // Estimated
                channel: Channel::from_number(0, WifiBand::Band2_4GHz),
                security: SecurityType::open(), // Not easily available
                wifi_standard: WifiStandard::default(),
                is_hidden: false,
                is_connected: true,
                noise: None,
                snr: None,
                country_code: None,
                supported_rates: Vec::new(),
                max_rate: None,
                beacon_interval: None,
                capabilities: AccessPointCapabilities::default(),
            };

            Some(WifiConnection {
                access_point,
                state: WifiConnectionState::Connected,
                auth_state: WifiAuthState::Open,
                tx_rate: None,
                rx_rate: None,
                spatial_streams: None,
                channel_width: None,
                connected_duration: None,
                last_roam: None,
            })
        }
    }

    #[cfg(not(target_os = "ios"))]
    fn get_current_wifi_info(&self) -> Option<WifiConnection> {
        None
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
            mac_address: None,
            powered_on: true, // We can't easily query this without more APIs
            connected: self.get_current_wifi_info().is_some(),
            country_code: None,
        }])
    }

    async fn scan_access_points(&self, _interface: &str) -> Result<Vec<AccessPoint>> {
        // WiFi scanning requires NEHotspotHelper entitlement
        // which is restricted to certain app categories
        Err(Error::UnsupportedOnPlatform {
            feature: "WiFi scanning".to_string(),
            platform: "iOS".to_string(),
            alternative: Some(
                "Use NEHotspotHelper with proper entitlements or check Settings app".to_string(),
            ),
        })
    }

    async fn get_current_connection(&self, _interface: &str) -> Result<Option<WifiConnection>> {
        // This requires the "Access WiFi Information" entitlement
        Ok(self.get_current_wifi_info())
    }

    async fn get_signal_strength(&self, _interface: &str) -> Result<Option<i32>> {
        // Signal strength is not directly available to iOS apps
        // The RSSI value in CNCopyCurrentNetworkInfo was removed in iOS 13
        Ok(None)
    }

    async fn get_noise_level(&self, _interface: &str) -> Result<Option<i32>> {
        // Noise level is not available on iOS
        Ok(None)
    }

    async fn get_channel_utilization(&self, _channel: Channel) -> Result<ChannelUtilization> {
        Err(Error::unsupported("Channel utilization", "iOS"))
    }

    async fn analyze_channels(&self, _interface: &str) -> Result<Vec<ChannelUtilization>> {
        // Channel analysis requires WiFi scanning which is restricted
        Ok(Vec::new())
    }

    fn supports_enterprise(&self) -> bool {
        // iOS supports enterprise WiFi but we can't query it programmatically
        false
    }

    async fn trigger_scan(&self, _interface: &str) -> Result<()> {
        Err(Error::unsupported("WiFi scan trigger", "iOS"))
    }

    async fn get_supported_standards(&self, _interface: &str) -> Result<Vec<WifiStandard>> {
        // Modern iOS devices support up to WiFi 6E
        // but we can't query device capabilities directly
        Ok(vec![
            WifiStandard::Dot11b,
            WifiStandard::Dot11g,
            WifiStandard::Dot11n,
            WifiStandard::Dot11ac,
            WifiStandard::Dot11ax,
        ])
    }
}

/// Parses a MAC address string (e.g., "AA:BB:CC:DD:EE:FF") into a Bssid.
fn parse_bssid(s: &str) -> Option<Bssid> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 6 {
        return None;
    }

    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        bytes[i] = u8::from_str_radix(part, 16).ok()?;
    }

    Some(Bssid::new(bytes))
}
