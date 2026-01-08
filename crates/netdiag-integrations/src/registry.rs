//! Regional Internet Registry (RIPE, ARIN, etc) client.
//!
//! Provides WHOIS-like information from RIR databases.

use crate::error::{IntegrationError, IntegrationResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

/// Regional Internet Registry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryType {
    /// ARIN (North America).
    Arin,
    /// RIPE NCC (Europe, Middle East, Central Asia).
    Ripe,
    /// APNIC (Asia Pacific).
    Apnic,
    /// LACNIC (Latin America and Caribbean).
    Lacnic,
    /// AFRINIC (Africa).
    Afrinic,
}

impl RegistryType {
    /// Get the API base URL for this registry.
    fn api_url(&self) -> &'static str {
        match self {
            RegistryType::Arin => "https://rdap.arin.net/registry",
            RegistryType::Ripe => "https://rdap.db.ripe.net",
            RegistryType::Apnic => "https://rdap.apnic.net",
            RegistryType::Lacnic => "https://rdap.lacnic.net/rdap",
            RegistryType::Afrinic => "https://rdap.afrinic.net/rdap",
        }
    }

    /// Get display name.
    pub fn name(&self) -> &'static str {
        match self {
            RegistryType::Arin => "ARIN",
            RegistryType::Ripe => "RIPE NCC",
            RegistryType::Apnic => "APNIC",
            RegistryType::Lacnic => "LACNIC",
            RegistryType::Afrinic => "AFRINIC",
        }
    }
}

/// Registry client for WHOIS queries.
pub struct RegistryClient {
    client: Client,
}

