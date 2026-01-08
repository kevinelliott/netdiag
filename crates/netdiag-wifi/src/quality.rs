//! Signal quality assessment.

use netdiag_types::wifi::WifiConnection;
use serde::{Deserialize, Serialize};

/// Signal quality rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SignalQuality {
    /// Excellent signal (-50 dBm or better).
    Excellent,
    /// Good signal (-60 to -51 dBm).
    Good,
    /// Fair signal (-70 to -61 dBm).
    Fair,
    /// Weak signal (-80 to -71 dBm).
    Weak,
    /// Very weak signal (below -80 dBm).
    VeryWeak,
}

impl SignalQuality {
    /// Create signal quality from RSSI value.
    pub fn from_rssi(rssi: i32) -> Self {
        match rssi {
            r if r >= -50 => SignalQuality::Excellent,
            r if r >= -60 => SignalQuality::Good,
            r if r >= -70 => SignalQuality::Fair,
            r if r >= -80 => SignalQuality::Weak,
            _ => SignalQuality::VeryWeak,
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            SignalQuality::Excellent => "Excellent - best possible connection quality",
            SignalQuality::Good => "Good - reliable for all activities",
            SignalQuality::Fair => "Fair - may experience occasional issues",
            SignalQuality::Weak => "Weak - may drop or be slow",
            SignalQuality::VeryWeak => "Very weak - connection unstable",
        }
    }

    /// Get score out of 100.
    pub fn score(&self) -> u8 {
        match self {
            SignalQuality::Excellent => 100,
            SignalQuality::Good => 80,
            SignalQuality::Fair => 60,
            SignalQuality::Weak => 40,
            SignalQuality::VeryWeak => 20,
        }
    }
}

impl std::fmt::Display for SignalQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SignalQuality::Excellent => "Excellent",
            SignalQuality::Good => "Good",
            SignalQuality::Fair => "Fair",
            SignalQuality::Weak => "Weak",
            SignalQuality::VeryWeak => "Very Weak",
        };
        write!(f, "{}", s)
    }
}

/// Comprehensive quality metrics for a WiFi connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Signal strength (RSSI) in dBm.
    pub rssi: i32,

    /// Noise level in dBm (if available).
    pub noise: Option<i32>,

    /// Signal-to-noise ratio in dB.
    pub snr: Option<f64>,

    /// Signal quality rating.
    pub quality: SignalQuality,

    /// Link quality percentage (0-100).
    pub link_quality: u8,

    /// Transmit rate in Mbps.
    pub tx_rate: Option<f64>,

    /// Receive rate in Mbps.
    pub rx_rate: Option<f64>,

    /// MCS index (if available).
    pub mcs_index: Option<u8>,

    /// Channel width in MHz.
    pub channel_width: Option<u16>,

    /// Number of spatial streams.
    pub spatial_streams: Option<u8>,

    /// Overall score (0-100).
    pub overall_score: u8,

    /// Recommendations for improvement.
    pub recommendations: Vec<String>,
}

impl QualityMetrics {
    /// Create quality metrics from raw signal data.
    pub fn new(rssi: i32, noise: Option<i32>) -> Self {
        let quality = SignalQuality::from_rssi(rssi);

        let snr = noise.map(|n| (rssi - n) as f64);

        let link_quality = Self::calculate_link_quality(rssi);

        let overall_score = Self::calculate_overall_score(rssi, snr);

        let recommendations = Self::generate_recommendations(rssi, snr, &quality);

        Self {
            rssi,
            noise,
            snr,
            quality,
            link_quality,
            tx_rate: None,
            rx_rate: None,
            mcs_index: None,
            channel_width: None,
            spatial_streams: None,
            overall_score,
            recommendations,
        }
    }

    /// Create from a WiFi connection.
    pub fn from_connection(connection: &WifiConnection) -> Self {
        let rssi = connection.access_point.rssi;
        let noise = connection.access_point.noise;
        let mut metrics = Self::new(rssi, noise);
        metrics.tx_rate = connection.tx_rate.map(|r| r as f64);
        metrics.channel_width = Some(connection.access_point.channel.width.mhz() as u16);
        metrics
    }

    /// Calculate link quality percentage from RSSI.
    fn calculate_link_quality(rssi: i32) -> u8 {
        // Convert RSSI to percentage (roughly -100 dBm = 0%, -30 dBm = 100%)
        let quality = ((rssi + 100).clamp(0, 70) as f64 / 70.0 * 100.0) as u8;
        quality.clamp(0, 100)
    }

    /// Calculate overall score considering multiple factors.
    fn calculate_overall_score(rssi: i32, snr: Option<f64>) -> u8 {
        let rssi_score = Self::calculate_link_quality(rssi) as f64;

        let snr_score = snr
            .map(|s| {
                // SNR: 20+ dB is excellent, 10-20 dB is good, below 10 dB is poor
                if s >= 25.0 {
                    100.0
                } else if s >= 15.0 {
                    80.0
                } else if s >= 10.0 {
                    60.0
                } else {
                    40.0
                }
            })
            .unwrap_or(rssi_score);

        // Weight RSSI and SNR equally
        ((rssi_score + snr_score) / 2.0) as u8
    }

    /// Generate recommendations based on metrics.
    fn generate_recommendations(
        rssi: i32,
        snr: Option<f64>,
        quality: &SignalQuality,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        match quality {
            SignalQuality::VeryWeak => {
                recommendations.push(
                    "Move closer to the access point or add a WiFi extender".to_string(),
                );
                recommendations.push(
                    "Check for physical obstructions (walls, large metal objects)".to_string(),
                );
            }
            SignalQuality::Weak => {
                recommendations
                    .push("Consider repositioning for better signal strength".to_string());
            }
            SignalQuality::Fair => {
                recommendations.push(
                    "Signal is adequate but could be improved by reducing distance to AP"
                        .to_string(),
                );
            }
            _ => {}
        }

        if let Some(snr) = snr {
            if snr < 15.0 {
                recommendations
                    .push("High noise environment - check for interference sources".to_string());
            }
        }

        if rssi > -60 && quality != &SignalQuality::Excellent {
            recommendations.push(
                "Good signal but high noise - look for nearby interference".to_string(),
            );
        }

        recommendations
    }

    /// Check if connection is suitable for video streaming.
    pub fn suitable_for_video(&self) -> bool {
        self.quality >= SignalQuality::Fair && self.tx_rate.unwrap_or(10.0) >= 5.0
    }

    /// Check if connection is suitable for gaming.
    pub fn suitable_for_gaming(&self) -> bool {
        self.quality >= SignalQuality::Good && self.snr.unwrap_or(20.0) >= 15.0
    }

    /// Check if connection is suitable for video conferencing.
    pub fn suitable_for_conferencing(&self) -> bool {
        self.quality >= SignalQuality::Fair
            && self.tx_rate.unwrap_or(2.0) >= 1.5
            && self.snr.unwrap_or(15.0) >= 10.0
    }
}
