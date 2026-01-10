//! Routing table types.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use strum::{Display, EnumString};

/// Represents a route in the routing table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Destination network (None for default route)
    pub destination: Option<IpAddr>,
    /// Destination prefix length
    pub prefix_len: u8,
    /// Gateway address (next hop)
    pub gateway: Option<IpAddr>,
    /// Interface name
    pub interface: String,
    /// Route metric/priority
    pub metric: u32,
    /// Route flags
    pub flags: RouteFlags,
    /// Route type
    pub route_type: RouteType,
}

impl Route {
    /// Checks if this is the default route.
    #[must_use]
    pub fn is_default(&self) -> bool {
        self.destination.is_none()
            || (self.prefix_len == 0
                && self.destination.is_some_and(|d| {
                    d == IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED)
                        || d == IpAddr::V6(std::net::Ipv6Addr::UNSPECIFIED)
                }))
    }
}

/// Route flags.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RouteFlags {
    /// Route is up
    pub up: bool,
    /// Route is a gateway
    pub gateway: bool,
    /// Route is a host route
    pub host: bool,
    /// Route was dynamically created
    pub dynamic: bool,
    /// Route was modified dynamically
    pub modified: bool,
    /// Route is cloned
    pub cloned: bool,
    /// Route is being rejected
    pub reject: bool,
    /// Route is a local address
    pub local: bool,
    /// Route is a broadcast address
    pub broadcast: bool,
    /// Route is static
    pub static_route: bool,
}

/// Route type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum RouteType {
    /// Unicast route
    #[default]
    Unicast,
    /// Local route
    Local,
    /// Broadcast route
    Broadcast,
    /// Multicast route
    Multicast,
    /// Blackhole (drop packets)
    Blackhole,
    /// Unreachable (send ICMP unreachable)
    Unreachable,
    /// Prohibit (send ICMP prohibited)
    Prohibit,
    /// Unknown type
    Unknown,
}

/// Default gateway information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gateway {
    /// Gateway IP address
    pub address: IpAddr,
    /// Interface used to reach the gateway
    pub interface: String,
    /// MAC address of the gateway (if resolved)
    pub mac_address: Option<super::MacAddress>,
    /// Gateway hostname (if resolved)
    pub hostname: Option<String>,
    /// Is this a virtual/VPN gateway?
    pub is_virtual: bool,
}

/// ISP (Internet Service Provider) information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IspInfo {
    /// ISP name
    pub name: String,
    /// ASN (Autonomous System Number)
    pub asn: Option<u32>,
    /// Organization name
    pub organization: Option<String>,
    /// External/public IP address
    pub external_ip: IpAddr,
    /// Country code
    pub country: Option<String>,
    /// Region/state
    pub region: Option<String>,
    /// City
    pub city: Option<String>,
    /// Connection type (if detectable)
    pub connection_type: Option<ConnectionType>,
}

/// Internet connection type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ConnectionType {
    /// Fiber optic
    Fiber,
    /// Cable (DOCSIS)
    Cable,
    /// DSL
    Dsl,
    /// Fixed wireless
    FixedWireless,
    /// Satellite
    Satellite,
    /// Cellular/mobile
    Cellular,
    /// Dial-up
    Dialup,
    /// Unknown
    Unknown,
}
