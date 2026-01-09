//! Privilege provider trait.

use async_trait::async_trait;
use netdiag_types::{
    error::Result,
    system::{ElevationRequest, PrivilegeLevel},
};

/// Provider for privilege/elevation operations.
#[async_trait]
pub trait PrivilegeProvider: Send + Sync {
    /// Gets the current privilege level.
    fn current_privilege_level(&self) -> PrivilegeLevel;

    /// Checks if running with elevated privileges.
    fn is_elevated(&self) -> bool {
        self.current_privilege_level().is_elevated()
    }

    /// Requests privilege elevation.
    ///
    /// Returns `true` if elevation was granted, `false` if denied.
    async fn request_elevation(&self, request: &ElevationRequest) -> Result<bool>;

    /// Checks if a specific capability is available.
    fn has_capability(&self, capability: Capability) -> bool;

    /// Gets all available capabilities.
    fn available_capabilities(&self) -> Vec<Capability>;

    /// Gets capabilities that require elevation.
    fn capabilities_requiring_elevation(&self) -> Vec<Capability>;
}

/// System capability that may require privileges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Raw socket access (for ICMP ping)
    RawSockets,
    /// Promiscuous mode for packet capture
    PromiscuousMode,
    /// Network configuration changes
    NetworkConfig,
    /// DNS configuration changes
    DnsConfig,
    /// Interface control (enable/disable)
    InterfaceControl,
    /// Routing table modifications
    RoutingTable,
    /// Firewall rule changes
    Firewall,
    /// Driver access/updates
    DriverAccess,
    /// Service/daemon management
    ServiceManagement,
    /// System registry/config access (Windows)
    SystemRegistry,
    /// WiFi scanning (may require location permission on mobile)
    WifiScan,
}

impl Capability {
    /// Returns a human-readable description of the capability.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::RawSockets => "Raw socket access for ICMP operations",
            Self::PromiscuousMode => "Promiscuous mode for packet capture",
            Self::NetworkConfig => "Network configuration changes",
            Self::DnsConfig => "DNS configuration changes",
            Self::InterfaceControl => "Network interface control",
            Self::RoutingTable => "Routing table modifications",
            Self::Firewall => "Firewall rule changes",
            Self::DriverAccess => "Network driver access",
            Self::ServiceManagement => "Service/daemon management",
            Self::SystemRegistry => "System registry access",
            Self::WifiScan => "WiFi network scanning",
        }
    }

    /// Returns the typical privilege level required for this capability.
    #[must_use]
    pub const fn typical_required_level(&self) -> PrivilegeLevel {
        match self {
            Self::WifiScan => PrivilegeLevel::User,
            Self::RawSockets | Self::PromiscuousMode => PrivilegeLevel::Elevated,
            Self::NetworkConfig
            | Self::DnsConfig
            | Self::InterfaceControl
            | Self::RoutingTable
            | Self::Firewall
            | Self::DriverAccess
            | Self::ServiceManagement
            | Self::SystemRegistry => PrivilegeLevel::Elevated,
        }
    }
}

/// Extension trait for privilege operations.
#[async_trait]
pub trait PrivilegeProviderExt: PrivilegeProvider {
    /// Checks if all required capabilities are available.
    fn has_all_capabilities(&self, capabilities: &[Capability]) -> bool {
        capabilities.iter().all(|c| self.has_capability(*c))
    }

    /// Gets missing capabilities from a list.
    fn missing_capabilities(&self, capabilities: &[Capability]) -> Vec<Capability> {
        capabilities
            .iter()
            .filter(|c| !self.has_capability(**c))
            .copied()
            .collect()
    }

    /// Requests elevation for specific capabilities if needed.
    async fn ensure_capabilities(&self, capabilities: &[Capability]) -> Result<bool> {
        let missing = self.missing_capabilities(capabilities);
        if missing.is_empty() {
            return Ok(true);
        }

        let request = ElevationRequest {
            reason: format!(
                "Required capabilities: {}",
                missing
                    .iter()
                    .map(|c| c.description())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            required_level: missing
                .iter()
                .map(|c| c.typical_required_level())
                .max()
                .unwrap_or(PrivilegeLevel::User),
            features: missing
                .iter()
                .map(|c| c.description().to_string())
                .collect(),
        };

        self.request_elevation(&request).await
    }
}

// Blanket implementation
impl<T: PrivilegeProvider + ?Sized> PrivilegeProviderExt for T {}
