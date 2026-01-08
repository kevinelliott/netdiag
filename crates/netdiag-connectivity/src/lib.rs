//! # netdiag-connectivity
//!
//! Connectivity testing module for netdiag.
//!
//! Provides ping, traceroute, and jitter testing capabilities.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod dns;
mod ping;
mod traceroute;

pub use dns::{DnsResolver, DnsResult};
pub use ping::{Pinger, PingConfig};
pub use traceroute::{Tracer, TracerouteConfig};

use netdiag_types::error::Result;
use std::net::IpAddr;
use std::time::Duration;

/// Default ping count
pub const DEFAULT_PING_COUNT: u32 = 4;

/// Default ping timeout in seconds
pub const DEFAULT_PING_TIMEOUT_SECS: u64 = 2;

/// Default traceroute max hops
pub const DEFAULT_MAX_HOPS: u8 = 30;

/// Default traceroute probes per hop
pub const DEFAULT_PROBES_PER_HOP: u8 = 3;

/// Connectivity test result
#[derive(Debug, Clone)]
pub struct ConnectivityResult {
    /// Target that was tested
    pub target: String,
    /// Resolved IP address
    pub resolved_ip: Option<IpAddr>,
    /// DNS resolution time
    pub dns_time: Option<Duration>,
    /// Whether the target is reachable
    pub reachable: bool,
    /// Ping statistics (if ping was performed)
    pub ping_stats: Option<netdiag_types::diagnostics::PingStats>,
    /// Traceroute result (if traceroute was performed)
    pub traceroute: Option<netdiag_types::diagnostics::TracerouteResult>,
    /// Error message if test failed
    pub error: Option<String>,
}

impl ConnectivityResult {
    /// Creates a successful connectivity result.
    pub fn success(target: String, resolved_ip: IpAddr, dns_time: Duration) -> Self {
        Self {
            target,
            resolved_ip: Some(resolved_ip),
            dns_time: Some(dns_time),
            reachable: true,
            ping_stats: None,
            traceroute: None,
            error: None,
        }
    }

    /// Creates a failed connectivity result.
    pub fn failed(target: String, error: impl Into<String>) -> Self {
        Self {
            target,
            resolved_ip: None,
            dns_time: None,
            reachable: false,
            ping_stats: None,
            traceroute: None,
            error: Some(error.into()),
        }
    }
}

/// Quick connectivity check to a target.
pub async fn check_connectivity(target: &str) -> Result<ConnectivityResult> {
    let resolver = DnsResolver::new()?;
    let pinger = Pinger::new();

    // First, resolve DNS
    let dns_result = resolver.resolve(target).await;

    match dns_result {
        Ok(result) => {
            if result.addresses.is_empty() {
                return Ok(ConnectivityResult::failed(
                    target.to_string(),
                    "DNS resolution returned no addresses",
                ));
            }

            let ip = result.addresses[0];
            let dns_time = result.duration;

            // Then ping
            let config = PingConfig {
                count: 3,
                timeout: Duration::from_secs(2),
                interval: Duration::from_millis(500),
                size: 64,
            };

            let ping_stats = pinger.ping(ip, &config).await?;
            let reachable = ping_stats.received > 0;

            Ok(ConnectivityResult {
                target: target.to_string(),
                resolved_ip: Some(ip),
                dns_time: Some(dns_time),
                reachable,
                ping_stats: Some(ping_stats),
                traceroute: None,
                error: None,
            })
        }
        Err(e) => Ok(ConnectivityResult::failed(target.to_string(), e.to_string())),
    }
}

/// Full connectivity diagnosis including traceroute.
pub async fn diagnose_connectivity(target: &str) -> Result<ConnectivityResult> {
    let resolver = DnsResolver::new()?;
    let pinger = Pinger::new();
    let tracer = Tracer::new();

    // Resolve DNS
    let dns_result = resolver.resolve(target).await;

    match dns_result {
        Ok(result) => {
            if result.addresses.is_empty() {
                return Ok(ConnectivityResult::failed(
                    target.to_string(),
                    "DNS resolution returned no addresses",
                ));
            }

            let ip = result.addresses[0];
            let dns_time = result.duration;

            // Ping
            let ping_config = PingConfig::default();
            let ping_stats = pinger.ping(ip, &ping_config).await?;

            // Traceroute
            let trace_config = TracerouteConfig::default();
            let traceroute = tracer.trace(ip, &trace_config).await?;

            let reachable = ping_stats.received > 0;

            Ok(ConnectivityResult {
                target: target.to_string(),
                resolved_ip: Some(ip),
                dns_time: Some(dns_time),
                reachable,
                ping_stats: Some(ping_stats),
                traceroute: Some(traceroute),
                error: None,
            })
        }
        Err(e) => Ok(ConnectivityResult::failed(target.to_string(), e.to_string())),
    }
}
