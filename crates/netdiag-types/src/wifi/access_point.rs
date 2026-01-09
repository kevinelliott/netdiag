//! `WiFi` access point types.

use super::{Channel, SecurityType, WifiStandard};
use serde::{Deserialize, Serialize};

/// Represents a `WiFi` access point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPoint {
    /// SSID (network name)
    pub ssid: Ssid,
    /// BSSID (MAC address of access point)
    pub bssid: Bssid,
    /// Signal strength in dBm
    pub rssi: i32,
    /// Signal quality as percentage (0-100)
    pub signal_quality: u8,
    /// Channel information
    pub channel: Channel,
    /// Security type
    pub security: SecurityType,
    /// `WiFi` standard (802.11a/b/g/n/ac/ax)
    pub wifi_standard: WifiStandard,
    /// Is this network hidden?
    pub is_hidden: bool,
    /// Is this the currently connected network?
    pub is_connected: bool,
    /// Noise level in dBm (if available)
    pub noise: Option<i32>,
    /// SNR (Signal-to-Noise Ratio) in dB
    pub snr: Option<i32>,
    /// Country code
    pub country_code: Option<String>,
    /// Supported data rates in Mbps
    pub supported_rates: Vec<f32>,
    /// Maximum data rate in Mbps
    pub max_rate: Option<f32>,
    /// Beacon interval in milliseconds
    pub beacon_interval: Option<u16>,
    /// Capabilities
    pub capabilities: AccessPointCapabilities,
}

impl AccessPoint {
    /// Returns the signal strength as a human-readable string.
    #[must_use]
    pub fn signal_strength_label(&self) -> &'static str {
        match self.rssi {
            -30..=0 => "Excellent",
            -50..=-31 => "Very Good",
            -60..=-51 => "Good",
            -70..=-61 => "Fair",
            -80..=-71 => "Weak",
            _ => "Very Weak",
        }
    }

    /// Calculates SNR from RSSI and noise.
    #[must_use]
    pub fn calculate_snr(&self) -> Option<i32> {
        self.snr.or_else(|| self.noise.map(|n| self.rssi - n))
    }
}

/// SSID (Service Set Identifier).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ssid(String);

impl Ssid {
    /// Creates a new SSID.
    #[must_use]
    pub fn new(ssid: impl Into<String>) -> Self {
        Self(ssid.into())
    }

    /// Creates an SSID for a hidden network.
    #[must_use]
    pub fn hidden() -> Self {
        Self(String::new())
    }

    /// Returns true if this is a hidden SSID.
    #[must_use]
    pub fn is_hidden(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the SSID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Ssid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_hidden() {
            write!(f, "<hidden>")
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl From<String> for Ssid {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for Ssid {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// BSSID (Basic Service Set Identifier - MAC address of AP).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bssid([u8; 6]);

impl Bssid {
    /// Creates a new BSSID from bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Returns the bytes of the BSSID.
    #[must_use]
    pub const fn octets(&self) -> [u8; 6] {
        self.0
    }
}

impl std::fmt::Display for Bssid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// Access point capabilities.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AccessPointCapabilities {
    /// Supports WMM (`WiFi` Multimedia)
    pub wmm: bool,
    /// Supports WPS (`WiFi` Protected Setup)
    pub wps: bool,
    /// Is an infrastructure (AP) mode
    pub infrastructure: bool,
    /// Is ad-hoc mode
    pub adhoc: bool,
    /// Supports short slot time
    pub short_slot_time: bool,
    /// Supports short preamble
    pub short_preamble: bool,
    /// Supports ESS (Extended Service Set)
    pub ess: bool,
    /// Supports IBSS (Independent Basic Service Set)
    pub ibss: bool,
    /// Privacy enabled (encryption required)
    pub privacy: bool,
    /// Supports spectrum management
    pub spectrum_management: bool,
    /// Supports radio measurement
    pub radio_measurement: bool,
    /// Supports 802.11k (neighbor reports)
    pub dot11k: bool,
    /// Supports 802.11r (fast BSS transition)
    pub dot11r: bool,
    /// Supports 802.11v (BSS transition management)
    pub dot11v: bool,
}

/// `WiFi` connection information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConnection {
    /// Connected access point
    pub access_point: AccessPoint,
    /// Connection state
    pub state: WifiConnectionState,
    /// Authentication state
    pub auth_state: WifiAuthState,
    /// TX (transmit) rate in Mbps
    pub tx_rate: Option<f32>,
    /// RX (receive) rate in Mbps
    pub rx_rate: Option<f32>,
    /// Number of spatial streams (MIMO)
    pub spatial_streams: Option<u8>,
    /// Channel width in use
    pub channel_width: Option<super::ChannelWidth>,
    /// Connection duration
    pub connected_duration: Option<std::time::Duration>,
    /// Last roamed time (if applicable)
    pub last_roam: Option<chrono::DateTime<chrono::Utc>>,
}

/// `WiFi` connection state.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WifiConnectionState {
    /// Disconnected
    Disconnected,
    /// Scanning for networks
    Scanning,
    /// Associating with AP
    Associating,
    /// Authenticating
    Authenticating,
    /// Connected
    Connected,
    /// Connection failed
    Failed,
}

/// `WiFi` authentication state.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, strum::EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WifiAuthState {
    /// Not authenticated
    None,
    /// Open (no authentication)
    Open,
    /// PSK (Pre-Shared Key) authenticated
    Psk,
    /// EAP authenticated
    Eap,
    /// SAE (Simultaneous Authentication of Equals - WPA3)
    Sae,
    /// OWE (Opportunistic Wireless Encryption)
    Owe,
    /// Authentication failed
    Failed,
}
