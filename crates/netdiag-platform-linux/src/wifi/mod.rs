//! Linux WiFi provider implementation.

use async_trait::async_trait;
use netdiag_platform::WifiProvider;
use netdiag_types::error::{Error, Result};
use netdiag_types::wifi::{
    AccessPoint, Channel, ChannelWidth, ConnectionInfo, FrequencyBand, SecurityType, WifiInterface,
};
use std::process::Command;
use tracing::debug;

/// Linux WiFi provider using iw and NetworkManager.
pub struct LinuxWifiProvider;

impl LinuxWifiProvider {
    /// Creates a new Linux WiFi provider.
    pub fn new() -> Self {
        Self
    }

    /// Parse iw scan output.
    fn parse_iw_scan(&self, output: &str) -> Vec<AccessPoint> {
        let mut aps = Vec::new();
        let mut current_ap: Option<AccessPointBuilder> = None;

        for line in output.lines() {
            let line = line.trim();

            // New BSS (access point)
            if line.starts_with("BSS ") {
                // Save previous AP if exists
                if let Some(builder) = current_ap.take() {
                    if let Some(ap) = builder.build() {
                        aps.push(ap);
                    }
                }

                // Start new AP
                let bssid = line
                    .strip_prefix("BSS ")
                    .and_then(|s| s.split('(').next())
                    .map(|s| s.trim().to_uppercase());

                current_ap = Some(AccessPointBuilder::new(bssid));
            } else if let Some(ref mut builder) = current_ap {
                // Parse AP properties
                if let Some(ssid) = line.strip_prefix("SSID: ") {
                    builder.ssid = Some(ssid.to_string());
                } else if let Some(freq) = line.strip_prefix("freq: ") {
                    if let Ok(f) = freq.parse::<u32>() {
                        builder.frequency = Some(f);
                        builder.band = Some(if f < 3000 {
                            FrequencyBand::Band2_4GHz
                        } else if f < 6000 {
                            FrequencyBand::Band5GHz
                        } else {
                            FrequencyBand::Band6GHz
                        });
                    }
                } else if let Some(signal) = line.strip_prefix("signal: ") {
                    // Signal format: "-XX.XX dBm"
                    if let Some(dbm_str) = signal.split_whitespace().next() {
                        if let Ok(dbm) = dbm_str.parse::<f32>() {
                            builder.signal_strength = Some(dbm as i32);
                        }
                    }
                } else if line.contains("WPA:") || line.contains("RSN:") {
                    builder.has_wpa = true;
                } else if line.contains("WEP") {
                    builder.has_wep = true;
                } else if line.contains("Privacy") {
                    builder.has_privacy = true;
                }
            }
        }

        // Don't forget the last AP
        if let Some(builder) = current_ap {
            if let Some(ap) = builder.build() {
                aps.push(ap);
            }
        }

        aps
    }

    /// Parse nmcli connection info.
    fn parse_nmcli_connection(&self, output: &str) -> Option<ConnectionInfo> {
        let mut ssid = None;
        let mut bssid = None;
        let mut signal = None;
        let mut frequency = None;
        let mut tx_rate = None;

        for line in output.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim();
            let value = parts[1].trim();

            match key {
                "GENERAL.CONNECTION" | "WIFI.SSID" => {
                    if !value.is_empty() && value != "--" {
                        ssid = Some(value.to_string());
                    }
                }
                "WIFI.BSSID" | "AP.BSSID" => {
                    if !value.is_empty() && value != "--" {
                        bssid = Some(value.to_uppercase());
                    }
                }
                "WIFI.SIGNAL" | "AP.SIGNAL" => {
                    if let Ok(s) = value.parse::<i32>() {
                        // nmcli returns percentage, convert to approximate dBm
                        signal = Some(-100 + s);
                    }
                }
                "WIFI.FREQ" | "AP.FREQ" => {
                    // Format: "2437 MHz"
                    if let Some(freq_str) = value.split_whitespace().next() {
                        frequency = freq_str.parse().ok();
                    }
                }
                "WIFI.RATE" | "AP.RATE" => {
                    // Format: "54 Mbit/s"
                    if let Some(rate_str) = value.split_whitespace().next() {
                        tx_rate = rate_str.parse().ok();
                    }
                }
                _ => {}
            }
        }

