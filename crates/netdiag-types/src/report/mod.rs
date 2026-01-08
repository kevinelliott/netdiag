//! Report types.

use crate::config::ReportFormat;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Diagnostic report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// Report ID
    pub id: uuid::Uuid,
    /// Report title
    pub title: String,
    /// Report timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Report duration (how long the diagnostics took)
    pub duration: std::time::Duration,
    /// Overall status
    pub status: ReportStatus,
    /// Executive summary
    pub summary: ReportSummary,
    /// Report sections
    pub sections: Vec<ReportSection>,
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
    /// Metadata
    pub metadata: ReportMetadata,
}

impl DiagnosticReport {
    /// Creates a new empty report.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            title: title.into(),
            timestamp: chrono::Utc::now(),
            duration: std::time::Duration::ZERO,
            status: ReportStatus::Pending,
            summary: ReportSummary::default(),
            sections: Vec::new(),
            recommendations: Vec::new(),
            metadata: ReportMetadata::default(),
        }
    }

    /// Adds a section to the report.
    pub fn add_section(&mut self, section: ReportSection) {
        self.sections.push(section);
    }

    /// Adds a recommendation to the report.
    pub fn add_recommendation(&mut self, recommendation: Recommendation) {
        self.recommendations.push(recommendation);
    }
}

/// Report status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReportStatus {
    /// Report is pending
    #[default]
    Pending,
    /// Report is in progress
    InProgress,
    /// Report completed with all checks passing
    Healthy,
    /// Report completed with warnings
    Warning,
    /// Report completed with errors/issues
    Unhealthy,
    /// Report failed to complete
    Failed,
}

/// Executive summary of the report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    /// One-line status
    pub headline: String,
    /// Brief description
    pub description: String,
    /// Number of issues found
    pub issues_count: u32,
    /// Number of warnings
    pub warnings_count: u32,
    /// Number of passed checks
    pub passed_count: u32,
    /// Overall health score (0-100)
    pub health_score: u8,
}

/// A section of the report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    /// Section ID
    pub id: String,
    /// Section title
    pub title: String,
    /// Section status
    pub status: ReportStatus,
    /// Section items
    pub items: Vec<ReportItem>,
    /// Subsections
    pub subsections: Vec<ReportSection>,
}

impl ReportSection {
    /// Creates a new report section.
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            status: ReportStatus::Pending,
            items: Vec::new(),
            subsections: Vec::new(),
        }
    }
}

/// A single item in a report section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportItem {
    /// Item label/name
    pub label: String,
    /// Item value
    pub value: String,
    /// Item status
    pub status: Option<ItemStatus>,
    /// Additional details
    pub details: Option<String>,
    /// Raw data (for JSON export)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_data: Option<serde_json::Value>,
}

impl ReportItem {
    /// Creates a simple key-value item.
    #[must_use]
    pub fn kv(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            status: None,
            details: None,
            raw_data: None,
        }
    }

    /// Creates an item with a status.
    #[must_use]
    pub fn with_status(
        label: impl Into<String>,
        value: impl Into<String>,
        status: ItemStatus,
    ) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            status: Some(status),
            details: None,
            raw_data: None,
        }
    }
}

/// Status for a report item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    /// Check passed
    Pass,
    /// Warning
    Warn,
    /// Check failed
    Fail,
    /// Informational
    Info,
    /// Skipped
    Skip,
}

/// A recommendation for fixing an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation ID
    pub id: String,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Category
    pub category: RecommendationCategory,
    /// Short title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Steps to fix
    pub steps: Vec<String>,
    /// Can this be auto-fixed?
    pub auto_fixable: bool,
    /// Related issue IDs
    pub related_issues: Vec<String>,
}

/// Recommendation priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationPriority {
    /// Critical - should fix immediately
    Critical,
    /// High priority
    High,
    /// Medium priority
    Medium,
    /// Low priority
    Low,
    /// Informational
    Info,
}

/// Recommendation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum RecommendationCategory {
    /// Network configuration
    Network,
    /// WiFi settings
    Wifi,
    /// DNS configuration
    Dns,
    /// Security
    Security,
    /// Performance
    Performance,
    /// Hardware/driver
    Hardware,
    /// System settings
    System,
    /// ISP-related
    Isp,
}

/// Report metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Tool version
    pub version: String,
    /// Operating system
    pub os: String,
    /// Hostname
    pub hostname: Option<String>,
    /// User who ran the report
    pub user: Option<String>,
    /// Command line arguments
    pub args: Vec<String>,
    /// Output format
    pub format: Option<ReportFormat>,
    /// Output file path
    pub output_path: Option<PathBuf>,
}
