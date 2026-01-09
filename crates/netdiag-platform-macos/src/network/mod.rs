//! macOS network provider implementation.

use async_trait::async_trait;
use netdiag_platform::NetworkProvider;
use netdiag_types::{
    error::{Error, Result},
    network::{
        DhcpInfo, DnsProtocol, DnsServer, DnsSource, Gateway, InterfaceFlags, InterfaceType,
        IpSubnet, Ipv4Info, Ipv4Subnet, Ipv6Info, Ipv6Scope, IspInfo, MacAddress, NetworkInterface,
        Route, RouteFlags, RouteType,
    },
};
use std::net::IpAddr;
use std::process::Command;
use std::sync::RwLock;
use tracing::debug;

/// macOS network provider using netdev and system commands.
pub struct MacosNetworkProvider {
    /// Cached interface list
    cache: RwLock<Option<Vec<NetworkInterface>>>,
}

impl MacosNetworkProvider {
    /// Creates a new macOS network provider.
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(None),
        }
    }

    /// Converts netdev interface to our NetworkInterface type.
    fn convert_interface(iface: &netdev::Interface) -> NetworkInterface {
        let interface_type = Self::detect_interface_type(&iface.name);

        let mac_address = iface.mac_addr.map(|mac| MacAddress::new(mac.octets()));

        let ipv4_addresses: Vec<Ipv4Info> = iface
            .ipv4
            .iter()
            .map(|net| Ipv4Info {
                address: net.addr(),
                subnet: IpSubnet::V4(Ipv4Subnet::new(net.addr(), net.prefix_len())),
                broadcast: Some(net.broadcast()),
            })
            .collect();

        let ipv6_addresses: Vec<Ipv6Info> = iface
            .ipv6
            .iter()
            .map(|net| {
                let addr = net.addr();
                let scope = if addr.is_loopback() {
                    Ipv6Scope::Loopback
                } else if addr.segments()[0] == 0xfe80 {
                    Ipv6Scope::LinkLocal
                } else if addr.segments()[0] & 0xfe00 == 0xfc00 {
                    Ipv6Scope::UniqueLocal
                } else {
                    Ipv6Scope::Global
                };
                Ipv6Info {
                    address: addr,
                    prefix_len: net.prefix_len(),
                    scope,
                }
            })
            .collect();

        let flags = InterfaceFlags {
            up: iface.is_up(),
            running: iface.is_running(),
            broadcast: !iface.is_point_to_point(),
            loopback: iface.is_loopback(),
            point_to_point: iface.is_point_to_point(),
            multicast: iface.is_multicast(),
            promiscuous: false,
        };

        NetworkInterface {
            name: iface.name.clone(),
            display_name: iface.friendly_name.clone(),
            index: iface.index,
            interface_type,
            mac_address,
            ipv4_addresses,
            ipv6_addresses,
            flags,
            mtu: None, // MTU not exposed by netdev
            speed_mbps: None,
            is_default: false,
        }
    }

    /// Detects interface type from name.
    fn detect_interface_type(name: &str) -> InterfaceType {
        if name.starts_with("lo") {
            InterfaceType::Loopback
        } else if name.starts_with("en") {
            // en0 is usually WiFi on modern Macs, en1+ can be Ethernet
            // We'll refine this with WiFi detection
            InterfaceType::Wifi
        } else if name.starts_with("bridge") {
            InterfaceType::Bridge
        } else if name.starts_with("awdl") || name.starts_with("llw") {
            InterfaceType::Wifi // Apple Wireless Direct Link
        } else if name.starts_with("utun") || name.starts_with("ipsec") {
            InterfaceType::Vpn
        } else if name.starts_with("gif") || name.starts_with("stf") {
            InterfaceType::Virtual
        } else if name.starts_with("p2p") {
            InterfaceType::Wifi
        } else {
            InterfaceType::Ethernet
        }
    }

    /// Parses DNS servers from scutil output.
    fn parse_dns_servers(&self) -> Result<Vec<DnsServer>> {
        let output = Command::new("scutil")
            .arg("--dns")
            .output()
            .map_err(|e| Error::Other {
                context: "DNS lookup".to_string(),
                message: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut servers = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.starts_with("nameserver[") {
                if let Some(addr_str) = line.split(':').nth(1) {
                    let addr_str = addr_str.trim();
                    if let Ok(addr) = addr_str.parse::<IpAddr>() {
                        if seen.insert(addr) {
                            let (name, provider) = Self::identify_dns_provider(addr);
                            servers.push(DnsServer {
                                address: addr,
                                port: 53,
                                name,
                                provider,
                                protocol: DnsProtocol::Udp,
                                is_primary: servers.is_empty(),
                                source: DnsSource::System,
                            });
                        }
                    }
                }
            }
        }

        Ok(servers)
    }

    /// Identifies well-known DNS providers.
    fn identify_dns_provider(addr: IpAddr) -> (Option<String>, Option<String>) {
        match addr {
            IpAddr::V4(v4) => {
                let octets = v4.octets();
                match octets {
                    [8, 8, 8, 8] => (Some("Google DNS".to_string()), Some("Google".to_string())),
                    [8, 8, 4, 4] => (
                        Some("Google DNS Secondary".to_string()),
                        Some("Google".to_string()),
                    ),
                    [1, 1, 1, 1] => (
                        Some("Cloudflare DNS".to_string()),
                        Some("Cloudflare".to_string()),
                    ),
                    [1, 0, 0, 1] => (
                        Some("Cloudflare DNS Secondary".to_string()),
                        Some("Cloudflare".to_string()),
                    ),
                    [9, 9, 9, 9] => (Some("Quad9 DNS".to_string()), Some("Quad9".to_string())),
                    [208, 67, 222, 222] => (Some("OpenDNS".to_string()), Some("Cisco".to_string())),
                    [208, 67, 220, 220] => (
                        Some("OpenDNS Secondary".to_string()),
                        Some("Cisco".to_string()),
                    ),
                    _ => (None, None),
                }
            }
            IpAddr::V6(v6) => {
                let segments = v6.segments();
                match segments {
                    [0x2001, 0x4860, 0x4860, _, _, _, _, 0x8888] => (
                        Some("Google DNS IPv6".to_string()),
                        Some("Google".to_string()),
                    ),
                    [0x2606, 0x4700, 0x4700, _, _, _, _, 0x1111] => (
                        Some("Cloudflare DNS IPv6".to_string()),
                        Some("Cloudflare".to_string()),
                    ),
                    _ => (None, None),
                }
            }
        }
    }

    /// Gets the default gateway using netstat.
    fn get_gateway_from_netstat(&self) -> Result<Option<Gateway>> {
        let output = Command::new("netstat")
            .args(["-rn", "-f", "inet"])
            .output()
            .map_err(|e| Error::Other {
                context: "netstat".to_string(),
                message: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 && parts[0] == "default" {
                if let Ok(gateway_ip) = parts[1].parse::<IpAddr>() {
                    let interface = parts.get(3).unwrap_or(&"").to_string();
                    return Ok(Some(Gateway {
                        address: gateway_ip,
                        interface,
                        mac_address: None,
                        hostname: None,
                        is_virtual: false,
                    }));
                }
            }
        }

        Ok(None)
    }
}

impl Default for MacosNetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkProvider for MacosNetworkProvider {
    async fn list_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        // Check cache first
        if let Ok(guard) = self.cache.read() {
            if let Some(ref cached) = *guard {
                return Ok(cached.clone());
            }
        }

        // Get interfaces from netdev
        let interfaces = netdev::get_interfaces();
        let mut result: Vec<NetworkInterface> =
            interfaces.iter().map(Self::convert_interface).collect();

        // Mark default interface
        if let Ok(Some(gateway)) = self.get_gateway_from_netstat() {
            for iface in &mut result {
                if iface.name == gateway.interface {
                    iface.is_default = true;
                    break;
                }
            }
        }

        // Cache results
        if let Ok(mut guard) = self.cache.write() {
            *guard = Some(result.clone());
        }

        Ok(result)
    }

    async fn get_interface(&self, name: &str) -> Result<Option<NetworkInterface>> {
        let interfaces = self.list_interfaces().await?;
        Ok(interfaces.into_iter().find(|i| i.name == name))
    }

    async fn get_default_interface(&self) -> Result<Option<NetworkInterface>> {
        let interfaces = self.list_interfaces().await?;
        Ok(interfaces.into_iter().find(|i| i.is_default))
    }

    async fn get_default_route(&self) -> Result<Option<Route>> {
        if let Some(gateway) = self.get_gateway_from_netstat()? {
            Ok(Some(Route {
                destination: None,
                prefix_len: 0,
                gateway: Some(gateway.address),
                interface: gateway.interface,
                metric: 0,
                flags: RouteFlags {
                    up: true,
                    gateway: true,
                    ..Default::default()
                },
                route_type: RouteType::Unicast,
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_routes(&self) -> Result<Vec<Route>> {
        let output = Command::new("netstat")
            .args(["-rn"])
            .output()
            .map_err(|e| Error::Other {
                context: "netstat".to_string(),
                message: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut routes = Vec::new();

        for line in stdout.lines().skip(4) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let destination = if parts[0] == "default" {
                    None
                } else {
                    parts[0]
                        .split('/')
                        .next()
                        .and_then(|s| s.parse::<IpAddr>().ok())
                };

                let gateway = parts[1].parse::<IpAddr>().ok();
                let interface = parts.get(3).unwrap_or(&"").to_string();

                routes.push(Route {
                    destination,
                    prefix_len: 0,
                    gateway,
                    interface,
                    metric: 0,
                    flags: RouteFlags::default(),
                    route_type: RouteType::Unicast,
                });
            }
        }

        Ok(routes)
    }

    async fn get_default_gateway(&self) -> Result<Option<Gateway>> {
        self.get_gateway_from_netstat()
    }

    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>> {
        self.parse_dns_servers()
    }

    async fn get_dhcp_info(&self, interface: &str) -> Result<Option<DhcpInfo>> {
        // Use ipconfig getpacket to get DHCP info
        let output = Command::new("ipconfig")
            .args(["getpacket", interface])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                debug!("DHCP info for {}: {}", interface, stdout);
                // Parse DHCP packet - simplified for now
                // Full implementation would parse the plist output
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    async fn detect_isp(&self) -> Result<Option<IspInfo>> {
        // Use external service to detect ISP
        // For now, return None - will be implemented with netdiag-integrations
        Ok(None)
    }

    fn supports_promiscuous(&self, _interface: &str) -> bool {
        // Requires elevated privileges on macOS
        unsafe { libc::geteuid() == 0 }
    }

    async fn refresh(&self) -> Result<()> {
        if let Ok(mut guard) = self.cache.write() {
            *guard = None;
        }
        Ok(())
    }
}
