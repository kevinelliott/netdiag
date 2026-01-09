//! iOS network provider implementation.

use async_trait::async_trait;
use netdiag_platform::NetworkProvider;
use netdiag_types::{
    error::Result,
    network::{
        DhcpInfo, DnsProtocol, DnsServer, DnsSource, Gateway, InterfaceFlags, InterfaceType,
        IpSubnet, Ipv4Info, Ipv4Subnet, Ipv6Info, Ipv6Scope, IspInfo, MacAddress, NetworkInterface,
        Route, RouteFlags, RouteType,
    },
    Error,
};
use std::collections::HashMap;
use std::ffi::CStr;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::ptr;

/// iOS network provider using native APIs.
pub struct IosNetworkProvider {
    // No persistent state needed
}

impl IosNetworkProvider {
    /// Creates a new iOS network provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets network interfaces using getifaddrs.
    fn get_interfaces_impl(&self) -> Result<Vec<NetworkInterface>> {
        let mut interfaces_map: HashMap<String, NetworkInterface> = HashMap::new();

        unsafe {
            let mut ifaddrs: *mut libc::ifaddrs = ptr::null_mut();

            if libc::getifaddrs(&mut ifaddrs) != 0 {
                return Err(Error::NetworkInterface {
                    interface: None,
                    message: format!(
                        "Failed to get network interfaces: {}",
                        std::io::Error::last_os_error()
                    ),
                });
            }

            let mut current = ifaddrs;
            while !current.is_null() {
                let ifa = &*current;

                if !ifa.ifa_name.is_null() {
                    let name = CStr::from_ptr(ifa.ifa_name)
                        .to_string_lossy()
                        .to_string();

                    // Get or create interface entry
                    let iface = interfaces_map.entry(name.clone()).or_insert_with(|| {
                        NetworkInterface {
                            name: name.clone(),
                            display_name: Some(get_display_name(&name)),
                            index: libc::if_nametoindex(ifa.ifa_name) as u32,
                            interface_type: detect_interface_type(&name),
                            mac_address: None,
                            ipv4_addresses: Vec::new(),
                            ipv6_addresses: Vec::new(),
                            flags: parse_flags(ifa.ifa_flags as u32),
                            mtu: None,
                            speed_mbps: None,
                            is_default: is_default_interface(&name),
                        }
                    });

                    // Parse address based on family
                    if !ifa.ifa_addr.is_null() {
                        let family = (*ifa.ifa_addr).sa_family as i32;

                        match family {
                            libc::AF_INET => {
                                if let Some(ipv4_info) = parse_ipv4_address(ifa) {
                                    iface.ipv4_addresses.push(ipv4_info);
                                }
                            }
                            libc::AF_INET6 => {
                                if let Some(ipv6_info) = parse_ipv6_address(ifa) {
                                    iface.ipv6_addresses.push(ipv6_info);
                                }
                            }
                            libc::AF_LINK => {
                                // Get MAC address from link-layer info
                                if iface.mac_address.is_none() {
                                    iface.mac_address = parse_mac_address(ifa);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                current = ifa.ifa_next;
            }

            libc::freeifaddrs(ifaddrs);
        }

        Ok(interfaces_map.into_values().collect())
    }

    /// Gets the default gateway by checking routing information.
    fn get_default_gateway_impl(&self) -> Option<Gateway> {
        // On iOS, we can try to infer the gateway from the default interface
        // A more complete implementation would parse routing tables

        // For WiFi (en0), the gateway is typically at .1 of the subnet
        // This is a heuristic that works in most cases
        if let Ok(interfaces) = self.get_interfaces_impl() {
            for iface in interfaces {
                if iface.name == "en0" || iface.is_default {
                    if let Some(ipv4) = iface.ipv4_addresses.first() {
                        // Assume gateway is at .1 of the network
                        let octets = ipv4.address.octets();
                        let gateway_addr = Ipv4Addr::new(octets[0], octets[1], octets[2], 1);
                        return Some(Gateway {
                            address: IpAddr::V4(gateway_addr),
                            interface: iface.name.clone(),
                            mac_address: None,
                            hostname: None,
                            is_virtual: false,
                        });
                    }
                }
            }
        }
        None
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
        // Prefer en0 (WiFi) or pdp_ip0 (cellular) as default
        Ok(interfaces.into_iter().find(|i| {
            i.is_default || i.name == "en0" || i.name.starts_with("pdp_ip")
        }))
    }

    async fn get_default_route(&self) -> Result<Option<Route>> {
        // iOS doesn't expose routing table to apps
        if let Some(gateway) = self.get_default_gateway_impl() {
            Ok(Some(Route {
                destination: None, // None for default route
                prefix_len: 0,
                gateway: Some(gateway.address),
                interface: gateway.interface.clone(),
                metric: 0,
                flags: RouteFlags::default(),
                route_type: RouteType::Unicast,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_routes(&self) -> Result<Vec<Route>> {
        // iOS doesn't expose routing table to apps
        Ok(Vec::new())
    }

    async fn get_default_gateway(&self) -> Result<Option<Gateway>> {
        Ok(self.get_default_gateway_impl())
    }

    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>> {
        // Try to read from resolv.conf (may not be accessible on iOS)
        // In a real app, you'd use SCDynamicStoreCopyDHCPInfo or similar
        let mut servers = Vec::new();

        // Common default DNS servers
        if let Ok(content) = std::fs::read_to_string("/etc/resolv.conf") {
            let mut is_first = true;
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    if let Some(addr_str) = line.split_whitespace().nth(1) {
                        if let Ok(addr) = addr_str.parse() {
                            servers.push(DnsServer {
                                address: addr,
                                port: 53,
                                name: None,
                                provider: None,
                                protocol: DnsProtocol::Udp,
                                is_primary: is_first,
                                source: DnsSource::System,
                            });
                            is_first = false;
                        }
                    }
                }
            }
        }

        Ok(servers)
    }

    async fn get_dhcp_info(&self, _interface: &str) -> Result<Option<DhcpInfo>> {
        // DHCP info is not directly accessible on iOS without private APIs
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

/// Gets a human-readable display name for an interface.
fn get_display_name(name: &str) -> String {
    match name {
        "lo0" => "Loopback".to_string(),
        "en0" => "Wi-Fi".to_string(),
        "en1" => "Ethernet Adapter".to_string(),
        n if n.starts_with("pdp_ip") => "Cellular".to_string(),
        n if n.starts_with("utun") => "VPN Tunnel".to_string(),
        n if n.starts_with("ipsec") => "IPSec VPN".to_string(),
        n if n.starts_with("bridge") => "Bridge".to_string(),
        n if n.starts_with("awdl") => "Apple Wireless Direct Link".to_string(),
        n if n.starts_with("llw") => "Low Latency WLAN".to_string(),
        _ => name.to_string(),
    }
}

/// Detects the interface type from its name.
fn detect_interface_type(name: &str) -> InterfaceType {
    match name {
        "lo0" => InterfaceType::Loopback,
        "en0" => InterfaceType::Wifi,
        "en1" | "en2" => InterfaceType::Ethernet,
        n if n.starts_with("pdp_ip") => InterfaceType::Cellular,
        n if n.starts_with("utun") => InterfaceType::Vpn,
        n if n.starts_with("ipsec") => InterfaceType::Vpn,
        n if n.starts_with("bridge") => InterfaceType::Bridge,
        n if n.starts_with("awdl") => InterfaceType::Wifi,
        n if n.starts_with("llw") => InterfaceType::Wifi,
        _ => InterfaceType::Unknown,
    }
}

/// Determines if this is likely the default interface.
fn is_default_interface(name: &str) -> bool {
    // On iOS, en0 (WiFi) or pdp_ip0 (cellular) is typically default
    name == "en0" || name == "pdp_ip0"
}

/// Parses interface flags.
fn parse_flags(flags: u32) -> InterfaceFlags {
    InterfaceFlags {
        up: flags & libc::IFF_UP as u32 != 0,
        running: flags & libc::IFF_RUNNING as u32 != 0,
        broadcast: flags & libc::IFF_BROADCAST as u32 != 0,
        loopback: flags & libc::IFF_LOOPBACK as u32 != 0,
        point_to_point: flags & libc::IFF_POINTOPOINT as u32 != 0,
        multicast: flags & libc::IFF_MULTICAST as u32 != 0,
        promiscuous: flags & libc::IFF_PROMISC as u32 != 0,
    }
}

/// Parses an IPv4 address from ifaddrs.
unsafe fn parse_ipv4_address(ifa: &libc::ifaddrs) -> Option<Ipv4Info> {
    if ifa.ifa_addr.is_null() {
        return None;
    }

    let sockaddr = ifa.ifa_addr as *const libc::sockaddr_in;
    let addr = Ipv4Addr::from(u32::from_be((*sockaddr).sin_addr.s_addr));

    // Get netmask
    let prefix_len = if !ifa.ifa_netmask.is_null() {
        let netmask = ifa.ifa_netmask as *const libc::sockaddr_in;
        let mask = u32::from_be((*netmask).sin_addr.s_addr);
        mask.count_ones() as u8
    } else {
        24
    };

    // Calculate network address
    let addr_u32 = u32::from_be((*sockaddr).sin_addr.s_addr);
    let mask = !((1u32 << (32 - prefix_len)) - 1);
    let network = Ipv4Addr::from((addr_u32 & mask).to_be());

    // Get broadcast
    let broadcast = if !ifa.ifa_dstaddr.is_null() {
        let bcast = ifa.ifa_dstaddr as *const libc::sockaddr_in;
        Some(Ipv4Addr::from(u32::from_be((*bcast).sin_addr.s_addr)))
    } else {
        None
    };

    Some(Ipv4Info {
        address: addr,
        subnet: IpSubnet::V4(Ipv4Subnet::new(network, prefix_len)),
        broadcast,
    })
}

/// Parses an IPv6 address from ifaddrs.
unsafe fn parse_ipv6_address(ifa: &libc::ifaddrs) -> Option<Ipv6Info> {
    if ifa.ifa_addr.is_null() {
        return None;
    }

    let sockaddr = ifa.ifa_addr as *const libc::sockaddr_in6;
    let addr_bytes = (*sockaddr).sin6_addr.s6_addr;
    let addr = Ipv6Addr::from(addr_bytes);

    // Get prefix length from netmask
    let prefix_len = if !ifa.ifa_netmask.is_null() {
        let netmask = ifa.ifa_netmask as *const libc::sockaddr_in6;
        let mask_bytes = (*netmask).sin6_addr.s6_addr;
        mask_bytes.iter().map(|b| b.count_ones() as u8).sum()
    } else {
        64
    };

    // Determine scope
    let scope = if addr.is_loopback() {
        Ipv6Scope::Loopback
    } else if addr.segments()[0] & 0xffc0 == 0xfe80 {
        Ipv6Scope::LinkLocal
    } else if addr.segments()[0] & 0xff00 == 0xff00 {
        // Multicast addresses - treat as unknown scope
        Ipv6Scope::Unknown
    } else {
        Ipv6Scope::Global
    };

    Some(Ipv6Info {
        address: addr,
        prefix_len,
        scope,
    })
}

/// Parses a MAC address from link-layer info.
unsafe fn parse_mac_address(ifa: &libc::ifaddrs) -> Option<MacAddress> {
    if ifa.ifa_addr.is_null() {
        return None;
    }

    // On iOS/macOS, AF_LINK addresses contain the MAC address
    let sdl = ifa.ifa_addr as *const libc::sockaddr_dl;
    let nlen = (*sdl).sdl_nlen as usize;
    let alen = (*sdl).sdl_alen as usize;

    if alen == 6 {
        let data_ptr = (*sdl).sdl_data.as_ptr().add(nlen) as *const u8;
        let mut mac = [0u8; 6];
        ptr::copy_nonoverlapping(data_ptr, mac.as_mut_ptr(), 6);

        // Skip all-zero MAC addresses
        if mac.iter().all(|&b| b == 0) {
            return None;
        }

        Some(MacAddress::new(mac))
    } else {
        None
    }
}
