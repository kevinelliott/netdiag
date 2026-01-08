//! Network provider trait.

use async_trait::async_trait;
use netdiag_types::{
    error::Result,
    network::{DhcpInfo, DnsServer, Gateway, IspInfo, NetworkInterface, Route},
};

/// Provider for network interface and routing information.
#[async_trait]
pub trait NetworkProvider: Send + Sync {
    /// Lists all network interfaces.
    async fn list_interfaces(&self) -> Result<Vec<NetworkInterface>>;

    /// Gets a specific interface by name.
    async fn get_interface(&self, name: &str) -> Result<Option<NetworkInterface>>;

    /// Gets the default network interface.
    async fn get_default_interface(&self) -> Result<Option<NetworkInterface>>;

    /// Gets the default route/gateway.
    async fn get_default_route(&self) -> Result<Option<Route>>;

    /// Gets all routes in the routing table.
    async fn get_routes(&self) -> Result<Vec<Route>>;

    /// Gets the default gateway information.
    async fn get_default_gateway(&self) -> Result<Option<Gateway>>;

    /// Gets configured DNS servers.
    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>>;

    /// Gets DHCP information for an interface.
    async fn get_dhcp_info(&self, interface: &str) -> Result<Option<DhcpInfo>>;

    /// Detects ISP information.
    async fn detect_isp(&self) -> Result<Option<IspInfo>>;

    /// Checks if promiscuous mode is supported for an interface.
    fn supports_promiscuous(&self, interface: &str) -> bool;

    /// Refreshes cached network information.
    async fn refresh(&self) -> Result<()>;
}

/// Extension trait for network operations.
#[async_trait]
pub trait NetworkProviderExt: NetworkProvider {
    /// Gets all active (up and running) interfaces.
    async fn get_active_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        let interfaces = self.list_interfaces().await?;
        Ok(interfaces.into_iter().filter(|i| i.is_up()).collect())
    }

    /// Gets all physical interfaces.
    async fn get_physical_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        let interfaces = self.list_interfaces().await?;
        Ok(interfaces
            .into_iter()
            .filter(|i| i.interface_type.is_physical())
            .collect())
    }

    /// Checks if there's a working internet connection.
    async fn has_internet_connection(&self) -> bool {
        if let Ok(Some(_gateway)) = self.get_default_gateway().await {
            // We have a gateway, likely have internet
            true
        } else {
            false
        }
    }
}

// Blanket implementation
impl<T: NetworkProvider + ?Sized> NetworkProviderExt for T {}