        ssid.map(|ssid| ConnectionInfo {
            ssid,
            bssid,
            signal_strength: signal,
            noise_level: None,
            channel: frequency.map(|f| freq_to_channel(f)),
            tx_rate,
            security: SecurityType::unknown(),
            connected_at: None,
        })
    }
}

impl Default for LinuxWifiProvider {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to convert frequency to channel number.
fn freq_to_channel(freq: u32) -> Channel {
    let number = match freq {
        2412 => 1,
        2417 => 2,
        2422 => 3,
        2427 => 4,
        2432 => 5,
        2437 => 6,
        2442 => 7,
        2447 => 8,
        2452 => 9,
        2457 => 10,
        2462 => 11,
        2467 => 12,
        2472 => 13,
        2484 => 14,
        5180 => 36,
        5200 => 40,
        5220 => 44,
        5240 => 48,
        5260 => 52,
        5280 => 56,
        5300 => 60,
        5320 => 64,
        5500 => 100,
        5520 => 104,
        5540 => 108,
        5560 => 112,
        5580 => 116,
        5600 => 120,
        5620 => 124,
        5640 => 128,
        5660 => 132,
        5680 => 136,
        5700 => 140,
        5720 => 144,
        5745 => 149,
        5765 => 153,
        5785 => 157,
        5805 => 161,
        5825 => 165,
        _ => 0,
    };

    let band = if freq < 3000 {
        FrequencyBand::Band2_4GHz
    } else if freq < 6000 {
        FrequencyBand::Band5GHz
    } else {
        FrequencyBand::Band6GHz
    };

    Channel {
        number: number as u8,
        frequency: freq,
        band,
        width: ChannelWidth::Width20MHz,
        is_dfs: (52..=144).contains(&number),
    }
}

/// Builder for constructing AccessPoint from parsed data.
struct AccessPointBuilder {
    bssid: Option<String>,
    ssid: Option<String>,
    signal_strength: Option<i32>,
    frequency: Option<u32>,
    band: Option<FrequencyBand>,
    has_wpa: bool,
    has_wep: bool,
    has_privacy: bool,
}

impl AccessPointBuilder {
    fn new(bssid: Option<String>) -> Self {
        Self {
            bssid,
            ssid: None,
            signal_strength: None,
            frequency: None,
            band: None,
            has_wpa: false,
            has_wep: false,
            has_privacy: false,
        }
    }

