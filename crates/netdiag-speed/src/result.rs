//! Speed test result types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Speed test result containing download, upload, and latency measurements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    /// Test timestamp.
    pub timestamp: DateTime<Utc>,

    /// Server used for the test.
    pub server: SpeedTestServer,

    /// Download measurement.
    pub download: Option<BandwidthMeasurement>,

    /// Upload measurement.
    pub upload: Option<BandwidthMeasurement>,

    /// Latency to the server.
    pub latency: Option<Duration>,

    /// Jitter (latency variance).
    pub jitter: Option<Duration>,

    /// Provider name.
    pub provider: String,

    /// Test duration.
    pub test_duration: Duration,

    /// Buffer bloat analysis results.
    pub buffer_bloat: Option<BufferBloatAnalysis>,

    /// Speed consistency analysis.
    pub consistency: Option<SpeedConsistency>,
}

/// Buffer bloat analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferBloatAnalysis {
    /// Baseline latency (unloaded).
    pub baseline_latency: Duration,
    /// Latency during download.
    pub download_latency: Option<Duration>,
    /// Latency during upload.
    pub upload_latency: Option<Duration>,
    /// Maximum latency observed under load.
    pub peak_latency: Duration,
    /// Latency increase from baseline.
    pub latency_increase: Duration,
    /// Latency increase percentage.
    pub latency_increase_percent: f64,
    /// Buffer bloat grade.
    pub grade: BufferBloatGrade,
}

impl BufferBloatAnalysis {
    /// Create a new buffer bloat analysis.
    pub fn new(
        baseline_latency: Duration,
        loaded_latency: Duration,
    ) -> Self {
        let latency_increase = loaded_latency.saturating_sub(baseline_latency);
        let latency_increase_percent = if baseline_latency.as_nanos() > 0 {
            (latency_increase.as_secs_f64() / baseline_latency.as_secs_f64()) * 100.0
        } else {
            0.0
        };

        let grade = BufferBloatGrade::from_increase(latency_increase);

        Self {
            baseline_latency,
            download_latency: None,
            upload_latency: None,
            peak_latency: loaded_latency,
            latency_increase,
            latency_increase_percent,
            grade,
        }
    }

    /// Get human-readable description.
    pub fn description(&self) -> &'static str {
        match self.grade {
            BufferBloatGrade::A => "Excellent - minimal buffer bloat",
            BufferBloatGrade::B => "Good - minor buffer bloat",
            BufferBloatGrade::C => "Fair - moderate buffer bloat, may affect real-time apps",
            BufferBloatGrade::D => "Poor - significant buffer bloat",
            BufferBloatGrade::F => "Very Poor - severe buffer bloat affecting performance",
        }
    }
}

/// Buffer bloat grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BufferBloatGrade {
    /// Excellent (< 5ms increase or < 20% increase)
    A,
    /// Good (< 30ms increase or < 50% increase)
    B,
    /// Fair (< 60ms increase or < 100% increase)
    C,
    /// Poor (< 200ms increase or < 300% increase)
    D,
    /// Very poor (> 200ms increase)
    F,
}

impl BufferBloatGrade {
    /// Determine grade from latency increase.
    pub fn from_increase(increase: Duration) -> Self {
        let ms = increase.as_millis() as u64;
        match ms {
            0..=5 => Self::A,
            6..=30 => Self::B,
            31..=60 => Self::C,
            61..=200 => Self::D,
            _ => Self::F,
        }
    }
}

impl std::fmt::Display for BufferBloatGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
            Self::F => write!(f, "F"),
        }
    }
}

/// Speed consistency analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedConsistency {
    /// Minimum speed observed (Mbps).
    pub min_speed_mbps: f64,
    /// Maximum speed observed (Mbps).
    pub max_speed_mbps: f64,
    /// Average speed (Mbps).
    pub avg_speed_mbps: f64,
    /// Standard deviation (Mbps).
    pub stddev_mbps: f64,
    /// Coefficient of variation (0-1).
    pub coefficient_of_variation: f64,
    /// Consistency rating.
    pub rating: ConsistencyRating,
}

