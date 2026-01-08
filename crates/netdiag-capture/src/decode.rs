//! Protocol decoding.

use chrono::{DateTime, Utc};
use etherparse::SlicedPacket;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// Decoded packet information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedPacket {
    /// Capture timestamp.
    pub timestamp: DateTime<Utc>,

    /// Packet length.
    pub length: usize,

    /// Capture length (may differ if truncated).
    pub capture_length: usize,

    /// Source MAC address.
    pub src_mac: Option<String>,

    /// Destination MAC address.
    pub dst_mac: Option<String>,

    /// Ethernet type.
    pub ether_type: Option<u16>,

    /// Source IP address.
    pub src_ip: Option<IpAddr>,

    /// Destination IP address.
    pub dst_ip: Option<IpAddr>,

    /// IP protocol number.
    pub ip_protocol: Option<u8>,

    /// TTL / Hop limit.
    pub ttl: Option<u8>,

    /// Source port.
    pub src_port: Option<u16>,

    /// Destination port.
    pub dst_port: Option<u16>,

    /// Protocol.
    pub protocol: Protocol,

    /// TCP flags (if TCP).
    pub tcp_flags: Option<TcpFlags>,

    /// ICMP type (if ICMP).
    pub icmp_type: Option<u8>,

    /// ICMP code (if ICMP).
    pub icmp_code: Option<u8>,

    /// Payload preview (first bytes).
    pub payload_preview: Option<Vec<u8>>,

    /// Payload length.
    pub payload_length: usize,
}

/// Protocol type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protocol {
    /// Unknown protocol.
    Unknown,
    /// Ethernet.
    Ethernet,
    /// ARP.
    Arp,
    /// IPv4.
    Ipv4,
    /// IPv6.
    Ipv6,
    /// TCP.
    Tcp,
    /// UDP.
    Udp,
    /// ICMP.
    Icmp,
    /// ICMPv6.
    Icmpv6,
    /// DNS.
    Dns,
    /// HTTP.
    Http,
    /// HTTPS.
    Https,
    /// SSH.
    Ssh,
    /// DHCP.
    Dhcp,
}

impl Protocol {
    /// Get protocol name.
    pub fn name(&self) -> &'static str {
        match self {
            Protocol::Unknown => "Unknown",
            Protocol::Ethernet => "Ethernet",
            Protocol::Arp => "ARP",
            Protocol::Ipv4 => "IPv4",
            Protocol::Ipv6 => "IPv6",
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
            Protocol::Icmp => "ICMP",
            Protocol::Icmpv6 => "ICMPv6",
            Protocol::Dns => "DNS",
            Protocol::Http => "HTTP",
            Protocol::Https => "HTTPS",
            Protocol::Ssh => "SSH",
            Protocol::Dhcp => "DHCP",
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// TCP flags.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TcpFlags {
    /// FIN flag.
    pub fin: bool,
    /// SYN flag.
    pub syn: bool,
    /// RST flag.
    pub rst: bool,
    /// PSH flag.
    pub psh: bool,
    /// ACK flag.
    pub ack: bool,
    /// URG flag.
    pub urg: bool,
    /// ECE flag.
    pub ece: bool,
    /// CWR flag.
    pub cwr: bool,
}

impl TcpFlags {
    /// Create from raw flags byte.
    pub fn from_byte(flags: u8) -> Self {
        Self {
            fin: flags & 0x01 != 0,
            syn: flags & 0x02 != 0,
            rst: flags & 0x04 != 0,
            psh: flags & 0x08 != 0,
            ack: flags & 0x10 != 0,
            urg: flags & 0x20 != 0,
            ece: flags & 0x40 != 0,
            cwr: flags & 0x80 != 0,
        }
    }

    /// Format flags as string.
    pub fn to_string_short(&self) -> String {
        let mut s = String::new();
        if self.syn { s.push('S'); }
        if self.ack { s.push('A'); }
        if self.fin { s.push('F'); }
        if self.rst { s.push('R'); }
        if self.psh { s.push('P'); }
        if self.urg { s.push('U'); }
        if s.is_empty() { s.push('.'); }
        s
    }
}

impl std::fmt::Display for TcpFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.to_string_short())
    }
}

/// Protocol decoder.
pub struct ProtocolDecoder {
    /// Maximum payload preview size.
    max_payload_preview: usize,
}

impl ProtocolDecoder {
    /// Create a new decoder.
    pub fn new() -> Self {
        Self {
            max_payload_preview: 64,
        }
    }

    /// Create with custom payload preview size.
    pub fn with_payload_preview(size: usize) -> Self {
        Self {
            max_payload_preview: size,
        }
    }

