//! Android system information provider.

use async_trait::async_trait;
use netdiag_platform::SystemInfoProvider;
use netdiag_types::{
    error::Result,
    system::{OsType, SystemInfo},
};
use std::time::Duration;

/// Android system information provider.
///
/// Uses Android's Build class and other system APIs through JNI
/// to get system information.
pub struct AndroidSystemInfoProvider {
    // JNI references would be stored here
}

impl AndroidSystemInfoProvider {
    /// Creates a new Android system info provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the Android version.
    ///
    /// In a real implementation, would use Build.VERSION.RELEASE
    fn get_android_version(&self) -> String {
        // Would use JNI to get Build.VERSION.RELEASE
        "Android".to_string()
    }

    /// Gets the device model.
    ///
    /// In a real implementation, would use Build.MODEL
    fn get_device_model(&self) -> String {
        // Would use JNI to get Build.MODEL
        "Android Device".to_string()
    }
}

impl Default for AndroidSystemInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemInfoProvider for AndroidSystemInfoProvider {
    async fn get_system_info(&self) -> Result<SystemInfo> {
        Ok(SystemInfo {
            hostname: self.get_hostname().await?,
            os_type: OsType::Android,
            os_version: self.get_android_version(),
            os_build: None,       // Would be Build.ID
            kernel_version: None, // Would be from /proc/version
            architecture: std::env::consts::ARCH.to_string(),
            cpu: None,
            memory: None,
            uptime: self.get_uptime().await.ok(),
        })
    }

    async fn get_hostname(&self) -> Result<String> {
        // Android devices typically use Settings.Global.DEVICE_NAME
        // or Build.MODEL for identification
        Ok(self.get_device_model())
    }

    async fn get_uptime(&self) -> Result<Duration> {
        // Would use SystemClock.elapsedRealtime()
        // For now, return zero
        Ok(Duration::from_secs(0))
    }

    fn get_timezone(&self) -> String {
        // Would use TimeZone.getDefault().getID()
        "UTC".to_string()
    }
}
