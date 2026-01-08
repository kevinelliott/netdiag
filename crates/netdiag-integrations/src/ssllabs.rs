//! SSL Labs API client.
//!
//! Provides SSL/TLS analysis using Qualys SSL Labs.

use crate::error::{IntegrationError, IntegrationResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::debug;

const SSLLABS_API_URL: &str = "https://api.ssllabs.com/api/v3";

/// SSL Labs API client.
pub struct SslLabsClient {
    client: Client,
}

impl SslLabsClient {
    /// Create a new SSL Labs client.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent("netdiag/0.1")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    /// Analyze a host's SSL/TLS configuration.
    ///
    /// This will start a new analysis if one is not already running.
    pub async fn analyze(&self, host: &str) -> IntegrationResult<SslAnalysis> {
        let url = format!("{}/analyze", SSLLABS_API_URL);
        debug!("SSL Labs analyze: {} - host: {}", url, host);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("host", host),
                ("all", "done"),
                ("startNew", "on"),
            ])
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get cached analysis result without starting new analysis.
    pub async fn get_cached(&self, host: &str) -> IntegrationResult<Option<SslAnalysis>> {
        let url = format!("{}/analyze", SSLLABS_API_URL);
        debug!("SSL Labs get cached: {} - host: {}", url, host);

        let response = self
            .client
            .get(&url)
            .query(&[
                ("host", host),
                ("all", "done"),
                ("fromCache", "on"),
                ("maxAge", "24"), // Max 24 hours old
            ])
            .send()
            .await?;

        match self.handle_response(response).await {
            Ok(analysis) => Ok(Some(analysis)),
            Err(IntegrationError::NotFound(_)) => Ok(None),
            Err(IntegrationError::AnalysisInProgress) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get the current status of an ongoing analysis.
    pub async fn status(&self, host: &str) -> IntegrationResult<SslAnalysis> {
        let url = format!("{}/analyze", SSLLABS_API_URL);
        debug!("SSL Labs status: {} - host: {}", url, host);

        let response = self
            .client
            .get(&url)
            .query(&[("host", host)])
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get API info (limits, etc).
    pub async fn info(&self) -> IntegrationResult<SslLabsInfo> {
        let url = format!("{}/info", SSLLABS_API_URL);
        debug!("SSL Labs info: {}", url);

        let response = self.client.get(&url).send().await?;

        match response.status().as_u16() {
            200 => Ok(response.json().await?),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }

    async fn handle_response(
        &self,
        response: reqwest::Response,
    ) -> IntegrationResult<SslAnalysis> {
        match response.status().as_u16() {
            200 => {
                let analysis: SslLabsResponse = response.json().await?;

                // Check status
                match analysis.status.as_str() {
                    "READY" => Ok(SslAnalysis::from(analysis)),
                    "IN_PROGRESS" | "DNS" => Err(IntegrationError::AnalysisInProgress),
                    "ERROR" => Err(IntegrationError::Api(
                        analysis.status_message.unwrap_or_else(|| "Unknown error".to_string()),
                    )),
                    _ => Err(IntegrationError::Api(format!(
                        "Unknown status: {}",
                        analysis.status
                    ))),
                }
            }
            429 => Err(IntegrationError::RateLimited("SSL Labs")),
            503 => Err(IntegrationError::ServiceUnavailable("SSL Labs".to_string())),
            _ => {
                let text = response.text().await.unwrap_or_default();
                Err(IntegrationError::Api(text))
            }
        }
    }
}

impl Default for SslLabsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// SSL Labs API response.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsResponse {
    host: String,
    port: u16,
    protocol: String,
    #[serde(rename = "isPublic")]
    is_public: Option<bool>,
    status: String,
    #[serde(rename = "statusMessage")]
    status_message: Option<String>,
    #[serde(rename = "startTime")]
    start_time: Option<u64>,
    #[serde(rename = "testTime")]
    test_time: Option<u64>,
    #[serde(rename = "engineVersion")]
    engine_version: Option<String>,
    #[serde(rename = "criteriaVersion")]
    criteria_version: Option<String>,
    #[serde(default)]
    endpoints: Vec<SslLabsEndpoint>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsEndpoint {
    #[serde(rename = "ipAddress")]
    ip_address: String,
    #[serde(rename = "serverName")]
    server_name: Option<String>,
    #[serde(rename = "statusMessage")]
    status_message: Option<String>,
    grade: Option<String>,
    #[serde(rename = "gradeTrustIgnored")]
    grade_trust_ignored: Option<String>,
    #[serde(rename = "hasWarnings")]
    has_warnings: Option<bool>,
    #[serde(rename = "isExceptional")]
    is_exceptional: Option<bool>,
    progress: Option<i32>,
    duration: Option<u64>,
    delegation: Option<i32>,
    details: Option<SslLabsDetails>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsDetails {
    #[serde(rename = "hostStartTime")]
    host_start_time: Option<u64>,
    #[serde(default)]
    protocols: Vec<SslLabsProtocol>,
    #[serde(default)]
    suites: Vec<SslLabsSuiteList>,
    #[serde(rename = "serverSignature")]
    server_signature: Option<String>,
    #[serde(rename = "prefixDelegation")]
    prefix_delegation: Option<bool>,
    #[serde(rename = "nonPrefixDelegation")]
    non_prefix_delegation: Option<bool>,
    #[serde(rename = "vulnBeast")]
    vuln_beast: Option<bool>,
    #[serde(rename = "heartbleed")]
    heartbleed: Option<bool>,
    #[serde(rename = "heartbeat")]
    heartbeat: Option<bool>,
    #[serde(rename = "openSslCcs")]
    openssl_ccs: Option<i32>,
    #[serde(rename = "openSSLLuckyMinus20")]
    openssl_lucky_minus_20: Option<i32>,
    poodle: Option<bool>,
    #[serde(rename = "poodleTls")]
    poodle_tls: Option<i32>,
    #[serde(rename = "fallbackScsv")]
    fallback_scsv: Option<bool>,
    freak: Option<bool>,
    #[serde(rename = "hasSct")]
    has_sct: Option<i32>,
    #[serde(rename = "dhPrimes")]
    dh_primes: Option<Vec<String>>,
    #[serde(rename = "dhUsesKnownPrimes")]
    dh_uses_known_primes: Option<i32>,
    #[serde(rename = "dhYsReuse")]
    dh_ys_reuse: Option<bool>,
    logjam: Option<bool>,
    #[serde(rename = "chaCha20Preference")]
    chacha20_preference: Option<bool>,
    #[serde(rename = "hstsPolicy")]
    hsts_policy: Option<SslLabsHstsPolicy>,
    #[serde(rename = "hstsPreloads")]
    hsts_preloads: Option<Vec<SslLabsHstsPreload>>,
    #[serde(rename = "staticPkpPolicy")]
    static_pkp_policy: Option<SslLabsPkpPolicy>,
    #[serde(rename = "httpTransactions")]
    http_transactions: Option<Vec<SslLabsHttpTransaction>>,
    #[serde(rename = "supportsAlpn")]
    supports_alpn: Option<bool>,
    #[serde(rename = "npnProtocols")]
    npn_protocols: Option<String>,
    #[serde(rename = "alpnProtocols")]
    alpn_protocols: Option<String>,
    #[serde(rename = "supportsMustStaple")]
    supports_must_staple: Option<bool>,
    cert: Option<SslLabsCert>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsProtocol {
    id: i32,
    name: String,
    version: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsSuiteList {
    protocol: i32,
    #[serde(default)]
    list: Vec<SslLabsSuite>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsSuite {
    id: u32,
    name: String,
    #[serde(rename = "cipherStrength")]
    cipher_strength: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsHstsPolicy {
    status: Option<String>,
    #[serde(rename = "maxAge")]
    max_age: Option<u64>,
    #[serde(rename = "includeSubDomains")]
    include_subdomains: Option<bool>,
    preload: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsHstsPreload {
    source: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsPkpPolicy {
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsHttpTransaction {
    #[serde(rename = "requestUrl")]
    request_url: Option<String>,
    #[serde(rename = "statusCode")]
    status_code: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SslLabsCert {
    subject: Option<String>,
    issuer: Option<String>,
    #[serde(rename = "commonNames")]
    common_names: Option<Vec<String>>,
    #[serde(rename = "altNames")]
    alt_names: Option<Vec<String>>,
    #[serde(rename = "notBefore")]
    not_before: Option<u64>,
    #[serde(rename = "notAfter")]
    not_after: Option<u64>,
    #[serde(rename = "sigAlg")]
    sig_alg: Option<String>,
    #[serde(rename = "keyAlg")]
    key_alg: Option<String>,
    #[serde(rename = "keySize")]
    key_size: Option<u32>,
    #[serde(rename = "keyStrength")]
    key_strength: Option<u32>,
}

/// SSL analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslAnalysis {
    /// Host analyzed.
    pub host: String,

    /// Port.
    pub port: u16,

    /// Overall grade (A+, A, A-, B, C, D, E, F, T, M).
    pub grade: Option<SslGrade>,

    /// Analysis timestamp.
    pub test_time: Option<u64>,

    /// Engine version used.
    pub engine_version: Option<String>,

    /// Endpoints (multiple IPs may serve the same host).
    pub endpoints: Vec<SslEndpoint>,

    /// Does the host support HSTS?
    pub hsts_enabled: bool,

    /// Does the host have HSTS preload?
    pub hsts_preload: bool,

    /// Vulnerabilities detected.
    pub vulnerabilities: Vec<String>,
}

impl From<SslLabsResponse> for SslAnalysis {
    fn from(r: SslLabsResponse) -> Self {
        let endpoints: Vec<SslEndpoint> = r.endpoints.into_iter().map(SslEndpoint::from).collect();

        // Get overall grade from first endpoint
        let grade = endpoints.first().and_then(|e| e.grade);

        // Check for HSTS
        let hsts_enabled = endpoints.iter().any(|e| e.hsts_enabled);
        let hsts_preload = endpoints.iter().any(|e| e.hsts_preload);

        // Collect vulnerabilities
        let mut vulnerabilities = Vec::new();
        for endpoint in &endpoints {
            vulnerabilities.extend(endpoint.vulnerabilities.clone());
        }
        vulnerabilities.sort();
        vulnerabilities.dedup();

        Self {
            host: r.host,
            port: r.port,
            grade,
            test_time: r.test_time,
            engine_version: r.engine_version,
            endpoints,
            hsts_enabled,
            hsts_preload,
            vulnerabilities,
        }
    }
}

/// SSL endpoint analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslEndpoint {
    /// IP address.
    pub ip_address: String,

    /// Server name.
    pub server_name: Option<String>,

    /// Grade.
    pub grade: Option<SslGrade>,

    /// Has warnings.
    pub has_warnings: bool,

    /// Supported protocols.
    pub protocols: Vec<String>,

    /// Certificate subject.
    pub cert_subject: Option<String>,

    /// Certificate issuer.
    pub cert_issuer: Option<String>,

    /// Certificate expiry.
    pub cert_not_after: Option<u64>,

    /// Key algorithm.
    pub key_algorithm: Option<String>,

    /// Key size.
    pub key_size: Option<u32>,

    /// HSTS enabled.
    pub hsts_enabled: bool,

    /// HSTS preload.
    pub hsts_preload: bool,

    /// Vulnerabilities.
    pub vulnerabilities: Vec<String>,
}

impl From<SslLabsEndpoint> for SslEndpoint {
    fn from(e: SslLabsEndpoint) -> Self {
        let mut vulnerabilities = Vec::new();

        if let Some(ref details) = e.details {
            if details.vuln_beast == Some(true) {
                vulnerabilities.push("BEAST".to_string());
            }
            if details.heartbleed == Some(true) {
                vulnerabilities.push("Heartbleed".to_string());
            }
            if details.poodle == Some(true) {
                vulnerabilities.push("POODLE".to_string());
            }
            if details.freak == Some(true) {
                vulnerabilities.push("FREAK".to_string());
            }
            if details.logjam == Some(true) {
                vulnerabilities.push("Logjam".to_string());
            }
            if details.openssl_ccs == Some(1) || details.openssl_ccs == Some(2) {
                vulnerabilities.push("OpenSSL CCS".to_string());
            }
        }

        let protocols: Vec<String> = e
            .details
            .as_ref()
            .map(|d| {
                d.protocols
                    .iter()
                    .map(|p| format!("{} {}", p.name, p.version))
                    .collect()
            })
            .unwrap_or_default();

        let hsts_enabled = e
            .details
            .as_ref()
            .and_then(|d| d.hsts_policy.as_ref())
            .and_then(|h| h.status.as_ref())
            .map(|s| s == "present")
            .unwrap_or(false);

        let hsts_preload = e
            .details
            .as_ref()
            .and_then(|d| d.hsts_policy.as_ref())
            .and_then(|h| h.preload)
            .unwrap_or(false);

        Self {
            ip_address: e.ip_address,
            server_name: e.server_name,
            grade: e.grade.as_ref().and_then(|g| SslGrade::from_str(g)),
            has_warnings: e.has_warnings.unwrap_or(false),
            protocols,
            cert_subject: e.details.as_ref().and_then(|d| d.cert.as_ref()).and_then(|c| c.subject.clone()),
            cert_issuer: e.details.as_ref().and_then(|d| d.cert.as_ref()).and_then(|c| c.issuer.clone()),
            cert_not_after: e.details.as_ref().and_then(|d| d.cert.as_ref()).and_then(|c| c.not_after),
            key_algorithm: e.details.as_ref().and_then(|d| d.cert.as_ref()).and_then(|c| c.key_alg.clone()),
            key_size: e.details.as_ref().and_then(|d| d.cert.as_ref()).and_then(|c| c.key_size),
            hsts_enabled,
            hsts_preload,
            vulnerabilities,
        }
    }
}

/// SSL grade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SslGrade {
    /// A+ grade.
    APlus,
    /// A grade.
    A,
    /// A- grade.
    AMinus,
    /// B grade.
    B,
    /// C grade.
    C,
    /// D grade.
    D,
    /// E grade.
    E,
    /// F grade.
    F,
    /// T grade (trust issues).
    T,
    /// M grade (certificate mismatch).
    M,
}

impl SslGrade {
    /// Parse grade from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "A+" => Some(SslGrade::APlus),
            "A" => Some(SslGrade::A),
            "A-" => Some(SslGrade::AMinus),
            "B" => Some(SslGrade::B),
            "C" => Some(SslGrade::C),
            "D" => Some(SslGrade::D),
            "E" => Some(SslGrade::E),
            "F" => Some(SslGrade::F),
            "T" => Some(SslGrade::T),
            "M" => Some(SslGrade::M),
            _ => None,
        }
    }

    /// Get grade as string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SslGrade::APlus => "A+",
            SslGrade::A => "A",
            SslGrade::AMinus => "A-",
            SslGrade::B => "B",
            SslGrade::C => "C",
            SslGrade::D => "D",
            SslGrade::E => "E",
            SslGrade::F => "F",
            SslGrade::T => "T",
            SslGrade::M => "M",
        }
    }

    /// Is this grade passing?
    pub fn is_passing(&self) -> bool {
        matches!(self, SslGrade::APlus | SslGrade::A | SslGrade::AMinus | SslGrade::B)
    }
}

impl std::fmt::Display for SslGrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// SSL Labs API info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslLabsInfo {
    /// Engine version.
    #[serde(rename = "engineVersion")]
    pub engine_version: String,

    /// Criteria version.
    #[serde(rename = "criteriaVersion")]
    pub criteria_version: String,

    /// Max assessments allowed.
    #[serde(rename = "maxAssessments")]
    pub max_assessments: u32,

    /// Current assessments running.
    #[serde(rename = "currentAssessments")]
    pub current_assessments: u32,
}