impl SpeedConsistency {
    /// Calculate consistency from samples.
    pub fn from_samples(samples: &[BandwidthSample]) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }

        let speeds: Vec<f64> = samples.iter().map(|s| s.mbps()).collect();
        let min_speed_mbps = speeds.iter().copied().fold(f64::INFINITY, f64::min);
        let max_speed_mbps = speeds.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let avg_speed_mbps = speeds.iter().sum::<f64>() / speeds.len() as f64;

        let variance = speeds
            .iter()
            .map(|s| (s - avg_speed_mbps).powi(2))
            .sum::<f64>()
            / speeds.len() as f64;
        let stddev_mbps = variance.sqrt();

        let coefficient_of_variation = if avg_speed_mbps > 0.0 {
            stddev_mbps / avg_speed_mbps
        } else {
            0.0
        };

        let rating = ConsistencyRating::from_cv(coefficient_of_variation);

        Some(Self {
            min_speed_mbps,
            max_speed_mbps,
            avg_speed_mbps,
            stddev_mbps,
            coefficient_of_variation,
            rating,
        })
    }
}

/// Consistency rating based on coefficient of variation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyRating {
    /// Excellent consistency (CV < 0.1)
    Excellent,
    /// Good consistency (CV < 0.2)
    Good,
    /// Fair consistency (CV < 0.3)
    Fair,
    /// Poor consistency (CV < 0.5)
    Poor,
    /// Very poor consistency (CV >= 0.5)
    VeryPoor,
}

impl ConsistencyRating {
    /// Determine rating from coefficient of variation.
    pub fn from_cv(cv: f64) -> Self {
        if cv < 0.1 {
            Self::Excellent
        } else if cv < 0.2 {
            Self::Good
        } else if cv < 0.3 {
            Self::Fair
        } else if cv < 0.5 {
            Self::Poor
        } else {
            Self::VeryPoor
        }
    }
}

impl std::fmt::Display for ConsistencyRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Excellent => write!(f, "Excellent"),
            Self::Good => write!(f, "Good"),
            Self::Fair => write!(f, "Fair"),
            Self::Poor => write!(f, "Poor"),
            Self::VeryPoor => write!(f, "Very Poor"),
        }
    }
}

impl SpeedTestResult {
    /// Create a new speed test result.
    pub fn new(server: SpeedTestServer, provider: &str) -> Self {
        Self {
            timestamp: Utc::now(),
            server,
            download: None,
            upload: None,
            latency: None,
            jitter: None,
            provider: provider.to_string(),
            test_duration: Duration::ZERO,
            buffer_bloat: None,
            consistency: None,
        }
    }

    /// Get download speed in Mbps.
    pub fn download_mbps(&self) -> Option<f64> {
        self.download.as_ref().map(|d| d.mbps())
    }

    /// Get upload speed in Mbps.
    pub fn upload_mbps(&self) -> Option<f64> {
        self.upload.as_ref().map(|u| u.mbps())
    }

    /// Calculate and set speed consistency from download samples.
    pub fn calculate_consistency(&mut self) {
        if let Some(ref download) = self.download {
            self.consistency = SpeedConsistency::from_samples(&download.samples);
        }
    }
}

/// Bandwidth measurement for download or upload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMeasurement {
    /// Bytes transferred.
    pub bytes: u64,

    /// Duration of the transfer.
    pub duration: Duration,

    /// Number of parallel connections used.
    pub connections: usize,

    /// Individual samples taken during the test.
    pub samples: Vec<BandwidthSample>,
}

impl BandwidthMeasurement {
    /// Create a new bandwidth measurement.
    pub fn new(bytes: u64, duration: Duration, connections: usize) -> Self {
        Self {
            bytes,
            duration,
            connections,
            samples: Vec::new(),
        }
    }

