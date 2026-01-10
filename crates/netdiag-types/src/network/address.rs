//! Network address types.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

/// MAC address representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Creates a new MAC address from bytes.
    #[must_use]
    pub const fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Creates a broadcast MAC address (FF:FF:FF:FF:FF:FF).
    #[must_use]
    pub const fn broadcast() -> Self {
        Self([0xFF; 6])
    }

    /// Creates a zero MAC address (00:00:00:00:00:00).
    #[must_use]
    pub const fn zero() -> Self {
        Self([0x00; 6])
    }

    /// Returns the bytes of the MAC address.
    #[must_use]
    pub const fn octets(&self) -> [u8; 6] {
        self.0
    }

    /// Checks if this is a broadcast address.
    #[must_use]
    pub fn is_broadcast(&self) -> bool {
        self.0 == [0xFF; 6]
    }

    /// Checks if this is a zero/null address.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == [0x00; 6]
    }

    /// Checks if this is a multicast address.
    #[must_use]
    pub fn is_multicast(&self) -> bool {
        self.0[0] & 0x01 != 0
    }

    /// Checks if this is a locally administered address.
    #[must_use]
    pub fn is_local(&self) -> bool {
        self.0[0] & 0x02 != 0
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl FromStr for MacAddress {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split([':', '-']).collect();
        if parts.len() != 6 {
            return Err(format!("Invalid MAC address format: {s}"));
        }

        let mut bytes = [0u8; 6];
        for (i, part) in parts.iter().enumerate() {
            bytes[i] = u8::from_str_radix(part, 16)
                .map_err(|_| format!("Invalid hex digit in MAC address: {part}"))?;
        }

        Ok(Self(bytes))
    }
}

/// IPv4 subnet with CIDR notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ipv4Subnet {
    /// Network address
    pub network: Ipv4Addr,
    /// Prefix length (0-32)
    pub prefix_len: u8,
}

impl Ipv4Subnet {
    /// Creates a new IPv4 subnet.
    #[must_use]
    pub const fn new(network: Ipv4Addr, prefix_len: u8) -> Self {
        Self {
            network,
            prefix_len,
        }
    }

    /// Returns the subnet mask.
    #[must_use]
    pub fn netmask(&self) -> Ipv4Addr {
        if self.prefix_len == 0 {
            Ipv4Addr::UNSPECIFIED
        } else if self.prefix_len >= 32 {
            Ipv4Addr::BROADCAST
        } else {
            let mask = !((1u32 << (32 - self.prefix_len)) - 1);
            Ipv4Addr::from(mask)
        }
    }

    /// Checks if an IP address is within this subnet.
    #[must_use]
    pub fn contains(&self, addr: Ipv4Addr) -> bool {
        let network = u32::from(self.network);
        let addr = u32::from(addr);
        let mask = u32::from(self.netmask());
        (network & mask) == (addr & mask)
    }
}

impl fmt::Display for Ipv4Subnet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix_len)
    }
}

/// IPv6 subnet with CIDR notation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ipv6Subnet {
    /// Network address
    pub network: Ipv6Addr,
    /// Prefix length (0-128)
    pub prefix_len: u8,
}

impl Ipv6Subnet {
    /// Creates a new IPv6 subnet.
    #[must_use]
    pub const fn new(network: Ipv6Addr, prefix_len: u8) -> Self {
        Self {
            network,
            prefix_len,
        }
    }
}

impl fmt::Display for Ipv6Subnet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.network, self.prefix_len)
    }
}

/// Generic IP subnet (v4 or v6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IpSubnet {
    /// IPv4 subnet
    V4(Ipv4Subnet),
    /// IPv6 subnet
    V6(Ipv6Subnet),
}

impl fmt::Display for IpSubnet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V4(s) => write!(f, "{s}"),
            Self::V6(s) => write!(f, "{s}"),
        }
    }
}
