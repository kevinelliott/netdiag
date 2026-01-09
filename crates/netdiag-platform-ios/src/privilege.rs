//! iOS privilege provider implementation.

use async_trait::async_trait;
use netdiag_platform::{Capability, PrivilegeProvider};
use netdiag_types::{
    error::Result,
    system::{ElevationRequest, PrivilegeLevel},
};

/// iOS privilege provider.
///
/// iOS has a sandboxed security model where apps run with limited privileges.
/// Privilege elevation is not possible - apps must request capabilities
/// at install time through entitlements.
pub struct IosPrivilegeProvider {
    // No state needed
}

impl IosPrivilegeProvider {
    /// Creates a new iOS privilege provider.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for IosPrivilegeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PrivilegeProvider for IosPrivilegeProvider {
    fn current_privilege_level(&self) -> PrivilegeLevel {
        // iOS apps always run as a regular user in a sandbox
        PrivilegeLevel::User
    }

    async fn request_elevation(&self, _request: &ElevationRequest) -> Result<bool> {
        // iOS doesn't support runtime privilege elevation
        // All capabilities must be declared in entitlements
        Ok(false)
    }

    fn has_capability(&self, capability: Capability) -> bool {
        match capability {
            // WiFi scanning requires NEHotspotHelper entitlement (restricted)
            Capability::WifiScan => false,

            // None of these are available on iOS due to sandboxing
            Capability::RawSockets => false,
            Capability::PromiscuousMode => false,
            Capability::NetworkConfig => false,
            Capability::DnsConfig => false,
            Capability::InterfaceControl => false,
            Capability::RoutingTable => false,
            Capability::Firewall => false,
            Capability::DriverAccess => false,
            Capability::ServiceManagement => false,
            Capability::SystemRegistry => false,
        }
    }

    fn available_capabilities(&self) -> Vec<Capability> {
        // iOS apps have very limited network capabilities
        // Most operations require special entitlements
        Vec::new()
    }

    fn capabilities_requiring_elevation(&self) -> Vec<Capability> {
        // On iOS, elevation isn't possible, so return capabilities
        // that would require special entitlements (none are elevatable)
        vec![
            Capability::RawSockets,
            Capability::PromiscuousMode,
            Capability::NetworkConfig,
            Capability::DnsConfig,
            Capability::InterfaceControl,
            Capability::RoutingTable,
            Capability::Firewall,
            Capability::DriverAccess,
            Capability::ServiceManagement,
            Capability::WifiScan,
        ]
    }
}
