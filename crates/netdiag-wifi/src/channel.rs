//! Channel analysis and recommendations.

use netdiag_types::wifi::{AccessPoint, Channel, ChannelUtilization, InterferenceLevel, WifiBand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Channel analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAnalysis {
    /// Current channel.
    pub current_channel: Option<Channel>,

    /// Utilization of current channel.
    pub current_utilization: Option<ChannelUtilization>,

    /// All analyzed channels.
    pub channels: Vec<ChannelInfo>,

    /// Channel recommendations.
    pub recommendations: Vec<ChannelRecommendation>,

    /// Overall channel health score (0-100).
    pub health_score: u8,
}

/// Detailed information about a channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// The channel.
    pub channel: Channel,

    /// Utilization data.
    pub utilization: Option<ChannelUtilization>,

    /// Number of access points on this channel.
    pub ap_count: usize,

    /// Strongest AP signal on this channel.
    pub max_signal: Option<i32>,

    /// Average AP signal on this channel.
    pub avg_signal: Option<f64>,

    /// Is this a DFS channel?
    pub is_dfs: bool,

    /// Score for this channel (0-100, higher is better).
    pub score: u8,
}

/// Channel recommendation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelRecommendation {
    /// Recommended channel.
    pub channel: Channel,

    /// Reason for recommendation.
    pub reason: String,

    /// Expected improvement.
    pub expected_improvement: String,

    /// Priority (1 = highest).
    pub priority: u8,

    /// Score improvement estimate.
    pub score_improvement: i8,
}

impl ChannelAnalysis {
    /// Analyze channels from scan results.
    pub fn analyze(
        access_points: &[AccessPoint],
        current_channel: Option<Channel>,
        utilization_data: &[ChannelUtilization],
    ) -> Self {
        let mut channel_map: HashMap<u8, Vec<&AccessPoint>> = HashMap::new();

        // Group APs by channel
        for ap in access_points {
            channel_map.entry(ap.channel.number).or_default().push(ap);
        }

        // Build channel info
        let mut channels: Vec<ChannelInfo> = Vec::new();

        // Process 2.4 GHz channels (1, 6, 11 recommended)
        for ch in 1..=14 {
            let channel = Channel::from_number(ch, WifiBand::Band2_4GHz);

            let aps = channel_map.get(&ch).map(|v| v.as_slice()).unwrap_or(&[]);
            let utilization = utilization_data.iter().find(|u| u.channel.number == ch);

            let score = Self::calculate_channel_score(&channel, aps, utilization);

            channels.push(ChannelInfo {
                channel: channel.clone(),
                utilization: utilization.cloned(),
                ap_count: aps.len(),
                max_signal: aps.iter().map(|ap| ap.rssi).max(),
                avg_signal: if aps.is_empty() {
                    None
                } else {
                    Some(aps.iter().map(|ap| ap.rssi as f64).sum::<f64>() / aps.len() as f64)
                },
                is_dfs: false,
                score,
            });
        }

        // Process 5 GHz channels
        let ghz5_channels = [
            36, 40, 44, 48, // UNII-1
            52, 56, 60, 64, // UNII-2A (DFS)
            100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144, // UNII-2C (DFS)
            149, 153, 157, 161, 165, // UNII-3
        ];

        for ch in ghz5_channels {
            let channel = Channel::from_number(ch, WifiBand::Band5GHz);

            let aps = channel_map.get(&ch).map(|v| v.as_slice()).unwrap_or(&[]);
            let utilization = utilization_data.iter().find(|u| u.channel.number == ch);

            let is_dfs = (52..=144).contains(&ch);
            let score = Self::calculate_channel_score(&channel, aps, utilization);

            channels.push(ChannelInfo {
                channel: channel.clone(),
                utilization: utilization.cloned(),
                ap_count: aps.len(),
                max_signal: aps.iter().map(|ap| ap.rssi).max(),
                avg_signal: if aps.is_empty() {
                    None
                } else {
                    Some(aps.iter().map(|ap| ap.rssi as f64).sum::<f64>() / aps.len() as f64)
                },
                is_dfs,
                score,
            });
        }

        // Calculate current channel utilization
        let current_utilization = current_channel.as_ref().and_then(|ch| {
            utilization_data
                .iter()
                .find(|u| u.channel.number == ch.number)
                .cloned()
        });

        // Generate recommendations
        let recommendations = Self::generate_recommendations(&channels, current_channel.as_ref());

        // Calculate overall health score
        let health_score = Self::calculate_health_score(&current_utilization, &channels);

        Self {
            current_channel,
            current_utilization,
            channels,
            recommendations,
            health_score,
        }
    }

