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
    /// Latency percentiles for detailed analysis
    pub latency_percentiles: Option<LatencyPercentiles>,
    /// `VoIP` quality metrics
    pub voip_quality: Option<VoipQuality>,
}

/// Latency percentiles for detailed analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    /// 50th percentile (median)
    pub p50: Duration,
    /// 75th percentile
    pub p75: Duration,
    /// 90th percentile
    pub p90: Duration,
    /// 95th percentile
    pub p95: Duration,
    /// 99th percentile
    pub p99: Duration,
    /// Interquartile range (P75 - P25)
    pub iqr: Duration,
}

impl LatencyPercentiles {
    /// Calculates percentiles from a slice of RTT durations.
    #[must_use]
    pub fn from_rtts(rtts: &[Duration]) -> Option<Self> {
        if rtts.is_empty() {
            return None;
        }

        let mut sorted: Vec<Duration> = rtts.to_vec();
        sorted.sort();

        let percentile = |p: f64| -> Duration {
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
            sorted[idx.min(sorted.len() - 1)]
        };

        let p25 = percentile(25.0);
        let p50 = percentile(50.0);
        let p75 = percentile(75.0);
        let p90 = percentile(90.0);
        let p95 = percentile(95.0);
        let p99 = percentile(99.0);

        Some(Self {
            p50,
            p75,
            p90,
            p95,
            p99,
            iqr: p75.saturating_sub(p25),
        })
    }
}

/// `VoIP` quality metrics based on ITU-T E-model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoipQuality {
    /// Mean Opinion Score (1.0-5.0)
    pub mos: f64,
    /// R-factor (0-100)
    pub r_factor: f64,
    /// Quality rating
    pub rating: VoipRating,
    /// Effective latency (one-way delay estimate)
    pub effective_latency_ms: f64,
    /// Impact factors
    pub impact: VoipImpact,
}

/// Breakdown of factors affecting `VoIP` quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoipImpact {
    /// Delay impairment factor
    pub delay_factor: f64,
    /// Jitter impairment factor
    pub jitter_factor: f64,
    /// Packet loss impairment factor
    pub loss_factor: f64,
}

impl VoipQuality {
    /// Calculates `VoIP` quality from ping statistics.
    ///
    /// Uses a simplified E-model based on:
    /// - One-way delay (RTT/2)
    /// - Jitter
    /// - Packet loss
    #[must_use]
    pub fn from_stats(avg_rtt: Duration, jitter: Option<Duration>, loss_percent: f64) -> Self {
        // Calculate one-way delay (RTT / 2)
        let one_way_delay_ms = avg_rtt.as_secs_f64() * 1000.0 / 2.0;

        // Calculate effective latency (delay + jitter buffer)
        let jitter_ms = jitter.map_or(0.0, |j| j.as_secs_f64() * 1000.0);
        let jitter_buffer_ms = jitter_ms * 2.0; // Typical jitter buffer = 2x jitter
        let effective_latency_ms = one_way_delay_ms + jitter_buffer_ms;

        // Delay impairment factor (Id)
        // Based on ITU-T G.107 simplified model
        let delay_factor = if effective_latency_ms < 177.3 {
            0.024 * effective_latency_ms
        } else {
            0.024 * effective_latency_ms + 0.11 * (effective_latency_ms - 177.3)
        };

        // Jitter impairment (simplified)
        let jitter_factor = jitter_ms * 0.4;

        // Packet loss impairment factor (Ie-eff)
        // Using codec-agnostic approximation
        let loss_factor = if loss_percent > 0.0 {
            loss_percent * 2.5 + loss_percent.powf(2.0) * 0.1
        } else {
            0.0
        };

        // Calculate R-factor (R = 93.2 - Id - Ie-eff)
        // 93.2 is baseline R assuming no impairments
        let r_factor = (93.2 - delay_factor - jitter_factor - loss_factor).clamp(0.0, 100.0);

        // Convert R-factor to MOS using standard formula
        // MOS = 1 + 0.035*R + R*(R-60)*(100-R)*7*10^-6
        let mos = if r_factor < 0.0 {
            1.0
        } else if r_factor > 100.0 {
            4.5
        } else {
            1.0 + 0.035 * r_factor + r_factor * (r_factor - 60.0) * (100.0 - r_factor) * 7.0e-6
        };

        let rating = VoipRating::from_mos(mos);

        Self {
            mos,
            r_factor,
            rating,
            effective_latency_ms,
            impact: VoipImpact {
                delay_factor,
                jitter_factor,
                loss_factor,
            },
        }
    }
}

