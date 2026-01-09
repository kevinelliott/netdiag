//! System information provider trait.

use async_trait::async_trait;
use netdiag_types::{error::Result, system::SystemInfo};

/// Provider for system information.
#[async_trait]
pub trait SystemInfoProvider: Send + Sync {
    /// Gets system information.
    async fn get_system_info(&self) -> Result<SystemInfo>;

    /// Gets the hostname.
    async fn get_hostname(&self) -> Result<String>;

    /// Gets system uptime.
    async fn get_uptime(&self) -> Result<std::time::Duration>;

    /// Gets the current time zone.
    fn get_timezone(&self) -> String;

    /// Gets the netdiag version.
    fn get_version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// Extension trait for system information.
#[async_trait]
pub trait SystemInfoProviderExt: SystemInfoProvider {
    /// Checks if the system is a virtual machine.
    async fn is_virtual_machine(&self) -> bool {
        // This can be overridden by platform-specific implementations
        false
    }

    /// Gets a formatted system summary.
    async fn get_summary(&self) -> Result<String> {
        let info = self.get_system_info().await?;
        Ok(format!(
            "{} {} ({}) - {}",
            info.os_type, info.os_version, info.architecture, info.hostname
        ))
    }
}

// Blanket implementation
impl<T: SystemInfoProvider + ?Sized> SystemInfoProviderExt for T {}
