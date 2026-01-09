//! Shodan API client.
//!
//! Provides access to Shodan's internet device search engine.

use crate::error::{IntegrationError, IntegrationResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;
use tracing::debug;

const SHODAN_BASE_URL: &str = "https://api.shodan.io";

/// Shodan API client.
pub struct ShodanClient {
    client: Client,
    api_key: String,
}

impl ShodanClient {
    /// Create a new Shodan client.
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("netdiag/0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            api_key: api_key.into(),
        }
    }

    /// Get host information.
    pub async fn host(&self, ip: &str) -> IntegrationResult<ShodanHost> {
        // Validate IP
        let _: IpAddr = ip
            .parse()
            .map_err(|_| IntegrationError::InvalidInput(format!("Invalid IP address: {}", ip)))?;

        let url = format!("{}/shodan/host/{}", SHODAN_BASE_URL, ip);
        debug!("Shodan host lookup: {}", url);

        let response = self
            .client
            .get(&url)
            .query(&[("key", &self.api_key)])
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let host: ShodanHostResponse = response.json().await?;
                Ok(ShodanHost::from(host))
            }
            401 => Err(IntegrationError::InvalidApiKey("Shodan")),
            404 => Err(IntegrationError::NotFound(format!(
                "Host not found: {}",
                ip
            ))),
            429 => Err(IntegrationError::RateLimited("Shodan")),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Search for hosts.
    pub async fn search(&self, query: &str) -> IntegrationResult<Vec<ShodanHost>> {
        let url = format!("{}/shodan/host/search", SHODAN_BASE_URL);
        debug!("Shodan search: {} - query: {}", url, query);

        let response = self
            .client
            .get(&url)
            .query(&[("key", &self.api_key), ("query", &query.to_string())])
            .send()
            .await?;

        match response.status().as_u16() {
            200 => {
                let result: ShodanSearchResponse = response.json().await?;
                Ok(result.matches.into_iter().map(ShodanHost::from).collect())
            }
            401 => Err(IntegrationError::InvalidApiKey("Shodan")),
            429 => Err(IntegrationError::RateLimited("Shodan")),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    /// Get API info (credits, usage, etc).
    pub async fn info(&self) -> IntegrationResult<ShodanApiInfo> {
        let url = format!("{}/api-info", SHODAN_BASE_URL);
        debug!("Shodan API info: {}", url);

        let response = self
            .client
            .get(&url)
            .query(&[("key", &self.api_key)])
            .send()
            .await?;

        match response.status().as_u16() {
            200 => Ok(response.json().await?),
            401 => Err(IntegrationError::InvalidApiKey("Shodan")),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }
}

/// Shodan host response.
#[derive(Debug, Deserialize)]
struct ShodanHostResponse {
    ip_str: String,
    hostnames: Option<Vec<String>>,
    domains: Option<Vec<String>>,
    country_code: Option<String>,
    country_name: Option<String>,
    city: Option<String>,
    region_code: Option<String>,
    postal_code: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    asn: Option<String>,
    isp: Option<String>,
    org: Option<String>,
    os: Option<String>,
    ports: Option<Vec<u16>>,
    vulns: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    data: Option<Vec<ShodanServiceResponse>>,
    last_update: Option<String>,
}

/// Shodan service response.
#[derive(Debug, Deserialize)]
struct ShodanServiceResponse {
    port: u16,
    transport: Option<String>,
    product: Option<String>,
    version: Option<String>,
    #[serde(rename = "ssl")]
    ssl_info: Option<ShodanSslInfo>,
    http: Option<ShodanHttpInfo>,
    data: Option<String>,
    timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ShodanSslInfo {
    cert: Option<ShodanCertInfo>,
    cipher: Option<ShodanCipherInfo>,
    versions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ShodanCertInfo {
    subject: Option<ShodanCertSubject>,
    issuer: Option<ShodanCertSubject>,
    expires: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ShodanCertSubject {
    #[serde(rename = "CN")]
    cn: Option<String>,
    #[serde(rename = "O")]
    organization: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ShodanCipherInfo {
    name: Option<String>,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ShodanHttpInfo {
    host: Option<String>,
    title: Option<String>,
    server: Option<String>,
    status: Option<u16>,
}

/// Shodan search response.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ShodanSearchResponse {
    matches: Vec<ShodanHostResponse>,
    total: u64,
}

/// Shodan host information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShodanHost {
    /// IP address.
    pub ip: String,

    /// Hostnames.
    pub hostnames: Vec<String>,

    /// Domains.
    pub domains: Vec<String>,

    /// Country code.
    pub country_code: Option<String>,

    /// Country name.
    pub country_name: Option<String>,

    /// City.
    pub city: Option<String>,

    /// Region code.
    pub region_code: Option<String>,

    /// Postal code.
    pub postal_code: Option<String>,

    /// Latitude.
    pub latitude: Option<f64>,

    /// Longitude.
    pub longitude: Option<f64>,

    /// ASN.
    pub asn: Option<String>,

    /// ISP.
    pub isp: Option<String>,

    /// Organization.
    pub organization: Option<String>,

    /// Operating system.
    pub os: Option<String>,

    /// Open ports.
    pub ports: Vec<u16>,

    /// Known vulnerabilities (CVEs).
    pub vulnerabilities: Vec<String>,

    /// Tags.
    pub tags: Vec<String>,

    /// Services.
    pub services: Vec<ShodanService>,

    /// Last update.
    pub last_update: Option<String>,
}

impl From<ShodanHostResponse> for ShodanHost {
    fn from(r: ShodanHostResponse) -> Self {
        Self {
            ip: r.ip_str,
            hostnames: r.hostnames.unwrap_or_default(),
            domains: r.domains.unwrap_or_default(),
            country_code: r.country_code,
            country_name: r.country_name,
            city: r.city,
            region_code: r.region_code,
            postal_code: r.postal_code,
            latitude: r.latitude,
            longitude: r.longitude,
            asn: r.asn,
            isp: r.isp,
            organization: r.org,
            os: r.os,
            ports: r.ports.unwrap_or_default(),
            vulnerabilities: r.vulns.unwrap_or_default(),
            tags: r.tags.unwrap_or_default(),
            services: r
                .data
                .unwrap_or_default()
                .into_iter()
                .map(ShodanService::from)
                .collect(),
            last_update: r.last_update,
        }
    }
}

/// Shodan service information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShodanService {
    /// Port number.
    pub port: u16,

    /// Transport protocol (tcp/udp).
    pub transport: Option<String>,

    /// Product name.
    pub product: Option<String>,

    /// Product version.
    pub version: Option<String>,

    /// SSL/TLS info.
    pub ssl_versions: Vec<String>,

    /// SSL certificate CN.
    pub ssl_cert_cn: Option<String>,

    /// HTTP title.
    pub http_title: Option<String>,

    /// HTTP server.
    pub http_server: Option<String>,

    /// HTTP status code.
    pub http_status: Option<u16>,

    /// Banner data (first 200 chars).
    pub banner: Option<String>,

    /// Timestamp.
    pub timestamp: Option<String>,
}

impl From<ShodanServiceResponse> for ShodanService {
    fn from(r: ShodanServiceResponse) -> Self {
        Self {
            port: r.port,
            transport: r.transport,
            product: r.product,
            version: r.version,
            ssl_versions: r
                .ssl_info
                .as_ref()
                .and_then(|s| s.versions.clone())
                .unwrap_or_default(),
            ssl_cert_cn: r
                .ssl_info
                .as_ref()
                .and_then(|s| s.cert.as_ref())
                .and_then(|c| c.subject.as_ref())
                .and_then(|s| s.cn.clone()),
            http_title: r.http.as_ref().and_then(|h| h.title.clone()),
            http_server: r.http.as_ref().and_then(|h| h.server.clone()),
            http_status: r.http.as_ref().and_then(|h| h.status),
            banner: r.data.map(|d| {
                if d.len() > 200 {
                    format!("{}...", &d[..200])
                } else {
                    d
                }
            }),
            timestamp: r.timestamp,
        }
    }
}

/// Shodan API info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShodanApiInfo {
    /// Query credits remaining.
    pub query_credits: u64,

    /// Scan credits remaining.
    pub scan_credits: u64,

    /// Whether this is a paid plan.
    pub plan: String,

    /// Usage limits.
    pub usage_limits: ShodanUsageLimits,
}

/// Shodan usage limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShodanUsageLimits {
    /// Scan credits limit.
    pub scan_credits: u64,

    /// Query credits limit.
    pub query_credits: u64,

    /// Monitored IPs limit.
    pub monitored_ips: u64,
}
