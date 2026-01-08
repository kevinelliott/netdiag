//! BGP Looking Glass client.
//!
//! Provides BGP route information from public looking glasses.

use crate::error::{IntegrationError, IntegrationResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

const BGPVIEW_API_URL: &str = "https://api.bgpview.io";
const RIPESTAT_API_URL: &str = "https://stat.ripe.net/data";

/// BGP Looking Glass client.
pub struct BgpLookingGlass {
    client: Client,
}

impl BgpLookingGlass {
    /// Create a new BGP looking glass client.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("netdiag/0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Look up route information for a prefix.
    pub async fn lookup_route(&self, prefix: &str) -> IntegrationResult<RouteInfo> {
        let url = format!("{}/prefix/{}", BGPVIEW_API_URL, prefix);
        debug!("BGP route lookup: {}", url);

        let response = self.client.get(&url).send().await?;

        match response.status().as_u16() {
            200 => {
                let result: BgpViewResponse<BgpViewPrefix> = response.json().await?;
                if result.status == "ok" {
                    Ok(RouteInfo::from_bgpview(result.data))
                } else {
                    Err(IntegrationError::NotFound(prefix.to_string()))
                }
            }
            404 => Err(IntegrationError::NotFound(prefix.to_string())),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Look up ASN information.
    pub async fn lookup_asn(&self, asn: u32) -> IntegrationResult<BgpAsn> {
        let url = format!("{}/asn/{}", BGPVIEW_API_URL, asn);
        debug!("BGP ASN lookup: {}", url);

        let response = self.client.get(&url).send().await?;

        match response.status().as_u16() {
            200 => {
                let result: BgpViewResponse<BgpViewAsn> = response.json().await?;
                if result.status == "ok" {
                    Ok(BgpAsn::from(result.data))
                } else {
                    Err(IntegrationError::NotFound(format!("AS{}", asn)))
                }
            }
            404 => Err(IntegrationError::NotFound(format!("AS{}", asn))),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Get prefixes announced by an ASN.
    pub async fn asn_prefixes(&self, asn: u32) -> IntegrationResult<Vec<String>> {
        let url = format!("{}/asn/{}/prefixes", BGPVIEW_API_URL, asn);
        debug!("BGP ASN prefixes: {}", url);

        let response = self.client.get(&url).send().await?;

        match response.status().as_u16() {
            200 => {
                let result: BgpViewResponse<BgpViewAsnPrefixes> = response.json().await?;
                if result.status == "ok" {
                    let mut prefixes: Vec<String> = result
                        .data
                        .ipv4_prefixes
                        .into_iter()
                        .map(|p| p.prefix)
                        .collect();
                    prefixes.extend(result.data.ipv6_prefixes.into_iter().map(|p| p.prefix));
                    Ok(prefixes)
                } else {
                    Err(IntegrationError::NotFound(format!("AS{}", asn)))
                }
            }
            404 => Err(IntegrationError::NotFound(format!("AS{}", asn))),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Get peers of an ASN.
    pub async fn asn_peers(&self, asn: u32) -> IntegrationResult<Vec<BgpPeer>> {
        let url = format!("{}/asn/{}/peers", BGPVIEW_API_URL, asn);
        debug!("BGP ASN peers: {}", url);

        let response = self.client.get(&url).send().await?;

        match response.status().as_u16() {
            200 => {
                let result: BgpViewResponse<BgpViewAsnPeers> = response.json().await?;
                if result.status == "ok" {
                    let mut peers: Vec<BgpPeer> = result
                        .data
                        .ipv4_peers
                        .into_iter()
                        .map(BgpPeer::from)
                        .collect();
                    peers.extend(result.data.ipv6_peers.into_iter().map(BgpPeer::from));
                    Ok(peers)
                } else {
                    Err(IntegrationError::NotFound(format!("AS{}", asn)))
                }
            }
            404 => Err(IntegrationError::NotFound(format!("AS{}", asn))),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Get AS path for a prefix using RIPE stat.
    pub async fn as_path(&self, prefix: &str) -> IntegrationResult<Vec<AsPath>> {
        let url = format!("{}/looking-glass/data.json", RIPESTAT_API_URL);
        debug!("RIPE stat looking glass: {} - prefix: {}", url, prefix);

        let response = self
            .client
            .get(&url)
            .query(&[("resource", prefix)])
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let result: RipeStatLookingGlass = response.json().await?;
                let paths: Vec<AsPath> = result
                    .data
                    .rrcs
                    .into_iter()
                    .flat_map(|rrc| rrc.peers.into_iter().map(move |peer| AsPath {
                        collector: rrc.rrc.clone(),
                        peer_asn: peer.asn_origin,
                        as_path: peer.as_path.split(' ').map(|s| s.to_string()).collect(),
                        prefix: peer.prefix,
                        origin: peer.origin,
                    }))
                    .collect();
                Ok(paths)
            }
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }
}

impl Default for BgpLookingGlass {
    fn default() -> Self {
        Self::new()
    }
}

/// BGPView API response wrapper.
#[derive(Debug, Deserialize)]
struct BgpViewResponse<T> {
    status: String,
    #[serde(default)]
    data: T,
}

/// BGPView prefix data.
#[derive(Debug, Default, Deserialize)]
struct BgpViewPrefix {
    prefix: String,
    ip: String,
    cidr: u8,
    #[serde(default)]
    asns: Vec<BgpViewAsnShort>,
    name: Option<String>,
    description_short: Option<String>,
    country_code: Option<String>,
    parent: Option<BgpViewParent>,
    rir_allocation: Option<BgpViewRirAllocation>,
    date_updated: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BgpViewAsnShort {
    asn: u32,
    name: Option<String>,
    description: Option<String>,
    country_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BgpViewParent {
    prefix: Option<String>,
    name: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BgpViewRirAllocation {
    rir_name: Option<String>,
    country_code: Option<String>,
    date_allocated: Option<String>,
    allocation_status: Option<String>,
}

/// BGPView ASN data.
#[derive(Debug, Default, Deserialize)]
struct BgpViewAsn {
    asn: u32,
    name: Option<String>,
    description_short: Option<String>,
    description_full: Option<Vec<String>>,
    country_code: Option<String>,
    website: Option<String>,
    email_contacts: Option<Vec<String>>,
    abuse_contacts: Option<Vec<String>>,
    looking_glass: Option<String>,
    traffic_estimation: Option<String>,
    traffic_ratio: Option<String>,
    owner_address: Option<Vec<String>>,
    rir_allocation: Option<BgpViewAsnRir>,
    date_updated: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BgpViewAsnRir {
    rir_name: Option<String>,
    country_code: Option<String>,
    date_allocated: Option<String>,
}

/// BGPView ASN prefixes.
#[derive(Debug, Default, Deserialize)]
struct BgpViewAsnPrefixes {
    #[serde(default)]
    ipv4_prefixes: Vec<BgpViewPrefixShort>,
    #[serde(default)]
    ipv6_prefixes: Vec<BgpViewPrefixShort>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BgpViewPrefixShort {
    prefix: String,
    name: Option<String>,
    description: Option<String>,
    country_code: Option<String>,
}

/// BGPView ASN peers.
#[derive(Debug, Default, Deserialize)]
struct BgpViewAsnPeers {
    #[serde(default)]
    ipv4_peers: Vec<BgpViewPeer>,
    #[serde(default)]
    ipv6_peers: Vec<BgpViewPeer>,
}

#[derive(Debug, Deserialize)]
struct BgpViewPeer {
    asn: u32,
    name: Option<String>,
    description: Option<String>,
    country_code: Option<String>,
}

/// RIPE stat looking glass response.
#[derive(Debug, Deserialize)]
struct RipeStatLookingGlass {
    data: RipeStatLookingGlassData,
}

#[derive(Debug, Deserialize)]
struct RipeStatLookingGlassData {
    #[serde(default)]
    rrcs: Vec<RipeStatRrc>,
}

#[derive(Debug, Deserialize)]
struct RipeStatRrc {
    rrc: String,
    #[serde(default)]
    peers: Vec<RipeStatPeer>,
}

#[derive(Debug, Deserialize)]
struct RipeStatPeer {
    asn_origin: u32,
    as_path: String,
    prefix: String,
    origin: Option<String>,
}

/// Route information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteInfo {
    /// IP prefix.
    pub prefix: String,

    /// Network address.
    pub ip: String,

    /// CIDR mask length.
    pub cidr: u8,

    /// Origin ASNs.
    pub origin_asns: Vec<u32>,

    /// Origin ASN names.
    pub origin_names: Vec<String>,

    /// Country code.
    pub country_code: Option<String>,

    /// Network name.
    pub name: Option<String>,

    /// Description.
    pub description: Option<String>,

    /// Parent prefix.
    pub parent_prefix: Option<String>,

    /// RIR name (ARIN, RIPE, etc).
    pub rir_name: Option<String>,

    /// Allocation date.
    pub allocation_date: Option<String>,

    /// Last updated.
    pub date_updated: Option<String>,
}

impl RouteInfo {
    fn from_bgpview(p: BgpViewPrefix) -> Self {
        Self {
            prefix: p.prefix,
            ip: p.ip,
            cidr: p.cidr,
            origin_asns: p.asns.iter().map(|a| a.asn).collect(),
            origin_names: p
                .asns
                .iter()
                .filter_map(|a| a.name.clone())
                .collect(),
            country_code: p.country_code,
            name: p.name,
            description: p.description_short,
            parent_prefix: p.parent.and_then(|p| p.prefix),
            rir_name: p.rir_allocation.as_ref().and_then(|r| r.rir_name.clone()),
            allocation_date: p.rir_allocation.as_ref().and_then(|r| r.date_allocated.clone()),
            date_updated: p.date_updated,
        }
    }
}

/// BGP ASN information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgpAsn {
    /// ASN number.
    pub asn: u32,

    /// Name.
    pub name: Option<String>,

    /// Short description.
    pub description: Option<String>,

    /// Full description.
    pub description_full: Vec<String>,

    /// Country code.
    pub country_code: Option<String>,

    /// Website.
    pub website: Option<String>,

    /// Email contacts.
    pub email_contacts: Vec<String>,

    /// Abuse contacts.
    pub abuse_contacts: Vec<String>,

    /// Looking glass URL.
    pub looking_glass: Option<String>,

    /// Traffic estimation.
    pub traffic_estimation: Option<String>,

    /// Traffic ratio.
    pub traffic_ratio: Option<String>,

    /// Owner address.
    pub owner_address: Vec<String>,

    /// RIR name.
    pub rir_name: Option<String>,

    /// Date updated.
    pub date_updated: Option<String>,
}

impl From<BgpViewAsn> for BgpAsn {
    fn from(a: BgpViewAsn) -> Self {
        Self {
            asn: a.asn,
            name: a.name,
            description: a.description_short,
            description_full: a.description_full.unwrap_or_default(),
            country_code: a.country_code,
            website: a.website,
            email_contacts: a.email_contacts.unwrap_or_default(),
            abuse_contacts: a.abuse_contacts.unwrap_or_default(),
            looking_glass: a.looking_glass,
            traffic_estimation: a.traffic_estimation,
            traffic_ratio: a.traffic_ratio,
            owner_address: a.owner_address.unwrap_or_default(),
            rir_name: a.rir_allocation.and_then(|r| r.rir_name),
            date_updated: a.date_updated,
        }
    }
}

/// BGP peer information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgpPeer {
    /// Peer ASN.
    pub asn: u32,

    /// Peer name.
    pub name: Option<String>,

    /// Peer description.
    pub description: Option<String>,

    /// Peer country.
    pub country_code: Option<String>,
}

impl From<BgpViewPeer> for BgpPeer {
    fn from(p: BgpViewPeer) -> Self {
        Self {
            asn: p.asn,
            name: p.name,
            description: p.description,
            country_code: p.country_code,
        }
    }
}

/// AS path information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsPath {
    /// Route collector.
    pub collector: String,

    /// Peer ASN that reported this path.
    pub peer_asn: u32,

    /// AS path as list of ASNs.
    pub as_path: Vec<String>,

    /// Prefix.
    pub prefix: String,

    /// Origin.
    pub origin: Option<String>,
}
