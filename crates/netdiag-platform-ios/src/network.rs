//! iOS network provider implementation.

use async_trait::async_trait;
use netdiag_platform::NetworkProvider;
use netdiag_types::{
    error::Result,
    network::{
        DhcpInfo, DnsServer, Gateway, InterfaceFlags, InterfaceType, IpSubnet,
        Ipv4Info, Ipv6Info, Ipv6Scope, IspInfo, MacAddress, NetworkInterface, Route,
    },
    Error,
};
use std::net::{IpAddr, Ipv4Addr};

/// iOS network provider.
pub struct IosNetworkProvider {
    // iOS doesn't have persistent state for network info
}

impl IosNetworkProvider {
    /// Creates a new iOS network provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets network interfaces using getifaddrs.
    fn get_interfaces_impl(&self) -> Result<Vec<NetworkInterface>> {
        // On iOS, we can use the same getifaddrs approach as macOS
        // but with more limited information available
        let mut interfaces: Vec<NetworkInterface> = Vec::new();

        // Use libc getifaddrs to enumerate interfaces
        // This is a simplified implementation - in production you'd use
        // the full getifaddrs API

        // For now, return a basic set based on typical iOS interfaces
        // In a real implementation, you'd iterate through getifaddrs

        // iOS typically has: lo0 (loopback), en0 (WiFi), pdp_ip0 (cellular)

        // Loopback
        interfaces.push(NetworkInterface {
            name: "lo0".to_string(),
            display_name: Some("Loopback".to_string()),
            index: 1,
            interface_type: InterfaceType::Loopback,
            mac_address: None,
            ipv4_addresses: vec![Ipv4Info {
                address: "127.0.0.1".parse().unwrap(),
                subnet: IpSubnet::V4 {
                    address: "127.0.0.0".parse().unwrap(),
                    prefix_len: 8,
                },
                broadcast: None,
            }],
            ipv6_addresses: vec![Ipv6Info {
                address: "::1".parse().unwrap(),
                prefix_len: 128,
                scope: Ipv6Scope::Loopback,
            }],
            flags: InterfaceFlags {
                up: true,
                running: true,
                broadcast: false,
                loopback: true,
                point_to_point: false,
                multicast: true,
                promiscuous: false,
            },
            mtu: Some(16384),
            speed_mbps: None,
            is_default: false,
        });

        Ok(interfaces)
    }
}

impl Default for IosNetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkProvider for IosNetworkProvider {
    async fn list_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        self.get_interfaces_impl()
    }

    async fn get_interface(&self, name: &str) -> Result<Option<NetworkInterface>> {
        let interfaces = self.get_interfaces_impl()?;
        Ok(interfaces.into_iter().find(|i| i.name == name))
    }

    async fn get_default_interface(&self) -> Result<Option<NetworkInterface>> {
        let interfaces = self.get_interfaces_impl()?;
        // On iOS, the default interface is typically en0 (WiFi) or pdp_ip0 (cellular)
        Ok(interfaces.into_iter().find(|i| i.is_default))
    }

    async fn get_default_route(&self) -> Result<Option<Route>> {
        // iOS doesn't expose routing table to apps
        Ok(None)
    }

    async fn get_routes(&self) -> Result<Vec<Route>> {
        // iOS doesn't expose routing table to apps
        Ok(Vec::new())
    }

    async fn get_default_gateway(&self) -> Result<Option<Gateway>> {
        // On iOS, we can't directly query the gateway
        // In a real implementation, you might try to infer it from
        // network reachability or other APIs
        Ok(None)
    }

    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>> {
        // iOS doesn't expose DNS server configuration to apps directly
        // You would need to use private APIs or system configuration
        Ok(Vec::new())
    }

    async fn get_dhcp_info(&self, _interface: &str) -> Result<Option<DhcpInfo>> {
        // DHCP info is not directly accessible on iOS
        Ok(None)
    }

    async fn detect_isp(&self) -> Result<Option<IspInfo>> {
        // ISP detection would require an external API call
        Ok(None)
    }

    fn supports_promiscuous(&self, _interface: &str) -> bool {
        // iOS doesn't support promiscuous mode for apps
        false
    }

    async fn refresh(&self) -> Result<()> {
        // No cached state to refresh
        Ok(())
    }
}
