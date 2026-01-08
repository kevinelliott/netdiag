//! Linux network provider implementation.

use async_trait::async_trait;
use netdiag_platform::NetworkProvider;
use netdiag_types::error::{Error, Result};
use netdiag_types::network::{
    DhcpInfo, DnsServer, InterfaceType, IpSubnet, Ipv4Info, Ipv4Subnet, Ipv6Info, Ipv6Subnet,
    IspInfo, MacAddress, NetworkInterface, Route,
};
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::process::Command;
use tracing::debug;

/// Linux network provider using netdev and system commands.
pub struct LinuxNetworkProvider {
    // Can store configuration or cached data here
}

impl LinuxNetworkProvider {
    /// Creates a new Linux network provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Convert netdev interface type to our InterfaceType.
    fn convert_interface_type(if_type: netdev::interface::InterfaceType) -> InterfaceType {
        match if_type {
            netdev::interface::InterfaceType::Ethernet => InterfaceType::Ethernet,
            netdev::interface::InterfaceType::Loopback => InterfaceType::Loopback,
            netdev::interface::InterfaceType::Bridge => InterfaceType::Bridge,
            netdev::interface::InterfaceType::Tun => InterfaceType::Tunnel,
            _ => InterfaceType::Unknown,
        }
    }

    /// Parse /etc/resolv.conf for DNS servers.
    fn parse_resolv_conf(&self) -> Vec<DnsServer> {
        let mut servers = Vec::new();

        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("nameserver") {
                    if let Some(addr_str) = line.split_whitespace().nth(1) {
                        if let Ok(ip) = addr_str.parse::<IpAddr>() {
                            servers.push(DnsServer {
                                address: ip,
                                interface: None,
                                is_dhcp: None,
                            });
                        }
                    }
                }
            }
        }

        servers
    }

    /// Get default gateway from /proc/net/route.
    fn get_default_gateway_from_proc(&self) -> Option<(String, Ipv4Addr)> {
        if let Ok(content) = fs::read_to_string("/proc/net/route") {
            for line in content.lines().skip(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let iface = parts[0];
                    let dest = parts[1];
                    let gateway = parts[2];

                    // Default route has destination 00000000
                    if dest == "00000000" && gateway != "00000000" {
                        // Gateway is in little-endian hex
                        if let Ok(gw_int) = u32::from_str_radix(gateway, 16) {
                            let gw_ip = Ipv4Addr::from(gw_int.to_be());
                            return Some((iface.to_string(), gw_ip));
                        }
                    }
                }
            }
        }
        None
    }
}

impl Default for LinuxNetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkProvider for LinuxNetworkProvider {
    async fn list_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        debug!("Listing network interfaces on Linux");

        let interfaces = netdev::get_interfaces();
        let mut result = Vec::new();

        for iface in interfaces {
            let interface_type = Self::convert_interface_type(iface.if_type);

            // Convert IPv4 addresses
            let ipv4_addresses: Vec<Ipv4Info> = iface
                .ipv4
                .iter()
                .map(|net| Ipv4Info {
                    address: net.addr(),
                    subnet: IpSubnet::V4(Ipv4Subnet::new(net.addr(), net.prefix_len())),
                    broadcast: Some(net.broadcast()),
                })
                .collect();

            // Convert IPv6 addresses
            let ipv6_addresses: Vec<Ipv6Info> = iface
                .ipv6
                .iter()
                .map(|net| Ipv6Info {
                    address: net.addr(),
                    subnet: IpSubnet::V6(Ipv6Subnet::new(net.addr(), net.prefix_len())),
                    scope: None,
                })
                .collect();

            // Convert MAC address
            let mac_address = iface.mac_addr.map(|mac| {
                MacAddress::new([
                    mac.octets()[0],
                    mac.octets()[1],
                    mac.octets()[2],
                    mac.octets()[3],
                    mac.octets()[4],
                    mac.octets()[5],
                ])
            });

            // Get MTU from /sys/class/net/{iface}/mtu
            let mtu = fs::read_to_string(format!("/sys/class/net/{}/mtu", iface.name))
                .ok()
                .and_then(|s| s.trim().parse().ok());

            // Check if default interface
            let is_default = netdev::get_default_interface()
                .map(|d| d.name == iface.name)
                .unwrap_or(false);

            let net_iface = NetworkInterface {
                name: iface.name.clone(),
                display_name: iface.friendly_name.clone().unwrap_or_else(|| iface.name.clone()),
                interface_type,
                mac_address,
                ipv4_addresses,
                ipv6_addresses,
                mtu,
                is_up: iface.is_up(),
                is_running: iface.is_running(),
                is_loopback: iface.is_loopback(),
                is_point_to_point: iface.is_point_to_point(),
                is_default,
                flags: HashMap::new(),
            };

            result.push(net_iface);
        }

