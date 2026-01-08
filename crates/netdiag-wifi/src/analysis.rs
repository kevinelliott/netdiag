//! Main WiFi analysis functionality.

use crate::{
    channel::ChannelAnalysis,
    error::{WifiError, WifiResult},
    interference::InterferenceAnalysis,
    quality::QualityMetrics,
    security::SecurityAnalysis,
};
use chrono::{DateTime, Utc};
use netdiag_platform::WifiProvider;
use netdiag_types::wifi::{AccessPoint, WifiConnection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Complete WiFi analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiAnalysis {
    /// Analysis timestamp.
    pub timestamp: DateTime<Utc>,

    /// Interface analyzed.
    pub interface: String,

    /// Current connection (if connected).
    pub connection: Option<WifiConnection>,

    /// Signal quality metrics.
    pub quality: Option<QualityMetrics>,

    /// Channel analysis.
    pub channel_analysis: ChannelAnalysis,

    /// Interference analysis.
    pub interference: InterferenceAnalysis,

    /// Security analysis (if connected).
    pub security: Option<SecurityAnalysis>,

    /// Nearby access points.
    pub nearby_networks: Vec<AccessPoint>,

    /// Overall WiFi health score (0-100).
    pub health_score: u8,

    /// Summary recommendations.
    pub recommendations: Vec<String>,
}

/// WiFi environment summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiEnvironment {
    /// Number of networks in 2.4 GHz band.
    pub networks_2_4ghz: usize,

    /// Number of networks in 5 GHz band.
    pub networks_5ghz: usize,

    /// Number of networks in 6 GHz band.
    pub networks_6ghz: usize,

    /// Most congested 2.4 GHz channel.
    pub most_congested_2_4: Option<u8>,

    /// Most congested 5 GHz channel.
    pub most_congested_5: Option<u8>,

    /// Cleanest 2.4 GHz channel.
    pub cleanest_2_4: Option<u8>,

    /// Cleanest 5 GHz channel.
    pub cleanest_5: Option<u8>,

    /// Average signal strength of nearby networks.
    pub avg_signal_strength: Option<f64>,

    /// Environment congestion level (0-100).
    pub congestion_level: u8,
}

impl WifiEnvironment {
    /// Analyze environment from access points.
    pub fn from_aps(aps: &[AccessPoint]) -> Self {
        use netdiag_types::wifi::WifiBand;
        use std::collections::HashMap;

        let mut channel_counts: HashMap<u8, usize> = HashMap::new();
        let mut networks_2_4 = 0;
        let mut networks_5 = 0;
        let mut networks_6 = 0;
        let mut signal_sum = 0i64;

        for ap in aps {
            *channel_counts.entry(ap.channel.number).or_insert(0) += 1;
            signal_sum += ap.rssi as i64;

            match ap.channel.band {
                WifiBand::Band2_4GHz => networks_2_4 += 1,
                WifiBand::Band5GHz => networks_5 += 1,
                WifiBand::Band6GHz => networks_6 += 1,
            }
        }

        let avg_signal = if !aps.is_empty() {
            Some(signal_sum as f64 / aps.len() as f64)
        } else {
            None
        };

        // Find most/least congested channels
        let channels_2_4: Vec<_> = channel_counts
            .iter()
            .filter(|(ch, _)| **ch <= 14)
            .collect();
        let channels_5: Vec<_> = channel_counts
            .iter()
            .filter(|(ch, _)| **ch > 14)
            .collect();

        let most_congested_2_4 = channels_2_4
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ch, _)| **ch);

        let most_congested_5 = channels_5
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ch, _)| **ch);

        // For cleanest, prefer non-overlapping 2.4 GHz channels
        let cleanest_2_4 = [1u8, 6, 11]
            .iter()
            .min_by_key(|ch| channel_counts.get(ch).unwrap_or(&0))
            .copied();

        let cleanest_5 = channels_5
            .iter()
            .min_by_key(|(_, count)| *count)
            .map(|(ch, _)| **ch)
            .or_else(|| Some(36)); // Default to channel 36

        // Calculate congestion level
        let congestion = ((networks_2_4 + networks_5) * 5).min(100) as u8;

        Self {
            networks_2_4ghz: networks_2_4,
            networks_5ghz: networks_5,
            networks_6ghz: networks_6,
            most_congested_2_4,
            most_congested_5,
            cleanest_2_4,
            cleanest_5,
            avg_signal_strength: avg_signal,
            congestion_level: congestion,
        }
    }
}

/// WiFi analyzer for comprehensive analysis.
pub struct WifiAnalyzer {
    provider: Arc<dyn WifiProvider>,
}

impl WifiAnalyzer {
    /// Create a new analyzer.
    pub fn new(provider: Arc<dyn WifiProvider>) -> Self {
        Self { provider }
    }

