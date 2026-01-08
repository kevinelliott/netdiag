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
            // Basic network capabilities are available
            Capability::NetworkRead => true,
            Capability::DnsResolve => true,
            Capability::Ping => true, // With some limitations
            Capability::Traceroute => true, // With some limitations

            // These require special entitlements or are not available
            Capability::RawSocket => false,
            Capability::PacketCapture => false,
            Capability::NetworkWrite => false,
            Capability::WifiScan => false, // Requires NEHotspotHelper
            Capability::WifiConnect => false,
            Capability::SystemModify => false,
            Capability::ServiceManage => false,
        }
    }

    fn available_capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::NetworkRead,
            Capability::DnsResolve,
            Capability::Ping,
            Capability::Traceroute,
        ]
    }

    fn capabilities_requiring_elevation(&self) -> Vec<Capability> {
        // On iOS, elevation isn't possible, so return capabilities
        // that would require special entitlements
        vec![
            Capability::RawSocket,
            Capability::PacketCapture,
            Capability::WifiScan,
            Capability::WifiConnect,
            Capability::SystemModify,
            Capability::ServiceManage,
        ]
    }
}
