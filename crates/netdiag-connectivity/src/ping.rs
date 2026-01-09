//! Ping implementation.
//!
//! Uses system ping command as a fallback since raw sockets require root.

use netdiag_types::diagnostics::{PingResult, PingStats};
use netdiag_types::error::{Error, Result};
use std::net::IpAddr;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::debug;

/// Ping configuration.
#[derive(Debug, Clone)]
pub struct PingConfig {
    /// Number of pings to send
    pub count: u32,
    /// Timeout per ping
    pub timeout: Duration,
    /// Interval between pings
    pub interval: Duration,
    /// Packet size in bytes
    pub size: usize,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            count: 4,
            timeout: Duration::from_secs(2),
            interval: Duration::from_secs(1),
            size: 64,
        }
    }
}

/// Pinger for connectivity testing.
pub struct Pinger {
    /// Whether to use system ping (fallback)
    use_system_ping: bool,
}

impl Pinger {
    /// Creates a new pinger.
    pub fn new() -> Self {
        // Check if we have raw socket capability
        let _has_raw_socket = unsafe { libc::geteuid() == 0 };

        Self {
            // For now, always use system ping
            // Raw socket implementation would require more complex ICMP handling
            use_system_ping: true,
        }
    }

    /// Pings a target IP address.
    pub async fn ping(&self, target: IpAddr, config: &PingConfig) -> Result<PingStats> {
        if self.use_system_ping {
            self.system_ping(target, config).await
        } else {
            // Raw socket ping would go here
            self.system_ping(target, config).await
        }
    }

    /// Uses system ping command.
    async fn system_ping(&self, target: IpAddr, config: &PingConfig) -> Result<PingStats> {
        let start = Instant::now();

        debug!("Pinging {} with {} packets", target, config.count);

        // Build ping command based on OS
        let output = if cfg!(target_os = "macos") {
            Command::new("ping")
                .args([
                    "-c",
                    &config.count.to_string(),
                    "-t",
                    &config.timeout.as_secs().to_string(),
                    "-i",
                    &format!("{:.1}", config.interval.as_secs_f64()),
                    "-s",
                    &config.size.to_string(),
                    &target.to_string(),
                ])
                .output()
        } else if cfg!(target_os = "linux") {
            Command::new("ping")
                .args([
                    "-c",
                    &config.count.to_string(),
                    "-W",
                    &config.timeout.as_secs().to_string(),
                    "-i",
                    &format!("{:.1}", config.interval.as_secs_f64()),
                    "-s",
                    &config.size.to_string(),
                    &target.to_string(),
                ])
                .output()
        } else {
            Command::new("ping")
                .args([
                    "-n",
                    &config.count.to_string(),
                    "-w",
                    &(config.timeout.as_millis() as u64).to_string(),
                    &target.to_string(),
                ])
                .output()
        };

        let output = output.map_err(|e| Error::Ping {
            target,
            message: format!("Failed to execute ping: {}", e),
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let duration = start.elapsed();

        debug!("Ping output: {}", stdout);

        // Parse ping output
        let results = self.parse_ping_output(&stdout, target, config);
        let stats = PingStats::from_results(target, results, duration);

        Ok(stats)
    }

    /// Parses ping command output.
    fn parse_ping_output(
        &self,
        output: &str,
        target: IpAddr,
        config: &PingConfig,
    ) -> Vec<PingResult> {
        let mut results = Vec::new();
        let mut seq: u16 = 0;

        for line in output.lines() {
            let line = line.trim();

            // Parse individual ping replies
            // macOS/Linux format: "64 bytes from X.X.X.X: icmp_seq=1 ttl=64 time=1.234 ms"
            if line.contains("bytes from") && line.contains("time=") {
                if let Some(result) = self.parse_ping_line(line, seq, target, config.size) {
                    results.push(result);
                    seq += 1;
                }
            }
            // Handle timeout lines
            else if line.contains("Request timeout") || line.contains("timed out") {
                results.push(PingResult::timeout(seq, target, config.size));
                seq += 1;
            }
        }

        // If no results parsed, try to parse summary statistics
        if results.is_empty() {
            if let Some(summary) = self.parse_summary(output, target, config.count) {
                return summary;
            }
        }

        results
    }

    /// Parses a single ping reply line.
    fn parse_ping_line(
        &self,
        line: &str,
        seq: u16,
        target: IpAddr,
        size: usize,
    ) -> Option<PingResult> {
        // Extract TTL
        let ttl = line
            .split("ttl=")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(64);

        // Extract time
        let time_ms = line
            .split("time=")
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| s.trim_end_matches("ms").trim().parse::<f64>().ok());

        match time_ms {
            Some(ms) => {
                let rtt = Duration::from_secs_f64(ms / 1000.0);
                Some(PingResult::success(seq, target, rtt, ttl, size))
            }
            None => Some(PingResult::timeout(seq, target, size)),
        }
    }

    /// Parses ping summary statistics when individual lines aren't available.
    fn parse_summary(&self, output: &str, target: IpAddr, count: u32) -> Option<Vec<PingResult>> {
        // Look for summary line like "4 packets transmitted, 4 packets received"
        for line in output.lines() {
            if line.contains("packets transmitted") && line.contains("received") {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let transmitted = parts[0]
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(count);

                    let received = parts[1]
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(0);

                    // Parse RTT stats from "min/avg/max/stddev = X/Y/Z/W ms"
                    let avg_rtt =
                        output
                            .lines()
                            .find(|l| l.contains("min/avg/max"))
                            .and_then(|l| {
                                l.split('=')
                                    .nth(1)
                                    .and_then(|s| s.split('/').nth(1))
                                    .and_then(|s| s.trim().parse::<f64>().ok())
                                    .map(|ms| Duration::from_secs_f64(ms / 1000.0))
                            });

                    // Create synthetic results based on summary
                    let mut results = Vec::new();
                    for i in 0..transmitted {
                        if i < received {
                            results.push(PingResult::success(
                                i as u16,
                                target,
                                avg_rtt.unwrap_or(Duration::from_millis(50)),
                                64,
                                64,
                            ));
                        } else {
                            results.push(PingResult::timeout(i as u16, target, 64));
                        }
                    }

                    return Some(results);
                }
            }
        }

        None
    }
}

impl Default for Pinger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ping_localhost() {
        let pinger = Pinger::new();
        let config = PingConfig {
            count: 3,
            timeout: Duration::from_secs(2),
            interval: Duration::from_millis(500),
            size: 64,
        };

        let result = pinger
            .ping("127.0.0.1".parse().unwrap(), &config)
            .await
            .unwrap();

        assert!(result.received > 0);
        assert!(result.avg_rtt.is_some());
    }
}
