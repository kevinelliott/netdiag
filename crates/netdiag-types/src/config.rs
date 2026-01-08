//! Configuration types for netdiag.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration for netdiag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    /// Ping settings
    pub ping: PingConfig,
    /// Traceroute settings
    pub traceroute: TracerouteConfig,
    /// Speed test settings
    pub speed_test: SpeedTestConfig,
    /// WiFi settings
    pub wifi: WifiConfig,
    /// Report settings
    pub report: ReportConfig,
    /// Storage settings
    pub storage: StorageConfig,
    /// Daemon settings
    pub daemon: DaemonConfig,
    /// Autofix settings
    pub autofix: AutofixConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ping: PingConfig::default(),
            traceroute: TracerouteConfig::default(),
            speed_test: SpeedTestConfig::default(),
            wifi: WifiConfig::default(),
            report: ReportConfig::default(),
            storage: StorageConfig::default(),
            daemon: DaemonConfig::default(),
            autofix: AutofixConfig::default(),
        }
    }
}

/// General configuration settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    /// Enable verbose output
    pub verbose: bool,
    /// Enable debug mode
    pub debug: bool,
    /// Default timeout for operations in seconds
    pub timeout_secs: u64,
    /// Number of parallel operations
    pub parallelism: usize,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            debug: false,
            timeout_secs: 30,
            parallelism: 4,
        }
    }
}

/// Ping configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PingConfig {
    /// Number of pings to send
    pub count: u32,
    /// Interval between pings in milliseconds
    pub interval_ms: u64,
    /// Timeout per ping in milliseconds
    pub timeout_ms: u64,
    /// Packet size in bytes
    pub packet_size: usize,
    /// Time to live
    pub ttl: u8,
    /// Default targets to ping
    pub default_targets: Vec<String>,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            count: 10,
            interval_ms: 1000,
            timeout_ms: 5000,
            packet_size: 64,
            ttl: 64,
            default_targets: vec![
                "8.8.8.8".to_string(),
                "1.1.1.1".to_string(),
                "google.com".to_string(),
            ],
        }
    }
}

/// Traceroute configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TracerouteConfig {
    /// Maximum number of hops
    pub max_hops: u8,
    /// Number of probes per hop
    pub probes_per_hop: u8,
    /// Timeout per probe in milliseconds
    pub timeout_ms: u64,
    /// Protocol to use (icmp, udp, tcp)
    pub protocol: TracerouteProtocol,
    /// Starting port for UDP/TCP probes
    pub start_port: u16,
}

impl Default for TracerouteConfig {
    fn default() -> Self {
        Self {
            max_hops: 30,
            probes_per_hop: 3,
            timeout_ms: 5000,
            protocol: TracerouteProtocol::Icmp,
            start_port: 33434,
        }
    }
}

/// Traceroute protocol options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TracerouteProtocol {
    /// ICMP Echo Request
    #[default]
    Icmp,
    /// UDP packets
    Udp,
    /// TCP SYN packets
    Tcp,
}

/// Speed test configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SpeedTestConfig {
    /// Duration of each test phase in seconds
    pub duration_secs: u64,
    /// Number of parallel connections
    pub connections: usize,
    /// Warmup duration in seconds
    pub warmup_secs: u64,
    /// Buffer size in bytes
    pub buffer_size: usize,
    /// Preferred speed test providers
    pub providers: Vec<SpeedTestProvider>,
    /// Custom speed test servers
    pub custom_servers: Vec<SpeedTestServer>,
}

impl Default for SpeedTestConfig {
    fn default() -> Self {
        Self {
            duration_secs: 10,
            connections: 4,
            warmup_secs: 2,
            buffer_size: 131_072, // 128KB
            providers: vec![SpeedTestProvider::Ookla, SpeedTestProvider::Cloudflare],
            custom_servers: Vec::new(),
        }
    }
}

/// Speed test provider options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeedTestProvider {
    /// Ookla Speedtest.net
    Ookla,
    /// Cloudflare speed test
    Cloudflare,
    /// Netflix Fast.com
    Netflix,
    /// iPerf3 server
    Iperf3,
    /// ISP official speed test
    Isp,
    /// Custom server
    Custom,
}

/// Custom speed test server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestServer {
    /// Server name
    pub name: String,
    /// Server URL or host
    pub url: String,
    /// Server type
    pub server_type: SpeedTestProvider,
}

/// WiFi configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WifiConfig {
    /// Enable channel analysis
    pub analyze_channels: bool,
    /// Enable interference detection
    pub detect_interference: bool,
    /// Scan interval in seconds for continuous monitoring
    pub scan_interval_secs: u64,
    /// Signal strength threshold for warnings (dBm)
    pub signal_warning_threshold: i32,
}

impl Default for WifiConfig {
    fn default() -> Self {
        Self {
            analyze_channels: true,
            detect_interference: true,
            scan_interval_secs: 30,
            signal_warning_threshold: -70,
        }
    }
}

/// Report configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportConfig {
    /// Default output format
    pub default_format: ReportFormat,
    /// Output directory for reports
    pub output_dir: PathBuf,
    /// Include raw data in reports
    pub include_raw_data: bool,
    /// Include recommendations
    pub include_recommendations: bool,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            default_format: ReportFormat::Terminal,
            output_dir: PathBuf::from("."),
            include_raw_data: false,
            include_recommendations: true,
        }
    }
}

/// Report format options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    /// Terminal colored output
    #[default]
    Terminal,
    /// Plain text
    Text,
    /// JSON
    Json,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// PDF
    Pdf,
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StorageConfig {
    /// Database path
    pub database_path: PathBuf,
    /// Enable cloud sync
    pub cloud_sync: bool,
    /// Cloud provider
    pub cloud_provider: Option<CloudProvider>,
    /// Cloud project ID (for Firebase)
    pub cloud_project_id: Option<String>,
    /// History retention in days
    pub history_retention_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("netdiag.db"),
            cloud_sync: false,
            cloud_provider: None,
            cloud_project_id: None,
            history_retention_days: 30,
        }
    }
}

/// Cloud storage provider options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CloudProvider {
    /// Firebase
    Firebase,
    /// Supabase
    Supabase,
}

/// Daemon configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DaemonConfig {
    /// Enable daemon mode
    pub enabled: bool,
    /// Monitoring interval in seconds
    pub interval_secs: u64,
    /// Enable notifications
    pub notifications: bool,
    /// Socket path for IPC
    pub socket_path: PathBuf,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_secs: 60,
            notifications: true,
            socket_path: PathBuf::from("/tmp/netdiag.sock"),
        }
    }
}

/// Autofix configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutofixConfig {
    /// Enable autofix prompts
    pub enabled: bool,
    /// Autofix level
    pub level: AutofixLevel,
    /// Create rollback points
    pub create_rollback: bool,
    /// Require confirmation for each fix
    pub require_confirmation: bool,
}

impl Default for AutofixConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: AutofixLevel::Aggressive,
            create_rollback: true,
            require_confirmation: true,
        }
    }
}

/// Autofix aggressiveness level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AutofixLevel {
    /// Conservative: Only safe, reversible fixes
    Conservative,
    /// Moderate: Include driver updates, profile changes
    Moderate,
    /// Aggressive: Include system config changes
    #[default]
    Aggressive,
}
