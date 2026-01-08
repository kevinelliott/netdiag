//! Android network provider implementation.

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

/// Android network provider.
///
/// Uses Android's ConnectivityManager and NetworkInterface APIs
/// through JNI to get network information.
pub struct AndroidNetworkProvider {
    // JNI references would be stored here in a real implementation
}

impl AndroidNetworkProvider {
    /// Creates a new Android network provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets network interfaces.
    ///
    /// In a real implementation, this would use JNI to call:
    /// - ConnectivityManager.getAllNetworks()
    /// - NetworkInterface.getNetworkInterfaces()
    fn get_interfaces_impl(&self) -> Result<Vec<NetworkInterface>> {
        let mut interfaces: Vec<NetworkInterface> = Vec::new();

        // Android typically has: lo (loopback), wlan0 (WiFi), rmnet0+ (cellular)

        // Loopback
        interfaces.push(NetworkInterface {
            name: "lo".to_string(),
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
            mtu: Some(65536),
            speed_mbps: None,
            is_default: false,
        });

        Ok(interfaces)
    }
}

impl Default for AndroidNetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkProvider for AndroidNetworkProvider {
    async fn list_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        self.get_interfaces_impl()
    }

    async fn get_interface(&self, name: &str) -> Result<Option<NetworkInterface>> {
        let interfaces = self.get_interfaces_impl()?;
        Ok(interfaces.into_iter().find(|i| i.name == name))
    }

    async fn get_default_interface(&self) -> Result<Option<NetworkInterface>> {
        let interfaces = self.get_interfaces_impl()?;
        // On Android, check for wlan0 (WiFi) or rmnet (cellular)
        Ok(interfaces.into_iter().find(|i| i.is_default))
    }

    async fn get_default_route(&self) -> Result<Option<Route>> {
        // Would use ConnectivityManager.getLinkProperties().getRoutes()
        Ok(None)
    }

    async fn get_routes(&self) -> Result<Vec<Route>> {
        // Routing table access is limited on Android
        Ok(Vec::new())
    }

    async fn get_default_gateway(&self) -> Result<Option<Gateway>> {
        // Would use LinkProperties.getRoutes() and find default gateway
        Ok(None)
    }

    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>> {
        // Would use LinkProperties.getDnsServers()
        Ok(Vec::new())
    }

    async fn get_dhcp_info(&self, _interface: &str) -> Result<Option<DhcpInfo>> {
        // Would use WifiManager.getDhcpInfo() for WiFi
        Ok(None)
    }

    async fn detect_isp(&self) -> Result<Option<IspInfo>> {
        // Would need external API call
        Ok(None)
    }

    fn supports_promiscuous(&self, _interface: &str) -> bool {
        // Promiscuous mode requires root on Android
        false
    }

    async fn refresh(&self) -> Result<()> {
        Ok(())
    }
}
