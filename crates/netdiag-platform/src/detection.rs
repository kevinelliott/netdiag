//! Platform detection utilities.

use netdiag_types::system::OsType;

/// Detected platform information.
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// Operating system type
    pub os_type: OsType,
    /// OS version string
    pub os_version: String,
    /// Architecture
    pub arch: &'static str,
    /// Is running in a container?
    pub is_container: bool,
    /// Is running in a virtual machine?
    pub is_vm: bool,
}

impl PlatformInfo {
    /// Detects the current platform.
    #[must_use]
    pub fn detect() -> Self {
        Self {
            os_type: OsType::current(),
            os_version: Self::detect_os_version(),
            arch: std::env::consts::ARCH,
            is_container: Self::detect_container(),
            is_vm: Self::detect_vm(),
        }
    }

    fn detect_os_version() -> String {
        #[cfg(target_os = "macos")]
        {
            // Try sw_vers
            if let Ok(output) = std::process::Command::new("sw_vers")
                .arg("-productVersion")
                .output()
            {
                if output.status.success() {
                    return String::from_utf8_lossy(&output.stdout).trim().to_string();
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Try /etc/os-release
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if let Some(version) = line.strip_prefix("VERSION_ID=") {
                        return version.trim_matches('"').to_string();
                    }
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Would use winapi to get version
        }

        "unknown".to_string()
    }

    fn detect_container() -> bool {
        // Check for Docker
        if std::path::Path::new("/.dockerenv").exists() {
            return true;
        }

        // Check for container in cgroup
        if let Ok(cgroup) = std::fs::read_to_string("/proc/1/cgroup") {
            if cgroup.contains("docker") || cgroup.contains("lxc") || cgroup.contains("kubepods") {
                return true;
            }
        }

        false
    }

    fn detect_vm() -> bool {
        #[cfg(target_os = "macos")]
        {
            // Check for VM indicators
            if let Ok(output) = std::process::Command::new("sysctl")
                .arg("-n")
                .arg("machdep.cpu.brand_string")
                .output()
            {
                let brand = String::from_utf8_lossy(&output.stdout);
                if brand.contains("Virtual") || brand.contains("QEMU") {
                    return true;
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Check /sys/class/dmi/id/product_name
            if let Ok(product) = std::fs::read_to_string("/sys/class/dmi/id/product_name") {
                let product = product.to_lowercase();
                if product.contains("virtual")
                    || product.contains("vmware")
                    || product.contains("kvm")
                    || product.contains("qemu")
                {
                    return true;
                }
            }

            // Check for hypervisor flag in cpuinfo
            if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
                if cpuinfo.contains("hypervisor") {
                    return true;
                }
            }
        }

        false
    }
}

/// Returns the current platform name.
#[must_use]
pub fn platform_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "linux")]
    {
        if std::path::Path::new("/system/build.prop").exists() {
            "android"
        } else {
            "linux"
        }
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "ios")]
    {
        "ios"
    }
    #[cfg(not(any(
        target_os = "macos",
        target_os = "linux",
        target_os = "windows",
        target_os = "ios"
    )))]
    {
        "unknown"
    }
}

/// Returns true if running on a mobile platform.
#[must_use]
pub fn is_mobile() -> bool {
    cfg!(any(target_os = "ios", target_os = "android"))
}

/// Returns true if running on a Unix-like platform.
#[must_use]
pub fn is_unix() -> bool {
    cfg!(unix)
}

/// Returns true if running on Windows.
#[must_use]
pub fn is_windows() -> bool {
    cfg!(windows)
}
