//! iOS system information provider.

use async_trait::async_trait;
use netdiag_platform::SystemInfoProvider;
use netdiag_types::{
    error::Result,
    system::{OsType, SystemInfo},
};
use std::time::Duration;

/// iOS system information provider.
pub struct IosSystemInfoProvider {
    // No persistent state
}

impl IosSystemInfoProvider {
    /// Creates a new iOS system info provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the device model identifier.
    fn get_device_model(&self) -> String {
        // In a real implementation, you'd use utsname or UIDevice
        // For now, return a generic iOS identifier
        "iOS Device".to_string()
    }

    /// Gets the iOS version.
    fn get_ios_version(&self) -> String {
        // In a real implementation, use UIDevice.current.systemVersion
        // or ProcessInfo.processInfo.operatingSystemVersion
        "iOS".to_string()
    }
}

impl Default for IosSystemInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemInfoProvider for IosSystemInfoProvider {
    async fn get_system_info(&self) -> Result<SystemInfo> {
        Ok(SystemInfo {
            hostname: self.get_hostname().await?,
            os_type: OsType::IOS,
            os_version: self.get_ios_version(),
            os_build: None,
            kernel_version: None,
            architecture: std::env::consts::ARCH.to_string(),
            cpu: None,
            memory: None,
            uptime: self.get_uptime().await.ok(),
        })
    }

    async fn get_hostname(&self) -> Result<String> {
        // iOS apps can get device name through UIDevice
        // For security/privacy, we return a generic name
        Ok("iPhone".to_string())
    }

    async fn get_uptime(&self) -> Result<Duration> {
        // iOS provides uptime through ProcessInfo
        // This is a placeholder - real implementation would use:
        // ProcessInfo.processInfo.systemUptime
        Ok(Duration::from_secs(0))
    }

    fn get_timezone(&self) -> String {
        // Get current timezone
        // In real implementation: TimeZone.current.identifier
        "UTC".to_string()
    }
}
