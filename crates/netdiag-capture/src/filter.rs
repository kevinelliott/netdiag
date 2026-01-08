//! BPF filter support.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// Capture filter using BPF syntax.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureFilter {
    /// Raw BPF filter string.
    filter: String,
}

impl CaptureFilter {
    /// Create a filter from a raw BPF string.
    pub fn new(filter: impl Into<String>) -> Self {
        Self {
            filter: filter.into(),
        }
    }

    /// Create an empty filter (capture all).
    pub fn all() -> Self {
        Self {
            filter: String::new(),
        }
    }

    /// Filter by host IP address.
    pub fn host(addr: IpAddr) -> Self {
        Self {
            filter: format!("host {}", addr),
        }
    }

    /// Filter by source IP address.
    pub fn src_host(addr: IpAddr) -> Self {
        Self {
            filter: format!("src host {}", addr),
        }
    }

    /// Filter by destination IP address.
    pub fn dst_host(addr: IpAddr) -> Self {
        Self {
            filter: format!("dst host {}", addr),
        }
    }

    /// Filter by port.
    pub fn port(port: u16) -> Self {
        Self {
            filter: format!("port {}", port),
        }
    }

    /// Filter by source port.
    pub fn src_port(port: u16) -> Self {
        Self {
            filter: format!("src port {}", port),
        }
    }

    /// Filter by destination port.
    pub fn dst_port(port: u16) -> Self {
        Self {
            filter: format!("dst port {}", port),
        }
    }

    /// Filter by protocol.
    pub fn protocol(proto: &str) -> Self {
        Self {
            filter: proto.to_lowercase(),
        }
    }

    /// Filter TCP traffic.
    pub fn tcp() -> Self {
        Self::protocol("tcp")
    }

    /// Filter UDP traffic.
    pub fn udp() -> Self {
        Self::protocol("udp")
    }

    /// Filter ICMP traffic.
    pub fn icmp() -> Self {
        Self::protocol("icmp")
    }

    /// Filter DNS traffic.
    pub fn dns() -> Self {
        Self {
            filter: "port 53".to_string(),
        }
    }

    /// Filter HTTP traffic.
    pub fn http() -> Self {
        Self {
            filter: "port 80 or port 443".to_string(),
        }
    }

    /// Combine with AND.
    pub fn and(self, other: CaptureFilter) -> Self {
        if self.filter.is_empty() {
            other
        } else if other.filter.is_empty() {
            self
        } else {
            Self {
                filter: format!("({}) and ({})", self.filter, other.filter),
            }
        }
    }

    /// Combine with OR.
    pub fn or(self, other: CaptureFilter) -> Self {
        if self.filter.is_empty() {
            other
        } else if other.filter.is_empty() {
            self
        } else {
            Self {
                filter: format!("({}) or ({})", self.filter, other.filter),
            }
        }
    }

    /// Negate the filter.
    pub fn not(self) -> Self {
        if self.filter.is_empty() {
            self
        } else {
            Self {
                filter: format!("not ({})", self.filter),
            }
        }
    }

    /// Get the BPF filter string.
    pub fn as_str(&self) -> &str {
        &self.filter
    }

    /// Check if filter is empty.
    pub fn is_empty(&self) -> bool {
        self.filter.is_empty()
    }
}

impl Default for CaptureFilter {
    fn default() -> Self {
        Self::all()
    }
}

impl std::fmt::Display for CaptureFilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.filter.is_empty() {
            write!(f, "(all)")
        } else {
            write!(f, "{}", self.filter)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_filter_host() {
        let filter = CaptureFilter::host(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert_eq!(filter.as_str(), "host 192.168.1.1");
    }

    #[test]
    fn test_filter_port() {
        let filter = CaptureFilter::port(80);
        assert_eq!(filter.as_str(), "port 80");
    }

    #[test]
    fn test_filter_and() {
        let filter = CaptureFilter::tcp().and(CaptureFilter::port(80));
        assert_eq!(filter.as_str(), "(tcp) and (port 80)");
    }

    #[test]
    fn test_filter_or() {
        let filter = CaptureFilter::port(80).or(CaptureFilter::port(443));
        assert_eq!(filter.as_str(), "(port 80) or (port 443)");
    }
}
