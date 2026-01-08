//! Site survey functionality.

use crate::error::{WifiError, WifiResult};
use crate::quality::{QualityMetrics, SignalQuality};
use chrono::{DateTime, Utc};
use netdiag_platform::WifiProvider;
use netdiag_types::wifi::Channel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// Site survey tool for comprehensive WiFi mapping.
pub struct SiteSurvey {
    provider: Arc<dyn WifiProvider>,
}

impl SiteSurvey {
    /// Create a new site survey tool.
    pub fn new(provider: Arc<dyn WifiProvider>) -> Self {
        Self { provider }
    }

    /// Record a single survey point.
    pub async fn record_point(
        &self,
        interface: &str,
        location: &str,
    ) -> WifiResult<SurveyPoint> {
        debug!("Recording survey point at: {}", location);

        let connection = self
            .provider
            .get_current_connection(interface)
            .await
            .map_err(WifiError::Platform)?
            .ok_or(WifiError::NotConnected)?;

        let noise = self
            .provider
            .get_noise_level(interface)
            .await
            .ok()
            .flatten();

        let quality = QualityMetrics::from_connection(&connection);

        let nearby = self
            .provider
            .scan_access_points(interface)
            .await
            .map_err(WifiError::Platform)?;

        Ok(SurveyPoint {
            timestamp: Utc::now(),
            location: location.to_string(),
            ssid: connection.access_point.ssid.to_string(),
            bssid: connection.access_point.bssid.to_string(),
            channel: connection.access_point.channel,
            rssi: connection.access_point.rssi,
            noise,
            quality,
            tx_rate: connection.tx_rate.unwrap_or(0.0) as f64,
            nearby_ap_count: nearby.len(),
            co_channel_aps: nearby
                .iter()
                .filter(|ap| ap.channel.number == connection.access_point.channel.number)
                .count(),
        })
    }

    /// Start a continuous survey (returns points as they're collected).
    pub async fn continuous_survey(
        &self,
        interface: &str,
        interval_secs: u64,
        count: usize,
    ) -> WifiResult<Vec<SurveyPoint>> {
        info!(
            "Starting continuous survey: {} points at {}s intervals",
            count, interval_secs
        );

        let mut points = Vec::with_capacity(count);

        for i in 0..count {
            let location = format!("Point {}", i + 1);
            match self.record_point(interface, &location).await {
                Ok(point) => {
                    points.push(point);
                }
                Err(e) => {
                    debug!("Failed to record point {}: {}", i + 1, e);
                }
            }

            if i < count - 1 {
                tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
            }
        }

        Ok(points)
    }

    /// Analyze survey results.
    pub fn analyze_survey(points: &[SurveyPoint]) -> SurveyResult {
        if points.is_empty() {
            return SurveyResult::default();
        }

        // Calculate statistics
        let rssi_values: Vec<i32> = points.iter().map(|p| p.rssi).collect();
        let rssi_avg = rssi_values.iter().sum::<i32>() as f64 / rssi_values.len() as f64;
        let rssi_min = *rssi_values.iter().min().unwrap();
        let rssi_max = *rssi_values.iter().max().unwrap();

        // Calculate variance
        let rssi_variance = rssi_values
            .iter()
            .map(|&r| (r as f64 - rssi_avg).powi(2))
            .sum::<f64>()
            / rssi_values.len() as f64;
        let rssi_std_dev = rssi_variance.sqrt();

        // Quality distribution
        let mut quality_dist: HashMap<SignalQuality, usize> = HashMap::new();
        for point in points {
            *quality_dist.entry(point.quality.quality).or_insert(0) += 1;
        }

        // Find problem areas
        let problem_areas: Vec<String> = points
            .iter()
            .filter(|p| p.quality.quality <= SignalQuality::Weak)
            .map(|p| p.location.clone())
            .collect();

        // Coverage score (percentage of points with good or better signal)
        let good_coverage = points
            .iter()
            .filter(|p| p.quality.quality >= SignalQuality::Fair)
            .count();
        let coverage_score = (good_coverage as f64 / points.len() as f64 * 100.0) as u8;

        // Roaming analysis (detect BSSID changes)
        let mut roaming_events = 0;
        let mut last_bssid: Option<&str> = None;
        for point in points {
            if let Some(prev) = last_bssid {
                if prev != point.bssid {
                    roaming_events += 1;
                }
            }
            last_bssid = Some(&point.bssid);
        }

        // Generate recommendations
        let recommendations = Self::generate_survey_recommendations(
            coverage_score,
            &problem_areas,
            rssi_std_dev,
            roaming_events,
        );

        SurveyResult {
            timestamp: Utc::now(),
            total_points: points.len(),
            rssi_average: rssi_avg,
            rssi_min,
            rssi_max,
            rssi_std_deviation: rssi_std_dev,
            quality_distribution: quality_dist,
            problem_areas,
            coverage_score,
            roaming_events,
            recommendations,
            points: points.to_vec(),
        }
    }

