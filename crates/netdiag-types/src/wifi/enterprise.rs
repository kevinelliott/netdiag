//! Enterprise `WiFi` (802.1X) types.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// EAP (Extensible Authentication Protocol) method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum EapMethod {
    /// EAP-TLS (certificate-based)
    Tls,
    /// EAP-TTLS (tunneled TLS)
    Ttls,
    /// PEAP (Protected EAP)
    Peap,
    /// EAP-FAST
    Fast,
    /// EAP-SIM
    Sim,
    /// EAP-AKA
    Aka,
    /// EAP-AKA'
    AkaPrime,
    /// EAP-PWD
    Pwd,
    /// LEAP (Cisco, deprecated)
    Leap,
    /// EAP-MD5 (deprecated)
    Md5,
}

/// Phase 2 (inner) authentication method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum Phase2Method {
    /// None/not applicable
    None,
    /// MS-CHAPv2
    MsChapV2,
    /// MS-CHAP
    MsChap,
    /// PAP
    Pap,
    /// CHAP
    Chap,
    /// GTC (Generic Token Card)
    Gtc,
    /// EAP-TLS (for EAP-TTLS)
    Tls,
}

/// Enterprise `WiFi` configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// EAP method
    pub eap_method: EapMethod,
    /// Phase 2 method (for PEAP/TTLS)
    pub phase2_method: Option<Phase2Method>,
    /// Identity (username)
    pub identity: String,
    /// Anonymous identity (for outer authentication)
    pub anonymous_identity: Option<String>,
    /// Domain (for server certificate validation)
    pub domain: Option<String>,
    /// CA certificate path
    pub ca_cert_path: Option<String>,
    /// Client certificate path (for EAP-TLS)
    pub client_cert_path: Option<String>,
    /// Client private key path
    pub private_key_path: Option<String>,
    /// Whether to validate server certificate
    pub validate_server_cert: bool,
}

/// RADIUS server information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusServer {
    /// Server hostname or IP
    pub host: String,
    /// Authentication port (usually 1812)
    pub auth_port: u16,
    /// Accounting port (usually 1813)
    pub accounting_port: u16,
    /// Is this the primary server?
    pub is_primary: bool,
    /// Server timeout in seconds
    pub timeout_secs: u32,
    /// Maximum retries
    pub max_retries: u32,
}

impl Default for RadiusServer {
    fn default() -> Self {
        Self {
            host: String::new(),
            auth_port: 1812,
            accounting_port: 1813,
            is_primary: true,
            timeout_secs: 5,
            max_retries: 3,
        }
    }
}

/// Enterprise authentication result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseAuthResult {
    /// Whether authentication succeeded
    pub success: bool,
    /// EAP method used
    pub eap_method: EapMethod,
    /// Authentication time in milliseconds
    pub auth_time_ms: u64,
    /// RADIUS server that responded
    pub radius_server: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Server certificate info
    pub server_cert: Option<CertificateInfo>,
    /// Session info
    pub session: Option<EapSession>,
}

/// Certificate information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    /// Certificate subject
    pub subject: String,
    /// Certificate issuer
    pub issuer: String,
    /// Valid from date
    pub valid_from: chrono::DateTime<chrono::Utc>,
    /// Valid to date
    pub valid_to: chrono::DateTime<chrono::Utc>,
    /// SHA-256 fingerprint
    pub fingerprint_sha256: String,
    /// Is the certificate valid?
    pub is_valid: bool,
    /// Is the certificate trusted?
    pub is_trusted: bool,
}

/// EAP session information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EapSession {
    /// Session ID
    pub session_id: String,
    /// Session timeout in seconds
    pub timeout_secs: Option<u32>,
    /// Reauthentication interval in seconds
    pub reauth_interval_secs: Option<u32>,
    /// VLAN assigned
    pub vlan_id: Option<u16>,
}