/// `VoIP` quality rating based on MOS score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum VoipRating {
    /// Excellent quality (MOS >= 4.3)
    Excellent,
    /// Good quality (MOS >= 4.0)
    Good,
    /// Fair quality (MOS >= 3.6)
    Fair,
    /// Poor quality (MOS >= 3.1)
    Poor,
    /// Bad quality (MOS >= 2.6)
    Bad,
    /// Very bad quality (MOS < 2.6)
    VeryBad,
}

impl VoipRating {
    /// Determines rating from MOS score.
    #[must_use]
    pub fn from_mos(mos: f64) -> Self {
        if mos >= 4.3 {
            Self::Excellent
        } else if mos >= 4.0 {
            Self::Good
        } else if mos >= 3.6 {
            Self::Fair
        } else if mos >= 3.1 {
            Self::Poor
        } else if mos >= 2.6 {
            Self::Bad
        } else {
            Self::VeryBad
        }
    }
}

impl PingStats {
    /// Creates new ping statistics from results.
    ///
    /// # Panics
    ///
    /// Panics if the RTTs vector is empty when calculating min/max, which should never happen
    /// since we check `rtts.is_empty()` before accessing these values.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_precision_loss)]
    pub fn from_results(target: IpAddr, results: Vec<PingResult>, duration: Duration) -> Self {
        let transmitted = results.len() as u32;
        let received = results.iter().filter(|r| r.success).count() as u32;
        let lost = transmitted - received;
        let loss_percent = if transmitted > 0 {
            (f64::from(lost) / f64::from(transmitted)) * 100.0
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
            #[allow(clippy::cast_possible_truncation)]
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
                #[allow(clippy::cast_precision_loss)]
                let len_minus_one = (rtts.len() - 1) as f64;
                Some(Duration::from_secs_f64(sum_diff / len_minus_one))
            } else {
                None
            };

            (Some(min), Some(max), Some(avg), Some(stddev), jitter)
        };

        // Calculate latency percentiles
        let latency_percentiles = LatencyPercentiles::from_rtts(&rtts);

        // Calculate VoIP quality metrics
        let voip_quality = avg_rtt.map(|avg| VoipQuality::from_stats(avg, jitter, loss_percent));

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
            latency_percentiles,
            voip_quality,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn test_latency_percentiles_empty() {
        assert!(LatencyPercentiles::from_rtts(&[]).is_none());
    }

    #[test]
    fn test_latency_percentiles_single() {
        let rtts = vec![ms(50)];
        let percentiles = LatencyPercentiles::from_rtts(&rtts).unwrap();
        assert_eq!(percentiles.p50, ms(50));
        assert_eq!(percentiles.p99, ms(50));
    }

    #[test]
    fn test_latency_percentiles_multiple() {
        // 10 samples: 10, 20, 30, 40, 50, 60, 70, 80, 90, 100 ms
        let rtts: Vec<Duration> = (1..=10).map(|i| ms(i * 10)).collect();
        let percentiles = LatencyPercentiles::from_rtts(&rtts).unwrap();

        // P50 should be around 50-60ms (median)
        assert!(percentiles.p50 >= ms(50) && percentiles.p50 <= ms(60));
        // P95 should be around 90-100ms
        assert!(percentiles.p95 >= ms(90));
        // P99 should be near 100ms
        assert!(percentiles.p99 >= ms(90));
    }

    #[test]
    fn test_latency_percentiles_iqr() {
        let rtts: Vec<Duration> = (1..=100).map(|i| ms(i)).collect();
        let percentiles = LatencyPercentiles::from_rtts(&rtts).unwrap();

        // IQR = P75 - P25, should be around 50ms for 1-100 range
        assert!(percentiles.iqr >= ms(45) && percentiles.iqr <= ms(55));
    }

    #[test]
    fn test_voip_rating_from_mos() {
        assert_eq!(VoipRating::from_mos(4.5), VoipRating::Excellent);
        assert_eq!(VoipRating::from_mos(4.3), VoipRating::Excellent);
        assert_eq!(VoipRating::from_mos(4.1), VoipRating::Good);
        assert_eq!(VoipRating::from_mos(4.0), VoipRating::Good);
        assert_eq!(VoipRating::from_mos(3.8), VoipRating::Fair);
        assert_eq!(VoipRating::from_mos(3.6), VoipRating::Fair);
        assert_eq!(VoipRating::from_mos(3.3), VoipRating::Poor);
        assert_eq!(VoipRating::from_mos(3.1), VoipRating::Poor);
        assert_eq!(VoipRating::from_mos(2.8), VoipRating::Bad);
        assert_eq!(VoipRating::from_mos(2.6), VoipRating::Bad);
        assert_eq!(VoipRating::from_mos(2.0), VoipRating::VeryBad);
        assert_eq!(VoipRating::from_mos(1.0), VoipRating::VeryBad);
    }

