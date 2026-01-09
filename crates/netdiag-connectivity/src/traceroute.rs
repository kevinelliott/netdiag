//! Traceroute implementation.
//!
//! Uses system traceroute command.

use netdiag_types::diagnostics::{
    IcmpResponse, TracerouteHop, TracerouteProbe, TracerouteProtocol, TracerouteResult,
};
use netdiag_types::error::{Error, Result};
use std::net::IpAddr;
use std::process::Command;
use std::time::{Duration, Instant};
use tracing::debug;

/// Traceroute configuration.
#[derive(Debug, Clone)]
pub struct TracerouteConfig {
    /// Maximum number of hops
    pub max_hops: u8,
    /// Number of probes per hop
    pub probes_per_hop: u8,
    /// Timeout per probe
    pub timeout: Duration,
    /// Protocol to use
    pub protocol: TracerouteProtocol,
    /// Resolve hostnames
    pub resolve_hostnames: bool,
}

impl Default for TracerouteConfig {
    fn default() -> Self {
        Self {
            max_hops: 30,
            probes_per_hop: 3,
            timeout: Duration::from_secs(3),
            protocol: TracerouteProtocol::Icmp,
            resolve_hostnames: true,
        }
    }
}

/// Tracer for path analysis.
pub struct Tracer {
    // Configuration can be stored here if needed
}

impl Tracer {
    /// Creates a new tracer.
    pub fn new() -> Self {
        Self {}
    }

