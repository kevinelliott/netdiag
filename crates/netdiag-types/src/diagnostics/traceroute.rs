//! Traceroute result types.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Result of a traceroute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteResult {
    /// Target address
    pub target: IpAddr,
    /// Resolved target hostname
    pub target_hostname: Option<String>,
    /// Hops along the path
    pub hops: Vec<TracerouteHop>,
    /// Whether the target was reached
    pub reached: bool,
    /// Total duration of traceroute
    pub duration: Duration,
    /// Protocol used
    pub protocol: TracerouteProtocol,
}

impl TracerouteResult {
    /// Returns the number of hops to reach the target.
    #[must_use]
    pub fn hop_count(&self) -> usize {
        if self.reached {
            self.hops.len()
        } else {
            0
        }
    }

    /// Finds hops with high latency.
    #[must_use]
    pub fn high_latency_hops(&self, threshold_ms: u64) -> Vec<&TracerouteHop> {
        self.hops
            .iter()
            .filter(|h| {
                h.avg_rtt.is_some_and(|rtt| {
                    #[allow(clippy::cast_possible_truncation)]
                    let millis = rtt.as_millis() as u64;
                    millis > threshold_ms
                })
            })
            .collect()
    }

    /// Finds hops where latency increases significantly.
    ///
    /// # Panics
    ///
    /// Panics if `curr` is less than `prev` when both are `Some`, which should never happen
    /// as latency should only increase along the path.
    #[must_use]
    pub fn latency_jumps(&self, threshold_ms: u64) -> Vec<(&TracerouteHop, Duration)> {
        let mut jumps = Vec::new();
        let mut prev_rtt: Option<Duration> = None;

        for hop in &self.hops {
            if let (Some(prev), Some(curr)) = (prev_rtt, hop.avg_rtt) {
                if curr > prev {
                    let diff = curr.checked_sub(prev).unwrap();
                    #[allow(clippy::cast_possible_truncation)]
                    let diff_millis = diff.as_millis() as u64;
                    if diff_millis > threshold_ms {
                        jumps.push((hop, diff));
                    }
                }
            }
            prev_rtt = hop.avg_rtt;
        }

        jumps
    }
}

/// A single hop in a traceroute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteHop {
    /// Hop number (TTL)
    pub hop: u8,
    /// Probes sent to this hop
    pub probes: Vec<TracerouteProbe>,
    /// IP address (if responded)
    pub address: Option<IpAddr>,
    /// Hostname (if resolved)
    pub hostname: Option<String>,
    /// ASN (Autonomous System Number)
    pub asn: Option<u32>,
    /// AS name
    pub as_name: Option<String>,
    /// Geographic location
    pub location: Option<HopLocation>,
    /// Average RTT for this hop
    pub avg_rtt: Option<Duration>,
    /// Whether all probes timed out
    pub all_timeout: bool,
}

impl TracerouteHop {
    /// Creates a timeout hop.
    #[must_use]
    pub fn timeout(hop: u8, probe_count: u8) -> Self {
        let probes = (0..probe_count)
            .map(|_| TracerouteProbe::timeout())
            .collect();
        Self {
            hop,
            probes,
            address: None,
            hostname: None,
            asn: None,
            as_name: None,
            location: None,
            avg_rtt: None,
            all_timeout: true,
        }
    }

    /// Calculates statistics from probes.
    pub fn calculate_stats(&mut self) {
        let rtts: Vec<Duration> = self.probes.iter().filter_map(|p| p.rtt).collect();

        self.all_timeout = rtts.is_empty();

        if !rtts.is_empty() {
            let sum: Duration = rtts.iter().sum();
            #[allow(clippy::cast_possible_truncation)]
            let len = rtts.len() as u32;
            self.avg_rtt = Some(sum / len);
        }
    }
}

/// A single probe in a traceroute hop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteProbe {
    /// Round-trip time (None if timeout)
    pub rtt: Option<Duration>,
    /// Response address (may differ from hop address for ECMP)
    pub address: Option<IpAddr>,
    /// ICMP response type
    pub response_type: Option<IcmpResponse>,
}

impl TracerouteProbe {
    /// Creates a successful probe.
    #[must_use]
    pub fn success(rtt: Duration, address: IpAddr, response_type: IcmpResponse) -> Self {
        Self {
            rtt: Some(rtt),
            address: Some(address),
            response_type: Some(response_type),
        }
    }

    /// Creates a timeout probe.
    #[must_use]
    pub fn timeout() -> Self {
        Self {
            rtt: None,
            address: None,
            response_type: None,
        }
    }
}

/// ICMP response type for traceroute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum IcmpResponse {
    /// Time Exceeded
    TimeExceeded,
    /// Echo Reply (destination reached)
    EchoReply,
    /// Destination Unreachable
    DestinationUnreachable,
    /// Port Unreachable (for UDP traceroute)
    PortUnreachable,
    /// Administratively Prohibited
    AdminProhibited,
}

/// Traceroute protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "lowercase")]
pub enum TracerouteProtocol {
    /// ICMP Echo
    #[default]
    Icmp,
    /// UDP
    Udp,
    /// TCP SYN
    Tcp,
}

/// Geographic location of a hop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HopLocation {
    /// Country code
    pub country: Option<String>,
    /// City
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
}
