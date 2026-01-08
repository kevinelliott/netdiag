//! macOS WiFi provider implementation.

use async_trait::async_trait;
use netdiag_platform::{WifiInterface, WifiProvider};
use netdiag_types::{
    error::{Error, Result},
    network::MacAddress,
    wifi::{
        AccessPoint, AccessPointCapabilities, Bssid, Channel, ChannelUtilization, ChannelWidth,
        InterferenceLevel, KeyManagement, SecurityType, Ssid, WifiAuthentication, WifiAuthState,
        WifiBand, WifiConnection, WifiConnectionState, WifiEncryption, WifiStandard,
    },
};
use std::process::Command;

/// macOS WiFi provider using CoreWLAN via airport command.
pub struct MacosWifiProvider {
    /// Path to airport utility
    airport_path: String,
}

impl MacosWifiProvider {
    /// Creates a new macOS WiFi provider.
    pub fn new() -> Self {
        Self {
            airport_path: "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport".to_string(),
        }
    }

    /// Runs airport command with arguments.
    fn run_airport(&self, args: &[&str]) -> Result<String> {
        let output = Command::new(&self.airport_path)
            .args(args)
            .output()
            .map_err(|e| Error::Other {
                context: "airport command".to_string(),
                message: e.to_string(),
            })?;

        if !output.status.success() {
            return Err(Error::Other {
                context: "airport command".to_string(),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Gets current interface info from airport -I.
    fn get_airport_info(&self) -> Result<AirportInfo> {
        let output = self.run_airport(&["-I"])?;
        let mut info = AirportInfo::default();

        for line in output.lines() {
            let line = line.trim();
            if let Some((key, value)) = line.split_once(':') {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "SSID" => info.ssid = Some(value.to_string()),
                    "BSSID" => info.bssid = Self::parse_bssid(value),
                    "agrCtlRSSI" => info.rssi = value.parse().ok(),
                    "agrCtlNoise" => info.noise = value.parse().ok(),
                    "channel" => info.channel = Self::parse_channel(value),
                    "lastTxRate" => info.tx_rate = value.parse().ok(),
                    "maxRate" => info.max_rate = value.parse().ok(),
                    "link auth" => info.auth = Some(value.to_string()),
                    "802.11 auth" => info.auth_80211 = Some(value.to_string()),
                    "MCS" => info.mcs = value.parse().ok(),
                    "NSS" => info.nss = value.parse().ok(),
                    "state" => info.state = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        Ok(info)
    }

    /// Parses a BSSID string into bytes.
    fn parse_bssid(s: &str) -> Option<Bssid> {
        let parts: Vec<u8> = s
            .split(':')
            .filter_map(|p| u8::from_str_radix(p, 16).ok())
            .collect();
        if parts.len() == 6 {
            Some(Bssid::new([parts[0], parts[1], parts[2], parts[3], parts[4], parts[5]]))
        } else {
            None
        }
    }

    /// Parses channel string (e.g., "6" or "36,+1").
    fn parse_channel(s: &str) -> Option<(u8, Option<i8>)> {
        let parts: Vec<&str> = s.split(',').collect();
        let channel: u8 = parts.first()?.parse().ok()?;
        let secondary = parts.get(1).and_then(|s| {
            if s.contains('+') {
                Some(1i8)
            } else if s.contains('-') {
                Some(-1i8)
            } else {
                None
            }
        });
        Some((channel, secondary))
    }

    /// Determines WiFi band from channel number.
    fn channel_to_band(channel: u8) -> WifiBand {
        if channel <= 14 {
            WifiBand::Band2_4GHz
        } else if channel <= 177 {
            WifiBand::Band5GHz
        } else {
            WifiBand::Band6GHz
        }
    }

    /// Determines channel width from secondary channel info.
    fn determine_channel_width(channel: u8, secondary: Option<i8>) -> ChannelWidth {
        match secondary {
            Some(_) if channel > 14 => ChannelWidth::Mhz40, // Could be wider, simplified
            Some(_) => ChannelWidth::Mhz40,
            None => ChannelWidth::Mhz20,
        }
    }

    /// Parses security type from auth string.
    fn parse_security(auth: Option<&str>) -> SecurityType {
        match auth {
            Some("wpa3-personal") | Some("wpa3") => SecurityType::wpa3_personal(),
            Some("wpa3-enterprise") => Self::wpa3_enterprise(),
            Some("wpa2-personal") | Some("wpa2") => SecurityType::wpa2_personal(),
            Some("wpa2-enterprise") | Some("wpa2 802.1x") => SecurityType::wpa2_enterprise(),
            Some("wpa") | Some("wpa-personal") => Self::wpa_personal(),
            Some("wep") => Self::wep(),
            Some("open") | Some("none") | None => SecurityType::open(),
            Some(other) => {
                if other.contains("wpa3") {
                    SecurityType::wpa3_personal()
                } else if other.contains("wpa2") {
                    SecurityType::wpa2_personal()
                } else if other.contains("wpa") {
                    Self::wpa_personal()
                } else {
                    SecurityType::open()
                }
            }
        }
    }

    /// Creates a WPA-Personal security type.
    fn wpa_personal() -> SecurityType {
        SecurityType {
            authentication: WifiAuthentication::Wpa,
            encryption: WifiEncryption::Tkip,
            key_management: KeyManagement::Psk,
            pmf_required: false,
            transition_mode: false,
        }
    }

    /// Creates a WPA3-Enterprise security type.
    fn wpa3_enterprise() -> SecurityType {
        SecurityType {
            authentication: WifiAuthentication::Wpa3,
            encryption: WifiEncryption::Gcmp256,
            key_management: KeyManagement::EapSuiteB192,
            pmf_required: true,
            transition_mode: false,
        }
    }

    /// Creates a WEP security type.
    fn wep() -> SecurityType {
        SecurityType {
            authentication: WifiAuthentication::Wep,
            encryption: WifiEncryption::Wep,
            key_management: KeyManagement::None,
            pmf_required: false,
            transition_mode: false,
        }
    }

    /// Parses WiFi standard from rate and MCS info.
    fn determine_standard(max_rate: Option<f32>, mcs: Option<u8>) -> WifiStandard {
        match (max_rate, mcs) {
            (Some(rate), _) if rate >= 1200.0 => WifiStandard::Dot11ax,
            (Some(rate), _) if rate >= 400.0 => WifiStandard::Dot11ac,
            (Some(rate), Some(_)) if rate >= 100.0 => WifiStandard::Dot11n,
            (Some(rate), _) if rate >= 54.0 => WifiStandard::Dot11g,
            (Some(_), _) => WifiStandard::Dot11b,
            _ => WifiStandard::Unknown,
        }
    }

    /// Gets WiFi interface MAC address.
    fn get_interface_mac(&self, interface: &str) -> Option<MacAddress> {
        let output = Command::new("ifconfig")
            .arg(interface)
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("ether ") {
                if let Some(mac_str) = line.split("ether ").nth(1) {
                    let mac_str = mac_str.split_whitespace().next()?;
                    let parts: Vec<u8> = mac_str
                        .split(':')
                        .filter_map(|p| u8::from_str_radix(p, 16).ok())
                        .collect();
                    if parts.len() == 6 {
                        return Some(MacAddress::new([
                            parts[0], parts[1], parts[2], parts[3], parts[4], parts[5],
                        ]));
                    }
                }
            }
        }
        None
    }
}

impl Default for MacosWifiProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal airport info structure.
#[derive(Debug, Default)]
struct AirportInfo {
    ssid: Option<String>,
    bssid: Option<Bssid>,
    rssi: Option<i32>,
    noise: Option<i32>,
    channel: Option<(u8, Option<i8>)>,
    tx_rate: Option<f32>,
    max_rate: Option<f32>,
    auth: Option<String>,
    auth_80211: Option<String>,
    mcs: Option<u8>,
    nss: Option<u8>,
    state: Option<String>,
}

#[async_trait]
impl WifiProvider for MacosWifiProvider {
    fn is_available(&self) -> bool {
        std::path::Path::new(&self.airport_path).exists()
    }

    async fn list_wifi_interfaces(&self) -> Result<Vec<WifiInterface>> {
        // On macOS, WiFi is typically en0
        // Use networksetup to list WiFi interfaces
        let output = Command::new("networksetup")
            .args(["-listallhardwareports"])
            .output()
            .map_err(|e| Error::Other {
                context: "networksetup".to_string(),
                message: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();
        let mut current_is_wifi = false;
        let mut current_device = None;

        for line in stdout.lines() {
            if line.starts_with("Hardware Port:") {
                current_is_wifi = line.contains("Wi-Fi") || line.contains("AirPort");
            } else if line.starts_with("Device:") && current_is_wifi {
                current_device = line.split(':').nth(1).map(|s| s.trim().to_string());
            } else if line.is_empty() && current_device.is_some() && current_is_wifi {
                let device = current_device.take().unwrap();

                // Check if powered on and get connection status
                let info = self.get_airport_info();
                let powered_on = info.is_ok();
                let connected = info.as_ref().map(|i| i.ssid.is_some()).unwrap_or(false);

                interfaces.push(WifiInterface {
                    name: device.clone(),
                    mac_address: self.get_interface_mac(&device),
                    powered_on,
                    connected,
                    country_code: None, // Would need to parse from airport -I
                });

                current_is_wifi = false;
            }
        }

        Ok(interfaces)
    }

    async fn scan_access_points(&self, _interface: &str) -> Result<Vec<AccessPoint>> {
        let output = self.run_airport(&["-s"])?;
        let mut access_points = Vec::new();

        for line in output.lines().skip(1) {
            // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 7 {
                // Format: SSID BSSID RSSI CHANNEL HT CC SECURITY
                let ssid = parts[0].to_string();
                let bssid = Self::parse_bssid(parts[1]);
                let rssi: i32 = parts[2].parse().unwrap_or(-100);
                let channel_str = parts[3];
                let security_str = parts.get(6..).map(|p| p.join(" ")).unwrap_or_default();

                // Parse channel
                let (channel_num, secondary) = Self::parse_channel(channel_str).unwrap_or((1, None));
                let band = Self::channel_to_band(channel_num);
                let width = Self::determine_channel_width(channel_num, secondary);

                let channel = Channel {
                    number: channel_num,
                    frequency: if band == WifiBand::Band2_4GHz {
                        2407 + (channel_num as u32 * 5)
                    } else {
                        5000 + (channel_num as u32 * 5)
                    },
                    band,
                    width,
                    center_frequency: None,
                    secondary_position: None,
                };

                // Parse security
                let security = if security_str.contains("WPA3") {
                    SecurityType::wpa3_personal()
                } else if security_str.contains("WPA2") && security_str.contains("802.1X") {
                    SecurityType::wpa2_enterprise()
                } else if security_str.contains("WPA2") {
                    SecurityType::wpa2_personal()
                } else if security_str.contains("WPA") {
                    Self::wpa_personal()
                } else if security_str.contains("WEP") {
                    Self::wep()
                } else {
                    SecurityType::open()
                };

                // Signal quality (rough estimate)
                let signal_quality = match rssi {
                    -30..=0 => 100,
                    -50..=-31 => 80,
                    -60..=-51 => 60,
                    -70..=-61 => 40,
                    -80..=-71 => 20,
                    _ => 10,
                };

                if let Some(bssid) = bssid {
                    access_points.push(AccessPoint {
                        ssid: Ssid::new(ssid),
                        bssid,
                        rssi,
                        signal_quality,
                        channel,
                        security,
                        wifi_standard: WifiStandard::Unknown, // Would need more info
                        is_hidden: false,
                        is_connected: false,
                        noise: None,
                        snr: None,
                        country_code: None,
                        supported_rates: Vec::new(),
                        max_rate: None,
                        beacon_interval: None,
                        capabilities: AccessPointCapabilities::default(),
                    });
                }
            }
        }

        Ok(access_points)
    }

    async fn get_current_connection(&self, _interface: &str) -> Result<Option<WifiConnection>> {
        let info = self.get_airport_info()?;

        let ssid = match info.ssid {
            Some(s) => s,
            None => return Ok(None),
        };

        let bssid = info.bssid.unwrap_or(Bssid::new([0; 6]));
        let rssi = info.rssi.unwrap_or(-100);
        let noise = info.noise;

        let (channel_num, secondary) = info.channel.unwrap_or((1, None));
        let band = Self::channel_to_band(channel_num);
        let width = Self::determine_channel_width(channel_num, secondary);

        let channel = Channel {
            number: channel_num,
            frequency: if band == WifiBand::Band2_4GHz {
                2407 + (channel_num as u32 * 5)
            } else {
                5000 + (channel_num as u32 * 5)
            },
            band,
            width,
            center_frequency: None,
            secondary_position: None,
        };

        let security = Self::parse_security(info.auth.as_deref());
        let standard = Self::determine_standard(info.max_rate, info.mcs);

        let signal_quality = match rssi {
            -30..=0 => 100,
            -50..=-31 => 80,
            -60..=-51 => 60,
            -70..=-61 => 40,
            -80..=-71 => 20,
            _ => 10,
        };

        let access_point = AccessPoint {
            ssid: Ssid::new(ssid),
            bssid,
            rssi,
            signal_quality,
            channel,
            security,
            wifi_standard: standard,
            is_hidden: false,
            is_connected: true,
            noise,
            snr: noise.map(|n| rssi - n),
            country_code: None,
            supported_rates: Vec::new(),
            max_rate: info.max_rate,
            beacon_interval: None,
            capabilities: AccessPointCapabilities::default(),
        };

        let state = match info.state.as_deref() {
            Some("running") => WifiConnectionState::Connected,
            Some("init") => WifiConnectionState::Associating,
            Some("auth") => WifiConnectionState::Authenticating,
            _ => WifiConnectionState::Connected,
        };

        let auth_state = match info.auth.as_deref() {
            Some("open") => WifiAuthState::Open,
            Some(a) if a.contains("wpa3") => WifiAuthState::Sae,
            Some(a) if a.contains("802.1x") || a.contains("enterprise") => WifiAuthState::Eap,
            Some(_) => WifiAuthState::Psk,
            None => WifiAuthState::None,
        };

        Ok(Some(WifiConnection {
            access_point,
            state,
            auth_state,
            tx_rate: info.tx_rate,
            rx_rate: info.tx_rate, // airport doesn't separate TX/RX
            spatial_streams: info.nss,
            channel_width: Some(width),
            connected_duration: None,
            last_roam: None,
        }))
    }

    async fn get_signal_strength(&self, _interface: &str) -> Result<Option<i32>> {
        let info = self.get_airport_info()?;
        Ok(info.rssi)
    }

    async fn get_noise_level(&self, _interface: &str) -> Result<Option<i32>> {
        let info = self.get_airport_info()?;
        Ok(info.noise)
    }

    async fn get_channel_utilization(&self, channel: Channel) -> Result<ChannelUtilization> {
        // Scan for networks on the channel
        let aps = self.scan_access_points("en0").await?;
        let networks_on_channel: Vec<_> = aps
            .iter()
            .filter(|ap| ap.channel.number == channel.number)
            .collect();

        let network_count = networks_on_channel.len() as u32;
        let avg_rssi = if networks_on_channel.is_empty() {
            -100
        } else {
            networks_on_channel.iter().map(|ap| ap.rssi).sum::<i32>()
                / networks_on_channel.len() as i32
        };

        // Estimate utilization based on network count and signal strength
        let utilization_percent = match network_count {
            0 => 0.0,
            1..=2 => 10.0 + (avg_rssi.abs() as f32 * 0.2),
            3..=5 => 30.0 + (avg_rssi.abs() as f32 * 0.3),
            _ => 60.0 + (avg_rssi.abs() as f32 * 0.2),
        };

        let interference_level = match network_count {
            0..=1 => InterferenceLevel::Low,
            2..=3 => InterferenceLevel::Medium,
            4..=6 => InterferenceLevel::High,
            _ => InterferenceLevel::Severe,
        };

        let recommended = network_count <= 2 && avg_rssi < -70;

        Ok(ChannelUtilization {
            channel,
            network_count,
            utilization_percent: utilization_percent.min(100.0),
            avg_rssi,
            recommended,
            interference_level,
        })
    }

    async fn analyze_channels(&self, _interface: &str) -> Result<Vec<ChannelUtilization>> {
        let aps = self.scan_access_points("en0").await?;
        let mut utilizations = Vec::new();

        // Common 2.4GHz channels
        for ch in [1u8, 6, 11] {
            let channel = Channel::from_number(ch, WifiBand::Band2_4GHz);
            utilizations.push(self.get_channel_utilization(channel).await?);
        }

        // Common 5GHz channels
        for ch in [36u8, 40, 44, 48, 149, 153, 157, 161, 165] {
            let channel = Channel::from_number(ch, WifiBand::Band5GHz);
            utilizations.push(self.get_channel_utilization(channel).await?);
        }

        Ok(utilizations)
    }

    fn supports_enterprise(&self) -> bool {
        true // macOS supports 802.1X
    }

    async fn trigger_scan(&self, _interface: &str) -> Result<()> {
        // airport -s triggers a scan
        self.run_airport(&["-s"])?;
        Ok(())
    }

    async fn get_supported_standards(&self, _interface: &str) -> Result<Vec<WifiStandard>> {
        // Most modern Macs support these standards
        Ok(vec![
            WifiStandard::Dot11a,
            WifiStandard::Dot11b,
            WifiStandard::Dot11g,
            WifiStandard::Dot11n,
            WifiStandard::Dot11ac,
            WifiStandard::Dot11ax,
        ])
    }
}
