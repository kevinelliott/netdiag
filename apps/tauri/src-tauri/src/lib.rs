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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
