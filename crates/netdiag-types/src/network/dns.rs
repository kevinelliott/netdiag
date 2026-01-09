//! DNS-related types.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;
use strum::{Display, EnumString};

/// DNS server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServer {
    /// Server address
    pub address: IpAddr,
    /// Server port (usually 53)
    pub port: u16,
    /// Server name/hostname (if known)
    pub name: Option<String>,
    /// Provider name (e.g., "Google", "Cloudflare")
    pub provider: Option<String>,
    /// Protocol used
    pub protocol: DnsProtocol,
    /// Is this the primary DNS server?
    pub is_primary: bool,
    /// Source of this DNS configuration
    pub source: DnsSource,
}

impl Default for DnsServer {
    fn default() -> Self {
        Self {
            address: IpAddr::V4(std::net::Ipv4Addr::new(8, 8, 8, 8)),
            port: 53,
            name: Some("Google DNS".to_string()),
            provider: Some("Google".to_string()),
            protocol: DnsProtocol::Udp,
            is_primary: true,
            source: DnsSource::System,
        }
    }
}

/// DNS protocol.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DnsProtocol {
    /// Plain DNS over UDP
    #[default]
    Udp,
    /// Plain DNS over TCP
    Tcp,
    /// DNS over TLS (DoT)
    DoT,
    /// DNS over HTTPS (DoH)
    DoH,
    /// DNS over QUIC (DoQ)
    DoQ,
}

/// Source of DNS configuration.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DnsSource {
    /// System configuration
    #[default]
    System,
    /// DHCP
    Dhcp,
    /// Manual/static configuration
    Static,
    /// VPN
    Vpn,
    /// Router advertisement (IPv6)
    RouterAdvertisement,
}

/// DNS record type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum DnsRecordType {
    /// A record (IPv4 address)
    A,
    /// AAAA record (IPv6 address)
    Aaaa,
    /// CNAME record (canonical name)
    Cname,
    /// MX record (mail exchange)
    Mx,
    /// NS record (name server)
    Ns,
    /// PTR record (pointer/reverse lookup)
    Ptr,
    /// SOA record (start of authority)
    Soa,
    /// SRV record (service)
    Srv,
    /// TXT record (text)
    Txt,
    /// CAA record (certification authority authorization)
    Caa,
}

/// DNS resolution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResolutionResult {
    /// The query hostname
    pub hostname: String,
    /// Query type
    pub query_type: DnsRecordType,
    /// Resolved addresses
    pub addresses: Vec<IpAddr>,
    /// DNS server used
    pub server: DnsServer,
    /// Resolution time
    pub resolution_time: Duration,
    /// TTL (time to live)
    pub ttl: Option<u32>,
    /// Whether the response was from cache
    pub from_cache: bool,
    /// DNSSEC validation status
    pub dnssec_status: DnssecStatus,
}

/// DNSSEC validation status.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum DnssecStatus {
    /// DNSSEC not checked
    #[default]
    NotChecked,
    /// DNSSEC validation passed (secure)
    Secure,
    /// DNSSEC validation failed (bogus)
    Bogus,
    /// Domain not signed (insecure)
    Insecure,
    /// Indeterminate status
    Indeterminate,
}

/// DNS test result for a server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsServerTest {
    /// The DNS server tested
    pub server: DnsServer,
    /// Whether the server is reachable
    pub reachable: bool,
    /// Average response time
    pub avg_response_time: Option<Duration>,
    /// Minimum response time
    pub min_response_time: Option<Duration>,
    /// Maximum response time
    pub max_response_time: Option<Duration>,
    /// Number of successful queries
    pub successful_queries: u32,
    /// Number of failed queries
    pub failed_queries: u32,
    /// Test duration
    pub test_duration: Duration,
}
