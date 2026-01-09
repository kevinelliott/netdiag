//! macOS system information provider implementation.

use async_trait::async_trait;
use netdiag_platform::SystemInfoProvider;
use netdiag_types::{
    error::{Error, Result},
    system::{CpuInfo, MemoryInfo, OsType, SystemInfo},
};
use std::process::Command;
use std::time::Duration;

/// macOS system information provider.
pub struct MacosSystemInfoProvider {
    /// Cached system info (expensive to gather)
    cache: std::sync::RwLock<Option<SystemInfo>>,
}

impl MacosSystemInfoProvider {
    /// Creates a new macOS system info provider.
    pub fn new() -> Self {
        Self {
            cache: std::sync::RwLock::new(None),
        }
    }

    /// Gets macOS version from sw_vers.
    fn get_macos_version() -> Option<(String, String)> {
        let version_output = Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()?;

        let version = String::from_utf8_lossy(&version_output.stdout)
            .trim()
            .to_string();

        let build_output = Command::new("sw_vers").arg("-buildVersion").output().ok()?;

        let build = String::from_utf8_lossy(&build_output.stdout)
            .trim()
            .to_string();

        Some((version, build))
    }

    /// Gets the kernel version.
    fn get_kernel_version() -> Option<String> {
        let output = Command::new("uname").arg("-r").output().ok()?;

        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Gets the architecture.
    fn get_architecture() -> String {
        let output = Command::new("uname")
            .arg("-m")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Translate to more readable format
        match output.as_str() {
            "arm64" => "Apple Silicon (arm64)".to_string(),
            "x86_64" => "Intel x86_64".to_string(),
            other => other.to_string(),
        }
    }

    /// Gets CPU info using sysctl.
    fn get_cpu_info() -> Option<CpuInfo> {
        // Get CPU brand
        let brand = Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Get core count
        let cores: u32 = Command::new("sysctl")
            .args(["-n", "hw.physicalcpu"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
            .unwrap_or(1);

        // Get thread count
        let threads: u32 = Command::new("sysctl")
            .args(["-n", "hw.logicalcpu"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
            .unwrap_or(cores);

        // Get CPU frequency (may not be available on Apple Silicon)
        let frequency_mhz: Option<u32> = Command::new("sysctl")
            .args(["-n", "hw.cpufrequency"])
            .output()
            .ok()
            .and_then(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .trim()
                    .parse::<u64>()
                    .ok()
                    .map(|hz| (hz / 1_000_000) as u32)
            });

        Some(CpuInfo {
            model: brand,
            cores,
            threads,
            frequency_mhz,
        })
    }

    /// Gets memory info using sysctl.
    fn get_memory_info() -> Option<MemoryInfo> {
        // Get total memory
        let total: u64 = Command::new("sysctl")
            .args(["-n", "hw.memsize"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
            .unwrap_or(0);

        // Get page size
        let page_size: u64 = Command::new("sysctl")
            .args(["-n", "hw.pagesize"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8_lossy(&o.stdout).trim().parse().ok())
            .unwrap_or(4096);

        // Get VM statistics for used/available memory
        let vm_output = Command::new("vm_stat").output().ok();

        let (available, used) = if let Some(output) = vm_output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut free_pages: u64 = 0;
            let mut inactive_pages: u64 = 0;
            let mut speculative_pages: u64 = 0;
            let mut wired_pages: u64 = 0;
            let mut active_pages: u64 = 0;
            let mut compressed_pages: u64 = 0;

            for line in stdout.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let value = value
                        .trim()
                        .trim_end_matches('.')
                        .parse::<u64>()
                        .unwrap_or(0);

                    match key.trim() {
                        "Pages free" => free_pages = value,
                        "Pages inactive" => inactive_pages = value,
                        "Pages speculative" => speculative_pages = value,
                        "Pages wired down" => wired_pages = value,
                        "Pages active" => active_pages = value,
                        "Pages occupied by compressor" => compressed_pages = value,
                        _ => {}
                    }
                }
            }

            let available = (free_pages + inactive_pages + speculative_pages) * page_size;
            let used = (active_pages + wired_pages + compressed_pages) * page_size;
            (available, used)
        } else {
            (0, 0)
        };

        Some(MemoryInfo {
            total,
            available,
            used,
        })
    }

    /// Gets system uptime.
    fn get_system_uptime() -> Option<Duration> {
        // Use sysctl to get boot time
        let output = Command::new("sysctl")
            .args(["-n", "kern.boottime"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: { sec = 1234567890, usec = 123456 }
        if let Some(sec_start) = stdout.find("sec = ") {
            let sec_str = &stdout[sec_start + 6..];
            if let Some(sec_end) = sec_str.find(',') {
                if let Ok(boot_sec) = sec_str[..sec_end].trim().parse::<u64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()?
                        .as_secs();
                    return Some(Duration::from_secs(now - boot_sec));
                }
            }
        }

        None
    }
}

impl Default for MacosSystemInfoProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SystemInfoProvider for MacosSystemInfoProvider {
    async fn get_system_info(&self) -> Result<SystemInfo> {
        // Check cache first
        if let Ok(guard) = self.cache.read() {
            if let Some(ref cached) = *guard {
                return Ok(cached.clone());
            }
        }

        let hostname = self.get_hostname().await?;
        let (os_version, os_build) =
            Self::get_macos_version().unwrap_or_else(|| ("Unknown".to_string(), String::new()));

        let info = SystemInfo {
            hostname,
            os_type: OsType::MacOS,
            os_version,
            os_build: Some(os_build),
            kernel_version: Self::get_kernel_version(),
            architecture: Self::get_architecture(),
            cpu: Self::get_cpu_info(),
            memory: Self::get_memory_info(),
            uptime: Self::get_system_uptime(),
        };

        // Cache the result
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some(info.clone());
        }

        Ok(info)
    }

    async fn get_hostname(&self) -> Result<String> {
        let output = Command::new("hostname")
            .output()
            .map_err(|e| Error::Other {
                context: "hostname".to_string(),
                message: e.to_string(),
            })?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    async fn get_uptime(&self) -> Result<Duration> {
        Self::get_system_uptime().ok_or_else(|| Error::Other {
            context: "uptime".to_string(),
            message: "Could not determine system uptime".to_string(),
        })
    }

    fn get_timezone(&self) -> String {
        // Get timezone from date command
        Command::new("date")
            .args(["+%Z"])
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|| "UTC".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_hostname() {
        let provider = MacosSystemInfoProvider::new();
        let hostname = provider.get_hostname().await;
        assert!(hostname.is_ok());
        let hostname = hostname.unwrap();
        assert!(!hostname.is_empty());
        println!("Hostname: {}", hostname);
    }

    #[tokio::test]
    async fn test_get_system_info() {
        let provider = MacosSystemInfoProvider::new();
        let info = provider.get_system_info().await;
        assert!(info.is_ok());
        let info = info.unwrap();
        println!("System: {:?}", info);
        assert_eq!(info.os_type, OsType::MacOS);
        assert!(!info.hostname.is_empty());
    }

    #[test]
    fn test_get_timezone() {
        let provider = MacosSystemInfoProvider::new();
        let tz = provider.get_timezone();
        assert!(!tz.is_empty());
        println!("Timezone: {}", tz);
    }
}
