//! Network interface types.

use super::{IpSubnet, MacAddress};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use strum::{Display, EnumString};

/// Represents a network interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name (e.g., "en0", "eth0", "wlan0")
    pub name: String,
    /// Friendly display name
    pub display_name: Option<String>,
    /// Interface index
    pub index: u32,
    /// Interface type
    pub interface_type: InterfaceType,
    /// MAC/hardware address
    pub mac_address: Option<MacAddress>,
    /// IPv4 addresses
    pub ipv4_addresses: Vec<Ipv4Info>,
    /// IPv6 addresses
    pub ipv6_addresses: Vec<Ipv6Info>,
    /// Interface flags
    pub flags: InterfaceFlags,
    /// MTU (Maximum Transmission Unit)
    pub mtu: Option<u32>,
    /// Interface speed in Mbps (if known)
    pub speed_mbps: Option<u32>,
    /// Is this the default interface for routing?
    pub is_default: bool,
}

impl NetworkInterface {
    /// Gets the primary IPv4 address, if any.
    #[must_use]
    pub fn primary_ipv4(&self) -> Option<IpAddr> {
        self.ipv4_addresses
            .first()
            .map(|info| IpAddr::V4(info.address))
    }

    /// Gets the primary IPv6 address, if any.
    #[must_use]
    pub fn primary_ipv6(&self) -> Option<IpAddr> {
        self.ipv6_addresses
            .first()
            .map(|info| IpAddr::V6(info.address))
    }

    /// Checks if the interface is up and running.
    #[must_use]
    pub fn is_up(&self) -> bool {
        self.flags.up && self.flags.running
    }
}

/// IPv4 address information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv4Info {
    /// IP address
    pub address: std::net::Ipv4Addr,
    /// Subnet
    pub subnet: IpSubnet,
    /// Broadcast address
    pub broadcast: Option<std::net::Ipv4Addr>,
}

/// IPv6 address information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ipv6Info {
    /// IP address
    pub address: std::net::Ipv6Addr,
    /// Prefix length
    pub prefix_len: u8,
    /// Address scope
    pub scope: Ipv6Scope,
}

/// IPv6 address scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Ipv6Scope {
    /// Global scope (publicly routable)
    Global,
    /// Link-local scope
    LinkLocal,
    /// Site-local scope (deprecated)
    SiteLocal,
    /// Unique local (ULA)
    UniqueLocal,
    /// Loopback
    Loopback,
    /// Unknown scope
    Unknown,
}

/// Network interface type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum InterfaceType {
    /// Ethernet (wired)
    Ethernet,
    /// WiFi (802.11)
    Wifi,
    /// Loopback interface
    Loopback,
    /// Virtual/tunnel interface
    Virtual,
    /// VPN tunnel
    Vpn,
    /// Bridge interface
    Bridge,
    /// Bond/team interface
    Bond,
    /// VLAN interface
    Vlan,
    /// Cellular/mobile data
    Cellular,
    /// Bluetooth PAN
    Bluetooth,
    /// MoCA (Multimedia over Coax Alliance)
    Moca,
    /// Powerline adapter
    Powerline,
    /// PPP (Point-to-Point Protocol)
    Ppp,
    /// Unknown interface type
    Unknown,
}

impl InterfaceType {
    /// Returns true if this is a wireless interface type.
    #[must_use]
    pub fn is_wireless(&self) -> bool {
        matches!(self, Self::Wifi | Self::Cellular | Self::Bluetooth)
    }

    /// Returns true if this is a physical interface type.
    #[must_use]
    pub fn is_physical(&self) -> bool {
        matches!(
            self,
            Self::Ethernet
                | Self::Wifi
                | Self::Cellular
                | Self::Bluetooth
                | Self::Moca
                | Self::Powerline
        )
    }
}

/// Interface status flags.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct InterfaceFlags {
    /// Interface is up
    pub up: bool,
    /// Interface is running
    pub running: bool,
    /// Interface supports broadcast
    pub broadcast: bool,
    /// Interface is a loopback
    pub loopback: bool,
    /// Interface is point-to-point
    pub point_to_point: bool,
    /// Interface supports multicast
    pub multicast: bool,
    /// Interface is in promiscuous mode
    pub promiscuous: bool,
}

/// Interface statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InterfaceStats {
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Receive drops
    pub rx_drops: u64,
    /// Transmit drops
    pub tx_drops: u64,
}

/// DHCP information for an interface.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DhcpInfo {
    /// DHCP server address
    pub server: IpAddr,
    /// Lease obtained time
    pub lease_obtained: chrono::DateTime<chrono::Utc>,
    /// Lease expires time
    pub lease_expires: chrono::DateTime<chrono::Utc>,
    /// Subnet mask
    pub subnet_mask: Option<IpAddr>,
    /// Default gateway
    pub gateway: Option<IpAddr>,
    /// DNS servers from DHCP
    pub dns_servers: Vec<IpAddr>,
    /// Domain name
    pub domain: Option<String>,
}