    #[test]
    fn test_voip_quality_excellent() {
        // Low latency, no jitter, no loss
        let quality = VoipQuality::from_stats(ms(20), Some(ms(2)), 0.0);
        assert!(quality.mos >= 4.0);
        assert!(quality.r_factor >= 80.0);
        assert!(matches!(
            quality.rating,
            VoipRating::Excellent | VoipRating::Good
        ));
    }

    #[test]
    fn test_voip_quality_with_loss() {
        // 10% packet loss should degrade quality significantly
        let quality = VoipQuality::from_stats(ms(50), Some(ms(5)), 10.0);
        assert!(quality.mos < 4.0);
        assert!(quality.impact.loss_factor > 0.0);
    }

    #[test]
    fn test_voip_quality_high_latency() {
        // High latency (500ms RTT = 250ms one-way)
        let quality = VoipQuality::from_stats(ms(500), Some(ms(10)), 0.0);
        assert!(quality.mos < 4.0);
        assert!(quality.effective_latency_ms >= 250.0);
        assert!(quality.impact.delay_factor > 5.0);
    }

    #[test]
    fn test_voip_quality_high_jitter() {
        // High jitter
        let quality = VoipQuality::from_stats(ms(50), Some(ms(100)), 0.0);
        assert!(quality.impact.jitter_factor > 30.0);
    }

    #[test]
    fn test_ping_stats_from_results() {
        let target = "8.8.8.8".parse().unwrap();
        let results = vec![
            PingResult::success(1, target, ms(30), 64, 64),
            PingResult::success(2, target, ms(40), 64, 64),
            PingResult::success(3, target, ms(35), 64, 64),
            PingResult::success(4, target, ms(50), 64, 64),
            PingResult::timeout(5, target, 64),
        ];

        let stats = PingStats::from_results(target, results, Duration::from_secs(5));

        assert_eq!(stats.transmitted, 5);
        assert_eq!(stats.received, 4);
        assert_eq!(stats.lost, 1);
        assert!((stats.loss_percent - 20.0).abs() < 0.1);

        assert_eq!(stats.min_rtt, Some(ms(30)));
        assert_eq!(stats.max_rtt, Some(ms(50)));

        // Average should be (30+40+35+50)/4 = 38.75ms
        assert!(stats.avg_rtt.unwrap() >= ms(38) && stats.avg_rtt.unwrap() <= ms(40));

        // Should have percentiles and VoIP quality calculated
        assert!(stats.latency_percentiles.is_some());
        assert!(stats.voip_quality.is_some());
    }

    #[test]
    fn test_ping_stats_all_timeouts() {
        let target = "8.8.8.8".parse().unwrap();
        let results = vec![
            PingResult::timeout(1, target, 64),
            PingResult::timeout(2, target, 64),
        ];

        let stats = PingStats::from_results(target, results, Duration::from_secs(2));

        assert_eq!(stats.transmitted, 2);
        assert_eq!(stats.received, 0);
        assert_eq!(stats.loss_percent, 100.0);
        assert!(stats.min_rtt.is_none());
        assert!(stats.avg_rtt.is_none());
        assert!(stats.latency_percentiles.is_none());
        assert!(stats.voip_quality.is_none());
    }

    #[test]
    fn test_ping_quality_rating() {
        let target = "8.8.8.8".parse().unwrap();

        // Excellent: low latency, no loss
        let results = vec![PingResult::success(1, target, ms(30), 64, 64)];
        let stats = PingStats::from_results(target, results, Duration::from_secs(1));
        assert_eq!(stats.quality_rating(), PingQuality::Excellent);

        // Poor: high loss
        let results = vec![
            PingResult::success(1, target, ms(30), 64, 64),
            PingResult::timeout(2, target, 64),
            PingResult::timeout(3, target, 64),
        ];
        let stats = PingStats::from_results(target, results, Duration::from_secs(3));
        assert!(matches!(
            stats.quality_rating(),
            PingQuality::Poor | PingQuality::VeryPoor
        ));
    }
}
