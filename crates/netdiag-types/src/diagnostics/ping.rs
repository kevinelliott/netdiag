//! Ping result types.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Result of a single ping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    /// Sequence number
    pub seq: u16,
    /// Target address
    pub target: IpAddr,
    /// Round-trip time (None if timeout)
    pub rtt: Option<Duration>,
    /// TTL of reply
    pub ttl: Option<u8>,
    /// Packet size in bytes
    pub size: usize,
    /// Whether this ping succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl PingResult {
    /// Creates a successful ping result.
    #[must_use]
    pub fn success(seq: u16, target: IpAddr, rtt: Duration, ttl: u8, size: usize) -> Self {
        Self {
            seq,
            target,
            rtt: Some(rtt),
            ttl: Some(ttl),
            size,
            success: true,
            error: None,
        }
    }

    /// Creates a timeout ping result.
    #[must_use]
    pub fn timeout(seq: u16, target: IpAddr, size: usize) -> Self {
        Self {
            seq,
            target,
            rtt: None,
            ttl: None,
            size,
            success: false,
            error: Some("Request timed out".to_string()),
        }
    }

    /// Creates a failed ping result.
    #[must_use]
    pub fn failed(seq: u16, target: IpAddr, error: impl Into<String>) -> Self {
        Self {
            seq,
            target,
            rtt: None,
            ttl: None,
            size: 0,
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Aggregated ping statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingStats {
    /// Target address
    pub target: IpAddr,
    /// Resolved hostname (if any)
    pub hostname: Option<String>,
    /// Number of packets transmitted
    pub transmitted: u32,
    /// Number of packets received
    pub received: u32,
    /// Number of packets lost
    pub lost: u32,
    /// Packet loss percentage
    pub loss_percent: f64,
    /// Minimum RTT
    pub min_rtt: Option<Duration>,
    /// Maximum RTT
    pub max_rtt: Option<Duration>,
    /// Average RTT
    pub avg_rtt: Option<Duration>,
    /// Standard deviation of RTT
    pub stddev_rtt: Option<Duration>,
    /// Jitter (variation in RTT)
    pub jitter: Option<Duration>,
    /// All individual results
    pub results: Vec<PingResult>,
    /// Test duration
    pub duration: Duration,
}

impl PingStats {
    /// Creates new ping statistics from results.
    #[must_use]
    pub fn from_results(target: IpAddr, results: Vec<PingResult>, duration: Duration) -> Self {
        let transmitted = results.len() as u32;
        let received = results.iter().filter(|r| r.success).count() as u32;
        let lost = transmitted - received;
        let loss_percent = if transmitted > 0 {
            (lost as f64 / transmitted as f64) * 100.0
        } else {
            0.0
        };

        let rtts: Vec<Duration> = results.iter().filter_map(|r| r.rtt).collect();

        let (min_rtt, max_rtt, avg_rtt, stddev_rtt, jitter) = if rtts.is_empty() {
            (None, None, None, None, None)
        } else {
            let min = *rtts.iter().min().unwrap();
            let max = *rtts.iter().max().unwrap();
            let sum: Duration = rtts.iter().sum();
            let avg = sum / rtts.len() as u32;

            // Calculate standard deviation
            let variance: f64 = rtts
                .iter()
                .map(|rtt| {
                    let diff = rtt.as_secs_f64() - avg.as_secs_f64();
                    diff * diff
                })
                .sum::<f64>()
                / rtts.len() as f64;
            let stddev = Duration::from_secs_f64(variance.sqrt());

            // Calculate jitter (average difference between consecutive RTTs)
            let jitter = if rtts.len() > 1 {
                let sum_diff: f64 = rtts
                    .windows(2)
                    .map(|w| (w[1].as_secs_f64() - w[0].as_secs_f64()).abs())
                    .sum();
                Some(Duration::from_secs_f64(sum_diff / (rtts.len() - 1) as f64))
            } else {
                None
            };

            (Some(min), Some(max), Some(avg), Some(stddev), jitter)
        };

        Self {
            target,
            hostname: None,
            transmitted,
            received,
            lost,
            loss_percent,
            min_rtt,
            max_rtt,
            avg_rtt,
            stddev_rtt,
            jitter,
            results,
            duration,
        }
    }

    /// Returns a quality rating based on the statistics.
    #[must_use]
    pub fn quality_rating(&self) -> PingQuality {
        if self.loss_percent >= 50.0 {
            return PingQuality::VeryPoor;
        }
        if self.loss_percent >= 10.0 {
            return PingQuality::Poor;
        }
        if self.loss_percent >= 2.0 {
            return PingQuality::Fair;
        }

        match self.avg_rtt {
            Some(rtt) if rtt.as_millis() > 200 => PingQuality::Fair,
            Some(rtt) if rtt.as_millis() > 100 => PingQuality::Good,
            Some(rtt) if rtt.as_millis() > 50 => PingQuality::VeryGood,
            Some(_) => PingQuality::Excellent,
            None => PingQuality::VeryPoor,
        }
    }
}

/// Ping quality rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum PingQuality {
    /// Excellent quality (< 50ms, < 2% loss)
    Excellent,
    /// Very good quality (< 100ms, < 2% loss)
    VeryGood,
    /// Good quality (< 200ms, < 2% loss)
    Good,
    /// Fair quality (high latency or some loss)
    Fair,
    /// Poor quality (> 10% loss)
    Poor,
    /// Very poor quality (> 50% loss)
    VeryPoor,
}
