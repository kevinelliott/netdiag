//! # netdiag-integrations
//!
//! External API integrations for network diagnostics.
//!
//! Provides clients for:
//! - IPinfo (IP geolocation and ASN lookup)
//! - Shodan (Internet device search)
//! - RIPE/ARIN (Regional Internet Registry data)
//! - SSL Labs (SSL/TLS analysis)
//! - BGP Looking Glass (Route information)

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod ipinfo;
mod shodan;
mod registry;
mod ssllabs;
mod bgp;

pub use error::{IntegrationError, IntegrationResult};
pub use ipinfo::{IpInfoClient, IpInfo, AsnInfo};
pub use shodan::{ShodanClient, ShodanHost, ShodanService};
pub use registry::{RegistryClient, WhoisInfo, RegistryType};
pub use ssllabs::{SslLabsClient, SslAnalysis, SslGrade, SslEndpoint};
pub use bgp::{BgpLookingGlass, RouteInfo, AsPath, BgpPeer};

/// Configuration for external integrations.
#[derive(Debug, Clone, Default)]
pub struct IntegrationConfig {
    /// IPinfo API key (optional, increases rate limits).
    pub ipinfo_api_key: Option<String>,

    /// Shodan API key (required for Shodan lookups).
    pub shodan_api_key: Option<String>,

    /// Custom HTTP timeout in seconds.
    pub timeout_secs: u64,

    /// Enable caching of API responses.
    pub enable_cache: bool,
}

impl IntegrationConfig {
    /// Create a new configuration with default settings.
    pub fn new() -> Self {
        Self {
            timeout_secs: 30,
            enable_cache: true,
            ..Default::default()
        }
    }

    /// Set IPinfo API key.
    pub fn with_ipinfo_key(mut self, key: impl Into<String>) -> Self {
        self.ipinfo_api_key = Some(key.into());
        self
    }

    /// Set Shodan API key.
    pub fn with_shodan_key(mut self, key: impl Into<String>) -> Self {
        self.shodan_api_key = Some(key.into());
        self
    }

    /// Set timeout in seconds.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

/// Manager for all external integrations.
pub struct IntegrationManager {
    config: IntegrationConfig,
    ipinfo: IpInfoClient,
    shodan: Option<ShodanClient>,
    registry: RegistryClient,
    ssllabs: SslLabsClient,
    bgp: BgpLookingGlass,
}

impl IntegrationManager {
    /// Create a new integration manager.
    pub fn new(config: IntegrationConfig) -> Self {
        let ipinfo = IpInfoClient::new(config.ipinfo_api_key.clone());
        let shodan = config.shodan_api_key.as_ref().map(|k| ShodanClient::new(k.clone()));
        let registry = RegistryClient::new();
        let ssllabs = SslLabsClient::new();
        let bgp = BgpLookingGlass::new();

        Self {
            config,
            ipinfo,
            shodan,
            registry,
            ssllabs,
            bgp,
        }
    }

    /// Get IP information.
    pub async fn lookup_ip(&self, ip: &str) -> IntegrationResult<IpInfo> {
        self.ipinfo.lookup(ip).await
    }

    /// Get ASN information.
    pub async fn lookup_asn(&self, asn: u32) -> IntegrationResult<AsnInfo> {
        self.ipinfo.lookup_asn(asn).await
    }

    /// Get Shodan host information.
    pub async fn shodan_host(&self, ip: &str) -> IntegrationResult<ShodanHost> {
        match &self.shodan {
            Some(client) => client.host(ip).await,
            None => Err(IntegrationError::ApiKeyRequired("Shodan")),
        }
    }

    /// Get WHOIS information.
    pub async fn whois(&self, query: &str) -> IntegrationResult<WhoisInfo> {
        self.registry.whois(query).await
    }

    /// Analyze SSL/TLS for a host.
    pub async fn ssl_analyze(&self, host: &str) -> IntegrationResult<SslAnalysis> {
        self.ssllabs.analyze(host).await
    }

    /// Get cached SSL analysis result.
    pub async fn ssl_cached(&self, host: &str) -> IntegrationResult<Option<SslAnalysis>> {
        self.ssllabs.get_cached(host).await
    }

    /// Get BGP route information.
    pub async fn bgp_route(&self, prefix: &str) -> IntegrationResult<RouteInfo> {
        self.bgp.lookup_route(prefix).await
    }

    /// Get configuration.
    pub fn config(&self) -> &IntegrationConfig {
        &self.config
    }
}
