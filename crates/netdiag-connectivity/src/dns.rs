//! DNS resolution module.

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use netdiag_types::error::{Error, Result};
use std::net::IpAddr;
use std::time::{Duration, Instant};
use tracing::debug;

/// DNS resolution result.
#[derive(Debug, Clone)]
pub struct DnsResult {
    /// Query name
    pub query: String,
    /// Resolved addresses
    pub addresses: Vec<IpAddr>,
    /// Resolution time
    pub duration: Duration,
    /// Whether resolution was successful
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl DnsResult {
    /// Creates a successful DNS result.
    pub fn success(query: String, addresses: Vec<IpAddr>, duration: Duration) -> Self {
        Self {
            query,
            addresses,
            duration,
            success: true,
            error: None,
        }
    }

    /// Creates a failed DNS result.
    pub fn failed(query: String, duration: Duration, error: impl Into<String>) -> Self {
        Self {
            query,
            addresses: Vec::new(),
            duration,
            success: false,
            error: Some(error.into()),
        }
    }
}

/// DNS resolver for connectivity testing.
pub struct DnsResolver {
    resolver: TokioAsyncResolver,
}

impl DnsResolver {
    /// Creates a new DNS resolver with system configuration.
    pub fn new() -> Result<Self> {
        let resolver =
            TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        Ok(Self { resolver })
    }

    /// Creates a DNS resolver with specific servers.
    pub fn with_servers(servers: &[IpAddr]) -> Result<Self> {
        use hickory_resolver::config::{NameServerConfig, Protocol};
        use std::net::SocketAddr;

        let name_servers: Vec<NameServerConfig> = servers
            .iter()
            .map(|ip| NameServerConfig::new(SocketAddr::new(*ip, 53), Protocol::Udp))
            .collect();

        let config = ResolverConfig::from_parts(None, Vec::new(), name_servers);
        let resolver = TokioAsyncResolver::tokio(config, ResolverOpts::default());

        Ok(Self { resolver })
    }

    /// Resolves a hostname to IP addresses.
    pub async fn resolve(&self, target: &str) -> Result<DnsResult> {
        let start = Instant::now();

        // Check if target is already an IP address
        if let Ok(ip) = target.parse::<IpAddr>() {
            debug!("Target {} is already an IP address", target);
            return Ok(DnsResult::success(
                target.to_string(),
                vec![ip],
                start.elapsed(),
            ));
        }

        debug!("Resolving DNS for: {}", target);

        match self.resolver.lookup_ip(target).await {
            Ok(lookup) => {
                let addresses: Vec<IpAddr> = lookup.iter().collect();
                let duration = start.elapsed();

                debug!(
                    "Resolved {} to {} addresses in {:?}",
                    target,
                    addresses.len(),
                    duration
                );

                Ok(DnsResult::success(target.to_string(), addresses, duration))
            }
            Err(e) => {
                let _duration = start.elapsed();
                debug!("DNS resolution failed for {}: {}", target, e);

                Err(Error::DnsResolution {
                    host: target.to_string(),
                    message: e.to_string(),
                })
            }
        }
    }

    /// Performs reverse DNS lookup.
    pub async fn reverse_lookup(&self, ip: IpAddr) -> Result<Option<String>> {
        match self.resolver.reverse_lookup(ip).await {
            Ok(lookup) => {
                let names: Vec<_> = lookup.iter().map(|n| n.to_string()).collect();
                Ok(names.first().cloned())
            }
            Err(_) => Ok(None),
        }
    }

    /// Tests DNS resolution to multiple targets.
    pub async fn test_resolution(&self, targets: &[&str]) -> Vec<DnsResult> {
        let mut results = Vec::with_capacity(targets.len());

        for target in targets {
            let result = self.resolve(target).await.unwrap_or_else(|e| {
                DnsResult::failed(target.to_string(), Duration::ZERO, e.to_string())
            });
            results.push(result);
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_resolve_ip() {
        let resolver = DnsResolver::new().unwrap();
        let result = resolver.resolve("8.8.8.8").await.unwrap();

        assert!(result.success);
        assert_eq!(result.addresses.len(), 1);
        assert_eq!(result.addresses[0], "8.8.8.8".parse::<IpAddr>().unwrap());
    }

    #[tokio::test]
    async fn test_resolve_hostname() {
        let resolver = DnsResolver::new().unwrap();
        let result = resolver.resolve("google.com").await.unwrap();

        assert!(result.success);
        assert!(!result.addresses.is_empty());
    }
}
