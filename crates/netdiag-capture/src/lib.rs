//! # netdiag-capture
//!
//! Packet capture and protocol analysis module for netdiag.
//!
//! Provides pcap-based packet capture with:
//! - Live interface capture
//! - Protocol decoding (Ethernet, IP, TCP, UDP, ICMP, DNS, HTTP)
//! - BPF filter support
//! - Packet statistics
//! - PCAP file reading/writing

#![warn(missing_docs)]
#![warn(clippy::all)]

mod capture;
mod decode;
mod error;
mod filter;
mod stats;

pub use capture::{CaptureConfig, CaptureHandle, PacketCapture};
pub use decode::{DecodedPacket, Protocol, ProtocolDecoder};
pub use error::{CaptureError, CaptureResult};
pub use filter::CaptureFilter;
pub use stats::{CaptureStats, ProtocolStats};

use pcap::Device;

/// List available network devices for capture.
pub fn list_devices() -> CaptureResult<Vec<CaptureDevice>> {
    let devices = Device::list().map_err(|e| CaptureError::PcapError(e.to_string()))?;

    Ok(devices
        .into_iter()
        .map(|d| CaptureDevice {
            name: d.name.clone(),
            description: d.desc.clone(),
            addresses: d.addresses.iter().map(|a| a.addr.to_string()).collect(),
            is_loopback: d.flags.is_loopback(),
            is_up: d.flags.is_up(),
            is_running: d.flags.is_running(),
        })
        .collect())
}

/// Get the default capture device.
pub fn default_device() -> CaptureResult<CaptureDevice> {
    let device = Device::lookup()
        .map_err(|e| CaptureError::PcapError(e.to_string()))?
        .ok_or(CaptureError::NoDeviceFound)?;

    Ok(CaptureDevice {
        name: device.name.clone(),
        description: device.desc.clone(),
        addresses: device
            .addresses
            .iter()
            .map(|a| a.addr.to_string())
            .collect(),
        is_loopback: device.flags.is_loopback(),
        is_up: device.flags.is_up(),
        is_running: device.flags.is_running(),
    })
}

/// A network device available for capture.
#[derive(Debug, Clone)]
pub struct CaptureDevice {
    /// Device name (e.g., "eth0", "en0").
    pub name: String,
    /// Device description.
    pub description: Option<String>,
    /// IP addresses assigned to this device.
    pub addresses: Vec<String>,
    /// Is this a loopback device?
    pub is_loopback: bool,
    /// Is the device up?
    pub is_up: bool,
    /// Is the device running?
    pub is_running: bool,
}