impl RegistryClient {
    /// Create a new registry client.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("netdiag/0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Look up WHOIS information for an IP or ASN.
    pub async fn whois(&self, query: &str) -> IntegrationResult<WhoisInfo> {
        // Try each registry until we get a result
        let registries = [
            RegistryType::Arin,
            RegistryType::Ripe,
            RegistryType::Apnic,
            RegistryType::Lacnic,
            RegistryType::Afrinic,
        ];

        let mut last_error = None;

        for registry in &registries {
            match self.query_registry(*registry, query).await {
                Ok(info) => return Ok(info),
                Err(IntegrationError::NotFound(_)) => {
                    // Try next registry
                    last_error = Some(IntegrationError::NotFound(query.to_string()));
                }
                Err(e) => {
                    debug!("Registry {} failed for {}: {}", registry.name(), query, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(IntegrationError::NotFound(query.to_string())))
    }

    /// Query a specific registry.
    pub async fn query_registry(
        &self,
        registry: RegistryType,
        query: &str,
    ) -> IntegrationResult<WhoisInfo> {
        // Determine query type
        let query_type = if query.starts_with("AS") || query.parse::<u32>().is_ok() {
            "autnum"
        } else {
            "ip"
        };

        // Clean up query
        let clean_query = query.trim_start_matches("AS");

        let url = format!("{}/{}/{}", registry.api_url(), query_type, clean_query);
        debug!("Registry RDAP query: {}", url);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/rdap+json")
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let rdap: RdapResponse = response.json().await?;
                Ok(WhoisInfo::from_rdap(rdap, registry))
            }
            404 => Err(IntegrationError::NotFound(query.to_string())),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Query RIPE stat for more detailed information.
    pub async fn ripe_stat(&self, resource: &str) -> IntegrationResult<RipeStatInfo> {
        let url = format!(
            "https://stat.ripe.net/data/whois/data.json?resource={}",
            resource
        );
        debug!("RIPE stat query: {}", url);

        let response = self.client.get(&url).send().await?;

        match response.status().as_u16() {
            200 => {
                let stat: RipeStatResponse = response.json().await?;
                Ok(RipeStatInfo::from(stat))
            }
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

/// RDAP response (simplified).
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RdapResponse {
    #[serde(default)]
    handle: Option<String>,
    #[serde(rename = "startAddress")]
    start_address: Option<String>,
    #[serde(rename = "endAddress")]
    end_address: Option<String>,
    #[serde(rename = "ipVersion")]
    ip_version: Option<String>,
    name: Option<String>,
    #[serde(rename = "type")]
    network_type: Option<String>,
    country: Option<String>,
    #[serde(default)]
    entities: Vec<RdapEntity>,
    #[serde(default)]
    remarks: Vec<RdapRemark>,
    #[serde(default)]
    events: Vec<RdapEvent>,
    #[serde(default)]
    links: Vec<RdapLink>,
    port43: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RdapEntity {
    handle: Option<String>,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(rename = "vcardArray")]
    vcard_array: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RdapRemark {
    title: Option<String>,
    description: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RdapEvent {
    #[serde(rename = "eventAction")]
    event_action: Option<String>,
    #[serde(rename = "eventDate")]
    event_date: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RdapLink {
    rel: Option<String>,
    href: Option<String>,
}

/// WHOIS information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhoisInfo {
    /// Handle/identifier.
    pub handle: Option<String>,

    /// Network name.
    pub name: Option<String>,

    /// Network type.
    pub network_type: Option<String>,

    /// Start address (for IP ranges).
    pub start_address: Option<String>,

    /// End address (for IP ranges).
    pub end_address: Option<String>,

    /// IP version.
    pub ip_version: Option<String>,

    /// Country code.
    pub country: Option<String>,

    /// Registry that provided the information.
    pub registry: RegistryType,

    /// Organization name.
    pub organization: Option<String>,

    /// Abuse contact email.
    pub abuse_email: Option<String>,

    /// Technical contact email.
    pub tech_email: Option<String>,

    /// Registration date.
    pub registration_date: Option<String>,

    /// Last changed date.
    pub last_changed: Option<String>,

    /// Remarks/description.
    pub remarks: Vec<String>,
}

impl WhoisInfo {
    fn from_rdap(rdap: RdapResponse, registry: RegistryType) -> Self {
        // Extract emails from entities
        let mut org_name = None;
        let mut abuse_email = None;
        let mut tech_email = None;

        for entity in &rdap.entities {
            if entity.roles.contains(&"abuse".to_string()) {
                abuse_email = extract_email_from_vcard(&entity.vcard_array);
            }
            if entity.roles.contains(&"technical".to_string()) {
                tech_email = extract_email_from_vcard(&entity.vcard_array);
            }
            if entity.roles.contains(&"registrant".to_string()) {
                org_name = extract_org_from_vcard(&entity.vcard_array);
            }
        }

        // Extract dates from events
        let mut reg_date = None;
        let mut changed_date = None;

        for event in &rdap.events {
            match event.event_action.as_deref() {
                Some("registration") => reg_date = event.event_date.clone(),
                Some("last changed") => changed_date = event.event_date.clone(),
                _ => {}
            }
        }

        // Extract remarks
        let remarks: Vec<String> = rdap
            .remarks
            .iter()
            .filter_map(|r| r.description.as_ref())
            .flatten()
            .cloned()
            .collect();

        Self {
            handle: rdap.handle,
            name: rdap.name,
            network_type: rdap.network_type,
            start_address: rdap.start_address,
            end_address: rdap.end_address,
            ip_version: rdap.ip_version,
            country: rdap.country,
            registry,
            organization: org_name,
            abuse_email,
            tech_email,
            registration_date: reg_date,
            last_changed: changed_date,
            remarks,
        }
    }
}

/// Extract email from vCard array.
fn extract_email_from_vcard(vcard: &Option<serde_json::Value>) -> Option<String> {
    vcard.as_ref().and_then(|v| {
        if let Some(arr) = v.as_array() {
            for item in arr.iter().skip(1) {
                if let Some(props) = item.as_array() {
                    for prop in props {
                        if let Some(prop_arr) = prop.as_array() {
                            if prop_arr.first().and_then(|v| v.as_str()) == Some("email") {
                                return prop_arr.last().and_then(|v| v.as_str()).map(String::from);
                            }
                        }
                    }
                }
            }
        }
        None
    })
}

/// Extract organization from vCard array.
fn extract_org_from_vcard(vcard: &Option<serde_json::Value>) -> Option<String> {
    vcard.as_ref().and_then(|v| {
        if let Some(arr) = v.as_array() {
            for item in arr.iter().skip(1) {
                if let Some(props) = item.as_array() {
                    for prop in props {
                        if let Some(prop_arr) = prop.as_array() {
                            if prop_arr.first().and_then(|v| v.as_str()) == Some("fn") {
                                return prop_arr.last().and_then(|v| v.as_str()).map(String::from);
                            }
                        }
                    }
                }
            }
        }
        None
    })
}

/// RIPE stat response.
#[derive(Debug, Deserialize)]
struct RipeStatResponse {
    data: RipeStatData,
}

#[derive(Debug, Deserialize)]
struct RipeStatData {
    #[serde(default)]
    records: Vec<Vec<RipeStatRecord>>,
}

#[derive(Debug, Deserialize)]
struct RipeStatRecord {
    key: String,
    value: String,
}

/// RIPE stat information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipeStatInfo {
    /// All key-value pairs from WHOIS.
    pub records: Vec<(String, String)>,
}

impl From<RipeStatResponse> for RipeStatInfo {
    fn from(r: RipeStatResponse) -> Self {
        let records: Vec<(String, String)> = r
            .data
            .records
            .into_iter()
            .flatten()
            .map(|rec| (rec.key, rec.value))
            .collect();

        Self { records }
    }
}
