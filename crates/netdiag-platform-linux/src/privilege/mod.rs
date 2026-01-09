//! Linux privilege provider implementation.

use async_trait::async_trait;
use netdiag_platform::PrivilegeProvider;
use netdiag_types::error::Result;
use netdiag_types::system::{Capability, PrivilegeLevel};
use std::fs;
use tracing::debug;

/// Linux privilege provider using capabilities and euid checks.
pub struct LinuxPrivilegeProvider;

impl LinuxPrivilegeProvider {
    /// Creates a new Linux privilege provider.
    pub fn new() -> Self {
        Self
    }

    /// Check if a specific capability is available.
    fn has_capability(&self, cap_name: &str) -> bool {
        // Read /proc/self/status and check CapEff (effective capabilities)
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("CapEff:") {
                    if let Some(hex) = line.split_whitespace().nth(1) {
                        if let Ok(caps) = u64::from_str_radix(hex, 16) {
                            // Check specific capability bit
                            let cap_bit = match cap_name {
                                "CAP_NET_RAW" => 13,
                                "CAP_NET_ADMIN" => 12,
                                "CAP_SYS_ADMIN" => 21,
                                "CAP_DAC_OVERRIDE" => 1,
                                _ => return false,
                            };
                            return (caps >> cap_bit) & 1 == 1;
                        }
                    }
                }
            }
        }
        false
    }
}

impl Default for LinuxPrivilegeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PrivilegeProvider for LinuxPrivilegeProvider {
    async fn current_level(&self) -> Result<PrivilegeLevel> {
        debug!("Checking privilege level on Linux");

        let euid = unsafe { libc::geteuid() };

        if euid == 0 {
            return Ok(PrivilegeLevel::Root);
        }

        // Check if we have some elevated capabilities even without being root
        if self.has_capability("CAP_NET_RAW") || self.has_capability("CAP_NET_ADMIN") {
            return Ok(PrivilegeLevel::Elevated);
        }

        Ok(PrivilegeLevel::User)
    }

    async fn has_capability(&self, capability: Capability) -> Result<bool> {
        debug!("Checking capability {:?} on Linux", capability);

        let euid = unsafe { libc::geteuid() };
        if euid == 0 {
            // Root has all capabilities
            return Ok(true);
        }

        let result = match capability {
            Capability::RawSockets => self.has_capability("CAP_NET_RAW"),
            Capability::PromiscuousMode => {
                self.has_capability("CAP_NET_RAW") || self.has_capability("CAP_NET_ADMIN")
            }
            Capability::ModifyNetworkConfig => self.has_capability("CAP_NET_ADMIN"),
            Capability::AccessSystemConfig => {
                self.has_capability("CAP_DAC_OVERRIDE") || self.has_capability("CAP_SYS_ADMIN")
            }
            Capability::InstallService => {
                // Generally requires root on Linux
                false
            }
            Capability::ModifySystemFiles => self.has_capability("CAP_DAC_OVERRIDE"),
        };

        Ok(result)
    }

    async fn request_elevation(&self) -> Result<bool> {
        debug!("Elevation request on Linux");
        // On Linux, we can't elevate at runtime without external tools
        // The user would need to run with sudo or pkexec
        Ok(false)
    }

    async fn can_elevate(&self) -> Result<bool> {
        debug!("Checking if elevation is possible on Linux");

        // Check if sudo is available and user is in sudoers
        let output = std::process::Command::new("sudo")
            .args(["-n", "true"])
            .output();

        match output {
            Ok(out) => Ok(out.status.success()),
            Err(_) => Ok(false),
        }
    }

    async fn elevation_method(&self) -> Result<Option<String>> {
        // Check available elevation methods
        if std::process::Command::new("pkexec")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Ok(Some("pkexec".to_string()));
        }

        if std::process::Command::new("sudo")
            .arg("--version")
            .output()
            .is_ok()
        {
            return Ok(Some("sudo".to_string()));
        }

        Ok(None)
    }

    async fn drop_privileges(&self) -> Result<()> {
        debug!("Dropping privileges on Linux");

        // Get the original user's UID from SUDO_UID environment variable
        if let Ok(uid_str) = std::env::var("SUDO_UID") {
            if let Ok(uid) = uid_str.parse::<libc::uid_t>() {
                unsafe {
                    if libc::setuid(uid) != 0 {
                        return Err(netdiag_types::Error::platform(
                            "setuid",
                            "Failed to drop privileges",
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_current_level() {
        let provider = LinuxPrivilegeProvider::new();
        let level = provider.current_level().await.unwrap();
        // Should at least return a valid level
        assert!(matches!(
            level,
            PrivilegeLevel::User | PrivilegeLevel::Elevated | PrivilegeLevel::Root
        ));
    }
}
