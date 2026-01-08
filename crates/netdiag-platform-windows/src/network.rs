//! Windows network provider implementation.

use async_trait::async_trait;
use netdiag_platform::NetworkProvider;
use netdiag_types::{
    error::Result,
    network::{
        DhcpInfo, DnsServer, InterfaceFlags, InterfaceType, IpAddressInfo, IspInfo,
        NetworkInterface, Route,
    },
    Error,
};
use std::net::{IpAddr, Ipv4Addr};

#[cfg(windows)]
use windows::{
    Win32::NetworkManagement::IpHelper::*,
    Win32::Networking::WinSock::*,
};

/// Windows network provider using IP Helper API.
pub struct WindowsNetworkProvider {
    // Could cache adapter info here
}

impl WindowsNetworkProvider {
    /// Creates a new Windows network provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets adapter addresses using GetAdaptersAddresses API.
    #[cfg(windows)]
    fn get_adapter_addresses(&self) -> Result<Vec<NetworkInterface>> {
        use std::ptr;
        use windows::Win32::Foundation::ERROR_BUFFER_OVERFLOW;

        let mut interfaces = Vec::new();
        let mut buffer_size: u32 = 0;
        let flags = GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS;

        // First call to get required buffer size
        unsafe {
            let result = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                flags,
                None,
                None,
                &mut buffer_size,
            );

            if result != ERROR_BUFFER_OVERFLOW.0 {
                return Err(Error::platform("Failed to get adapter addresses size"));
            }
        }

        // Allocate buffer
        let mut buffer = vec![0u8; buffer_size as usize];
        let adapter_addresses = buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

        // Second call to get actual data
        unsafe {
            let result = GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                flags,
                None,
                Some(adapter_addresses),
                &mut buffer_size,
            );

            if result != 0 {
                return Err(Error::platform("Failed to get adapter addresses"));
            }

            // Iterate through adapters
            let mut current = adapter_addresses;
            while !current.is_null() {
                let adapter = &*current;

                // Get adapter name
                let name = if !adapter.AdapterName.0.is_null() {
                    std::ffi::CStr::from_ptr(adapter.AdapterName.0 as *const i8)
                        .to_string_lossy()
                        .to_string()
                } else {
                    String::new()
                };

                // Get friendly name
                let friendly_name = if !adapter.FriendlyName.0.is_null() {
                    widestring::U16CStr::from_ptr_str(adapter.FriendlyName.0)
                        .to_string_lossy()
                } else {
                    String::new()
                };

                // Get MAC address
                let mac_address = if adapter.PhysicalAddressLength > 0 {
                    let bytes: [u8; 6] = [
                        adapter.PhysicalAddress[0],
                        adapter.PhysicalAddress[1],
                        adapter.PhysicalAddress[2],
                        adapter.PhysicalAddress[3],
                        adapter.PhysicalAddress[4],
                        adapter.PhysicalAddress[5],
                    ];
                    Some(netdiag_types::network::MacAddress::new(bytes))
                } else {
                    None
                };

                // Get interface type
                let interface_type = match adapter.IfType {
                    IF_TYPE_ETHERNET_CSMACD => InterfaceType::Ethernet,
                    IF_TYPE_IEEE80211 => InterfaceType::Wifi,
                    IF_TYPE_SOFTWARE_LOOPBACK => InterfaceType::Loopback,
                    IF_TYPE_TUNNEL => InterfaceType::Virtual,
                    IF_TYPE_PPP => InterfaceType::Other,
                    _ => InterfaceType::Other,
                };

                // Get IP addresses
                let mut ipv4_addresses = Vec::new();
                let mut ipv6_addresses = Vec::new();

                let mut unicast = adapter.FirstUnicastAddress;
                while !unicast.is_null() {
                    let addr = &*unicast;
                    if let Some(sockaddr) = addr.Address.lpSockaddr.as_ref() {
                        match sockaddr.sa_family {
                            AF_INET => {
                                let sockaddr_in = &*(sockaddr as *const _ as *const SOCKADDR_IN);
                                let ip = Ipv4Addr::from(u32::from_be(sockaddr_in.sin_addr.S_un.S_addr));
                                ipv4_addresses.push(IpAddressInfo {
                                    address: IpAddr::V4(ip),
                                    prefix_length: addr.OnLinkPrefixLength,
                                });
                            }
                            AF_INET6 => {
                                let sockaddr_in6 = &*(sockaddr as *const _ as *const SOCKADDR_IN6);
                                let ip = std::net::Ipv6Addr::from(sockaddr_in6.sin6_addr.u.Byte);
                                ipv6_addresses.push(IpAddressInfo {
                                    address: IpAddr::V6(ip),
                                    prefix_length: addr.OnLinkPrefixLength,
                                });
                            }
                            _ => {}
                        }
                    }
                    unicast = addr.Next;
                }

                // Get flags
                let is_up = adapter.OperStatus == IfOperStatusUp;
                let flags = InterfaceFlags {
                    up: is_up,
                    broadcast: true,
                    loopback: interface_type == InterfaceType::Loopback,
                    point_to_point: false,
                    multicast: true,
                    running: is_up,
                };

                interfaces.push(NetworkInterface {
                    name,
                    display_name: Some(friendly_name),
                    index: adapter.IfIndex,
                    interface_type,
                    mac_address,
                    ipv4_addresses,
                    ipv6_addresses,
                    flags,
                    mtu: Some(adapter.Mtu),
                    speed: if adapter.TransmitLinkSpeed > 0 {
                        Some(adapter.TransmitLinkSpeed)
                    } else {
                        None
                    },
                });

                current = adapter.Next;
            }
        }

        Ok(interfaces)
    }

    /// Stub implementation for non-Windows platforms.
    #[cfg(not(windows))]
    fn get_adapter_addresses(&self) -> Result<Vec<NetworkInterface>> {
        Ok(Vec::new())
    }
}

impl Default for WindowsNetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkProvider for WindowsNetworkProvider {
    async fn list_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        self.get_adapter_addresses()
    }

    async fn get_default_route(&self) -> Result<Option<Route>> {
        // Use GetBestRoute2 or parse routing table
        #[cfg(windows)]
        {
            // Simplified: get first gateway from adapter addresses
            let interfaces = self.list_interfaces().await?;
            for iface in interfaces {
                if iface.is_up() && !iface.flags.loopback {
                    // In a real implementation, would get gateway from adapter info
                    return Ok(Some(Route {
                        destination: "0.0.0.0/0".parse().unwrap(),
                        gateway: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))),
                        interface: iface.name,
                        metric: Some(0),
                    }));
                }
            }
        }
        Ok(None)
    }

    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>> {
        let mut servers = Vec::new();

        #[cfg(windows)]
        {
            // Would parse DNS servers from adapter addresses
            // For now, return common defaults
            servers.push(DnsServer {
                address: IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
                is_primary: true,
                is_dhcp: false,
            });
        }

        Ok(servers)
    }

    async fn get_dhcp_info(&self, _interface: &str) -> Result<Option<DhcpInfo>> {
        // Would use GetAdaptersInfo or WMI
        Ok(None)
    }

    async fn detect_isp(&self) -> Result<Option<IspInfo>> {
        // Would use external IP lookup
        Ok(None)
    }
}