    /// Get speed in bits per second.
    pub fn bps(&self) -> f64 {
        if self.duration.as_secs_f64() > 0.0 {
            (self.bytes as f64 * 8.0) / self.duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Get speed in kilobits per second.
    pub fn kbps(&self) -> f64 {
        self.bps() / 1000.0
    }

    /// Get speed in megabits per second.
    pub fn mbps(&self) -> f64 {
        self.bps() / 1_000_000.0
    }

    /// Get speed in gigabits per second.
    pub fn gbps(&self) -> f64 {
        self.bps() / 1_000_000_000.0
    }

    /// Get speed in bytes per second.
    pub fn bytes_per_second(&self) -> f64 {
        if self.duration.as_secs_f64() > 0.0 {
            self.bytes as f64 / self.duration.as_secs_f64()
        } else {
            0.0
        }
    }

    /// Format speed as human-readable string.
    pub fn format_speed(&self) -> String {
        let mbps = self.mbps();
        if mbps >= 1000.0 {
            format!("{:.2} Gbps", self.gbps())
        } else if mbps >= 1.0 {
            format!("{:.2} Mbps", mbps)
        } else {
            format!("{:.2} Kbps", self.kbps())
        }
    }
}

/// A single bandwidth sample during a speed test.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthSample {
    /// Sample timestamp relative to test start.
    pub elapsed: Duration,

    /// Bytes transferred in this sample.
    pub bytes: u64,

    /// Duration of this sample.
    pub duration: Duration,
}

impl BandwidthSample {
    /// Get speed in Mbps for this sample.
    pub fn mbps(&self) -> f64 {
        if self.duration.as_secs_f64() > 0.0 {
            (self.bytes as f64 * 8.0) / self.duration.as_secs_f64() / 1_000_000.0
        } else {
            0.0
        }
    }
}

/// Speed test server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestServer {
    /// Server name.
    pub name: String,

    /// Server URL or address.
    pub url: String,

    /// Server location (city, region).
    pub location: Option<String>,

    /// Server country.
    pub country: Option<String>,

    /// Server sponsor/provider.
    pub sponsor: Option<String>,

    /// Distance to server (km).
    pub distance_km: Option<f64>,

    /// Server latency (measured).
    pub latency: Option<Duration>,
}

impl SpeedTestServer {
    /// Create a new speed test server.
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            location: None,
            country: None,
            sponsor: None,
            distance_km: None,
            latency: None,
        }
    }

    /// Create with location info.
    pub fn with_location(mut self, location: &str, country: &str) -> Self {
        self.location = Some(location.to_string());
        self.country = Some(country.to_string());
        self
    }
}