    /// Run full WiFi analysis.
    pub async fn analyze(&self, interface: &str) -> WifiResult<WifiAnalysis> {
        info!("Starting WiFi analysis for interface: {}", interface);

        if !self.provider.is_available() {
            return Err(WifiError::NotAvailable);
        }

        // Get current connection
        debug!("Getting current connection...");
        let connection = self
            .provider
            .get_current_connection(interface)
            .await
            .map_err(WifiError::Platform)?;

        // Scan for nearby networks
        debug!("Scanning for nearby networks...");
        let nearby_networks = self
            .provider
            .scan_access_points(interface)
            .await
            .map_err(WifiError::Platform)?;

        // Get current channel
        let current_channel = connection.as_ref().map(|c| c.access_point.channel.clone());

        // Get signal quality
        debug!("Getting signal quality...");
        let quality = if let Some(conn) = &connection {
            Some(QualityMetrics::from_connection(conn))
        } else {
            None
        };

        // Get channel utilization data
        debug!("Analyzing channels...");
        let utilization_data = self
            .provider
            .analyze_channels(interface)
            .await
            .unwrap_or_default();

        // Perform channel analysis
        let channel_analysis =
            ChannelAnalysis::analyze(&nearby_networks, current_channel.clone(), &utilization_data);

        // Analyze interference
        debug!("Analyzing interference...");
        let noise = self
            .provider
            .get_noise_level(interface)
            .await
            .ok()
            .flatten();
        let interference =
            InterferenceAnalysis::analyze(&nearby_networks, current_channel.as_ref(), noise);

        // Security analysis
        debug!("Analyzing security...");
        let security = connection.as_ref().map(SecurityAnalysis::analyze_connection);

        // Calculate overall health score
        let health_score = Self::calculate_health_score(&quality, &channel_analysis, &interference, &security);

        // Generate summary recommendations
        let recommendations = Self::generate_recommendations(
            &quality,
            &channel_analysis,
            &interference,
            &security,
        );

        info!("WiFi analysis complete. Health score: {}", health_score);

        Ok(WifiAnalysis {
            timestamp: Utc::now(),
            interface: interface.to_string(),
            connection,
            quality,
            channel_analysis,
            interference,
            security,
            nearby_networks,
            health_score,
            recommendations,
        })
    }

    /// Get environment summary without full analysis.
    pub async fn get_environment(&self, interface: &str) -> WifiResult<WifiEnvironment> {
        let aps = self
            .provider
            .scan_access_points(interface)
            .await
            .map_err(WifiError::Platform)?;

        Ok(WifiEnvironment::from_aps(&aps))
    }

    /// Quick check if connected and basic health.
    pub async fn quick_status(&self, interface: &str) -> WifiResult<(bool, u8)> {
        let connection = self
            .provider
            .get_current_connection(interface)
            .await
            .map_err(WifiError::Platform)?;

        if let Some(conn) = connection {
            let quality = QualityMetrics::from_connection(&conn);
            Ok((true, quality.overall_score))
        } else {
            Ok((false, 0))
        }
    }

    /// Calculate overall health score.
    fn calculate_health_score(
        quality: &Option<QualityMetrics>,
        channel: &ChannelAnalysis,
        interference: &InterferenceAnalysis,
        security: &Option<SecurityAnalysis>,
    ) -> u8 {
        let mut scores = Vec::new();
        let mut weights = Vec::new();

        // Signal quality (weight: 40%)
        if let Some(q) = quality {
            scores.push(q.overall_score as f64);
            weights.push(0.4);
        }

        // Channel health (weight: 25%)
        scores.push(channel.health_score as f64);
        weights.push(0.25);

        // Interference (weight: 20%, inverted)
        scores.push((100 - interference.interference_score) as f64);
        weights.push(0.2);

        // Security (weight: 15%)
        if let Some(s) = security {
            scores.push(s.score as f64);
            weights.push(0.15);
        }

        // Calculate weighted average
        if scores.is_empty() {
            return 50;
        }

        let total_weight: f64 = weights.iter().sum();
        let weighted_sum: f64 = scores
            .iter()
            .zip(weights.iter())
            .map(|(s, w)| s * w)
            .sum();

        (weighted_sum / total_weight) as u8
    }

    /// Generate summary recommendations.
    fn generate_recommendations(
        quality: &Option<QualityMetrics>,
        channel: &ChannelAnalysis,
        interference: &InterferenceAnalysis,
        security: &Option<SecurityAnalysis>,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Quality recommendations
        if let Some(q) = quality {
            recommendations.extend(q.recommendations.iter().take(2).cloned());
        }

        // Channel recommendations (top priority only)
        if let Some(rec) = channel.recommendations.first() {
            recommendations.push(rec.reason.clone());
        }

        // Interference recommendations
        if interference.is_significant() {
            recommendations.extend(interference.recommendations.iter().take(2).cloned());
        }

        // Security recommendations (critical only)
        if let Some(s) = security {
            for issue in s.critical_issues() {
                recommendations.push(issue.remediation.clone());
            }
        }

        // Limit to top 5 recommendations
        recommendations.truncate(5);

        recommendations
    }
}
