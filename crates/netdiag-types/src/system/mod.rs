//! System-related types.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// System information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Hostname
    pub hostname: String,
    /// Operating system type
    pub os_type: OsType,
    /// OS version
    pub os_version: String,
    /// OS build/release
    pub os_build: Option<String>,
    /// Kernel version
    pub kernel_version: Option<String>,
    /// Architecture
    pub architecture: String,
    /// CPU info
    pub cpu: Option<CpuInfo>,
    /// Memory info
    pub memory: Option<MemoryInfo>,
    /// Uptime
    pub uptime: Option<std::time::Duration>,
}

/// Operating system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OsType {
    /// macOS
    MacOS,
    /// Linux
    Linux,
    /// Windows
    Windows,
    /// iOS
    IOS,
    /// iPadOS
    IPadOS,
    /// Android
    Android,
    /// Other/unknown
    Other,
}

impl OsType {
    /// Detects the current OS type.
    #[must_use]
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self::MacOS
        }
        #[cfg(target_os = "linux")]
        {
            // Could be Android, need to check
            if std::path::Path::new("/system/build.prop").exists() {
                Self::Android
            } else {
                Self::Linux
            }
        }
        #[cfg(target_os = "windows")]
        {
            Self::Windows
        }
        #[cfg(target_os = "ios")]
        {
            Self::IOS
        }
        #[cfg(not(any(
            target_os = "macos",
            target_os = "linux",
            target_os = "windows",
            target_os = "ios"
        )))]
        {
            Self::Other
        }
    }

    /// Returns true if this is a mobile OS.
    #[must_use]
    pub fn is_mobile(&self) -> bool {
        matches!(self, Self::IOS | Self::IPadOS | Self::Android)
    }

    /// Returns true if this is a Unix-like OS.
    #[must_use]
    pub fn is_unix(&self) -> bool {
        matches!(
            self,
            Self::MacOS | Self::Linux | Self::IOS | Self::IPadOS | Self::Android
        )
    }
}

/// CPU information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    /// CPU model name
    pub model: String,
    /// Number of physical cores
    pub cores: u32,
    /// Number of logical processors
    pub threads: u32,
    /// Base frequency in MHz
    pub frequency_mhz: Option<u32>,
}

/// Memory information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    /// Total memory in bytes
    pub total: u64,
    /// Available memory in bytes
    pub available: u64,
    /// Used memory in bytes
    pub used: u64,
}

/// Privilege level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum PrivilegeLevel {
    /// Regular user
    User,
    /// Elevated/administrator
    Elevated,
    /// Root/system
    Root,
}

impl PrivilegeLevel {
    /// Returns true if this level is elevated (admin or root).
    #[must_use]
    pub fn is_elevated(&self) -> bool {
        matches!(self, Self::Elevated | Self::Root)
    }
}

/// Elevation request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationRequest {
    /// Reason for elevation
    pub reason: String,
    /// Required privilege level
    pub required_level: PrivilegeLevel,
    /// Features that need elevation
    pub features: Vec<String>,
}

/// Rollback identifier for autofix operations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RollbackId(pub String);

impl RollbackId {
    /// Creates a new rollback ID.
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for RollbackId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for RollbackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
