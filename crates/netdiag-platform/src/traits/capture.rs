//! Packet capture provider trait.

use async_trait::async_trait;
use netdiag_types::{
    capture::{CaptureFilter, CaptureHandle, CaptureStats, CapturedPacket},
    error::Result,
    system::PrivilegeLevel,
};
use tokio::sync::mpsc;

/// Provider for packet capture operations.
#[async_trait]
pub trait CaptureProvider: Send + Sync {
    /// Checks if packet capture is available on this platform.
    fn is_available(&self) -> bool;

    /// Lists available capture interfaces.
    async fn list_capture_interfaces(&self) -> Result<Vec<CaptureInterface>>;

    /// Starts a packet capture session.
    async fn start_capture(
        &self,
        interface: &str,
        filter: Option<CaptureFilter>,
        packet_tx: mpsc::Sender<CapturedPacket>,
    ) -> Result<CaptureHandle>;

    /// Stops a packet capture session.
    async fn stop_capture(&self, handle: CaptureHandle) -> Result<CaptureStats>;

    /// Gets statistics for an active capture.
    async fn get_capture_stats(&self, handle: CaptureHandle) -> Result<CaptureStats>;

    /// Gets the required privilege level for capture.
    fn required_privilege_level(&self) -> PrivilegeLevel;

    /// Compiles a BPF filter expression.
    fn compile_filter(&self, expression: &str) -> Result<String>;
}

/// Capture interface information.
#[derive(Debug, Clone)]
pub struct CaptureInterface {
    /// Interface name
    pub name: String,
    /// Interface description
    pub description: Option<String>,
    /// IPv4 addresses
    pub addresses: Vec<std::net::IpAddr>,
    /// Is this interface a loopback?
    pub is_loopback: bool,
    /// Is this interface up and running?
    pub is_up: bool,
    /// Can capture in promiscuous mode?
    pub can_promiscuous: bool,
}

/// Extension trait for capture operations.
#[async_trait]
pub trait CaptureProviderExt: CaptureProvider {
    /// Gets the best interface for capture (usually the default).
    async fn get_default_capture_interface(&self) -> Result<Option<CaptureInterface>> {
        let interfaces = self.list_capture_interfaces().await?;
        Ok(interfaces.into_iter().find(|i| i.is_up && !i.is_loopback))
    }

    /// Captures packets for a specified duration.
    async fn capture_for_duration(
        &self,
        interface: &str,
        filter: Option<CaptureFilter>,
        duration: std::time::Duration,
    ) -> Result<(Vec<CapturedPacket>, CaptureStats)> {
        let (tx, mut rx) = mpsc::channel(1000);
        let handle = self.start_capture(interface, filter, tx).await?;

        let mut packets = Vec::new();

        // Collect packets for the duration
        let start = std::time::Instant::now();
        while start.elapsed() < duration {
            match tokio::time::timeout(std::time::Duration::from_millis(100), rx.recv()).await {
                Ok(Some(packet)) => packets.push(packet),
                Ok(None) => break,  // Channel closed
                Err(_) => continue, // Timeout, continue collecting
            }
        }

        let stats = self.stop_capture(handle).await?;
        Ok((packets, stats))
    }
}

// Blanket implementation
impl<T: CaptureProvider + ?Sized> CaptureProviderExt for T {}
