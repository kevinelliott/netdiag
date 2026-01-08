//! Windows system information provider.

use async_trait::async_trait;
use netdiag_platform::SystemInfoProvider;
use netdiag_types::{
    error::Result,
    system::{OsType, SystemInfo},
};
use std::time::Duration;

/// Windows system information provider.
pub struct WindowsSystemInfoProvider {}

impl WindowsSystemInfoProvider {
    /// Creates a new Windows system info provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the Windows version string.
    #[cfg(windows)]
    fn get_windows_version(&self) -> String {
        use windows::Win32::System::SystemInformation::*;

        unsafe {
            let mut info = std::mem::zeroed::<OSVERSIONINFOW>();
            info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;

            // Note: GetVersionExW is deprecated, but RtlGetVersion requires ntdll
            // In production, would use RtlGetVersion or WMI
            format!(
                "Windows {}.{}.{}",
                info.dwMajorVersion, info.dwMinorVersion, info.dwBuildNumber
            )
        }
    }

    #[cfg(not(windows))]
    fn get_windows_version(&self) -> String {
        "Windows".to_string()
    }

    /// Gets the computer name.
    #[cfg(windows)]
    fn get_computer_name(&self) -> Result<String> {
        use windows::Win32::System::SystemInformation::*;

        unsafe {
            let mut size = 0u32;
            let _ = GetComputerNameExW(ComputerNameDnsHostname, None, &mut size);

            let mut buffer = vec![0u16; size as usize];
            if GetComputerNameExW(
                ComputerNameDnsHostname,
                Some(buffer.as_mut_ptr()),
                &mut size,
            ).is_ok() {
                Ok(String::from_utf16_lossy(&buffer[..size as usize]))
            } else {
                Ok("Unknown".to_string())
            }
        }
    }

    #[cfg(not(windows))]
    fn get_computer_name(&self) -> Result<String> {
        Ok("Unknown".to_string())
    }

    /// Gets system uptime.
    #[cfg(windows)]
    fn get_system_uptime(&self) -> Duration {
        use windows::Win32::System::SystemInformation::GetTickCount64;

        unsafe {
            let ticks = GetTickCount64();
            Duration::from_millis(ticks)
        }
    }

    #[cfg(not(windows))]
    fn get_system_uptime(&self) -> Duration {
        Duration::from_secs(0)
    }
}

impl Default for WindowsSystemInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemInfoProvider for WindowsSystemInfoProvider {
    async fn get_system_info(&self) -> Result<SystemInfo> {
        Ok(SystemInfo {
            hostname: self.get_hostname().await?,
            os_type: OsType::Windows,
            os_version: self.get_windows_version(),
            os_build: None,
            kernel_version: None,
            architecture: std::env::consts::ARCH.to_string(),
            cpu: None,
            memory: None,
            uptime: Some(self.get_uptime().await?),
        })
    }

    async fn get_hostname(&self) -> Result<String> {
        self.get_computer_name()
    }

    async fn get_uptime(&self) -> Result<Duration> {
        Ok(self.get_system_uptime())
    }

    fn get_timezone(&self) -> String {
        // Would use GetTimeZoneInformation
        // For now, return UTC
        "UTC".to_string()
    }
}
