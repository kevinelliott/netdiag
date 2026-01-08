//! Android privilege provider implementation.

use async_trait::async_trait;
use netdiag_platform::{Capability, PrivilegeProvider};
use netdiag_types::{
    error::Result,
    system::{ElevationRequest, PrivilegeLevel},
};

/// Android privilege provider.
///
/// Android uses a permission-based security model. Apps must declare
/// permissions in the manifest and request runtime permissions for
/// dangerous permissions.
pub struct AndroidPrivilegeProvider {
    // Would store granted permissions here
}

impl AndroidPrivilegeProvider {
    /// Creates a new Android privilege provider.
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for AndroidPrivilegeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PrivilegeProvider for AndroidPrivilegeProvider {
    fn current_privilege_level(&self) -> PrivilegeLevel {
        // Regular Android apps run as user
        // Root is only available on rooted devices
        PrivilegeLevel::User
    }

    async fn request_elevation(&self, _request: &ElevationRequest) -> Result<bool> {
        // Android doesn't support privilege elevation
        // Apps must request permissions through the permission system
        Ok(false)
    }

    fn has_capability(&self, capability: Capability) -> bool {
        match capability {
            // Basic capabilities available to all apps
            Capability::NetworkRead => true,
            Capability::DnsResolve => true,

            // These depend on permissions
            Capability::Ping => true, // INTERNET permission usually granted
            Capability::Traceroute => true,

            // These require special permissions or root
            Capability::RawSocket => false, // Requires root
            Capability::PacketCapture => false, // Requires root
            Capability::NetworkWrite => false, // Limited
            Capability::WifiScan => false, // Requires ACCESS_FINE_LOCATION
            Capability::WifiConnect => false, // Requires special permissions
            Capability::SystemModify => false, // Requires root
            Capability::ServiceManage => false, // Not available to apps
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
        // These capabilities require either special permissions or root
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
