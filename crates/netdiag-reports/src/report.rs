//! Report data structures and builder.

use chrono::{DateTime, Utc};
use netdiag_types::diagnostics::{PingStats, TracerouteResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Report format types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// JSON format
    Json,
    /// Human-readable text
    Text,
    /// Markdown format
    Markdown,
    /// HTML format
    Html,
    /// PDF format
    Pdf,
}

impl ReportFormat {
    /// Get the file extension for this format.
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Pdf => "pdf",
        }
    }
}

/// Report metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report ID
    pub id: Uuid,
    /// Report title
    pub title: String,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Tool version
    pub version: String,
    /// Hostname of the machine that generated the report
    pub hostname: Option<String>,
    /// Operating system information
    pub os_info: Option<String>,
    /// Additional metadata
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl Default for ReportMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: "Network Diagnostics Report".to_string(),
            generated_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            hostname: None,
            os_info: None,
            extra: HashMap::new(),
        }
    }
}

/// Network interface summary for reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSummary {
    /// Interface name
    pub name: String,
    /// Interface type
    pub interface_type: String,
    /// IPv4 addresses
    pub ipv4_addresses: Vec<String>,
    /// IPv6 addresses
    pub ipv6_addresses: Vec<String>,
    /// MAC address
    pub mac_address: Option<String>,
    /// Is up
    pub is_up: bool,
    /// Is default
    pub is_default: bool,
}

/// DNS result summary for reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSummary {
    /// Query
    pub query: String,
    /// Resolved addresses
    pub addresses: Vec<String>,
    /// Resolution time in milliseconds
    pub duration_ms: f64,
    /// Success
    pub success: bool,
    /// Error message
    pub error: Option<String>,
}

/// Ping result summary for reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingSummary {
    /// Target
    pub target: String,
    /// Packets transmitted
    pub transmitted: u32,
    /// Packets received
    pub received: u32,
    /// Packet loss percentage
    pub loss_percent: f64,
    /// Minimum RTT in milliseconds
    pub min_rtt_ms: Option<f64>,
    /// Average RTT in milliseconds
    pub avg_rtt_ms: Option<f64>,
    /// Maximum RTT in milliseconds
    pub max_rtt_ms: Option<f64>,
    /// Standard deviation
    pub stddev_ms: Option<f64>,
    /// Quality rating
    pub quality: String,
}

impl From<&PingStats> for PingSummary {
    fn from(stats: &PingStats) -> Self {
        Self {
            target: stats.target.to_string(),
            transmitted: stats.transmitted,
            received: stats.received,
            loss_percent: stats.loss_percent,
            min_rtt_ms: stats.min_rtt.map(|d| d.as_secs_f64() * 1000.0),
            avg_rtt_ms: stats.avg_rtt.map(|d| d.as_secs_f64() * 1000.0),
            max_rtt_ms: stats.max_rtt.map(|d| d.as_secs_f64() * 1000.0),
            stddev_ms: stats.stddev_rtt.map(|d| d.as_secs_f64() * 1000.0),
            quality: format!("{:?}", stats.quality_rating()),
        }
    }
}

/// Traceroute hop summary for reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteHopSummary {
    /// Hop number
    pub hop: u8,
    /// IP address
    pub address: Option<String>,
    /// Hostname
    pub hostname: Option<String>,
    /// RTT values in milliseconds
    pub rtt_ms: Vec<Option<f64>>,
    /// All timeout
    pub all_timeout: bool,
}

/// Traceroute result summary for reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteSummary {
    /// Target
    pub target: String,
    /// Reached destination
    pub reached: bool,
    /// Hop count
    pub hop_count: usize,
    /// Total duration in milliseconds
    pub duration_ms: f64,
    /// Protocol
    pub protocol: String,
    /// Hops
    pub hops: Vec<TracerouteHopSummary>,
}

impl From<&TracerouteResult> for TracerouteSummary {
    fn from(result: &TracerouteResult) -> Self {
        Self {
            target: result.target.to_string(),
            reached: result.reached,
            hop_count: result.hops.len(),
            duration_ms: result.duration.as_secs_f64() * 1000.0,
            protocol: format!("{:?}", result.protocol),
            hops: result
                .hops
                .iter()
                .map(|hop| TracerouteHopSummary {
                    hop: hop.hop,
                    address: hop.address.map(|a| a.to_string()),
                    hostname: hop.hostname.clone(),
                    rtt_ms: hop
                        .probes
                        .iter()
                        .map(|p| p.rtt.map(|d| d.as_secs_f64() * 1000.0))
                        .collect(),
                    all_timeout: hop.all_timeout,
                })
                .collect(),
        }
    }
}

/// Overall health assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthAssessment {
    /// Overall status (good, warning, critical)
    pub status: String,
    /// Score (0-100)
    pub score: u8,
    /// Issues found
    pub issues: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Complete diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Health assessment
    pub health: Option<HealthAssessment>,
    /// Network interfaces
    pub interfaces: Vec<InterfaceSummary>,
    /// DNS results
    pub dns_results: Vec<DnsSummary>,
    /// Ping results
    pub ping_results: Vec<PingSummary>,
    /// Traceroute results
    pub traceroute_results: Vec<TracerouteSummary>,
    /// Raw data (optional, for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<serde_json::Value>,
}

