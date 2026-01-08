//! Packet capture functionality.

use crate::decode::{DecodedPacket, ProtocolDecoder};
use crate::error::{CaptureError, CaptureResult};
use crate::filter::CaptureFilter;
use crate::stats::CaptureStats;
use chrono::Utc;
use pcap::{Active, Capture, Device};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Capture configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Device name to capture on.
    pub device: String,

    /// BPF filter.
    pub filter: CaptureFilter,

    /// Promiscuous mode.
    pub promiscuous: bool,

    /// Snapshot length (max bytes per packet).
    pub snaplen: i32,

    /// Buffer size (bytes).
    pub buffer_size: i32,

    /// Read timeout (milliseconds).
    pub timeout_ms: i32,

    /// Maximum packets to capture (0 = unlimited).
    pub max_packets: usize,

    /// Maximum capture duration.
    pub max_duration: Option<Duration>,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            device: String::new(),
            filter: CaptureFilter::all(),
            promiscuous: true,
            snaplen: 65535,
            buffer_size: 1024 * 1024, // 1 MB
            timeout_ms: 1000,
            max_packets: 0,
            max_duration: None,
        }
    }
}

impl CaptureConfig {
    /// Create config for a specific device.
    pub fn for_device(device: &str) -> Self {
        Self {
            device: device.to_string(),
            ..Default::default()
        }
    }

    /// Set BPF filter.
    pub fn with_filter(mut self, filter: CaptureFilter) -> Self {
        self.filter = filter;
        self
    }

    /// Set promiscuous mode.
    pub fn promiscuous(mut self, enabled: bool) -> Self {
        self.promiscuous = enabled;
        self
    }

    /// Set max packets.
    pub fn max_packets(mut self, max: usize) -> Self {
        self.max_packets = max;
        self
    }

    /// Set max duration.
    pub fn max_duration(mut self, duration: Duration) -> Self {
        self.max_duration = Some(duration);
        self
    }
}

/// Packet capture handle.
pub struct CaptureHandle {
    /// Stop flag.
    stop_flag: Arc<AtomicBool>,
}

impl CaptureHandle {
    /// Create a new handle.
    fn new(stop_flag: Arc<AtomicBool>) -> Self {
        Self { stop_flag }
    }

    /// Stop the capture.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// Check if stopped.
    pub fn is_stopped(&self) -> bool {
        self.stop_flag.load(Ordering::SeqCst)
    }
}

/// Packet capture manager.
pub struct PacketCapture {
    config: CaptureConfig,
    decoder: ProtocolDecoder,
}