    fn build(self) -> Option<AccessPoint> {
        let ssid = self.ssid.unwrap_or_default();
        let bssid = self.bssid?;

        let security = if self.has_wpa {
            SecurityType::wpa2_personal()
        } else if self.has_wep {
            SecurityType::wep()
        } else if self.has_privacy {
            SecurityType::unknown()
        } else {
            SecurityType::open()
        };

        let channel = self.frequency.map(freq_to_channel);

        Some(AccessPoint {
            ssid,
            bssid,
            signal_strength: self.signal_strength.unwrap_or(-100),
            noise_level: None,
            channel,
            security,
            is_hidden: false,
            vendor: None,
            supported_rates: Vec::new(),
            country_code: None,
        })
    }
}

#[async_trait]
impl WifiProvider for LinuxWifiProvider {
    async fn list_wifi_interfaces(&self) -> Result<Vec<WifiInterface>> {
        debug!("Listing WiFi interfaces on Linux");

        let output = Command::new("iw")
            .args(["dev"])
            .output()
            .map_err(|e| Error::platform("iw dev", &e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();
        let mut current_iface: Option<String> = None;

        for line in stdout.lines() {
            let line = line.trim();

            if line.starts_with("Interface ") {
                current_iface = line.strip_prefix("Interface ").map(|s| s.to_string());
            } else if let Some(ref iface) = current_iface {
                if line.starts_with("addr ") {
                    let mac = line.strip_prefix("addr ").map(|s| s.to_uppercase());

                    interfaces.push(WifiInterface {
                        name: iface.clone(),
                        mac_address: mac,
                        is_up: true,                 // Would need additional check
                        supports_monitor_mode: true, // Most Linux WiFi interfaces do
                        supported_bands: vec![FrequencyBand::Band2_4GHz, FrequencyBand::Band5GHz],
                    });
                }
            }
        }

        debug!("Found {} WiFi interfaces", interfaces.len());
        Ok(interfaces)
    }

    async fn scan_access_points(&self, interface: &str) -> Result<Vec<AccessPoint>> {
        debug!("Scanning for access points on {} (Linux)", interface);

        // Try iw scan first (requires root or CAP_NET_ADMIN)
        let output = Command::new("iw")
            .args(["dev", interface, "scan"])
            .output()
            .map_err(|e| Error::platform("iw scan", &e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(self.parse_iw_scan(&stdout));
        }

        // Fallback: try nmcli
        let output = Command::new("nmcli")
            .args([
                "-t",
                "-f",
                "BSSID,SSID,FREQ,SIGNAL,SECURITY",
                "device",
                "wifi",
                "list",
            ])
            .output()
            .map_err(|e| Error::platform("nmcli wifi list", &e.to_string()))?;

        if !output.status.success() {
            return Err(Error::platform(
                "WiFi scan",
                "Failed to scan - try running with sudo or ensure NetworkManager is running",
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut aps = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 5 {
                let bssid = parts[0].to_uppercase();
                let ssid = parts[1].to_string();
                let freq: u32 = parts[2].parse().unwrap_or(0);
                let signal: i32 = parts[3].parse().map(|s: i32| -100 + s).unwrap_or(-100);
                let security_str = parts[4];

                let security = if security_str.contains("WPA3") {
                    SecurityType::wpa3_personal()
                } else if security_str.contains("WPA2") {
                    SecurityType::wpa2_personal()
                } else if security_str.contains("WPA") {
                    SecurityType::wpa_personal()
                } else if security_str.contains("WEP") {
                    SecurityType::wep()
                } else {
                    SecurityType::open()
                };

                aps.push(AccessPoint {
                    ssid,
                    bssid,
                    signal_strength: signal,
                    noise_level: None,
                    channel: Some(freq_to_channel(freq)),
                    security,
                    is_hidden: false,
                    vendor: None,
                    supported_rates: Vec::new(),
                    country_code: None,
                });
            }
        }

        Ok(aps)
    }

    async fn get_current_connection(&self, interface: &str) -> Result<Option<ConnectionInfo>> {
        debug!("Getting current WiFi connection on {} (Linux)", interface);

        // Try nmcli first
        let output = Command::new("nmcli")
            .args(["-t", "device", "show", interface])
            .output()
            .map_err(|e| Error::platform("nmcli device show", &e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Ok(self.parse_nmcli_connection(&stdout));
        }

        // Fallback: try iw
        let output = Command::new("iw")
            .args(["dev", interface, "link"])
            .output()
            .map_err(|e| Error::platform("iw link", &e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if stdout.contains("Not connected") {
            return Ok(None);
        }

        let mut ssid = None;
        let mut bssid = None;
        let mut freq = None;
        let mut signal = None;
        let mut tx_rate = None;

        for line in stdout.lines() {
            let line = line.trim();

            if line.starts_with("Connected to ") {
                bssid = line
                    .strip_prefix("Connected to ")
                    .map(|s| s.split_whitespace().next().unwrap_or("").to_uppercase());
            } else if line.starts_with("SSID: ") {
                ssid = line.strip_prefix("SSID: ").map(|s| s.to_string());
            } else if line.starts_with("freq: ") {
                freq = line
                    .strip_prefix("freq: ")
                    .and_then(|s| s.parse::<u32>().ok());
            } else if line.starts_with("signal: ") {
                signal = line
                    .strip_prefix("signal: ")
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|s| s.parse::<i32>().ok());
            } else if line.starts_with("tx bitrate: ") {
                tx_rate = line
                    .strip_prefix("tx bitrate: ")
                    .and_then(|s| s.split_whitespace().next())
                    .and_then(|s| s.parse::<f32>().ok())
                    .map(|r| r as u32);
            }
        }

        Ok(ssid.map(|ssid| ConnectionInfo {
            ssid,
            bssid,
            signal_strength: signal,
            noise_level: None,
            channel: freq.map(freq_to_channel),
            tx_rate,
            security: SecurityType::unknown(),
            connected_at: None,
        }))
    }

    async fn get_signal_strength(&self, interface: &str) -> Result<Option<i32>> {
        if let Some(conn) = self.get_current_connection(interface).await? {
            return Ok(conn.signal_strength);
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_wifi_interfaces() {
        let provider = LinuxWifiProvider::new();
        // This test will only pass on Linux systems with WiFi
        let _ = provider.list_wifi_interfaces().await;
    }
}