    /// Generate recommendations based on survey data.
    fn generate_survey_recommendations(
        coverage: u8,
        problem_areas: &[String],
        variability: f64,
        roaming: usize,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if coverage < 70 {
            recommendations.push(
                "Coverage is below optimal. Consider adding access points or WiFi extenders."
                    .to_string(),
            );
        }

        if !problem_areas.is_empty() {
            recommendations.push(format!(
                "Weak signal detected in {} location(s): {}. Consider repositioning or adding coverage.",
                problem_areas.len(),
                problem_areas.join(", ")
            ));
        }

        if variability > 10.0 {
            recommendations.push(
                "High signal variability detected. Check for interference sources or physical obstructions."
                    .to_string(),
            );
        }

        if roaming > 2 {
            recommendations.push(format!(
                "Detected {} roaming events. Verify AP coverage overlap and roaming thresholds.",
                roaming
            ));
        }

        if coverage >= 90 && problem_areas.is_empty() {
            recommendations.push("Excellent coverage throughout surveyed area.".to_string());
        }

        recommendations
    }
}

/// A single point in a site survey.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyPoint {
    /// Timestamp of measurement.
    pub timestamp: DateTime<Utc>,

    /// Location description.
    pub location: String,

    /// Connected SSID.
    pub ssid: String,

    /// Connected BSSID (AP MAC).
    pub bssid: String,

    /// Channel.
    pub channel: Channel,

    /// Signal strength (RSSI).
    pub rssi: i32,

    /// Noise level.
    pub noise: Option<i32>,

    /// Quality metrics.
    pub quality: QualityMetrics,

    /// Transmit rate in Mbps.
    pub tx_rate: f64,

    /// Number of nearby access points.
    pub nearby_ap_count: usize,

    /// Number of APs on same channel.
    pub co_channel_aps: usize,
}

/// Site survey results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyResult {
    /// Analysis timestamp.
    pub timestamp: DateTime<Utc>,

    /// Total survey points.
    pub total_points: usize,

    /// Average RSSI.
    pub rssi_average: f64,

    /// Minimum RSSI (weakest signal).
    pub rssi_min: i32,

    /// Maximum RSSI (strongest signal).
    pub rssi_max: i32,

    /// RSSI standard deviation.
    pub rssi_std_deviation: f64,

    /// Distribution of signal quality.
    pub quality_distribution: HashMap<SignalQuality, usize>,

    /// Locations with weak signal.
    pub problem_areas: Vec<String>,

    /// Overall coverage score (0-100).
    pub coverage_score: u8,

    /// Number of roaming events detected.
    pub roaming_events: usize,

    /// Recommendations.
    pub recommendations: Vec<String>,

    /// All survey points.
    pub points: Vec<SurveyPoint>,
}

impl Default for SurveyResult {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            total_points: 0,
            rssi_average: 0.0,
            rssi_min: 0,
            rssi_max: 0,
            rssi_std_deviation: 0.0,
            quality_distribution: HashMap::new(),
            problem_areas: Vec::new(),
            coverage_score: 0,
            roaming_events: 0,
            recommendations: vec!["No survey data collected.".to_string()],
            points: Vec::new(),
        }
    }
}
