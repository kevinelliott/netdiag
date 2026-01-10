//! NetDiag GUI - Tauri backend
//!
//! Provides Tauri commands that wrap the netdiag core functionality.

use serde::{Deserialize, Serialize};

mod commands;
mod error;
mod state;

pub use error::{GuiError, GuiResult};
pub use state::AppState;

/// Network interface information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceInfo {
    pub name: String,
    pub friendly_name: Option<String>,
    pub mac_address: Option<String>,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub is_up: bool,
    pub is_loopback: bool,
    pub is_default: bool,
    pub interface_type: String,
}

/// System information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_type: String,
    pub os_version: String,
    pub architecture: String,
    pub uptime_seconds: Option<u64>,
}

/// Ping result for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub target: String,
    pub resolved_ip: Option<String>,
    pub sent: u32,
    pub received: u32,
    pub lost: u32,
    pub loss_percent: f64,
    pub min_ms: Option<f64>,
    pub avg_ms: Option<f64>,
    pub max_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub error: Option<String>,
}

/// Traceroute hop for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteHop {
    pub hop: u8,
    pub address: Option<String>,
    pub hostname: Option<String>,
    pub rtt_ms: Vec<Option<f64>>,
    pub is_timeout: bool,
}

/// Traceroute result for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracerouteResult {
    pub target: String,
    pub resolved_ip: Option<String>,
    pub hops: Vec<TracerouteHop>,
    pub reached_destination: bool,
    pub error: Option<String>,
}

/// DNS lookup result for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsResult {
    pub hostname: String,
    pub addresses: Vec<String>,
    pub duration_ms: f64,
    pub error: Option<String>,
}

/// WiFi interface information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiInterfaceInfo {
    pub name: String,
    pub powered_on: bool,
    pub mac_address: Option<String>,
}

/// WiFi connection info for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiConnectionInfo {
    pub interface: String,
    pub ssid: Option<String>,
    pub bssid: Option<String>,
    pub rssi: Option<i32>,
    pub noise: Option<i32>,
    pub snr: Option<i32>,
    pub channel: Option<u8>,
    pub band: Option<String>,
    pub security: Option<String>,
    pub tx_rate: Option<f32>,
    pub wifi_standard: Option<String>,
    pub signal_quality: String,
}

/// Speed test result for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedTestResultOutput {
    pub download_mbps: Option<f64>,
    pub upload_mbps: Option<f64>,
    pub latency_ms: Option<f64>,
    pub jitter_ms: Option<f64>,
    pub server_name: String,
    pub server_location: Option<String>,
    pub test_duration_secs: f64,
    pub buffer_bloat_grade: Option<String>,
    pub consistency_rating: Option<String>,
}

/// Access point information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPointInfo {
    pub ssid: String,
    pub bssid: String,
    pub rssi: i32,
    pub signal_quality: u8,
    pub channel: u8,
    pub band: String,
    pub security: String,
    pub wifi_standard: String,
    pub is_connected: bool,
}

/// Channel analysis for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAnalysis {
    pub channel: u8,
    pub band: String,
    pub network_count: usize,
    pub interference_level: String,
    pub is_dfs: bool,
    pub is_recommended: bool,
    pub is_current: bool,
}

/// WiFi interference report for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterferenceReport {
    pub current_channel: Option<u8>,
    pub snr_rating: String,
    pub channel_utilization: Option<f64>,
    pub overlapping_networks: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Individual diagnostic test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticTest {
    pub name: String,
    pub category: String,
    pub passed: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub duration_ms: u64,
}

/// Summary of all diagnostic tests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub overall_status: String,
}

/// Full diagnostics result for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsResult {
    pub tests: Vec<DiagnosticTest>,
    pub summary: DiagnosticsSummary,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Generated report for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedReport {
    /// Report content (base64 for PDF, plain text otherwise)
    pub content: String,
    /// MIME type of the report
    pub mime_type: String,
    /// File extension for the report
    pub file_extension: String,
    /// Whether the content is binary (base64 encoded)
    pub is_binary: bool,
    /// Health score (0-100)
    pub health_score: u8,
    /// Health status ("good", "warning", "critical")
    pub health_status: String,
}

/// Available fix action information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixActionInfo {
    /// Unique identifier for this action.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of what this fix does.
    pub description: String,
    /// Severity/impact level (low, medium, high, critical).
    pub severity: String,
    /// Category (dns, adapter, tcp_ip, wifi, routing, firewall, service).
    pub category: String,
    /// Whether this fix can be rolled back.
    pub reversible: bool,
    /// Estimated time to apply (seconds).
    pub estimated_time_secs: u32,
    /// Prerequisites for this fix.
    pub prerequisites: Vec<String>,
}

/// Result of applying a fix for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResultInfo {
    /// The action ID that was applied.
    pub action_id: String,
    /// Whether the fix was successful.
    pub success: bool,
    /// Output message.
    pub message: Option<String>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Time taken to apply (milliseconds).
    pub duration_ms: u64,
    /// Rollback point ID if created.
    pub rollback_id: Option<String>,
}

/// Rollback point information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPointInfo {
    /// Unique identifier.
    pub id: String,
    /// When this point was created (ISO 8601).
    pub created_at: String,
    /// Description of what was changed.
    pub description: String,
    /// Whether this rollback point is still valid.
    pub valid: bool,
}

/// Capture device information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureDeviceInfo {
    /// Device name.
    pub name: String,
    /// Device description.
    pub description: Option<String>,
    /// IP addresses assigned to this device.
    pub addresses: Vec<String>,
    /// Is this a loopback device?
    pub is_loopback: bool,
    /// Is the device up?
    pub is_up: bool,
}

/// Captured packet information for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedPacketInfo {
    /// Timestamp (ISO 8601).
    pub timestamp: String,
    /// Protocol name.
    pub protocol: String,
    /// Source IP address.
    pub src_ip: Option<String>,
    /// Destination IP address.
    pub dst_ip: Option<String>,
    /// Source port.
    pub src_port: Option<u16>,
    /// Destination port.
    pub dst_port: Option<u16>,
    /// Packet length.
    pub length: usize,
    /// Summary of the packet.
    pub summary: String,
}

/// Capture statistics for the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureStatsInfo {
    /// Total packets captured.
    pub total_packets: u64,
    /// Total bytes captured.
    pub total_bytes: u64,
    /// Packets dropped.
    pub packets_dropped: u64,
    /// Protocol breakdown.
    pub protocol_breakdown: Vec<(String, u64)>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
        .manage(AppState::new())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_system_info,
            commands::get_interfaces,
            commands::get_default_gateway,
            commands::get_dns_servers,
            commands::ping_target,
            commands::traceroute_target,
            commands::dns_lookup,
            commands::check_connectivity,
            commands::get_wifi_interfaces,
            commands::get_wifi_connection,
            commands::run_speed_test,
            commands::get_speed_test_providers,
            commands::scan_wifi_networks,
            commands::analyze_wifi_channels,
            commands::check_wifi_interference,
            commands::run_diagnostics,
            commands::generate_report,
            commands::get_available_fixes,
            commands::apply_fix,
            commands::list_rollback_points,
            commands::perform_rollback,
            commands::is_autofix_available,
            commands::list_capture_devices,
            commands::capture_packets,
            commands::is_capture_available,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
