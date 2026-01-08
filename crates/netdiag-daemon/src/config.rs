//! Daemon configuration.

use crate::error::{DaemonError, Result};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Daemon configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// General daemon settings.
    #[serde(default)]
    pub general: GeneralConfig,

    /// IPC settings.
    #[serde(default)]
    pub ipc: IpcConfig,

    /// Monitoring settings.
    #[serde(default)]
    pub monitoring: MonitoringConfig,

    /// Scheduled jobs.
    #[serde(default)]
    pub schedules: Vec<ScheduleConfig>,

    /// Alert settings.
    #[serde(default)]
    pub alerts: AlertConfig,

    /// Storage settings.
    #[serde(default)]
    pub storage: StorageConfig,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            ipc: IpcConfig::default(),
            monitoring: MonitoringConfig::default(),
            schedules: vec![
                // Default: run quick diagnostics every 5 minutes
                ScheduleConfig {
                    name: "quick-check".to_string(),
                    cron: "*/5 * * * *".to_string(),
                    diagnostic: DiagnosticType::Quick,
                    enabled: true,
                },
                // Default: run full diagnostics every hour
                ScheduleConfig {
                    name: "full-check".to_string(),
                    cron: "0 * * * *".to_string(),
                    diagnostic: DiagnosticType::Full,
                    enabled: true,
                },
            ],
            alerts: AlertConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

impl DaemonConfig {
    /// Loads configuration from a file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    /// Saves configuration to a file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content =
            toml::to_string_pretty(self).map_err(|e| DaemonError::config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<()> {
        // Validate cron expressions
        for schedule in &self.schedules {
            schedule.validate()?;
        }

        // Validate thresholds
        self.alerts.validate()?;

        Ok(())
    }

    /// Returns the default configuration file path.
    pub fn default_path() -> PathBuf {
        if cfg!(target_os = "macos") {
            PathBuf::from("/Library/Application Support/netdiag/daemon.toml")
        } else if cfg!(target_os = "linux") {
            PathBuf::from("/etc/netdiag/daemon.toml")
        } else if cfg!(target_os = "windows") {
            PathBuf::from(r"C:\ProgramData\netdiag\daemon.toml")
        } else {
            PathBuf::from("daemon.toml")
        }
    }

    /// Returns the user-specific configuration file path.
    pub fn user_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("netdiag").join("daemon.toml"))
    }
}

/// General daemon settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Whether to run in foreground mode (don't daemonize).
    #[serde(default)]
    pub foreground: bool,

    /// Log level.
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Log file path.
    pub log_file: Option<PathBuf>,

    /// PID file path.
    pub pid_file: Option<PathBuf>,

    /// Working directory.
    pub working_dir: Option<PathBuf>,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            foreground: false,
            log_level: default_log_level(),
            log_file: Some(default_log_file()),
            pid_file: Some(default_pid_file()),
            working_dir: None,
        }
    }
}

fn default_log_file() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/var/log/netdiag/daemon.log")
    } else if cfg!(target_os = "linux") {
        PathBuf::from("/var/log/netdiag/daemon.log")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\ProgramData\netdiag\logs\daemon.log")
    } else {
        PathBuf::from("daemon.log")
    }
}

fn default_pid_file() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/var/run/netdiag.pid")
    } else if cfg!(target_os = "linux") {
        PathBuf::from("/var/run/netdiag.pid")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\ProgramData\netdiag\netdiag.pid")
    } else {
        PathBuf::from("netdiag.pid")
    }
}

/// IPC configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfig {
    /// Unix socket path (Unix) or named pipe path (Windows).
    #[serde(default = "default_socket_path")]
    pub socket_path: String,

    /// Maximum concurrent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,
}

fn default_socket_path() -> String {
    if cfg!(windows) {
        r"\\.\pipe\netdiag".to_string()
    } else {
        "/var/run/netdiag.sock".to_string()
    }
}

fn default_max_connections() -> usize {
    10
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            socket_path: default_socket_path(),
            max_connections: default_max_connections(),
        }
    }
}

