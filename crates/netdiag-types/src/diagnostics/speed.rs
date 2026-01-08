//! Speed test result types.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Complete speed test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResult {
    /// Download speed in Mbps
    pub download_mbps: f64,
    /// Upload speed in Mbps
    pub upload_mbps: f64,
    /// Latency (ping) in milliseconds
    pub latency_ms: f64,
    /// Jitter in milliseconds
    pub jitter_ms: Option<f64>,
    /// Server used for the test
    pub server: SpeedTestServer,
    /// Download test details
    pub download: SpeedTestPhase,
    /// Upload test details
    pub upload: SpeedTestPhase,
    /// Test timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Test duration
    pub duration: Duration,
    /// Client IP address
    pub client_ip: Option<IpAddr>,
    /// ISP name
    pub isp: Option<String>,
}

impl SpeedTestResult {
    /// Returns the download speed in bytes per second.
    #[must_use]
    pub fn download_bytes_per_sec(&self) -> f64 {
        self.download_mbps * 125_000.0 // Mbps to bytes/sec
    }

    /// Returns the upload speed in bytes per second.
    #[must_use]
    pub fn upload_bytes_per_sec(&self) -> f64 {
        self.upload_mbps * 125_000.0
    }

    /// Returns a quality rating for the connection.
    #[must_use]
    pub fn quality_rating(&self) -> SpeedQuality {
        // Based on download speed primarily
        if self.download_mbps >= 100.0 && self.latency_ms < 20.0 {
            SpeedQuality::Excellent
        } else if self.download_mbps >= 50.0 && self.latency_ms < 50.0 {
            SpeedQuality::VeryGood
        } else if self.download_mbps >= 25.0 && self.latency_ms < 100.0 {
            SpeedQuality::Good
        } else if self.download_mbps >= 10.0 {
            SpeedQuality::Fair
        } else if self.download_mbps >= 1.0 {
            SpeedQuality::Poor
        } else {
            SpeedQuality::VeryPoor
        }
    }
}

/// Speed test phase (download or upload).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestPhase {
    /// Speed in Mbps
    pub speed_mbps: f64,
    /// Bytes transferred
    pub bytes_transferred: u64,
    /// Duration of this phase
    pub duration: Duration,
    /// Number of connections used
    pub connections: u32,
    /// Speed samples over time
    pub samples: Vec<SpeedSample>,
}

/// A speed sample during testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSample {
    /// Time offset from start
    pub time_offset: Duration,
    /// Speed at this point in Mbps
    pub speed_mbps: f64,
    /// Bytes transferred at this point
    pub bytes: u64,
}

/// Speed test server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestServer {
    /// Server ID
    pub id: String,
    /// Server name
    pub name: String,
    /// Server sponsor/provider
    pub sponsor: Option<String>,
    /// Server URL
    pub url: String,
    /// Server IP address
    pub address: Option<IpAddr>,
    /// Server location
    pub location: Option<ServerLocation>,
    /// Distance from client in km
    pub distance_km: Option<f64>,
    /// Latency to server in ms
    pub latency_ms: Option<f64>,
}

/// Server location information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerLocation {
    /// City
    pub city: Option<String>,
    /// Country
    pub country: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
}

/// Speed quality rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum SpeedQuality {
    /// Excellent (100+ Mbps, < 20ms latency)
    Excellent,
    /// Very good (50+ Mbps, < 50ms latency)
    VeryGood,
    /// Good (25+ Mbps, < 100ms latency)
    Good,
    /// Fair (10+ Mbps)
    Fair,
    /// Poor (1-10 Mbps)
    Poor,
    /// Very poor (< 1 Mbps)
    VeryPoor,
}

/// Bandwidth measurement for internal use.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bandwidth {
    /// Bits per second
    pub bits_per_sec: f64,
}

impl Bandwidth {
    /// Creates bandwidth from bits per second.
    #[must_use]
    pub const fn from_bps(bps: f64) -> Self {
        Self { bits_per_sec: bps }
    }

    /// Creates bandwidth from megabits per second.
    #[must_use]
    pub fn from_mbps(mbps: f64) -> Self {
        Self {
            bits_per_sec: mbps * 1_000_000.0,
        }
    }

    /// Returns the bandwidth in Mbps.
    #[must_use]
    pub fn as_mbps(&self) -> f64 {
        self.bits_per_sec / 1_000_000.0
    }

    /// Returns the bandwidth in Gbps.
    #[must_use]
    pub fn as_gbps(&self) -> f64 {
        self.bits_per_sec / 1_000_000_000.0
    }

    /// Returns the bandwidth in bytes per second.
    #[must_use]
    pub fn as_bytes_per_sec(&self) -> f64 {
        self.bits_per_sec / 8.0
    }
}

impl std::fmt::Display for Bandwidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.bits_per_sec >= 1_000_000_000.0 {
            write!(f, "{:.2} Gbps", self.as_gbps())
        } else if self.bits_per_sec >= 1_000_000.0 {
            write!(f, "{:.2} Mbps", self.as_mbps())
        } else if self.bits_per_sec >= 1_000.0 {
            write!(f, "{:.2} Kbps", self.bits_per_sec / 1000.0)
        } else {
            write!(f, "{:.2} bps", self.bits_per_sec)
        }
    }
}
