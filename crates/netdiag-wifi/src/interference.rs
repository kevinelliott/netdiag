//! Interference detection and analysis.

use netdiag_types::wifi::{AccessPoint, Channel, WifiBand};
use serde::{Deserialize, Serialize};

/// Interference analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceAnalysis {
    /// Overall interference level (0-100, higher is worse).
    pub interference_score: u8,

    /// Detected interference sources.
    pub sources: Vec<InterferenceSource>,

    /// Co-channel interference (same channel).
    pub co_channel_count: usize,

    /// Adjacent channel interference.
    pub adjacent_channel_count: usize,

    /// Non-WiFi interference detected.
    pub non_wifi_detected: bool,

    /// Recommendations.
    pub recommendations: Vec<String>,
}

/// Detected interference source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceSource {
    /// Type of interference.
    pub source_type: InterferenceType,

    /// Severity (0-100).
    pub severity: u8,

    /// Description of the source.
    pub description: String,

    /// Affected channel(s).
    pub affected_channels: Vec<u8>,

    /// Signal strength (if measurable).
    pub signal_strength: Option<i32>,

    /// SSID (if from WiFi AP).
    pub ssid: Option<String>,
}

/// Type of interference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InterferenceType {
    /// Same channel WiFi networks.
    CoChannel,
    /// Adjacent channel WiFi overlap.
    AdjacentChannel,
    /// Bluetooth devices (2.4 GHz).
    Bluetooth,
    /// Microwave ovens (2.4 GHz).
    Microwave,
    /// Cordless phones.
    CordlessPhone,
    /// Baby monitors.
    BabyMonitor,
    /// Wireless video (cameras, etc.).
    WirelessVideo,
    /// Radar (5 GHz DFS).
    Radar,
    /// Unknown 2.4 GHz interference.
    Unknown2_4GHz,
    /// Unknown 5 GHz interference.
    Unknown5GHz,
}

impl InterferenceType {
    /// Get a description of this interference type.
    pub fn description(&self) -> &'static str {
        match self {
            InterferenceType::CoChannel => "Same-channel WiFi network",
            InterferenceType::AdjacentChannel => "Overlapping adjacent channel WiFi",
            InterferenceType::Bluetooth => "Bluetooth device",
            InterferenceType::Microwave => "Microwave oven",
            InterferenceType::CordlessPhone => "Cordless phone",
            InterferenceType::BabyMonitor => "Baby monitor",
            InterferenceType::WirelessVideo => "Wireless video device",
            InterferenceType::Radar => "Weather/aviation radar",
            InterferenceType::Unknown2_4GHz => "Unknown 2.4 GHz interference",
            InterferenceType::Unknown5GHz => "Unknown 5 GHz interference",
        }
    }

    /// Get mitigation advice.
    pub fn mitigation(&self) -> &'static str {
        match self {
            InterferenceType::CoChannel => "Switch to a less crowded channel",
            InterferenceType::AdjacentChannel => "Use channels 1, 6, or 11 only (2.4 GHz)",
            InterferenceType::Bluetooth => "Keep Bluetooth devices away from router",
            InterferenceType::Microwave => "Position router away from kitchen",
            InterferenceType::CordlessPhone => "Use DECT phones or move router",
            InterferenceType::BabyMonitor => "Use digital baby monitor or 5 GHz WiFi",
            InterferenceType::WirelessVideo => "Use wired cameras or different frequency",
            InterferenceType::Radar => "Use non-DFS channels",
            InterferenceType::Unknown2_4GHz => "Switch to 5 GHz band if possible",
            InterferenceType::Unknown5GHz => "Try different 5 GHz channels",
        }
    }
}

impl std::fmt::Display for InterferenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl InterferenceAnalysis {
    /// Analyze interference from scan results.
    pub fn analyze(
        access_points: &[AccessPoint],
        current_channel: Option<&Channel>,
        noise_level: Option<i32>,
    ) -> Self {
        let mut sources = Vec::new();
        let mut co_channel_count = 0;
        let mut adjacent_channel_count = 0;

        let current_ch = current_channel.map(|c| c.number).unwrap_or(0);
        let current_band = current_channel
            .map(|c| c.band.clone())
            .unwrap_or(WifiBand::Band2_4GHz);

        // Analyze WiFi interference
        for ap in access_points {
            if ap.channel.number == current_ch {
                // Co-channel interference
                co_channel_count += 1;
                sources.push(InterferenceSource {
                    source_type: InterferenceType::CoChannel,
                    severity: Self::calculate_severity(ap.rssi),
                    description: format!("WiFi network '{}' on same channel", ap.ssid),
                    affected_channels: vec![current_ch],
                    signal_strength: Some(ap.rssi),
                    ssid: Some(ap.ssid.to_string()),
                });
            } else if Self::is_adjacent_channel(current_ch, ap.channel.number, &current_band) {
                // Adjacent channel interference (only relevant for 2.4 GHz)
                adjacent_channel_count += 1;
                sources.push(InterferenceSource {
                    source_type: InterferenceType::AdjacentChannel,
                    severity: Self::calculate_adjacent_severity(
                        ap.rssi,
                        current_ch,
                        ap.channel.number,
                    ),
                    description: format!(
                        "WiFi network '{}' on adjacent channel {}",
                        ap.ssid, ap.channel.number
                    ),
                    affected_channels: vec![current_ch, ap.channel.number],
                    signal_strength: Some(ap.rssi),
                    ssid: Some(ap.ssid.to_string()),
                });
            }
        }

        // Check for non-WiFi interference based on noise level
        let non_wifi_detected = Self::detect_non_wifi_interference(noise_level, &current_band);

        if non_wifi_detected {
            let source_type = if current_band == WifiBand::Band2_4GHz {
                InterferenceType::Unknown2_4GHz
            } else {
                InterferenceType::Unknown5GHz
            };

            sources.push(InterferenceSource {
                source_type,
                severity: 50,
                description: "Elevated noise floor suggests non-WiFi interference".to_string(),
                affected_channels: vec![current_ch],
                signal_strength: noise_level,
                ssid: None,
            });
        }

        // Calculate overall interference score
        let interference_score = Self::calculate_overall_score(&sources);

        // Generate recommendations
        let recommendations = Self::generate_recommendations(
            &sources,
            co_channel_count,
            adjacent_channel_count,
            &current_band,
        );

        // Sort sources by severity
        sources.sort_by(|a, b| b.severity.cmp(&a.severity));

        Self {
            interference_score,
            sources,
            co_channel_count,
            adjacent_channel_count,
            non_wifi_detected,
            recommendations,
        }
    }

