//! Storage model types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Stored diagnostic session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticSession {
    /// Unique session ID
    pub id: Uuid,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Session end time
    pub ended_at: Option<DateTime<Utc>>,
    /// Session type (quick, full, scheduled)
    pub session_type: SessionType,
    /// Session status
    pub status: SessionStatus,
    /// Summary of results
    pub summary: Option<String>,
    /// Metadata as JSON
    pub metadata: Option<serde_json::Value>,
}

/// Session type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    /// Quick diagnostic
    Quick,
    /// Full diagnostic
    Full,
    /// Scheduled diagnostic
    Scheduled,
    /// Manual test
    Manual,
}

impl SessionType {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Quick => "quick",
            Self::Full => "full",
            Self::Scheduled => "scheduled",
            Self::Manual => "manual",
        }
    }
}

impl std::str::FromStr for SessionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "quick" => Ok(Self::Quick),
            "full" => Ok(Self::Full),
            "scheduled" => Ok(Self::Scheduled),
            "manual" => Ok(Self::Manual),
            _ => Err(format!("Unknown session type: {}", s)),
        }
    }
}

/// Session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Running
    Running,
    /// Completed successfully
    Completed,
    /// Failed
    Failed,
    /// Cancelled
    Cancelled,
}

impl SessionStatus {
    /// Convert to string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(format!("Unknown session status: {}", s)),
        }
    }
}

/// Stored ping result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPingResult {
    /// Unique result ID
    pub id: Uuid,
    /// Session ID
    pub session_id: Option<Uuid>,
    /// Target hostname or IP
    pub target: String,
    /// Resolved IP address
    pub resolved_ip: Option<String>,
    /// Number of packets transmitted
    pub transmitted: u32,
    /// Number of packets received
    pub received: u32,
    /// Packet loss percentage
    pub loss_percent: f64,
    /// Minimum RTT in milliseconds
    pub min_rtt_ms: Option<f64>,
    /// Average RTT in milliseconds
    pub avg_rtt_ms: Option<f64>,
    /// Maximum RTT in milliseconds
    pub max_rtt_ms: Option<f64>,
    /// Standard deviation in milliseconds
    pub stddev_ms: Option<f64>,
    /// Quality rating
    pub quality: Option<String>,
    /// Timestamp
    pub created_at: DateTime<Utc>,
    /// Raw results as JSON
    pub raw_data: Option<serde_json::Value>,
}

/// Stored traceroute result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredTracerouteResult {
    /// Unique result ID
    pub id: Uuid,
    /// Session ID
    pub session_id: Option<Uuid>,
    /// Target hostname or IP
    pub target: String,
    /// Resolved IP address
    pub resolved_ip: Option<String>,
    /// Number of hops
    pub hop_count: u32,
    /// Whether destination was reached
    pub reached: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Protocol used
    pub protocol: String,
    /// Timestamp
    pub created_at: DateTime<Utc>,
    /// Hops data as JSON
    pub hops_data: serde_json::Value,
}

/// Stored speed test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSpeedResult {
    /// Unique result ID
    pub id: Uuid,
    /// Session ID
    pub session_id: Option<Uuid>,
    /// Server used
    pub server: Option<String>,
    /// Download speed in Mbps
    pub download_mbps: Option<f64>,
    /// Upload speed in Mbps
    pub upload_mbps: Option<f64>,
    /// Ping/latency in milliseconds
    pub ping_ms: Option<f64>,
    /// Jitter in milliseconds
    pub jitter_ms: Option<f64>,
    /// Timestamp
    pub created_at: DateTime<Utc>,
    /// Raw data as JSON
    pub raw_data: Option<serde_json::Value>,
}

/// Stored network interface snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredInterfaceSnapshot {
    /// Unique snapshot ID
    pub id: Uuid,
    /// Session ID
    pub session_id: Option<Uuid>,
    /// Interface name
    pub name: String,
    /// Interface type
    pub interface_type: String,
    /// MAC address
    pub mac_address: Option<String>,
    /// IPv4 addresses as JSON
    pub ipv4_addresses: serde_json::Value,
    /// IPv6 addresses as JSON
    pub ipv6_addresses: serde_json::Value,
    /// Is up
    pub is_up: bool,
    /// Is default
    pub is_default: bool,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Stored DNS result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredDnsResult {
    /// Unique result ID
    pub id: Uuid,
    /// Session ID
    pub session_id: Option<Uuid>,
    /// Query hostname
    pub query: String,
    /// Resolved addresses as JSON
    pub addresses: serde_json::Value,
    /// Resolution time in milliseconds
    pub duration_ms: f64,
    /// Success
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp
    pub created_at: DateTime<Utc>,
}

/// Query options for retrieving results.
#[derive(Debug, Clone, Default)]
pub struct QueryOptions {
    /// Maximum number of results
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
    /// Start time filter
    pub from: Option<DateTime<Utc>>,
    /// End time filter
    pub to: Option<DateTime<Utc>>,
    /// Target filter
    pub target: Option<String>,
    /// Session ID filter
    pub session_id: Option<Uuid>,
}

impl QueryOptions {
    /// Create new query options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set limit.
    pub fn limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset.
    pub fn offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Set from time.
    pub fn from(mut self, from: DateTime<Utc>) -> Self {
        self.from = Some(from);
        self
    }

    /// Set to time.
    pub fn to(mut self, to: DateTime<Utc>) -> Self {
        self.to = Some(to);
        self
    }

    /// Set target filter.
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Set session ID filter.
    pub fn session(mut self, session_id: Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }
}