        debug!("Found {} interfaces", result.len());
        Ok(result)
    }

    async fn get_default_interface(&self) -> Result<Option<NetworkInterface>> {
        let interfaces = self.list_interfaces().await?;
        Ok(interfaces.into_iter().find(|i| i.is_default))
    }

    async fn get_default_route(&self) -> Result<Option<Route>> {
        debug!("Getting default route on Linux");

        if let Some((iface, gateway)) = self.get_default_gateway_from_proc() {
            return Ok(Some(Route {
                destination: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                gateway: Some(IpAddr::V4(gateway)),
                interface: iface,
                metric: None,
            }));
        }

        // Fallback: use `ip route` command
        let output = Command::new("ip")
            .args(["route", "show", "default"])
            .output()
            .map_err(|e| Error::platform("ip route", &e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            // Format: default via 192.168.1.1 dev eth0 proto dhcp metric 100
            if line.starts_with("default") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                let mut gateway = None;
                let mut interface = String::new();
                let mut metric = None;

                let mut i = 1;
                while i < parts.len() {
                    match parts[i] {
                        "via" => {
                            if i + 1 < parts.len() {
                                gateway = parts[i + 1].parse().ok();
                                i += 1;
                            }
                        }
                        "dev" => {
                            if i + 1 < parts.len() {
                                interface = parts[i + 1].to_string();
                                i += 1;
                            }
                        }
                        "metric" => {
                            if i + 1 < parts.len() {
                                metric = parts[i + 1].parse().ok();
                                i += 1;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }

                return Ok(Some(Route {
                    destination: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                    gateway,
                    interface,
                    metric,
                }));
            }
        }

        Ok(None)
    }

    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>> {
        debug!("Getting DNS servers on Linux");
        Ok(self.parse_resolv_conf())
    }

    async fn get_dhcp_info(&self, interface: &str) -> Result<Option<DhcpInfo>> {
        debug!("Getting DHCP info for {} on Linux", interface);

        // Check dhclient lease file
        let lease_files = [
            format!("/var/lib/dhcp/dhclient.{}.leases", interface),
            format!("/var/lib/dhclient/dhclient-{}.leases", interface),
            "/var/lib/dhcp/dhclient.leases".to_string(),
        ];

        for lease_file in &lease_files {
            if let Ok(content) = fs::read_to_string(lease_file) {
                // Parse the last lease in the file
                let mut server_ip = None;
                let mut lease_time = None;
                let mut router = None;
                let mut dns_servers = Vec::new();

                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("option dhcp-server-identifier") {
                        if let Some(ip_str) = line.split_whitespace().nth(2) {
                            server_ip = ip_str.trim_end_matches(';').parse().ok();
                        }
                    } else if line.starts_with("option dhcp-lease-time") {
                        if let Some(time_str) = line.split_whitespace().nth(2) {
                            lease_time = time_str.trim_end_matches(';').parse().ok();
                        }
                    } else if line.starts_with("option routers") {
                        if let Some(ip_str) = line.split_whitespace().nth(2) {
                            router = ip_str.trim_end_matches(';').parse().ok();
                        }
                    } else if line.starts_with("option domain-name-servers") {
                        let servers_str = line
                            .trim_start_matches("option domain-name-servers")
                            .trim()
                            .trim_end_matches(';');
                        for addr in servers_str.split(',') {
                            if let Ok(ip) = addr.trim().parse() {
                                dns_servers.push(ip);
                            }
                        }
                    }
                }

                if server_ip.is_some() || router.is_some() {
                    return Ok(Some(DhcpInfo {
                        enabled: true,
                        server: server_ip,
                        lease_obtained: None,
                        lease_expires: None,
                        lease_duration: lease_time.map(std::time::Duration::from_secs),
                        router,
                        dns_servers,
                    }));
                }
            }
        }

        // Check if interface uses DHCP via NetworkManager
        let output = Command::new("nmcli")
            .args(["-t", "-f", "IP4.METHOD", "device", "show", interface])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("auto") || stdout.contains("dhcp") {
                return Ok(Some(DhcpInfo {
                    enabled: true,
                    server: None,
                    lease_obtained: None,
                    lease_expires: None,
                    lease_duration: None,
                    router: None,
                    dns_servers: Vec::new(),
                }));
            }
        }

        Ok(None)
    }

    async fn detect_isp(&self) -> Result<Option<IspInfo>> {
        debug!("Detecting ISP on Linux");

        // This would typically require an external API call
        // For now, return None - ISP detection is better done via external services
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_interfaces() {
        let provider = LinuxNetworkProvider::new();
        let interfaces = provider.list_interfaces().await.unwrap();
        assert!(!interfaces.is_empty(), "Should find at least one interface");
    }
}
