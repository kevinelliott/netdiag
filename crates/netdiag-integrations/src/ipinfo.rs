//! IPinfo API client.
//!
//! Provides IP geolocation and ASN information.

use crate::error::{IntegrationError, IntegrationResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;
use tracing::debug;

const IPINFO_BASE_URL: &str = "https://ipinfo.io";

/// IPinfo API client.
pub struct IpInfoClient {
    client: Client,
    api_key: Option<String>,
}

impl IpInfoClient {
    /// Create a new IPinfo client.
    pub fn new(api_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("netdiag/0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, api_key }
    }

    /// Look up IP information.
    pub async fn lookup(&self, ip: &str) -> IntegrationResult<IpInfo> {
        // Validate IP
        let _: IpAddr = ip.parse().map_err(|_| {
            IntegrationError::InvalidInput(format!("Invalid IP address: {}", ip))
        })?;

        let url = format!("{}/{}/json", IPINFO_BASE_URL, ip);
        debug!("IPinfo lookup: {}", url);

        let mut request = self.client.get(&url);
        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await?;

        match response.status().as_u16() {
            200 => {
                let info: IpInfoResponse = response.json().await?;
                Ok(IpInfo::from(info))
            }
            401 => Err(IntegrationError::InvalidApiKey("IPinfo")),
            404 => Err(IntegrationError::NotFound(format!("IP not found: {}", ip))),
            429 => Err(IntegrationError::RateLimited("IPinfo")),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Look up ASN information.
    pub async fn lookup_asn(&self, asn: u32) -> IntegrationResult<AsnInfo> {
        let url = format!("{}/AS{}/json", IPINFO_BASE_URL, asn);
        debug!("IPinfo ASN lookup: {}", url);

        let mut request = self.client.get(&url);
        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await?;

        match response.status().as_u16() {
            200 => {
                let info: AsnInfoResponse = response.json().await?;
                Ok(AsnInfo::from(info))
            }
            401 => Err(IntegrationError::InvalidApiKey("IPinfo")),
            404 => Err(IntegrationError::NotFound(format!("ASN not found: AS{}", asn))),
            429 => Err(IntegrationError::RateLimited("IPinfo")),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Get information for the current IP.
    pub async fn my_ip(&self) -> IntegrationResult<IpInfo> {
        let url = format!("{}/json", IPINFO_BASE_URL);
        debug!("IPinfo my IP lookup: {}", url);

        let mut request = self.client.get(&url);
        if let Some(ref key) = self.api_key {
            request = request.bearer_auth(key);
        }

        let response = request.send().await?;
        let info: IpInfoResponse = response.json().await?;
        Ok(IpInfo::from(info))
    }
}

/// IPinfo API response.
#[derive(Debug, Deserialize)]
struct IpInfoResponse {
    ip: String,
    hostname: Option<String>,
    city: Option<String>,
    region: Option<String>,
    country: Option<String>,
    loc: Option<String>,
    org: Option<String>,
    postal: Option<String>,
    timezone: Option<String>,
    asn: Option<IpInfoAsn>,
    company: Option<IpInfoCompany>,
    carrier: Option<IpInfoCarrier>,
    privacy: Option<IpInfoPrivacy>,
}

#[derive(Debug, Deserialize)]
struct IpInfoAsn {
    asn: String,
    name: String,
    domain: Option<String>,
    route: Option<String>,
    #[serde(rename = "type")]
    asn_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IpInfoCompany {
    name: String,
    domain: Option<String>,
    #[serde(rename = "type")]
    company_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IpInfoCarrier {
    name: String,
    mcc: Option<String>,
    mnc: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IpInfoPrivacy {
    vpn: bool,
    proxy: bool,
    tor: bool,
    relay: bool,
    hosting: bool,
}

/// IP information result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfo {
    /// IP address.
    pub ip: String,

    /// Hostname.
    pub hostname: Option<String>,

    /// City.
    pub city: Option<String>,

    /// Region/state.
    pub region: Option<String>,

    /// Country code.
    pub country: Option<String>,

    /// Latitude.
    pub latitude: Option<f64>,

    /// Longitude.
    pub longitude: Option<f64>,

    /// Organization (typically ISP or company).
    pub organization: Option<String>,

    /// Postal code.
    pub postal: Option<String>,

    /// Timezone.
    pub timezone: Option<String>,

    /// ASN number.
    pub asn: Option<u32>,

    /// ASN name.
    pub asn_name: Option<String>,

    /// Is this a VPN?
    pub is_vpn: bool,

    /// Is this a proxy?
    pub is_proxy: bool,

    /// Is this a Tor exit node?
    pub is_tor: bool,

    /// Is this a hosting provider?
    pub is_hosting: bool,
}

impl From<IpInfoResponse> for IpInfo {
    fn from(r: IpInfoResponse) -> Self {
        let (lat, lon) = r.loc.as_ref().and_then(|loc| {
            let parts: Vec<&str> = loc.split(',').collect();
            if parts.len() == 2 {
                Some((
                    parts[0].parse().ok(),
                    parts[1].parse().ok(),
                ))
            } else {
                None
            }
        }).unwrap_or((None, None));

        let asn = r.asn.as_ref().and_then(|a| {
            a.asn.trim_start_matches("AS").parse().ok()
        });

        Self {
            ip: r.ip,
            hostname: r.hostname,
            city: r.city,
            region: r.region,
            country: r.country,
            latitude: lat,
            longitude: lon,
            organization: r.org,
            postal: r.postal,
            timezone: r.timezone,
            asn,
            asn_name: r.asn.map(|a| a.name),
            is_vpn: r.privacy.as_ref().map_or(false, |p| p.vpn),
            is_proxy: r.privacy.as_ref().map_or(false, |p| p.proxy),
            is_tor: r.privacy.as_ref().map_or(false, |p| p.tor),
            is_hosting: r.privacy.as_ref().map_or(false, |p| p.hosting),
        }
    }
}

/// ASN API response.
#[derive(Debug, Deserialize)]
struct AsnInfoResponse {
    asn: String,
    name: String,
    country: Option<String>,
    allocated: Option<String>,
    registry: Option<String>,
    domain: Option<String>,
    num_ips: Option<u64>,
    #[serde(rename = "type")]
    asn_type: Option<String>,
    prefixes: Option<Vec<AsnPrefix>>,
    prefixes6: Option<Vec<AsnPrefix>>,
    peers: Option<Vec<String>>,
    upstreams: Option<Vec<String>>,
    downstreams: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AsnPrefix {
    netblock: String,
    id: String,
    name: String,
    country: Option<String>,
}

/// ASN information result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsnInfo {
    /// ASN number.
    pub asn: u32,

    /// ASN name.
    pub name: String,

    /// Country.
    pub country: Option<String>,

    /// Allocation date.
    pub allocated: Option<String>,

    /// Registry (ARIN, RIPE, etc).
    pub registry: Option<String>,

    /// Domain.
    pub domain: Option<String>,

    /// Number of IPs.
    pub num_ips: Option<u64>,

    /// ASN type (isp, business, education, etc).
    pub asn_type: Option<String>,

    /// IPv4 prefixes.
    pub prefixes: Vec<String>,

    /// IPv6 prefixes.
    pub prefixes6: Vec<String>,

    /// Peer ASNs.
    pub peers: Vec<String>,

    /// Upstream ASNs.
    pub upstreams: Vec<String>,

    /// Downstream ASNs.
    pub downstreams: Vec<String>,
}

impl From<AsnInfoResponse> for AsnInfo {
    fn from(r: AsnInfoResponse) -> Self {
        Self {
            asn: r.asn.trim_start_matches("AS").parse().unwrap_or(0),
            name: r.name,
            country: r.country,
            allocated: r.allocated,
            registry: r.registry,
            domain: r.domain,
            num_ips: r.num_ips,
            asn_type: r.asn_type,
            prefixes: r.prefixes.unwrap_or_default().into_iter().map(|p| p.netblock).collect(),
            prefixes6: r.prefixes6.unwrap_or_default().into_iter().map(|p| p.netblock).collect(),
            peers: r.peers.unwrap_or_default(),
            upstreams: r.upstreams.unwrap_or_default(),
            downstreams: r.downstreams.unwrap_or_default(),
        }
    }
}
