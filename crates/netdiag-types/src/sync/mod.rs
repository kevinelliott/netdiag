//! Cloud sync types.

use crate::config::CloudProvider;
use serde::{Deserialize, Serialize};

/// Sync configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Cloud provider
    pub provider: CloudProvider,
    /// Project ID (for Firebase)
    pub project_id: Option<String>,
    /// API key or token
    pub api_key: Option<String>,
    /// Sync enabled
    pub enabled: bool,
    /// Sync interval in seconds
    pub interval_secs: u64,
    /// Last sync timestamp
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            provider: CloudProvider::Firebase,
            project_id: None,
            api_key: None,
            enabled: false,
            interval_secs: 300, // 5 minutes
            last_sync: None,
        }
    }
}

/// Sync status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum SyncStatus {
    /// Not configured
    NotConfigured,
    /// Idle (not syncing)
    Idle,
    /// Currently syncing
    Syncing,
    /// Sync successful
    Success,
    /// Sync failed
    Failed,
    /// Offline
    Offline,
}

/// Sync result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    /// Sync status
    pub status: SyncStatus,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Number of records uploaded
    pub uploaded: u32,
    /// Number of records downloaded
    pub downloaded: u32,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Duration of sync
    pub duration: std::time::Duration,
}

/// A record to be synced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRecord {
    /// Record ID
    pub id: uuid::Uuid,
    /// Record type
    pub record_type: SyncRecordType,
    /// Record data
    pub data: serde_json::Value,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Sync status
    pub sync_status: RecordSyncStatus,
}

/// Type of sync record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum SyncRecordType {
    /// Diagnostic report
    Report,
    /// Configuration
    Config,
    /// History entry
    History,
    /// Device info
    Device,
}

/// Sync status of a record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::Display, Default)]
#[serde(rename_all = "lowercase")]
pub enum RecordSyncStatus {
    /// Pending sync
    #[default]
    Pending,
    /// Synced
    Synced,
    /// Conflict
    Conflict,
    /// Failed
    Failed,
}