    /// Check if two channels are adjacent (overlapping in 2.4 GHz).
    fn is_adjacent_channel(current: u8, other: u8, band: &WifiBand) -> bool {
        if *band != WifiBand::Band2_4GHz {
            return false;
        }

        let diff = (current as i16 - other as i16).unsigned_abs();
        // 2.4 GHz channels 5 apart don't overlap
        diff > 0 && diff < 5
    }

    /// Calculate severity from RSSI.
    fn calculate_severity(rssi: i32) -> u8 {
        match rssi {
            r if r >= -50 => 100,
            r if r >= -60 => 80,
            r if r >= -70 => 60,
            r if r >= -80 => 40,
            r if r >= -90 => 20,
            _ => 10,
        }
    }

    /// Calculate adjacent channel severity (depends on channel separation).
    fn calculate_adjacent_severity(rssi: i32, current: u8, other: u8) -> u8 {
        let base_severity = Self::calculate_severity(rssi);
        let separation = (current as i16 - other as i16).unsigned_abs();

        // Reduce severity based on channel separation
        // 1 channel apart: full severity
        // 2 channels apart: 75% severity
        // 3 channels apart: 50% severity
        // 4 channels apart: 25% severity
        let multiplier = match separation {
            1 => 100,
            2 => 75,
            3 => 50,
            4 => 25,
            _ => 0,
        };

        (base_severity as u16 * multiplier / 100) as u8
    }

    /// Detect potential non-WiFi interference from noise level.
    fn detect_non_wifi_interference(noise: Option<i32>, band: &WifiBand) -> bool {
        if let Some(n) = noise {
            match band {
                WifiBand::Band2_4GHz => n > -90, // Normal floor is around -95 dBm
                WifiBand::Band5GHz => n > -92,   // 5 GHz is typically cleaner
                WifiBand::Band6GHz => n > -93,
            }
        } else {
            false
        }
    }

    /// Calculate overall interference score.
    fn calculate_overall_score(sources: &[InterferenceSource]) -> u8 {
        if sources.is_empty() {
            return 0;
        }

        // Take weighted average of severities
        let total: u32 = sources.iter().map(|s| s.severity as u32).sum();
        let count = sources.len() as u32;

        // Also consider the maximum severity
        let max_severity = sources.iter().map(|s| s.severity).max().unwrap_or(0);

        // Blend average and max (max has more weight)
        let avg = (total / count) as u8;
        ((avg as u16 + max_severity as u16 * 2) / 3) as u8
    }

    /// Generate recommendations.
    fn generate_recommendations(
        sources: &[InterferenceSource],
        co_channel: usize,
        adjacent: usize,
        band: &WifiBand,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if co_channel > 3 {
            recommendations.push(format!(
                "High co-channel congestion ({} networks). Consider switching channels.",
                co_channel
            ));
        }

        if *band == WifiBand::Band2_4GHz && adjacent > 2 {
            recommendations.push(
                "Multiple adjacent channel interference. Use only channels 1, 6, or 11."
                    .to_string(),
            );
        }

        if *band == WifiBand::Band2_4GHz && (co_channel > 5 || adjacent > 5) {
            recommendations.push(
                "2.4 GHz band is very congested. Consider switching to 5 GHz if available."
                    .to_string(),
            );
        }

        // Add specific recommendations for detected interference types
        let seen_types: std::collections::HashSet<_> =
            sources.iter().map(|s| s.source_type).collect();

        for itype in seen_types {
            if itype != InterferenceType::CoChannel && itype != InterferenceType::AdjacentChannel {
                recommendations.push(format!(
                    "Detected {}: {}",
                    itype.description(),
                    itype.mitigation()
                ));
            }
        }

        if recommendations.is_empty() && sources.is_empty() {
            recommendations.push("No significant interference detected.".to_string());
        }

        recommendations
    }

    /// Check if interference is significant.
    pub fn is_significant(&self) -> bool {
        self.interference_score > 50
    }

    /// Get the most severe interference source.
    pub fn most_severe(&self) -> Option<&InterferenceSource> {
        self.sources.first()
    }
}