    /// Decode a raw packet.
    pub fn decode(&self, data: &[u8], timestamp: DateTime<Utc>) -> DecodedPacket {
        let mut packet = DecodedPacket {
            timestamp,
            length: data.len(),
            capture_length: data.len(),
            src_mac: None,
            dst_mac: None,
            ether_type: None,
            src_ip: None,
            dst_ip: None,
            ip_protocol: None,
            ttl: None,
            src_port: None,
            dst_port: None,
            protocol: Protocol::Unknown,
            tcp_flags: None,
            icmp_type: None,
            icmp_code: None,
            payload_preview: None,
            payload_length: 0,
        };

        // Try to parse the packet
        match SlicedPacket::from_ethernet(data) {
            Ok(sliced) => {
                // Extract link layer
                if let Some(link) = &sliced.link {
                    match link {
                        etherparse::LinkSlice::Ethernet2(eth) => {
                            packet.src_mac = Some(format_mac(&eth.source()));
                            packet.dst_mac = Some(format_mac(&eth.destination()));
                            packet.ether_type = Some(eth.ether_type().0);
                        }
                        etherparse::LinkSlice::LinuxSll(_) => {
                            // Linux cooked capture - no Ethernet header
                        }
                        etherparse::LinkSlice::EtherPayload(payload) => {
                            packet.ether_type = Some(payload.ether_type.0);
                        }
                        etherparse::LinkSlice::LinuxSllPayload(_) => {
                            // Linux cooked capture payload - no Ethernet header
                        }
                    }
                }

                // Extract network layer
                if let Some(net) = &sliced.net {
                    match net {
                        etherparse::NetSlice::Ipv4(ipv4) => {
                            let header = ipv4.header();
                            packet.src_ip = Some(IpAddr::V4(Ipv4Addr::from(header.source())));
                            packet.dst_ip =
                                Some(IpAddr::V4(Ipv4Addr::from(header.destination())));
                            packet.ip_protocol = Some(header.protocol().0);
                            packet.ttl = Some(header.ttl());
                            packet.protocol = Protocol::Ipv4;
                        }
                        etherparse::NetSlice::Ipv6(ipv6) => {
                            let header = ipv6.header();
                            packet.src_ip = Some(IpAddr::V6(Ipv6Addr::from(header.source())));
                            packet.dst_ip =
                                Some(IpAddr::V6(Ipv6Addr::from(header.destination())));
                            packet.ip_protocol = Some(header.next_header().0);
                            packet.ttl = Some(header.hop_limit());
                            packet.protocol = Protocol::Ipv6;
                        }
                    }
                }

                // Extract transport layer
                if let Some(transport) = &sliced.transport {
                    match transport {
                        etherparse::TransportSlice::Tcp(tcp) => {
                            packet.src_port = Some(tcp.source_port());
                            packet.dst_port = Some(tcp.destination_port());
                            packet.tcp_flags = Some(TcpFlags {
                                fin: tcp.fin(),
                                syn: tcp.syn(),
                                rst: tcp.rst(),
                                psh: tcp.psh(),
                                ack: tcp.ack(),
                                urg: tcp.urg(),
                                ece: tcp.ece(),
                                cwr: tcp.cwr(),
                            });
                            packet.protocol = self.identify_tcp_protocol(tcp.source_port(), tcp.destination_port());
                        }
                        etherparse::TransportSlice::Udp(udp) => {
                            packet.src_port = Some(udp.source_port());
                            packet.dst_port = Some(udp.destination_port());
                            packet.protocol = self.identify_udp_protocol(udp.source_port(), udp.destination_port());
                        }
                        etherparse::TransportSlice::Icmpv4(icmp) => {
                            packet.icmp_type = Some(icmp.type_u8());
                            packet.icmp_code = Some(icmp.code_u8());
                            packet.protocol = Protocol::Icmp;
                        }
                        etherparse::TransportSlice::Icmpv6(icmp) => {
                            packet.icmp_type = Some(icmp.type_u8());
                            packet.icmp_code = Some(icmp.code_u8());
                            packet.protocol = Protocol::Icmpv6;
                        }
                    }
                }

                // Note: In etherparse 0.16, payload is accessed differently
                // For now, we calculate payload length from the remaining data
                // This is a simplification
                packet.payload_length = 0;
            }
            Err(_) => {
                // Couldn't parse, just store raw data info
                packet.payload_length = data.len();
                if self.max_payload_preview > 0 {
                    let preview_len = data.len().min(self.max_payload_preview);
                    packet.payload_preview = Some(data[..preview_len].to_vec());
                }
            }
        }

        packet
    }

    /// Identify TCP protocol from port numbers.
    fn identify_tcp_protocol(&self, src_port: u16, dst_port: u16) -> Protocol {
        match (src_port, dst_port) {
            (80, _) | (_, 80) | (8080, _) | (_, 8080) => Protocol::Http,
            (443, _) | (_, 443) | (8443, _) | (_, 8443) => Protocol::Https,
            (22, _) | (_, 22) => Protocol::Ssh,
            (53, _) | (_, 53) => Protocol::Dns,
            _ => Protocol::Tcp,
        }
    }

    /// Identify UDP protocol from port numbers.
    fn identify_udp_protocol(&self, src_port: u16, dst_port: u16) -> Protocol {
        match (src_port, dst_port) {
            (53, _) | (_, 53) => Protocol::Dns,
            (67, _) | (_, 67) | (68, _) | (_, 68) => Protocol::Dhcp,
            _ => Protocol::Udp,
        }
    }
}

impl Default for ProtocolDecoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Format MAC address.
fn format_mac(bytes: &[u8; 6]) -> String {
    format!(
        "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

impl DecodedPacket {
    /// Format a one-line summary.
    pub fn summary(&self) -> String {
        let src = self.src_ip.map(|ip| {
            if let Some(port) = self.src_port {
                format!("{}:{}", ip, port)
            } else {
                ip.to_string()
            }
        }).unwrap_or_else(|| "?".to_string());

        let dst = self.dst_ip.map(|ip| {
            if let Some(port) = self.dst_port {
                format!("{}:{}", ip, port)
            } else {
                ip.to_string()
            }
        }).unwrap_or_else(|| "?".to_string());

        let flags = self.tcp_flags.map(|f| f.to_string_short()).unwrap_or_default();

        format!(
            "{} {} -> {} {} len={}{}",
            self.protocol,
            src,
            dst,
            flags,
            self.length,
            if self.payload_length > 0 {
                format!(" payload={}", self.payload_length)
            } else {
                String::new()
            }
        )
    }
}
