//! `WiFi` channel types.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// `WiFi` channel information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Channel {
    /// Channel number
    pub number: u8,
    /// Frequency in MHz
    pub frequency: u32,
    /// Frequency band
    pub band: WifiBand,
    /// Channel width
    pub width: ChannelWidth,
    /// Center frequency for bonded channels
    pub center_frequency: Option<u32>,
    /// Secondary channel position (for 40MHz+)
    pub secondary_position: Option<SecondaryChannelPosition>,
}

impl Channel {
    /// Creates a channel from a channel number.
    #[must_use]
    pub fn from_number(number: u8, band: WifiBand) -> Self {
        let frequency = match band {
            WifiBand::Band2_4GHz => 2407 + (u32::from(number) * 5),
            WifiBand::Band5GHz => {
                if (36..=64).contains(&number) || (100..=165).contains(&number) {
                    5000 + (u32::from(number) * 5)
                } else {
                    5000
                }
            }
            WifiBand::Band6GHz => 5950 + (u32::from(number) * 5),
        };

        Self {
            number,
            frequency,
            band,
            width: ChannelWidth::Mhz20,
            center_frequency: None,
            secondary_position: None,
        }
    }

    /// Creates a channel from a frequency.
    #[must_use]
    pub fn from_frequency(frequency: u32) -> Self {
        let (number, band) = if (2400..=2500).contains(&frequency) {
            #[allow(clippy::cast_possible_truncation)]
            let number = ((frequency - 2407) / 5) as u8;
            (number, WifiBand::Band2_4GHz)
        } else if (5000..=5900).contains(&frequency) {
            #[allow(clippy::cast_possible_truncation)]
            let number = ((frequency - 5000) / 5) as u8;
            (number, WifiBand::Band5GHz)
        } else if (5925..=7125).contains(&frequency) {
            #[allow(clippy::cast_possible_truncation)]
            let number = ((frequency - 5950) / 5) as u8;
            (number, WifiBand::Band6GHz)
        } else {
            (0, WifiBand::Band2_4GHz)
        };

        Self {
            number,
            frequency,
            band,
            width: ChannelWidth::Mhz20,
            center_frequency: None,
            secondary_position: None,
        }
    }

    /// Returns true if this is a DFS (Dynamic Frequency Selection) channel.
    #[must_use]
    pub fn is_dfs(&self) -> bool {
        if self.band != WifiBand::Band5GHz {
            return false;
        }
        // DFS channels: 52-64, 100-144 in the US
        (self.number >= 52 && self.number <= 64) || (self.number >= 100 && self.number <= 144)
    }

    /// Returns the list of overlapping channels in 2.4GHz band.
    #[must_use]
    pub fn overlapping_channels(&self) -> Vec<u8> {
        if self.band != WifiBand::Band2_4GHz {
            return Vec::new();
        }

        // 2.4GHz channels overlap with channels within +/- 4
        let min = self.number.saturating_sub(4);
        let max = std::cmp::min(self.number + 4, 14);
        (min..=max)
            .filter(|&c| c != self.number && c >= 1)
            .collect()
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} MHz, {})", self.number, self.frequency, self.band)
    }
}

/// `WiFi` frequency band.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum WifiBand {
    /// 2.4 GHz band
    #[strum(serialize = "2.4GHz")]
    #[serde(rename = "2.4ghz")]
    Band2_4GHz,
    /// 5 GHz band
    #[strum(serialize = "5GHz")]
    #[serde(rename = "5ghz")]
    Band5GHz,
    /// 6 GHz band (`WiFi` 6E)
    #[strum(serialize = "6GHz")]
    #[serde(rename = "6ghz")]
    Band6GHz,
}

/// `WiFi` channel width.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum ChannelWidth {
    /// 20 MHz
    #[default]
    #[strum(serialize = "20MHz")]
    Mhz20,
    /// 40 MHz
    #[strum(serialize = "40MHz")]
    Mhz40,
    /// 80 MHz
    #[strum(serialize = "80MHz")]
    Mhz80,
    /// 160 MHz
    #[strum(serialize = "160MHz")]
    Mhz160,
    /// 320 MHz (`WiFi` 7)
    #[strum(serialize = "320MHz")]
    Mhz320,
}

impl ChannelWidth {
    /// Returns the width in MHz.
    #[must_use]
    pub const fn mhz(&self) -> u32 {
        match self {
            Self::Mhz20 => 20,
            Self::Mhz40 => 40,
            Self::Mhz80 => 80,
            Self::Mhz160 => 160,
            Self::Mhz320 => 320,
        }
    }
}

/// Secondary channel position for bonded channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum SecondaryChannelPosition {
    /// Above the primary channel
    Above,
    /// Below the primary channel
    Below,
    /// Not applicable (20MHz only)
    None,
}

/// `WiFi` standard (802.11 variant).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum WifiStandard {
    /// 802.11a (5 GHz, up to 54 Mbps)
    #[strum(serialize = "802.11a")]
    Dot11a,
    /// 802.11b (2.4 GHz, up to 11 Mbps)
    #[strum(serialize = "802.11b")]
    Dot11b,
    /// 802.11g (2.4 GHz, up to 54 Mbps)
    #[strum(serialize = "802.11g")]
    Dot11g,
    /// 802.11n (`WiFi` 4, up to 600 Mbps)
    #[strum(serialize = "802.11n")]
    Dot11n,
    /// 802.11ac (`WiFi` 5, up to 3.5 Gbps)
    #[strum(serialize = "802.11ac")]
    Dot11ac,
    /// 802.11ax (`WiFi` 6/6E, up to 9.6 Gbps)
    #[default]
    #[strum(serialize = "802.11ax")]
    Dot11ax,
    /// 802.11be (`WiFi` 7, up to 46 Gbps)
    #[strum(serialize = "802.11be")]
    Dot11be,
    /// Unknown standard
    #[strum(serialize = "unknown")]
    Unknown,
}

impl WifiStandard {
    /// Returns the marketing name (`WiFi` 4, 5, 6, etc.)
    #[must_use]
    pub const fn marketing_name(&self) -> Option<&'static str> {
        match self {
            Self::Dot11n => Some("WiFi 4"),
            Self::Dot11ac => Some("WiFi 5"),
            Self::Dot11ax => Some("WiFi 6/6E"),
            Self::Dot11be => Some("WiFi 7"),
            _ => None,
        }
    }

    /// Returns the maximum theoretical speed in Mbps.
    #[must_use]
    pub const fn max_speed_mbps(&self) -> u32 {
        match self {
            Self::Dot11a | Self::Dot11g => 54,
            Self::Dot11b => 11,
            Self::Dot11n => 600,
            Self::Dot11ac => 3500,
            Self::Dot11ax => 9600,
            Self::Dot11be => 46_000,
            Self::Unknown => 0,
        }
    }
}

/// Channel utilization information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelUtilization {
    /// Channel
    pub channel: Channel,
    /// Number of networks on this channel
    pub network_count: u32,
    /// Estimated utilization percentage (0-100)
    pub utilization_percent: f32,
    /// Average signal strength of networks on this channel
    pub avg_rssi: i32,
    /// Is this a recommended channel?
    pub recommended: bool,
    /// Interference level (low, medium, high)
    pub interference_level: InterferenceLevel,
}

/// Interference level.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum InterferenceLevel {
    /// No significant interference
    #[default]
    Low,
    /// Moderate interference
    Medium,
    /// High interference
    High,
    /// Severe interference (consider changing channels)
    Severe,
}