    /// Calculate score for a channel (higher is better).
    fn calculate_channel_score(
        channel: &Channel,
        aps: &[&AccessPoint],
        utilization: Option<&ChannelUtilization>,
    ) -> u8 {
        let mut score = 100u8;

        // Penalize for number of APs (crowded channels)
        let ap_penalty = (aps.len() * 10).min(50) as u8;
        score = score.saturating_sub(ap_penalty);

        // Penalize for strong signals (nearby competitors)
        if let Some(max_rssi) = aps.iter().map(|ap| ap.rssi).max() {
            if max_rssi >= -50 {
                score = score.saturating_sub(30);
            } else if max_rssi >= -60 {
                score = score.saturating_sub(20);
            } else if max_rssi >= -70 {
                score = score.saturating_sub(10);
            }
        }

        // Consider utilization data if available
        if let Some(util) = utilization {
            let util_penalty = (util.utilization_percent * 0.5) as u8;
            score = score.saturating_sub(util_penalty);

            match util.interference_level {
                InterferenceLevel::Severe => score = score.saturating_sub(30),
                InterferenceLevel::High => score = score.saturating_sub(20),
                InterferenceLevel::Medium => score = score.saturating_sub(10),
                InterferenceLevel::Low => {}
            }
        }

        // Bonus for non-overlapping 2.4 GHz channels
        if channel.band == WifiBand::Band2_4GHz && [1, 6, 11].contains(&channel.number) {
            score = score.saturating_add(10).min(100);
        }

        // Small penalty for DFS channels (potential radar interference)
        if (52..=144).contains(&channel.number) {
            score = score.saturating_sub(5);
        }

        score
    }

    /// Generate channel recommendations.
    fn generate_recommendations(
        channels: &[ChannelInfo],
        current: Option<&Channel>,
    ) -> Vec<ChannelRecommendation> {
        let mut recommendations = Vec::new();

        let current_score = current
            .and_then(|ch| {
                channels
                    .iter()
                    .find(|c| c.channel.number == ch.number)
                    .map(|c| c.score)
            })
            .unwrap_or(0);

        // Find best channels in each band
        let best_2_4 = channels
            .iter()
            .filter(|c| c.channel.band == WifiBand::Band2_4GHz)
            .max_by_key(|c| c.score);

        let best_5 = channels
            .iter()
            .filter(|c| c.channel.band == WifiBand::Band5GHz && !c.is_dfs)
            .max_by_key(|c| c.score);

        let best_5_dfs = channels
            .iter()
            .filter(|c| c.channel.band == WifiBand::Band5GHz && c.is_dfs)
            .max_by_key(|c| c.score);

        // Recommend 5 GHz if significantly better
        if let Some(best) = best_5 {
            if best.score > current_score + 15 {
                recommendations.push(ChannelRecommendation {
                    channel: best.channel.clone(),
                    reason: format!(
                        "Channel {} has {} APs (score: {})",
                        best.channel.number, best.ap_count, best.score
                    ),
                    expected_improvement: "Better speed and less interference on 5 GHz".to_string(),
                    priority: 1,
                    score_improvement: (best.score as i8 - current_score as i8),
                });
            }
        }

        // Recommend 2.4 GHz improvement
        if let Some(best) = best_2_4 {
            if let Some(current_ch) = current {
                if current_ch.band == WifiBand::Band2_4GHz
                    && best.channel.number != current_ch.number
                    && best.score > current_score + 10
                {
                    recommendations.push(ChannelRecommendation {
                        channel: best.channel.clone(),
                        reason: format!(
                            "Channel {} is less congested ({} APs vs current)",
                            best.channel.number, best.ap_count
                        ),
                        expected_improvement: "Reduced interference on 2.4 GHz".to_string(),
                        priority: 2,
                        score_improvement: (best.score as i8 - current_score as i8),
                    });
                }
            }
        }

        // Mention DFS channels if they're significantly better
        if let Some(best) = best_5_dfs {
            if best.score > current_score + 25 {
                recommendations.push(ChannelRecommendation {
                    channel: best.channel.clone(),
                    reason: format!(
                        "DFS channel {} is very clean (score: {})",
                        best.channel.number, best.score
                    ),
                    expected_improvement:
                        "Much less congestion (but requires DFS support and radar checks)"
                            .to_string(),
                    priority: 3,
                    score_improvement: (best.score as i8 - current_score as i8),
                });
            }
        }

        // Sort by priority
        recommendations.sort_by_key(|r| r.priority);

        recommendations
    }

    /// Calculate overall channel health score.
    fn calculate_health_score(
        current_utilization: &Option<ChannelUtilization>,
        channels: &[ChannelInfo],
    ) -> u8 {
        if let Some(util) = current_utilization {
            let base_score = 100 - (util.utilization_percent as u8).min(100);

            // Adjust for interference
            let adjusted = match util.interference_level {
                InterferenceLevel::Severe => base_score.saturating_sub(40),
                InterferenceLevel::High => base_score.saturating_sub(30),
                InterferenceLevel::Medium => base_score.saturating_sub(15),
                InterferenceLevel::Low => base_score,
            };

            adjusted
        } else {
            // Estimate from channel data
            let current_info = channels.iter().find(|c| c.ap_count > 0);
            current_info.map(|c| c.score).unwrap_or(50)
        }
    }

    /// Get the best channel for a given band.
    pub fn best_channel(&self, band: WifiBand) -> Option<&ChannelInfo> {
        self.channels
            .iter()
            .filter(|c| c.channel.band == band)
            .max_by_key(|c| c.score)
    }

    /// Get non-overlapping 2.4 GHz channels.
    pub fn non_overlapping_2_4ghz(&self) -> Vec<&ChannelInfo> {
        self.channels
            .iter()
            .filter(|c| {
                c.channel.band == WifiBand::Band2_4GHz && [1, 6, 11].contains(&c.channel.number)
            })
            .collect()
    }
}