/// Monitoring configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable continuous monitoring.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Monitoring interval.
    #[serde(with = "humantime_serde", default = "default_interval")]
    pub interval: Duration,

    /// Targets to monitor.
    #[serde(default = "default_targets")]
    pub targets: Vec<MonitorTarget>,
}

fn default_enabled() -> bool {
    true
}

fn default_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_targets() -> Vec<MonitorTarget> {
    vec![
        MonitorTarget::Gateway,
        MonitorTarget::Dns,
        MonitorTarget::Internet,
    ]
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            interval: default_interval(),
            targets: default_targets(),
        }
    }
}

/// Monitor target types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MonitorTarget {
    /// Monitor the default gateway.
    Gateway,
    /// Monitor DNS servers.
    Dns,
    /// Monitor internet connectivity.
    Internet,
    /// Monitor a custom host.
    Host(String),
    /// Monitor a custom IP.
    Ip(IpAddr),
}

/// Scheduled diagnostic configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Name of the schedule.
    pub name: String,

    /// Cron expression.
    pub cron: String,

    /// Type of diagnostic to run.
    pub diagnostic: DiagnosticType,

    /// Whether this schedule is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl ScheduleConfig {
    /// Validates the schedule configuration.
    pub fn validate(&self) -> Result<()> {
        // Basic cron validation - just check it's not empty for now
        // Full validation happens when creating the scheduler
        if self.cron.is_empty() {
            return Err(DaemonError::config(format!(
                "Empty cron expression for schedule '{}'",
                self.name
            )));
        }
        Ok(())
    }
}

/// Type of diagnostic to run.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticType {
    /// Quick connectivity check.
    Quick,
    /// Full diagnostic run.
    Full,
    /// WiFi-only analysis.
    Wifi,
    /// Speed test only.
    Speed,
    /// Custom diagnostic profile.
    Custom,
}

/// Alert configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Enable alerts.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Latency threshold (ms) for alerts.
    #[serde(default = "default_latency_threshold")]
    pub latency_threshold_ms: u64,

    /// Packet loss threshold (%) for alerts.
    #[serde(default = "default_packet_loss_threshold")]
    pub packet_loss_threshold: f64,

    /// WiFi signal threshold (dBm) for alerts.
    #[serde(default = "default_wifi_threshold")]
    pub wifi_signal_threshold: i32,

    /// Alert methods.
    #[serde(default)]
    pub methods: Vec<AlertMethod>,
}

fn default_latency_threshold() -> u64 {
    100
}

fn default_packet_loss_threshold() -> f64 {
    5.0
}

fn default_wifi_threshold() -> i32 {
    -70
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            latency_threshold_ms: default_latency_threshold(),
            packet_loss_threshold: default_packet_loss_threshold(),
            wifi_signal_threshold: default_wifi_threshold(),
            methods: vec![AlertMethod::Log],
        }
    }
}

impl AlertConfig {
    /// Validates the alert configuration.
    pub fn validate(&self) -> Result<()> {
        if self.packet_loss_threshold < 0.0 || self.packet_loss_threshold > 100.0 {
            return Err(DaemonError::config(
                "Packet loss threshold must be between 0 and 100",
            ));
        }
        Ok(())
    }
}

/// Alert delivery method.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AlertMethod {
    /// Log to file.
    Log,
    /// System notification (desktop).
    Notification,
    /// Write to file.
    File(PathBuf),
    /// Custom command.
    Command(String),
}

/// Storage configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path.
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,

    /// Maximum history retention (days).
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,

    /// Enable cloud sync.
    #[serde(default)]
    pub cloud_sync: bool,
}

fn default_db_path() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/Library/Application Support/netdiag/data.db")
    } else if cfg!(target_os = "linux") {
        PathBuf::from("/var/lib/netdiag/data.db")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\ProgramData\netdiag\data.db")
    } else {
        PathBuf::from("data.db")
    }
}

fn default_retention_days() -> u32 {
    30
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            retention_days: default_retention_days(),
            cloud_sync: false,
        }
    }
}
