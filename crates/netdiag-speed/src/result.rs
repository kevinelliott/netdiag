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