    /// Traces the route to a target.
    pub async fn trace(
        &self,
        target: IpAddr,
        config: &TracerouteConfig,
    ) -> Result<TracerouteResult> {
        let start = Instant::now();

        debug!("Traceroute to {} with max {} hops", target, config.max_hops);

        // Build traceroute command based on OS
        let output = if cfg!(target_os = "macos") {
            let mut args = vec![
                "-m".to_string(),
                config.max_hops.to_string(),
                "-q".to_string(),
                config.probes_per_hop.to_string(),
                "-w".to_string(),
                config.timeout.as_secs().to_string(),
            ];

            // Add protocol flag
            match config.protocol {
                TracerouteProtocol::Icmp => args.push("-I".to_string()),
                TracerouteProtocol::Udp => {} // Default
                TracerouteProtocol::Tcp => args.push("-T".to_string()),
            }

            // Add hostname resolution flag
            if !config.resolve_hostnames {
                args.push("-n".to_string());
            }

            args.push(target.to_string());

            Command::new("traceroute").args(&args).output()
        } else if cfg!(target_os = "linux") {
            let mut args = vec![
                "-m".to_string(),
                config.max_hops.to_string(),
                "-q".to_string(),
                config.probes_per_hop.to_string(),
                "-w".to_string(),
                config.timeout.as_secs().to_string(),
            ];

            match config.protocol {
                TracerouteProtocol::Icmp => args.push("-I".to_string()),
                TracerouteProtocol::Udp => args.push("-U".to_string()),
                TracerouteProtocol::Tcp => args.push("-T".to_string()),
            }

            if !config.resolve_hostnames {
                args.push("-n".to_string());
            }

            args.push(target.to_string());

            Command::new("traceroute").args(&args).output()
        } else {
            // Windows uses tracert
            Command::new("tracert")
                .args([
                    "-h",
                    &config.max_hops.to_string(),
                    "-w",
                    &(config.timeout.as_millis() as u64).to_string(),
                    &target.to_string(),
                ])
                .output()
        };

        let output = output.map_err(|e| Error::Traceroute {
            target,
            message: format!("Failed to execute traceroute: {}", e),
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let duration = start.elapsed();

        debug!("Traceroute output:\n{}", stdout);

        // Parse traceroute output
        let hops = self.parse_traceroute_output(&stdout, config);
        let reached = hops
            .last()
            .map(|h| h.address == Some(target))
            .unwrap_or(false);

        Ok(TracerouteResult {
            target,
            target_hostname: None,
            hops,
            reached,
            duration,
            protocol: config.protocol,
        })
    }

    /// Parses traceroute command output.
    fn parse_traceroute_output(
        &self,
        output: &str,
        config: &TracerouteConfig,
    ) -> Vec<TracerouteHop> {
        let mut hops = Vec::new();

        for line in output.lines().skip(1) {
            // Skip header line
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if let Some(hop) = self.parse_hop_line(line, config.probes_per_hop) {
                hops.push(hop);
            }
        }

        hops
    }

    /// Parses a single hop line.
    /// Format: "1  hostname (IP)  1.234 ms  1.567 ms  1.890 ms"
    /// or:     "1  * * *"
    fn parse_hop_line(&self, line: &str, probes_per_hop: u8) -> Option<TracerouteHop> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        // First part is hop number
        let hop_num = parts[0].parse::<u8>().ok()?;

        // Check for all timeouts
        if parts.iter().skip(1).all(|p| *p == "*") {
            return Some(TracerouteHop::timeout(hop_num, probes_per_hop));
        }

        // Parse address and hostname
        let mut address: Option<IpAddr> = None;
        let mut hostname: Option<String> = None;
        let mut probes = Vec::new();

        let mut i = 1;
        while i < parts.len() {
            let part = parts[i];

            // Skip asterisks (timeouts)
            if part == "*" {
                probes.push(TracerouteProbe::timeout());
                i += 1;
                continue;
            }

            // Check for IP in parentheses: (X.X.X.X)
            if part.starts_with('(') && part.ends_with(')') {
                let ip_str = &part[1..part.len() - 1];
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    address = Some(ip);
                }
                i += 1;
                continue;
            }

            // Check for bare IP address
            if let Ok(ip) = part.parse::<IpAddr>() {
                address = Some(ip);
                i += 1;
                continue;
            }

            // Check for RTT (ends with "ms")
            if part.ends_with("ms") || (i + 1 < parts.len() && parts[i + 1] == "ms") {
                let time_str = if part.ends_with("ms") {
                    part.trim_end_matches("ms")
                } else {
                    part
                };

                if let Ok(time_f) = time_str.parse::<f64>() {
                    let rtt = Duration::from_secs_f64(time_f / 1000.0);
                    probes.push(TracerouteProbe::success(
                        rtt,
                        address.unwrap_or_else(|| "0.0.0.0".parse().unwrap()),
                        IcmpResponse::TimeExceeded,
                    ));
                }

                // Skip "ms" if separate
                if !part.ends_with("ms") && i + 1 < parts.len() && parts[i + 1] == "ms" {
                    i += 1;
                }
                i += 1;
                continue;
            }

            // Otherwise it's likely a hostname
            if hostname.is_none() && !part.chars().all(|c| c.is_ascii_digit() || c == '.') {
                hostname = Some(part.to_string());
            }

            i += 1;
        }

        // Calculate average RTT
        let mut hop = TracerouteHop {
            hop: hop_num,
            probes,
            address,
            hostname,
            asn: None,
            as_name: None,
            location: None,
            avg_rtt: None,
            all_timeout: false,
        };

        hop.calculate_stats();

        Some(hop)
    }
}

impl Default for Tracer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_traceroute_localhost() {
        let tracer = Tracer::new();
        let config = TracerouteConfig {
            max_hops: 5,
            probes_per_hop: 1,
            timeout: Duration::from_secs(1),
            protocol: TracerouteProtocol::Icmp,
            resolve_hostnames: false,
        };

        let result = tracer
            .trace("127.0.0.1".parse().unwrap(), &config)
            .await
            .unwrap();

        // Localhost traceroute should be very short
        assert!(result.hops.len() <= 2);
    }

    #[test]
    fn test_parse_hop_line() {
        let tracer = Tracer::new();

        // Test normal hop
        let line = "1  router.local (192.168.1.1)  1.234 ms  1.567 ms  1.890 ms";
        let hop = tracer.parse_hop_line(line, 3).unwrap();
        assert_eq!(hop.hop, 1);
        assert_eq!(hop.address, Some("192.168.1.1".parse().unwrap()));
        assert_eq!(hop.hostname, Some("router.local".to_string()));
        assert!(!hop.all_timeout);

        // Test timeout hop
        let line = "5  * * *";
        let hop = tracer.parse_hop_line(line, 3).unwrap();
        assert_eq!(hop.hop, 5);
        assert!(hop.all_timeout);
    }
}
