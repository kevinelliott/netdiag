//! Jitter and packet loss types.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Jitter statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitterStats {
    /// Average jitter
    pub average: Duration,
    /// Maximum jitter
    pub max: Duration,
    /// Minimum jitter
    pub min: Duration,
    /// Standard deviation
    pub stddev: Duration,
    /// Number of samples
    pub sample_count: u32,
    /// Individual jitter values
    pub samples: Vec<Duration>,
}

impl JitterStats {
    /// Creates jitter stats from RTT samples.
    #[must_use]
    pub fn from_rtts(rtts: &[Duration]) -> Option<Self> {
        if rtts.len() < 2 {
            return None;
        }

        let mut jitters: Vec<Duration> = Vec::with_capacity(rtts.len() - 1);
        for window in rtts.windows(2) {
            let diff = if window[1] > window[0] {
                window[1] - window[0]
            } else {
                window[0] - window[1]
            };
            jitters.push(diff);
        }

        if jitters.is_empty() {
            return None;
        }

        let sum: Duration = jitters.iter().sum();
        let average = sum / jitters.len() as u32;
        let min = *jitters.iter().min().unwrap();
        let max = *jitters.iter().max().unwrap();

        // Calculate standard deviation
        let variance: f64 = jitters
            .iter()
            .map(|j| {
                let diff = j.as_secs_f64() - average.as_secs_f64();
                diff * diff
            })
            .sum::<f64>()
            / jitters.len() as f64;
        let stddev = Duration::from_secs_f64(variance.sqrt());

        Some(Self {
            average,
            max,
            min,
            stddev,
            sample_count: jitters.len() as u32,
            samples: jitters,
        })
    }

    /// Returns a quality rating based on jitter.
    #[must_use]
    pub fn quality_rating(&self) -> JitterQuality {
        let avg_ms = self.average.as_millis() as u64;
        match avg_ms {
            0..=5 => JitterQuality::Excellent,
            6..=15 => JitterQuality::Good,
            16..=30 => JitterQuality::Fair,
            31..=50 => JitterQuality::Poor,
            _ => JitterQuality::VeryPoor,
        }
    }
}

/// Jitter quality rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum JitterQuality {
    /// Excellent (< 5ms)
    Excellent,
    /// Good (5-15ms)
    Good,
    /// Fair (15-30ms)
    Fair,
    /// Poor (30-50ms)
    Poor,
    /// Very poor (> 50ms)
    VeryPoor,
}

/// Packet loss statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketLossStats {
    /// Total packets sent
    pub sent: u32,
    /// Packets received
    pub received: u32,
    /// Packets lost
    pub lost: u32,
    /// Loss percentage (0-100)
    pub loss_percent: f64,
    /// Burst loss count (consecutive losses)
    pub burst_count: u32,
    /// Maximum burst length
    pub max_burst_length: u32,
    /// Loss pattern (indices of lost packets)
    pub loss_pattern: Vec<u32>,
}

impl PacketLossStats {
    /// Creates packet loss stats from success/failure pattern.
    #[must_use]
    pub fn from_pattern(results: &[bool]) -> Self {
        let sent = results.len() as u32;
        let received = results.iter().filter(|&&r| r).count() as u32;
        let lost = sent - received;
        let loss_percent = if sent > 0 {
            (lost as f64 / sent as f64) * 100.0
        } else {
            0.0
        };

        // Calculate loss pattern and bursts
        let mut loss_pattern = Vec::new();
        let mut burst_count = 0;
        let mut max_burst_length = 0;
        let mut current_burst = 0;

        for (i, &success) in results.iter().enumerate() {
            if !success {
                #[allow(clippy::cast_possible_truncation)]
                loss_pattern.push(i as u32);
                current_burst += 1;
            } else {
                if current_burst > 0 {
                    burst_count += 1;
                    max_burst_length = max_burst_length.max(current_burst);
                    current_burst = 0;
                }
            }
        }
        if current_burst > 0 {
            burst_count += 1;
            max_burst_length = max_burst_length.max(current_burst);
        }

        Self {
            sent,
            received,
            lost,
            loss_percent,
            burst_count,
            max_burst_length,
            loss_pattern,
        }
    }

    /// Returns a quality rating based on packet loss.
    #[must_use]
    pub fn quality_rating(&self) -> PacketLossQuality {
        if self.loss_percent == 0.0 {
            PacketLossQuality::Excellent
        } else if self.loss_percent < 1.0 {
            PacketLossQuality::Good
        } else if self.loss_percent < 2.5 {
            PacketLossQuality::Fair
        } else if self.loss_percent < 5.0 {
            PacketLossQuality::Poor
        } else {
            PacketLossQuality::VeryPoor
        }
    }
}

/// Packet loss quality rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum PacketLossQuality {
    /// Excellent (0% loss)
    Excellent,
    /// Good (< 1% loss)
    Good,
    /// Fair (1-2.5% loss)
    Fair,
    /// Poor (2.5-5% loss)
    Poor,
    /// Very poor (> 5% loss)
    VeryPoor,
}