impl PacketCapture {
    /// Create a new packet capture.
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            config,
            decoder: ProtocolDecoder::new(),
        }
    }

    /// Create capture for default device.
    pub fn default_device() -> CaptureResult<Self> {
        let device = Device::lookup()
            .map_err(|e| CaptureError::PcapError(e.to_string()))?
            .ok_or(CaptureError::NoDeviceFound)?;

        Ok(Self::new(CaptureConfig::for_device(&device.name)))
    }

    /// Open the capture device.
    fn open_capture(&self) -> CaptureResult<Capture<Active>> {
        let device = if self.config.device.is_empty() {
            Device::lookup()
                .map_err(|e| CaptureError::PcapError(e.to_string()))?
                .ok_or(CaptureError::NoDeviceFound)?
        } else {
            Device::list()
                .map_err(|e| CaptureError::PcapError(e.to_string()))?
                .into_iter()
                .find(|d| d.name == self.config.device)
                .ok_or_else(|| CaptureError::DeviceNotFound(self.config.device.clone()))?
        };

        debug!("Opening capture on device: {}", device.name);

        let mut cap = Capture::from_device(device)
            .map_err(|e| {
                if e.to_string().contains("permission") {
                    CaptureError::PermissionDenied
                } else {
                    CaptureError::PcapError(e.to_string())
                }
            })?
            .promisc(self.config.promiscuous)
            .snaplen(self.config.snaplen)
            .buffer_size(self.config.buffer_size)
            .timeout(self.config.timeout_ms)
            .open()
            .map_err(|e| {
                if e.to_string().contains("permission") {
                    CaptureError::PermissionDenied
                } else {
                    CaptureError::PcapError(e.to_string())
                }
            })?;

        // Apply filter
        if !self.config.filter.is_empty() {
            debug!("Applying filter: {}", self.config.filter);
            cap.filter(self.config.filter.as_str(), true)
                .map_err(|e| CaptureError::InvalidFilter(e.to_string()))?;
        }

        Ok(cap)
    }

    /// Start capturing packets.
    ///
    /// Returns a channel receiver for decoded packets and a handle to stop capture.
    pub fn start(&self) -> CaptureResult<(mpsc::Receiver<DecodedPacket>, CaptureHandle)> {
        let (tx, rx) = mpsc::channel(1000);
        let stop_flag = Arc::new(AtomicBool::new(false));
        let handle = CaptureHandle::new(stop_flag.clone());

        let mut cap = self.open_capture()?;
        let decoder = self.decoder.clone();
        let max_packets = self.config.max_packets;
        let max_duration = self.config.max_duration;
        let start_time = std::time::Instant::now();

        std::thread::spawn(move || {
            let mut packet_count = 0usize;

            loop {
                // Check stop conditions
                if stop_flag.load(Ordering::SeqCst) {
                    debug!("Capture stopped by handle");
                    break;
                }

                if max_packets > 0 && packet_count >= max_packets {
                    debug!("Max packets reached: {}", max_packets);
                    break;
                }

                if let Some(max_dur) = max_duration {
                    if start_time.elapsed() >= max_dur {
                        debug!("Max duration reached");
                        break;
                    }
                }

                // Try to get next packet
                match cap.next_packet() {
                    Ok(packet) => {
                        let decoded = decoder.decode(packet.data, Utc::now());

                        if tx.blocking_send(decoded).is_err() {
                            // Receiver dropped
                            debug!("Receiver dropped, stopping capture");
                            break;
                        }

                        packet_count += 1;
                    }
                    Err(pcap::Error::TimeoutExpired) => {
                        // Normal timeout, continue
                        continue;
                    }
                    Err(e) => {
                        warn!("Capture error: {}", e);
                        break;
                    }
                }
            }

            info!("Capture finished: {} packets", packet_count);
        });

        Ok((rx, handle))
    }

    /// Capture packets synchronously with callback.
    pub fn capture_sync<F>(&self, mut callback: F) -> CaptureResult<CaptureStats>
    where
        F: FnMut(DecodedPacket) -> bool,
    {
        let mut cap = self.open_capture()?;
        let mut stats = CaptureStats::new();
        let start_time = std::time::Instant::now();
        let mut packet_count = 0usize;

        loop {
            // Check stop conditions
            if self.config.max_packets > 0 && packet_count >= self.config.max_packets {
                break;
            }

            if let Some(max_dur) = self.config.max_duration {
                if start_time.elapsed() >= max_dur {
                    break;
                }
            }

            // Try to get next packet
            match cap.next_packet() {
                Ok(packet) => {
                    let decoded = self.decoder.decode(packet.data, Utc::now());

                    // Update stats
                    stats.update(
                        decoded.protocol,
                        decoded.length,
                        decoded.src_ip.as_ref().map(|ip| ip.to_string()).as_deref(),
                        decoded.dst_ip.as_ref().map(|ip| ip.to_string()).as_deref(),
                        decoded.src_port,
                        decoded.dst_port,
                    );

                    // Call user callback
                    if !callback(decoded) {
                        break;
                    }

                    packet_count += 1;
                }
                Err(pcap::Error::TimeoutExpired) => {
                    continue;
                }
                Err(e) => {
                    warn!("Capture error: {}", e);
                    break;
                }
            }
        }

        // Get pcap stats
        if let Ok(pcap_stats) = cap.stats() {
            stats.packets_dropped = pcap_stats.dropped as u64;
            stats.packets_dropped_interface = pcap_stats.if_dropped as u64;
        }

        stats.finalize();
        Ok(stats)
    }

    /// Quick capture of N packets.
    pub fn capture_packets(&self, count: usize) -> CaptureResult<Vec<DecodedPacket>> {
        let mut packets = Vec::with_capacity(count);
        let config = CaptureConfig {
            max_packets: count,
            ..self.config.clone()
        };
        let capture = PacketCapture::new(config);

        capture.capture_sync(|packet| {
            packets.push(packet);
            packets.len() < count
        })?;

        Ok(packets)
    }
}

impl Clone for ProtocolDecoder {
    fn clone(&self) -> Self {
        Self::new()
    }
}
