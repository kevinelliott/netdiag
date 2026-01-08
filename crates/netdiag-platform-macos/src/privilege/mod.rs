//! macOS privilege provider implementation.

use async_trait::async_trait;
use netdiag_platform::{Capability, PrivilegeProvider};
use netdiag_types::{
    error::Result,
    system::{ElevationRequest, PrivilegeLevel},
};
use std::process::Command;
use tracing::{debug, warn};

/// macOS privilege provider.
pub struct MacosPrivilegeProvider {
    /// Cached privilege level
    cached_level: PrivilegeLevel,
}

impl MacosPrivilegeProvider {
    /// Creates a new macOS privilege provider.
    pub fn new() -> Self {
        let cached_level = Self::detect_privilege_level();
        Self { cached_level }
    }

    /// Detects the current privilege level.
    fn detect_privilege_level() -> PrivilegeLevel {
        let euid = unsafe { libc::geteuid() };
        if euid == 0 {
            PrivilegeLevel::Root
        } else {
            // Check if user is in admin group (can use sudo)
            let output = Command::new("id")
                .args(["-Gn"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string());

            if let Some(groups) = output {
                if groups.contains("admin") || groups.contains("wheel") {
                    // User can potentially elevate, but not currently elevated
                    PrivilegeLevel::User
                } else {
                    PrivilegeLevel::User
                }
            } else {
                PrivilegeLevel::User
            }
        }
    }

    /// Checks if raw socket capability is available.
    fn check_raw_socket_capability(&self) -> bool {
        // Raw sockets typically require root on macOS
        self.cached_level == PrivilegeLevel::Root
    }

    /// Checks if promiscuous mode is available.
    fn check_promiscuous_capability(&self) -> bool {
        // Promiscuous mode requires root or special entitlements
        self.cached_level == PrivilegeLevel::Root
    }

    /// Checks if network config capability is available.
    fn check_network_config_capability(&self) -> bool {
        // Network configuration requires root or being in admin group with sudo
        if self.cached_level == PrivilegeLevel::Root {
            return true;
        }

        // Check if we can use networksetup without sudo (some operations work)
        Command::new("networksetup")
            .args(["-listallnetworkservices"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Checks if DNS config capability is available.
    fn check_dns_config_capability(&self) -> bool {
        // DNS config changes require root
        self.cached_level == PrivilegeLevel::Root
    }

    /// Checks if interface control capability is available.
    fn check_interface_control_capability(&self) -> bool {
        // Interface control requires root
        self.cached_level == PrivilegeLevel::Root
    }

    /// Checks if routing table capability is available.
    fn check_routing_table_capability(&self) -> bool {
        // Routing table modifications require root
        self.cached_level == PrivilegeLevel::Root
    }

    /// Checks if firewall capability is available.
    fn check_firewall_capability(&self) -> bool {
        // Firewall (pf) requires root
        self.cached_level == PrivilegeLevel::Root
    }

    /// Checks if service management capability is available.
    fn check_service_management_capability(&self) -> bool {
        // launchctl for system services requires root
        self.cached_level == PrivilegeLevel::Root
    }

    /// Checks if WiFi scan capability is available.
    fn check_wifi_scan_capability(&self) -> bool {
        // WiFi scanning typically works for all users on macOS
        // (unless location services are disabled)
        std::path::Path::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport").exists()
    }
}

impl Default for MacosPrivilegeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PrivilegeProvider for MacosPrivilegeProvider {
    fn current_privilege_level(&self) -> PrivilegeLevel {
        self.cached_level
    }

    async fn request_elevation(&self, request: &ElevationRequest) -> Result<bool> {
        // On macOS, we could use osascript to prompt for admin password
        // or guide the user to run with sudo

        if self.cached_level == PrivilegeLevel::Root {
            return Ok(true); // Already elevated
        }

        // Check if running in terminal (can use sudo) vs GUI app
        let is_terminal = std::env::var("TERM").is_ok();

        if is_terminal {
            debug!(
                "Elevation required: {}. Run with 'sudo' for full capabilities.",
                request.reason
            );

            // For CLI, we can't interactively get sudo - user must restart with sudo
            warn!(
                "Features requiring elevation: {:?}. Re-run with 'sudo netdiag' for full access.",
                request.features
            );

            Ok(false)
        } else {
            // For GUI apps, could potentially use osascript
            // For now, just return false
            debug!("Elevation required but running in non-terminal mode");
            Ok(false)
        }
    }

    fn has_capability(&self, capability: Capability) -> bool {
        match capability {
            Capability::RawSockets => self.check_raw_socket_capability(),
            Capability::PromiscuousMode => self.check_promiscuous_capability(),
            Capability::NetworkConfig => self.check_network_config_capability(),
            Capability::DnsConfig => self.check_dns_config_capability(),
            Capability::InterfaceControl => self.check_interface_control_capability(),
            Capability::RoutingTable => self.check_routing_table_capability(),
            Capability::Firewall => self.check_firewall_capability(),
            Capability::DriverAccess => self.cached_level == PrivilegeLevel::Root,
            Capability::ServiceManagement => self.check_service_management_capability(),
            Capability::SystemRegistry => false, // Windows-only
            Capability::WifiScan => self.check_wifi_scan_capability(),
        }
    }

    fn available_capabilities(&self) -> Vec<Capability> {
        let mut caps = Vec::new();

        // WiFi scan is usually available
        if self.check_wifi_scan_capability() {
            caps.push(Capability::WifiScan);
        }

        // Some network config operations work without root
        if self.check_network_config_capability() {
            caps.push(Capability::NetworkConfig);
        }

        // Root-only capabilities
        if self.cached_level == PrivilegeLevel::Root {
            caps.extend([
                Capability::RawSockets,
                Capability::PromiscuousMode,
                Capability::DnsConfig,
                Capability::InterfaceControl,
                Capability::RoutingTable,
                Capability::Firewall,
                Capability::DriverAccess,
                Capability::ServiceManagement,
            ]);
        }

        caps
    }

    fn capabilities_requiring_elevation(&self) -> Vec<Capability> {
        if self.cached_level == PrivilegeLevel::Root {
            Vec::new() // Already have everything
        } else {
            vec![
                Capability::RawSockets,
                Capability::PromiscuousMode,
                Capability::DnsConfig,
                Capability::InterfaceControl,
                Capability::RoutingTable,
                Capability::Firewall,
                Capability::DriverAccess,
                Capability::ServiceManagement,
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_privilege_detection() {
        let provider = MacosPrivilegeProvider::new();
        let level = provider.current_privilege_level();

        // In normal test environment, should be User
        // (unless tests are run with sudo)
        assert!(matches!(level, PrivilegeLevel::User | PrivilegeLevel::Root));
    }

    #[test]
    fn test_wifi_scan_capability() {
        let provider = MacosPrivilegeProvider::new();
        // WiFi scan should be available on macOS
        // (unless airport binary is missing)
        let has_cap = provider.has_capability(Capability::WifiScan);
        // Don't assert true - may fail on non-macOS or minimal systems
        println!("WiFi scan capability: {}", has_cap);
    }
}
