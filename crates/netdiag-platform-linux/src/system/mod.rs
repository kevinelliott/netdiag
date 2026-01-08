//! Linux system info provider implementation.

use async_trait::async_trait;
use netdiag_platform::SystemInfoProvider;
use netdiag_types::error::Result;
use netdiag_types::system::{CpuInfo, MemoryInfo, OsInfo, SystemInfo};
use std::fs;
use std::process::Command;
use tracing::debug;

/// Linux system info provider using /proc filesystem and system commands.
pub struct LinuxSystemInfoProvider;

impl LinuxSystemInfoProvider {
    /// Creates a new Linux system info provider.
    pub fn new() -> Self {
        Self
    }

    /// Parse /etc/os-release for OS info.
    fn parse_os_release(&self) -> Option<(String, String, String)> {
        let content = fs::read_to_string("/etc/os-release").ok()?;

        let mut name = None;
        let mut version = None;
        let mut id = None;

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("NAME=") {
                name = Some(value.trim_matches('"').to_string());
            } else if let Some(value) = line.strip_prefix("VERSION=") {
                version = Some(value.trim_matches('"').to_string());
            } else if let Some(value) = line.strip_prefix("VERSION_ID=") {
                id = Some(value.trim_matches('"').to_string());
            }
        }

        Some((
            name.unwrap_or_else(|| "Linux".to_string()),
            version.unwrap_or_default(),
            id.unwrap_or_default(),
        ))
    }

    /// Parse /proc/cpuinfo.
    fn parse_cpuinfo(&self) -> Option<CpuInfo> {
        let content = fs::read_to_string("/proc/cpuinfo").ok()?;

        let mut model = None;
        let mut cores = 0u32;
        let mut speed = None;

        for line in content.lines() {
            if line.starts_with("model name") {
                if model.is_none() {
                    model = line.split(':').nth(1).map(|s| s.trim().to_string());
                }
            } else if line.starts_with("processor") {
                cores += 1;
            } else if line.starts_with("cpu MHz") {
                if speed.is_none() {
                    speed = line
                        .split(':')
                        .nth(1)
                        .and_then(|s| s.trim().parse::<f64>().ok())
                        .map(|mhz| (mhz * 1_000_000.0) as u64);
                }
            }
        }

        Some(CpuInfo {
            model: model.unwrap_or_else(|| "Unknown".to_string()),
            cores,
            speed_hz: speed,
            architecture: std::env::consts::ARCH.to_string(),
        })
    }

    /// Parse /proc/meminfo.
    fn parse_meminfo(&self) -> Option<MemoryInfo> {
        let content = fs::read_to_string("/proc/meminfo").ok()?;

        let mut total = 0u64;
        let mut available = 0u64;

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let value: u64 = parts[1].parse().unwrap_or(0);
                // Values are in KB, convert to bytes
                let value_bytes = value * 1024;

                match parts[0].trim_end_matches(':') {
                    "MemTotal" => total = value_bytes,
                    "MemAvailable" => available = value_bytes,
                    _ => {}
                }
            }
        }

        Some(MemoryInfo {
            total,
            available,
            used: total.saturating_sub(available),
        })
    }

    /// Get kernel version from uname.
    fn get_kernel_version(&self) -> Option<String> {
        let output = Command::new("uname").arg("-r").output().ok()?;
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get hostname.
    fn get_hostname(&self) -> Option<String> {
        fs::read_to_string("/etc/hostname")
            .ok()
            .map(|s| s.trim().to_string())
            .or_else(|| {
                Command::new("hostname")
                    .output()
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            })
    }

    /// Get system uptime from /proc/uptime.
    fn get_uptime(&self) -> Option<std::time::Duration> {
        let content = fs::read_to_string("/proc/uptime").ok()?;
        let secs: f64 = content.split_whitespace().next()?.parse().ok()?;
        Some(std::time::Duration::from_secs_f64(secs))
    }

    /// Get system boot time.
    fn get_boot_time(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        let uptime = self.get_uptime()?;
        let now = chrono::Utc::now();
        Some(now - chrono::Duration::from_std(uptime).ok()?)
    }
}

impl Default for LinuxSystemInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemInfoProvider for LinuxSystemInfoProvider {
    async fn get_system_info(&self) -> Result<SystemInfo> {
        debug!("Getting system info on Linux");

        let (os_name, os_version, os_build) = self
            .parse_os_release()
            .unwrap_or_else(|| ("Linux".to_string(), String::new(), String::new()));

        let kernel_version = self.get_kernel_version();

        Ok(SystemInfo {
            os: OsInfo {
                name: os_name,
                version: os_version,
                build: os_build,
                kernel_version,
            },
            hostname: self.get_hostname(),
            cpu: self.parse_cpuinfo(),
            memory: self.parse_meminfo(),
            uptime: self.get_uptime(),
            boot_time: self.get_boot_time(),
        })
    }

    async fn get_hostname(&self) -> Result<Option<String>> {
        Ok(self.get_hostname())
    }

    async fn get_uptime(&self) -> Result<Option<std::time::Duration>> {
        Ok(self.get_uptime())
    }

    async fn get_os_info(&self) -> Result<OsInfo> {
        let (name, version, build) = self
            .parse_os_release()
            .unwrap_or_else(|| ("Linux".to_string(), String::new(), String::new()));

        Ok(OsInfo {
            name,
            version,
            build,
            kernel_version: self.get_kernel_version(),
        })
    }

    async fn get_cpu_info(&self) -> Result<Option<CpuInfo>> {
        Ok(self.parse_cpuinfo())
    }

    async fn get_memory_info(&self) -> Result<Option<MemoryInfo>> {
        Ok(self.parse_meminfo())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_system_info() {
        let provider = LinuxSystemInfoProvider::new();
        let info = provider.get_system_info().await.unwrap();
        assert!(!info.os.name.is_empty());
    }
}
