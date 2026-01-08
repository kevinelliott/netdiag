//! Packet capture types.

use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::time::Duration;

/// Captured packet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedPacket {
    /// Capture timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Packet length (captured)
    pub length: u32,
    /// Original packet length
    pub original_length: u32,
    /// Interface name
    pub interface: String,
    /// Decoded packet info
    pub info: PacketInfo,
    /// Raw packet data (if retained)
    pub data: Option<Vec<u8>>,
}

/// Decoded packet information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketInfo {
    /// Ethernet layer info
    pub ethernet: Option<EthernetInfo>,
    /// IP layer info
    pub ip: Option<IpInfo>,
    /// Transport layer info
    pub transport: Option<TransportInfo>,
    /// Application layer protocol
    pub application: Option<ApplicationProtocol>,
}

/// Ethernet frame information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetInfo {
    /// Source MAC address
    pub src_mac: String,
    /// Destination MAC address
    pub dst_mac: String,
    /// EtherType
    pub ether_type: u16,
}

/// IP layer information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfo {
    /// IP version (4 or 6)
    pub version: u8,
    /// Source IP address
    pub src_ip: IpAddr,
    /// Destination IP address
    pub dst_ip: IpAddr,
    /// Protocol number
    pub protocol: u8,
    /// TTL / Hop Limit
    pub ttl: u8,
    /// Total length
    pub length: u16,
    /// Fragment info
    pub fragment: Option<FragmentInfo>,
}

/// IP fragment information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentInfo {
    /// Fragment offset
    pub offset: u16,
    /// More fragments flag
    pub more_fragments: bool,
    /// Don't fragment flag
    pub dont_fragment: bool,
    /// Fragment ID
    pub id: u16,
}

/// Transport layer information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransportInfo {
    /// TCP segment
    Tcp(TcpInfo),
    /// UDP datagram
    Udp(UdpInfo),
    /// ICMP packet
    Icmp(IcmpInfo),
}

/// TCP segment information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpInfo {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Sequence number
    pub seq: u32,
    /// Acknowledgment number
    pub ack: u32,
    /// Flags
    pub flags: TcpFlags,
    /// Window size
    pub window: u16,
    /// Payload length
    pub payload_len: u16,
}

/// TCP flags.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TcpFlags {
    /// FIN flag
    pub fin: bool,
    /// SYN flag
    pub syn: bool,
    /// RST flag
    pub rst: bool,
    /// PSH flag
    pub psh: bool,
    /// ACK flag
    pub ack: bool,
    /// URG flag
    pub urg: bool,
    /// ECE flag
    pub ece: bool,
    /// CWR flag
    pub cwr: bool,
}

/// UDP datagram information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpInfo {
    /// Source port
    pub src_port: u16,
    /// Destination port
    pub dst_port: u16,
    /// Datagram length
    pub length: u16,
}

/// ICMP packet information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IcmpInfo {
    /// ICMP type
    pub icmp_type: u8,
    /// ICMP code
    pub code: u8,
    /// ICMP message type name
    pub type_name: String,
}

/// Application layer protocol.
#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum ApplicationProtocol {
    /// HTTP
    Http,
    /// HTTPS/TLS
    Https,
    /// DNS
    Dns,
    /// DHCP
    Dhcp,
    /// SSH
    Ssh,
    /// FTP
    Ftp,
    /// SMTP
    Smtp,
    /// IMAP
    Imap,
    /// POP3
    Pop3,
    /// NTP
    Ntp,
    /// SNMP
    Snmp,
    /// Unknown
    Unknown,
}

/// Capture filter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureFilter {
    /// BPF filter string
    pub bpf: Option<String>,
    /// Host filter (src or dst)
    pub host: Option<IpAddr>,
    /// Port filter
    pub port: Option<u16>,
    /// Protocol filter
    pub protocol: Option<String>,
    /// Capture limit (number of packets)
    pub limit: Option<u32>,
    /// Time limit
    pub duration: Option<Duration>,
}

impl Default for CaptureFilter {
    fn default() -> Self {
        Self {
            bpf: None,
            host: None,
            port: None,
            protocol: None,
            limit: None,
            duration: None,
        }
    }
}

/// Capture statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CaptureStats {
    /// Packets received
    pub packets_received: u64,
    /// Packets dropped by kernel
    pub packets_dropped: u64,
    /// Packets dropped by interface
    pub packets_if_dropped: u64,
    /// Bytes captured
    pub bytes_captured: u64,
    /// Capture duration
    pub duration: Duration,
    /// Protocol breakdown
    pub protocol_stats: ProtocolStats,
}

/// Protocol statistics from capture.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolStats {
    /// TCP packet count
    pub tcp: u64,
    /// UDP packet count
    pub udp: u64,
    /// ICMP packet count
    pub icmp: u64,
    /// Other protocol count
    pub other: u64,
    /// IPv4 packet count
    pub ipv4: u64,
    /// IPv6 packet count
    pub ipv6: u64,
}

/// Capture handle for managing active captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CaptureHandle(pub u64);