impl Default for DiagnosticReport {
    fn default() -> Self {
        Self {
            metadata: ReportMetadata::default(),
            health: None,
            interfaces: Vec::new(),
            dns_results: Vec::new(),
            ping_results: Vec::new(),
            traceroute_results: Vec::new(),
            raw_data: None,
        }
    }
}

/// Builder for diagnostic reports.
pub struct ReportBuilder {
    report: DiagnosticReport,
}

impl ReportBuilder {
    /// Create a new report builder.
    pub fn new() -> Self {
        Self {
            report: DiagnosticReport::default(),
        }
    }

    /// Set the report title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.report.metadata.title = title.into();
        self
    }

    /// Set the hostname.
    pub fn hostname(mut self, hostname: impl Into<String>) -> Self {
        self.report.metadata.hostname = Some(hostname.into());
        self
    }

    /// Set the OS info.
    pub fn os_info(mut self, os_info: impl Into<String>) -> Self {
        self.report.metadata.os_info = Some(os_info.into());
        self
    }

    /// Add metadata.
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.report.metadata.extra.insert(key.into(), value.into());
        self
    }

    /// Add an interface.
    pub fn add_interface(mut self, interface: InterfaceSummary) -> Self {
        self.report.interfaces.push(interface);
        self
    }

    /// Add a DNS result.
    pub fn add_dns_result(mut self, dns: DnsSummary) -> Self {
        self.report.dns_results.push(dns);
        self
    }

    /// Add a ping result.
    pub fn add_ping_result(mut self, ping: PingSummary) -> Self {
        self.report.ping_results.push(ping);
        self
    }

    /// Add ping stats.
    pub fn add_ping_stats(mut self, stats: &PingStats) -> Self {
        self.report.ping_results.push(stats.into());
        self
    }

    /// Add a traceroute result.
    pub fn add_traceroute_result(mut self, traceroute: TracerouteSummary) -> Self {
        self.report.traceroute_results.push(traceroute);
        self
    }

    /// Add traceroute result.
    pub fn add_traceroute(mut self, result: &TracerouteResult) -> Self {
        self.report.traceroute_results.push(result.into());
        self
    }

    /// Set the health assessment.
    pub fn health_assessment(mut self, health: HealthAssessment) -> Self {
        self.report.health = Some(health);
        self
    }

    /// Include raw data.
    pub fn raw_data(mut self, data: serde_json::Value) -> Self {
        self.report.raw_data = Some(data);
        self
    }

    /// Build the report.
    pub fn build(mut self) -> DiagnosticReport {
        // Calculate health assessment if not set
        if self.report.health.is_none() {
            self.report.health = Some(self.calculate_health());
        }
        self.report
    }

    /// Calculate health assessment from results.
    fn calculate_health(&self) -> HealthAssessment {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut score = 100u8;

        // Check ping results
        for ping in &self.report.ping_results {
            if ping.loss_percent > 0.0 {
                if ping.loss_percent > 10.0 {
                    issues.push(format!(
                        "High packet loss to {}: {:.1}%",
                        ping.target, ping.loss_percent
                    ));
                    score = score.saturating_sub(20);
                } else {
                    issues.push(format!(
                        "Minor packet loss to {}: {:.1}%",
                        ping.target, ping.loss_percent
                    ));
                    score = score.saturating_sub(5);
                }
            }

            if let Some(avg) = ping.avg_rtt_ms {
                if avg > 200.0 {
                    issues.push(format!("High latency to {}: {:.1}ms", ping.target, avg));
                    score = score.saturating_sub(15);
                    recommendations.push("Consider checking for network congestion".to_string());
                } else if avg > 100.0 {
                    issues.push(format!("Elevated latency to {}: {:.1}ms", ping.target, avg));
                    score = score.saturating_sub(5);
                }
            }
        }

        // Check traceroute results
        for traceroute in &self.report.traceroute_results {
            if !traceroute.reached {
                issues.push(format!(
                    "Could not reach destination: {}",
                    traceroute.target
                ));
                score = score.saturating_sub(25);
                recommendations.push("Check firewall settings and routing".to_string());
            }

            let timeout_hops: usize = traceroute.hops.iter().filter(|h| h.all_timeout).count();
            if timeout_hops > 2 {
                issues.push(format!(
                    "{} hops timing out on route to {}",
                    timeout_hops, traceroute.target
                ));
                score = score.saturating_sub(10);
            }
        }

        // Check DNS results
        for dns in &self.report.dns_results {
            if !dns.success {
                issues.push(format!("DNS resolution failed for {}", dns.query));
                score = score.saturating_sub(20);
                recommendations.push("Check DNS server configuration".to_string());
            } else if dns.duration_ms > 500.0 {
                issues.push(format!(
                    "Slow DNS resolution for {}: {:.1}ms",
                    dns.query, dns.duration_ms
                ));
                score = score.saturating_sub(5);
                recommendations.push("Consider using faster DNS servers".to_string());
            }
        }

        // Determine status
        let status = if score >= 80 {
            "good".to_string()
        } else if score >= 50 {
            "warning".to_string()
        } else {
            "critical".to_string()
        };

        // Add general recommendations
        if issues.is_empty() {
            recommendations.push("Network appears healthy".to_string());
        }

        HealthAssessment {
            status,
            score,
            issues,
            recommendations,
        }
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}
