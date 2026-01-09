//! Capture statistics.

use crate::decode::Protocol;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Capture statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaptureStats {
    /// Total packets captured.
    pub packets_captured: u64,

    /// Total packets dropped (by kernel).
    pub packets_dropped: u64,

    /// Total packets dropped by interface.
    pub packets_dropped_interface: u64,

    /// Total bytes captured.
    pub bytes_captured: u64,

    /// Capture start time.
    pub start_time: Option<DateTime<Utc>>,

    /// Capture end time.
    pub end_time: Option<DateTime<Utc>>,

    /// Capture duration.
    pub duration: Duration,

    /// Packets per second.
    pub packets_per_second: f64,

    /// Bytes per second.
    pub bytes_per_second: f64,

    /// Protocol breakdown.
    pub protocols: HashMap<String, ProtocolStats>,

    /// Top talkers (by IP).
    pub top_talkers: Vec<(String, u64)>,

    /// Top ports.
    pub top_ports: Vec<(u16, u64)>,
}

impl CaptureStats {
    /// Create new empty stats.
    pub fn new() -> Self {
        Self {
            start_time: Some(Utc::now()),
            ..Default::default()
        }
    }

    /// Update stats with a new packet.
    pub fn update(
        &mut self,
        protocol: Protocol,
        length: usize,
        src_ip: Option<&str>,
        dst_ip: Option<&str>,
        src_port: Option<u16>,
        dst_port: Option<u16>,
    ) {
        self.packets_captured += 1;
        self.bytes_captured += length as u64;

        // Update protocol stats
        let proto_name = protocol.name().to_string();
        let proto_stats = self
            .protocols
            .entry(proto_name)
            .or_insert_with(ProtocolStats::new);
        proto_stats.packets += 1;
        proto_stats.bytes += length as u64;

        // Track source IP
        if let Some(ip) = src_ip {
            proto_stats.update_ip(ip);
        }

        // Track destination IP
        if let Some(ip) = dst_ip {
            proto_stats.update_ip(ip);
        }

        // Track ports
        if let Some(port) = src_port {
            proto_stats.update_port(port);
        }
        if let Some(port) = dst_port {
            proto_stats.update_port(port);
        }
    }

    /// Finalize stats (calculate rates, sort top lists).
    pub fn finalize(&mut self) {
        self.end_time = Some(Utc::now());

        if let (Some(start), Some(end)) = (self.start_time, self.end_time) {
            let duration_secs = (end - start).num_milliseconds() as f64 / 1000.0;
            self.duration = Duration::from_secs_f64(duration_secs);

            if duration_secs > 0.0 {
                self.packets_per_second = self.packets_captured as f64 / duration_secs;
                self.bytes_per_second = self.bytes_captured as f64 / duration_secs;
            }
        }

        // Calculate top talkers across all protocols
        let mut ip_counts: HashMap<String, u64> = HashMap::new();
        let mut port_counts: HashMap<u16, u64> = HashMap::new();

        for stats in self.protocols.values() {
            for (ip, count) in &stats.ip_counts {
                *ip_counts.entry(ip.clone()).or_insert(0) += count;
            }
            for (port, count) in &stats.port_counts {
                *port_counts.entry(*port).or_insert(0) += count;
            }
        }

        // Sort and take top 10
        let mut talkers: Vec<_> = ip_counts.into_iter().collect();
        talkers.sort_by(|a, b| b.1.cmp(&a.1));
        self.top_talkers = talkers.into_iter().take(10).collect();

        let mut ports: Vec<_> = port_counts.into_iter().collect();
        ports.sort_by(|a, b| b.1.cmp(&a.1));
        self.top_ports = ports.into_iter().take(10).collect();
    }

    /// Get drop rate as percentage.
    pub fn drop_rate(&self) -> f64 {
        if self.packets_captured == 0 {
            return 0.0;
        }
        let total = self.packets_captured + self.packets_dropped;
        (self.packets_dropped as f64 / total as f64) * 100.0
    }

    /// Format bytes per second as human readable.
    pub fn format_bandwidth(&self) -> String {
        let bps = self.bytes_per_second * 8.0; // Convert to bits
        if bps >= 1_000_000_000.0 {
            format!("{:.2} Gbps", bps / 1_000_000_000.0)
        } else if bps >= 1_000_000.0 {
            format!("{:.2} Mbps", bps / 1_000_000.0)
        } else if bps >= 1_000.0 {
            format!("{:.2} Kbps", bps / 1_000.0)
        } else {
            format!("{:.0} bps", bps)
        }
    }
}

/// Per-protocol statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolStats {
    /// Number of packets.
    pub packets: u64,

    /// Total bytes.
    pub bytes: u64,

    /// IP address counts (for top talkers).
    #[serde(skip)]
    pub ip_counts: HashMap<String, u64>,

    /// Port counts.
    #[serde(skip)]
    pub port_counts: HashMap<u16, u64>,
}

impl ProtocolStats {
    /// Create new protocol stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update IP count.
    pub fn update_ip(&mut self, ip: &str) {
        *self.ip_counts.entry(ip.to_string()).or_insert(0) += 1;
    }

    /// Update port count.
    pub fn update_port(&mut self, port: u16) {
        *self.port_counts.entry(port).or_insert(0) += 1;
    }

    /// Get packets per protocol as percentage.
    pub fn percentage(&self, total: u64) -> f64 {
        if total == 0 {
            return 0.0;
        }
        (self.packets as f64 / total as f64) * 100.0
    }
}