impl Default for SpeedTestServer {
    fn default() -> Self {
        Self::new("Unknown", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_bloat_grade_from_increase() {
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(0)), BufferBloatGrade::A);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(5)), BufferBloatGrade::A);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(6)), BufferBloatGrade::B);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(30)), BufferBloatGrade::B);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(31)), BufferBloatGrade::C);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(60)), BufferBloatGrade::C);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(61)), BufferBloatGrade::D);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(200)), BufferBloatGrade::D);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(201)), BufferBloatGrade::F);
        assert_eq!(BufferBloatGrade::from_increase(Duration::from_millis(500)), BufferBloatGrade::F);
    }

    #[test]
    fn test_buffer_bloat_grade_display() {
        assert_eq!(format!("{}", BufferBloatGrade::A), "A");
        assert_eq!(format!("{}", BufferBloatGrade::B), "B");
        assert_eq!(format!("{}", BufferBloatGrade::C), "C");
        assert_eq!(format!("{}", BufferBloatGrade::D), "D");
        assert_eq!(format!("{}", BufferBloatGrade::F), "F");
    }

    #[test]
    fn test_buffer_bloat_analysis_new() {
        let baseline = Duration::from_millis(20);
        let loaded = Duration::from_millis(50);
        let analysis = BufferBloatAnalysis::new(baseline, loaded);

        assert_eq!(analysis.baseline_latency, baseline);
        assert_eq!(analysis.peak_latency, loaded);
        assert_eq!(analysis.latency_increase, Duration::from_millis(30));
        assert!((analysis.latency_increase_percent - 150.0).abs() < 0.1);
        assert_eq!(analysis.grade, BufferBloatGrade::B);
    }

    #[test]
    fn test_buffer_bloat_analysis_description() {
        let analysis = BufferBloatAnalysis::new(
            Duration::from_millis(20),
            Duration::from_millis(22),
        );
        assert!(analysis.description().contains("Excellent"));

        let analysis = BufferBloatAnalysis::new(
            Duration::from_millis(20),
            Duration::from_millis(300),
        );
        assert!(analysis.description().contains("Very Poor"));
    }

    #[test]
    fn test_consistency_rating_from_cv() {
        assert_eq!(ConsistencyRating::from_cv(0.05), ConsistencyRating::Excellent);
        assert_eq!(ConsistencyRating::from_cv(0.1), ConsistencyRating::Good);
        assert_eq!(ConsistencyRating::from_cv(0.15), ConsistencyRating::Good);
        assert_eq!(ConsistencyRating::from_cv(0.2), ConsistencyRating::Fair);
        assert_eq!(ConsistencyRating::from_cv(0.25), ConsistencyRating::Fair);
        assert_eq!(ConsistencyRating::from_cv(0.3), ConsistencyRating::Poor);
        assert_eq!(ConsistencyRating::from_cv(0.4), ConsistencyRating::Poor);
        assert_eq!(ConsistencyRating::from_cv(0.5), ConsistencyRating::VeryPoor);
        assert_eq!(ConsistencyRating::from_cv(0.8), ConsistencyRating::VeryPoor);
    }

    #[test]
    fn test_consistency_rating_display() {
        assert_eq!(format!("{}", ConsistencyRating::Excellent), "Excellent");
        assert_eq!(format!("{}", ConsistencyRating::Good), "Good");
        assert_eq!(format!("{}", ConsistencyRating::Fair), "Fair");
        assert_eq!(format!("{}", ConsistencyRating::Poor), "Poor");
        assert_eq!(format!("{}", ConsistencyRating::VeryPoor), "Very Poor");
    }

    #[test]
    fn test_speed_consistency_from_samples_empty() {
        assert!(SpeedConsistency::from_samples(&[]).is_none());
    }

    #[test]
    fn test_speed_consistency_from_samples() {
        // Create consistent samples (100 Mbps each)
        let samples: Vec<BandwidthSample> = (0..10)
            .map(|i| BandwidthSample {
                elapsed: Duration::from_secs(i),
                bytes: 12_500_000, // 100 Mbps for 1 second
                duration: Duration::from_secs(1),
            })
            .collect();

        let consistency = SpeedConsistency::from_samples(&samples).unwrap();

        assert!((consistency.avg_speed_mbps - 100.0).abs() < 1.0);
        assert!(consistency.stddev_mbps < 1.0);
        assert!(consistency.coefficient_of_variation < 0.1);
        assert_eq!(consistency.rating, ConsistencyRating::Excellent);
    }

    #[test]
    fn test_speed_consistency_variable_samples() {
        // Create variable samples (50-150 Mbps)
        let samples: Vec<BandwidthSample> = vec![
            BandwidthSample {
                elapsed: Duration::from_secs(0),
                bytes: 6_250_000, // 50 Mbps
                duration: Duration::from_secs(1),
            },
            BandwidthSample {
                elapsed: Duration::from_secs(1),
                bytes: 18_750_000, // 150 Mbps
                duration: Duration::from_secs(1),
            },
        ];

        let consistency = SpeedConsistency::from_samples(&samples).unwrap();

        assert!((consistency.avg_speed_mbps - 100.0).abs() < 1.0);
        assert!(consistency.min_speed_mbps < consistency.max_speed_mbps);
        assert!(consistency.coefficient_of_variation > 0.3);
        assert!(matches!(
            consistency.rating,
            ConsistencyRating::Poor | ConsistencyRating::VeryPoor
        ));
    }

    #[test]
    fn test_bandwidth_measurement_bps() {
        let measurement = BandwidthMeasurement::new(
            125_000_000, // 125 MB
            Duration::from_secs(10),
            4,
        );

        // 125 MB in 10s = 12.5 MB/s = 100 Mbps
        assert!((measurement.bps() - 100_000_000.0).abs() < 1000.0);
        assert!((measurement.kbps() - 100_000.0).abs() < 1.0);
        assert!((measurement.mbps() - 100.0).abs() < 0.01);
        assert!((measurement.gbps() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_bandwidth_measurement_format_speed() {
        // Test Kbps
        let measurement = BandwidthMeasurement::new(
            50_000, // Small bytes
            Duration::from_secs(1),
            1,
        );
        assert!(measurement.format_speed().contains("Kbps"));

        // Test Mbps
        let measurement = BandwidthMeasurement::new(
            12_500_000, // 100 Mbps
            Duration::from_secs(1),
            1,
        );
        assert!(measurement.format_speed().contains("Mbps"));

        // Test Gbps
        let measurement = BandwidthMeasurement::new(
            1_250_000_000, // 10 Gbps
            Duration::from_secs(1),
            1,
        );
        assert!(measurement.format_speed().contains("Gbps"));
    }

    #[test]
    fn test_bandwidth_sample_mbps() {
        let sample = BandwidthSample {
            elapsed: Duration::from_secs(0),
            bytes: 12_500_000, // 100 Mbps for 1 second
            duration: Duration::from_secs(1),
        };

        assert!((sample.mbps() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_bandwidth_sample_zero_duration() {
        let sample = BandwidthSample {
            elapsed: Duration::from_secs(0),
            bytes: 12_500_000,
            duration: Duration::ZERO,
        };

        assert_eq!(sample.mbps(), 0.0);
    }

    #[test]
    fn test_speed_test_server_new() {
        let server = SpeedTestServer::new("Test Server", "https://example.com");

        assert_eq!(server.name, "Test Server");
        assert_eq!(server.url, "https://example.com");
        assert!(server.location.is_none());
        assert!(server.country.is_none());
    }

    #[test]
    fn test_speed_test_server_with_location() {
        let server = SpeedTestServer::new("Test Server", "https://example.com")
            .with_location("San Francisco", "USA");

        assert_eq!(server.location, Some("San Francisco".to_string()));
        assert_eq!(server.country, Some("USA".to_string()));
    }

    #[test]
    fn test_speed_test_server_default() {
        let server = SpeedTestServer::default();
        assert_eq!(server.name, "Unknown");
        assert_eq!(server.url, "");
    }

    #[test]
    fn test_speed_test_result_new() {
        let server = SpeedTestServer::new("Test", "https://test.com");
        let result = SpeedTestResult::new(server, "cloudflare");

        assert_eq!(result.provider, "cloudflare");
        assert!(result.download.is_none());
        assert!(result.upload.is_none());
        assert!(result.latency.is_none());
        assert!(result.buffer_bloat.is_none());
        assert!(result.consistency.is_none());
    }

    #[test]
    fn test_speed_test_result_download_mbps() {
        let server = SpeedTestServer::new("Test", "https://test.com");
        let mut result = SpeedTestResult::new(server, "cloudflare");

        assert!(result.download_mbps().is_none());

        result.download = Some(BandwidthMeasurement::new(
            12_500_000,
            Duration::from_secs(1),
            4,
        ));

        assert!((result.download_mbps().unwrap() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_speed_test_result_calculate_consistency() {
        let server = SpeedTestServer::new("Test", "https://test.com");
        let mut result = SpeedTestResult::new(server, "cloudflare");

        let mut download = BandwidthMeasurement::new(
            125_000_000,
            Duration::from_secs(10),
            4,
        );
        download.samples = (0..10)
            .map(|i| BandwidthSample {
                elapsed: Duration::from_secs(i),
                bytes: 12_500_000,
                duration: Duration::from_secs(1),
            })
            .collect();

        result.download = Some(download);
        result.calculate_consistency();

        assert!(result.consistency.is_some());
        let consistency = result.consistency.unwrap();
        assert_eq!(consistency.rating, ConsistencyRating::Excellent);
    }
}
