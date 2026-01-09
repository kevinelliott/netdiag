//! Tauri commands for network diagnostics.

use crate::{
    AccessPointInfo, AppState, CaptureDeviceInfo, CaptureStatsInfo, CapturedPacketInfo,
    ChannelAnalysis, DiagnosticTest, DiagnosticsResult, DiagnosticsSummary, DnsResult,
    FixActionInfo, FixResultInfo, GeneratedReport, InterfaceInfo, InterferenceReport, PingResult,
    RollbackPointInfo, SpeedTestResultOutput, SystemInfo, TracerouteHop, TracerouteResult,
    WifiConnectionInfo, WifiInterfaceInfo,
};
use netdiag_connectivity::{DnsResolver, PingConfig, Pinger, Tracer, TracerouteConfig};
use netdiag_reports::{
    DnsSummary, HtmlFormatter, InterfaceSummary, JsonFormatter, MarkdownFormatter, PingSummary,
    ReportBuilder, ReportFormatter, TextFormatter,
};
use netdiag_speed::{SpeedTestConfig, SpeedTester};
use std::time::{Duration, Instant};
use tauri::State;

/// Get system information.
#[tauri::command]
pub async fn get_system_info(state: State<'_, AppState>) -> Result<SystemInfo, String> {
    let info = state
        .providers
        .system
        .get_system_info()
        .await
        .map_err(|e| e.to_string())?;

    Ok(SystemInfo {
        hostname: info.hostname,
        os_type: format!("{:?}", info.os_type),
        os_version: info.os_version,
        architecture: info.architecture,
        uptime_seconds: info.uptime.map(|d| d.as_secs()),
    })
}

/// Get network interfaces.
#[tauri::command]
pub async fn get_interfaces(state: State<'_, AppState>) -> Result<Vec<InterfaceInfo>, String> {
    let interfaces = state
        .providers
        .network
        .list_interfaces()
        .await
        .map_err(|e| e.to_string())?;

    let default_iface = state
        .providers
        .network
        .get_default_interface()
        .await
        .ok()
        .flatten();

    let default_name = default_iface.as_ref().map(|i| i.name.clone());

    Ok(interfaces
        .into_iter()
        .map(|iface| InterfaceInfo {
            name: iface.name.clone(),
            friendly_name: iface.display_name.clone(),
            mac_address: iface.mac_address.as_ref().map(|m| m.to_string()),
            ipv4_addresses: iface
                .ipv4_addresses
                .iter()
                .map(|a| a.address.to_string())
                .collect(),
            ipv6_addresses: iface
                .ipv6_addresses
                .iter()
                .map(|a| a.address.to_string())
                .collect(),
            is_up: iface.is_up(),
            is_loopback: iface.flags.loopback,
            is_default: Some(&iface.name) == default_name.as_ref(),
            interface_type: format!("{:?}", iface.interface_type),
        })
        .collect())
}

/// Get default gateway.
#[tauri::command]
pub async fn get_default_gateway(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let gateway = state
        .providers
        .network
        .get_default_gateway()
        .await
        .map_err(|e| e.to_string())?;

    Ok(gateway.map(|g| g.address.to_string()))
}

/// Get DNS servers.
#[tauri::command]
pub async fn get_dns_servers(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let servers = state
        .providers
        .network
        .get_dns_servers()
        .await
        .map_err(|e| e.to_string())?;

    Ok(servers.into_iter().map(|s| s.address.to_string()).collect())
}

/// Ping a target.
#[tauri::command]
pub async fn ping_target(
    target: String,
    count: Option<u32>,
    timeout_ms: Option<u64>,
) -> Result<PingResult, String> {
    let resolver = DnsResolver::new().map_err(|e| e.to_string())?;
    let pinger = Pinger::new();

    // Resolve DNS first
    let dns_result = resolver.resolve(&target).await;

    match dns_result {
        Ok(result) => {
            if result.addresses.is_empty() {
                return Ok(PingResult {
                    target: target.clone(),
                    resolved_ip: None,
                    sent: 0,
                    received: 0,
                    lost: 0,
                    loss_percent: 100.0,
                    min_ms: None,
                    avg_ms: None,
                    max_ms: None,
                    jitter_ms: None,
                    error: Some("DNS resolution returned no addresses".to_string()),
                });
            }

            let ip = result.addresses[0];
            let config = PingConfig {
                count: count.unwrap_or(4),
                timeout: Duration::from_millis(timeout_ms.unwrap_or(2000)),
                interval: Duration::from_millis(500),
                size: 64,
            };

            match pinger.ping(ip, &config).await {
                Ok(stats) => Ok(PingResult {
                    target,
                    resolved_ip: Some(ip.to_string()),
                    sent: stats.transmitted,
                    received: stats.received,
                    lost: stats.lost,
                    loss_percent: stats.loss_percent,
                    min_ms: stats.min_rtt.map(|d| d.as_secs_f64() * 1000.0),
                    avg_ms: stats.avg_rtt.map(|d| d.as_secs_f64() * 1000.0),
                    max_ms: stats.max_rtt.map(|d| d.as_secs_f64() * 1000.0),
                    jitter_ms: stats.jitter.map(|d| d.as_secs_f64() * 1000.0),
                    error: None,
                }),
                Err(e) => Ok(PingResult {
                    target,
                    resolved_ip: Some(ip.to_string()),
                    sent: 0,
                    received: 0,
                    lost: 0,
                    loss_percent: 100.0,
                    min_ms: None,
                    avg_ms: None,
                    max_ms: None,
                    jitter_ms: None,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Ok(PingResult {
            target,
            resolved_ip: None,
            sent: 0,
            received: 0,
            lost: 0,
            loss_percent: 100.0,
            min_ms: None,
            avg_ms: None,
            max_ms: None,
            jitter_ms: None,
            error: Some(format!("DNS resolution failed: {}", e)),
        }),
    }
}

/// Run traceroute to a target.
#[tauri::command]
pub async fn traceroute_target(
    target: String,
    max_hops: Option<u8>,
    timeout_ms: Option<u64>,
) -> Result<TracerouteResult, String> {
    let resolver = DnsResolver::new().map_err(|e| e.to_string())?;
    let tracer = Tracer::new();

    // Resolve DNS first
    let dns_result = resolver.resolve(&target).await;

    match dns_result {
        Ok(result) => {
            if result.addresses.is_empty() {
                return Ok(TracerouteResult {
                    target: target.clone(),
                    resolved_ip: None,
                    hops: vec![],
                    reached_destination: false,
                    error: Some("DNS resolution returned no addresses".to_string()),
                });
            }

            let ip = result.addresses[0];
            let config = TracerouteConfig {
                max_hops: max_hops.unwrap_or(30),
                probes_per_hop: 3,
                timeout: Duration::from_millis(timeout_ms.unwrap_or(2000)),
                ..Default::default()
            };

            match tracer.trace(ip, &config).await {
                Ok(trace) => {
                    let hops: Vec<TracerouteHop> = trace
                        .hops
                        .iter()
                        .map(|hop| {
                            // Extract RTTs from probes
                            let rtt_ms: Vec<Option<f64>> = hop
                                .probes
                                .iter()
                                .map(|p| p.rtt.map(|d| d.as_secs_f64() * 1000.0))
                                .collect();

                            TracerouteHop {
                                hop: hop.hop,
                                address: hop.address.map(|a| a.to_string()),
                                hostname: hop.hostname.clone(),
                                rtt_ms,
                                is_timeout: hop.all_timeout,
                            }
                        })
                        .collect();

                    Ok(TracerouteResult {
                        target,
                        resolved_ip: Some(ip.to_string()),
                        hops,
                        reached_destination: trace.reached,
                        error: None,
                    })
                }
                Err(e) => Ok(TracerouteResult {
                    target,
                    resolved_ip: Some(ip.to_string()),
                    hops: vec![],
                    reached_destination: false,
                    error: Some(e.to_string()),
                }),
            }
        }
        Err(e) => Ok(TracerouteResult {
            target,
            resolved_ip: None,
            hops: vec![],
            reached_destination: false,
            error: Some(format!("DNS resolution failed: {}", e)),
        }),
    }
}

/// Perform DNS lookup.
#[tauri::command]
pub async fn dns_lookup(hostname: String) -> Result<DnsResult, String> {
    let resolver = DnsResolver::new().map_err(|e| e.to_string())?;

    match resolver.resolve(&hostname).await {
        Ok(result) => Ok(DnsResult {
            hostname,
            addresses: result.addresses.iter().map(|a| a.to_string()).collect(),
            duration_ms: result.duration.as_secs_f64() * 1000.0,
            error: None,
        }),
        Err(e) => Ok(DnsResult {
            hostname,
            addresses: vec![],
            duration_ms: 0.0,
            error: Some(e.to_string()),
        }),
    }
}

/// Quick connectivity check.
#[tauri::command]
pub async fn check_connectivity(target: String) -> Result<PingResult, String> {
    ping_target(target, Some(3), Some(2000)).await
}

/// Get WiFi interfaces.
#[tauri::command]
pub async fn get_wifi_interfaces(
    state: State<'_, AppState>,
) -> Result<Vec<WifiInterfaceInfo>, String> {
    let interfaces = state
        .providers
        .wifi
        .list_wifi_interfaces()
        .await
        .map_err(|e| e.to_string())?;

    Ok(interfaces
        .into_iter()
        .map(|iface| WifiInterfaceInfo {
            name: iface.name.clone(),
            powered_on: iface.powered_on,
            mac_address: iface.mac_address.map(|m| m.to_string()),
        })
        .collect())
}

/// Get current WiFi connection info.
#[tauri::command]
pub async fn get_wifi_connection(
    state: State<'_, AppState>,
) -> Result<Option<WifiConnectionInfo>, String> {
    let interfaces = state
        .providers
        .wifi
        .list_wifi_interfaces()
        .await
        .map_err(|e| e.to_string())?;

    if interfaces.is_empty() {
        return Ok(None);
    }

    let iface = &interfaces[0];
    if !iface.powered_on {
        return Ok(Some(WifiConnectionInfo {
            interface: iface.name.clone(),
            ssid: None,
            bssid: None,
            rssi: None,
            noise: None,
            snr: None,
            channel: None,
            band: None,
            security: None,
            tx_rate: None,
            wifi_standard: None,
            signal_quality: "WiFi Off".to_string(),
        }));
    }

    match state
        .providers
        .wifi
        .get_current_connection(&iface.name)
        .await
    {
        Ok(Some(conn)) => {
            let rssi = conn.access_point.rssi;
            let noise = conn.access_point.noise;
            let snr = noise.map(|n| rssi - n);

            let signal_quality = match rssi {
                r if r >= -50 => "Excellent".to_string(),
                r if r >= -60 => "Good".to_string(),
                r if r >= -70 => "Fair".to_string(),
                r if r >= -80 => "Weak".to_string(),
                _ => "Very Weak".to_string(),
            };

            Ok(Some(WifiConnectionInfo {
                interface: iface.name.clone(),
                ssid: Some(conn.access_point.ssid.as_str().to_string()),
                bssid: Some(conn.access_point.bssid.to_string()),
                rssi: Some(rssi),
                noise,
                snr,
                channel: Some(conn.access_point.channel.number),
                band: Some(format!("{:?}", conn.access_point.channel.band)),
                security: Some(format!("{:?}", conn.access_point.security)),
                tx_rate: conn.tx_rate,
                wifi_standard: Some(format!("{:?}", conn.access_point.wifi_standard)),
                signal_quality,
            }))
        }
        Ok(None) => Ok(Some(WifiConnectionInfo {
            interface: iface.name.clone(),
            ssid: None,
            bssid: None,
            rssi: None,
            noise: None,
            snr: None,
            channel: None,
            band: None,
            security: None,
            tx_rate: None,
            wifi_standard: None,
            signal_quality: "Not Connected".to_string(),
        })),
        Err(e) => Err(e.to_string()),
    }
}

/// Run a speed test.
#[tauri::command]
pub async fn run_speed_test(
    duration_secs: Option<u64>,
    connections: Option<usize>,
    test_download: Option<bool>,
    test_upload: Option<bool>,
) -> Result<SpeedTestResultOutput, String> {
    let tester = SpeedTester::new();

    let config = SpeedTestConfig {
        duration: Duration::from_secs(duration_secs.unwrap_or(10)),
        connections: connections.unwrap_or(4),
        test_download: test_download.unwrap_or(true),
        test_upload: test_upload.unwrap_or(true),
        ..Default::default()
    };

    match tester.run_test(&config).await {
        Ok(result) => Ok(SpeedTestResultOutput {
            download_mbps: result.download_mbps(),
            upload_mbps: result.upload_mbps(),
            latency_ms: result.latency.map(|d| d.as_secs_f64() * 1000.0),
            jitter_ms: result.jitter.map(|d| d.as_secs_f64() * 1000.0),
            server_name: result.server.name,
            server_location: result.server.location,
            test_duration_secs: result.test_duration.as_secs_f64(),
            buffer_bloat_grade: result.buffer_bloat.map(|b| b.grade.to_string()),
            consistency_rating: result.consistency.map(|c| c.rating.to_string()),
        }),
        Err(e) => Err(e.to_string()),
    }
}

/// Get available speed test providers.
#[tauri::command]
pub async fn get_speed_test_providers() -> Result<Vec<String>, String> {
    let tester = SpeedTester::new();
    let providers = tester.available_providers().await;
    Ok(providers.into_iter().map(|s| s.to_string()).collect())
}

/// Scan for WiFi networks.
#[tauri::command]
pub async fn scan_wifi_networks(
    state: State<'_, AppState>,
    interface: Option<String>,
) -> Result<Vec<AccessPointInfo>, String> {
    // Get the interface name
    let iface_name = match interface {
        Some(name) => name,
        None => {
            let interfaces = state
                .providers
                .wifi
                .list_wifi_interfaces()
                .await
                .map_err(|e| e.to_string())?;
            if interfaces.is_empty() {
                return Err("No WiFi interfaces found".to_string());
            }
            interfaces[0].name.clone()
        }
    };

    // Perform scan
    let access_points = state
        .providers
        .wifi
        .scan_access_points(&iface_name)
        .await
        .map_err(|e| e.to_string())?;

    // Get current connection to determine which AP we're connected to
    let current_bssid = state
        .providers
        .wifi
        .get_current_connection(&iface_name)
        .await
        .ok()
        .flatten()
        .map(|conn| conn.access_point.bssid.to_string());

    Ok(access_points
        .into_iter()
        .map(|ap| {
            let is_connected = current_bssid
                .as_ref()
                .map(|b| b == &ap.bssid.to_string())
                .unwrap_or(false);

            AccessPointInfo {
                ssid: ap.ssid.as_str().to_string(),
                bssid: ap.bssid.to_string(),
                rssi: ap.rssi,
                signal_quality: ap.signal_quality,
                channel: ap.channel.number,
                band: format!("{:?}", ap.channel.band),
                security: format!("{:?}", ap.security),
                wifi_standard: format!("{:?}", ap.wifi_standard),
                is_connected,
            }
        })
        .collect())
}

/// Analyze WiFi channels.
#[tauri::command]
pub async fn analyze_wifi_channels(
    state: State<'_, AppState>,
    interface: Option<String>,
) -> Result<Vec<ChannelAnalysis>, String> {
    // Get the interface name
    let iface_name = match interface {
        Some(name) => name,
        None => {
            let interfaces = state
                .providers
                .wifi
                .list_wifi_interfaces()
                .await
                .map_err(|e| e.to_string())?;
            if interfaces.is_empty() {
                return Err("No WiFi interfaces found".to_string());
            }
            interfaces[0].name.clone()
        }
    };

    // Get channel utilization data
    let channel_data = state
        .providers
        .wifi
        .analyze_channels(&iface_name)
        .await
        .map_err(|e| e.to_string())?;

    // Get current connection to determine current channel
    let current_channel = state
        .providers
        .wifi
        .get_current_connection(&iface_name)
        .await
        .ok()
        .flatten()
        .map(|conn| conn.access_point.channel.number);

    Ok(channel_data
        .into_iter()
        .map(|ch| ChannelAnalysis {
            channel: ch.channel.number,
            band: format!("{:?}", ch.channel.band),
            network_count: ch.network_count as usize,
            interference_level: format!("{:?}", ch.interference_level),
            is_dfs: ch.channel.is_dfs(),
            is_recommended: ch.recommended,
            is_current: current_channel == Some(ch.channel.number),
        })
        .collect())
}

/// Check WiFi interference.
#[tauri::command]
pub async fn check_wifi_interference(
    state: State<'_, AppState>,
    interface: Option<String>,
) -> Result<InterferenceReport, String> {
    // Get the interface name
    let iface_name = match interface {
        Some(name) => name,
        None => {
            let interfaces = state
                .providers
                .wifi
                .list_wifi_interfaces()
                .await
                .map_err(|e| e.to_string())?;
            if interfaces.is_empty() {
                return Err("No WiFi interfaces found".to_string());
            }
            interfaces[0].name.clone()
        }
    };

    // Get current connection info
    let connection = state
        .providers
        .wifi
        .get_current_connection(&iface_name)
        .await
        .map_err(|e| e.to_string())?;

    let (current_channel, current_ssid, rssi, noise) = match connection {
        Some(conn) => (
            Some(conn.access_point.channel.number),
            Some(conn.access_point.ssid.as_str().to_string()),
            Some(conn.access_point.rssi),
            conn.access_point.noise,
        ),
        None => (None, None, None, None),
    };

    // Calculate SNR rating
    let snr_rating = match (rssi, noise) {
        (Some(r), Some(n)) => {
            let snr = r - n;
            if snr >= 40 {
                "Excellent"
            } else if snr >= 25 {
                "Good"
            } else if snr >= 15 {
                "Fair"
            } else {
                "Poor"
            }
        }
        (Some(r), None) => {
            // Estimate based on RSSI alone
            if r >= -50 {
                "Excellent"
            } else if r >= -60 {
                "Good"
            } else if r >= -70 {
                "Fair"
            } else {
                "Poor"
            }
        }
        _ => "Unknown",
    }
    .to_string();

    // Scan for overlapping networks
    let access_points = state
        .providers
        .wifi
        .scan_access_points(&iface_name)
        .await
        .unwrap_or_default();

    let overlapping_networks: Vec<String> = if let Some(current_ch) = current_channel {
        access_points
            .iter()
            .filter(|ap| {
                let channel_diff = (ap.channel.number as i16 - current_ch as i16).abs();
                // For 2.4GHz, channels overlap within 4 channels
                // For 5GHz, channels don't overlap
                channel_diff > 0 && channel_diff <= 4 && ap.channel.number < 36
            })
            .filter(|ap| Some(ap.ssid.as_str().to_string()) != current_ssid)
            .map(|ap| format!("{} (ch {})", ap.ssid.as_str(), ap.channel.number))
            .collect()
    } else {
        Vec::new()
    };

    // Generate recommendations
    let mut recommendations = Vec::new();

    if snr_rating == "Poor" || snr_rating == "Fair" {
        recommendations.push(
            "Consider moving closer to your router or reducing interference sources".to_string(),
        );
    }

    if overlapping_networks.len() > 3 {
        recommendations.push(
            "High channel congestion detected. Consider switching to a 5GHz network if available"
                .to_string(),
        );
    }

    if current_channel.map(|c| c < 36).unwrap_or(false) && access_points.len() > 10 {
        recommendations.push(
            "Many networks on 2.4GHz band. A 5GHz network would provide better performance"
                .to_string(),
        );
    }

    if recommendations.is_empty() {
        recommendations.push("Your WiFi connection appears to be in good condition".to_string());
    }

    Ok(InterferenceReport {
        current_channel,
        snr_rating,
        channel_utilization: None, // Would need additional platform-specific APIs
        overlapping_networks,
        recommendations,
    })
}

/// Run comprehensive network diagnostics.
#[tauri::command]
pub async fn run_diagnostics(
    state: State<'_, AppState>,
    quick: Option<bool>,
    include_speed: Option<bool>,
    include_wifi: Option<bool>,
) -> Result<DiagnosticsResult, String> {
    let quick_mode = quick.unwrap_or(false);
    let test_speed = include_speed.unwrap_or(!quick_mode);
    let test_wifi = include_wifi.unwrap_or(true);

    let mut tests = Vec::new();
    let mut issues = Vec::new();
    let mut recommendations = Vec::new();

    // Test 1: Network Interface Check
    let start = Instant::now();
    let interfaces_result = state.providers.network.list_interfaces().await;
    let duration = start.elapsed().as_millis() as u64;

    match interfaces_result {
        Ok(interfaces) => {
            let active_count = interfaces.iter().filter(|i| i.is_up()).count();
            let has_ipv4 = interfaces.iter().any(|i| !i.ipv4_addresses.is_empty());

            let passed = active_count > 0 && has_ipv4;
            tests.push(DiagnosticTest {
                name: "Network Interfaces".to_string(),
                category: "Network".to_string(),
                passed,
                message: if passed {
                    format!(
                        "{} active interface(s) with IPv4 connectivity",
                        active_count
                    )
                } else {
                    "No active network interfaces found".to_string()
                },
                details: Some(serde_json::json!({
                    "total_interfaces": interfaces.len(),
                    "active_interfaces": active_count,
                    "has_ipv4": has_ipv4
                })),
                duration_ms: duration,
            });

            if !passed {
                issues.push("No active network interfaces".to_string());
                recommendations.push(
                    "Check that your network cable is connected or WiFi is enabled".to_string(),
                );
            }
        }
        Err(e) => {
            tests.push(DiagnosticTest {
                name: "Network Interfaces".to_string(),
                category: "Network".to_string(),
                passed: false,
                message: format!("Failed to enumerate interfaces: {}", e),
                details: None,
                duration_ms: duration,
            });
            issues.push("Could not check network interfaces".to_string());
        }
    }

    // Test 2: Default Gateway Check
    let start = Instant::now();
    let gateway_result = state.providers.network.get_default_gateway().await;
    let duration = start.elapsed().as_millis() as u64;

    match gateway_result {
        Ok(Some(gateway)) => {
            tests.push(DiagnosticTest {
                name: "Default Gateway".to_string(),
                category: "Network".to_string(),
                passed: true,
                message: format!("Gateway: {} via {}", gateway.address, gateway.interface),
                details: Some(serde_json::json!({
                    "address": gateway.address.to_string(),
                    "interface": gateway.interface
                })),
                duration_ms: duration,
            });
        }
        Ok(None) => {
            tests.push(DiagnosticTest {
                name: "Default Gateway".to_string(),
                category: "Network".to_string(),
                passed: false,
                message: "No default gateway configured".to_string(),
                details: None,
                duration_ms: duration,
            });
            issues.push("No default gateway".to_string());
            recommendations.push("Check your router connection and DHCP settings".to_string());
        }
        Err(e) => {
            tests.push(DiagnosticTest {
                name: "Default Gateway".to_string(),
                category: "Network".to_string(),
                passed: false,
                message: format!("Failed to get gateway: {}", e),
                details: None,
                duration_ms: duration,
            });
        }
    }

    // Test 3: DNS Configuration
    let start = Instant::now();
    let dns_result = state.providers.network.get_dns_servers().await;
    let duration = start.elapsed().as_millis() as u64;

    match dns_result {
        Ok(servers) => {
            let passed = !servers.is_empty();
            tests.push(DiagnosticTest {
                name: "DNS Configuration".to_string(),
                category: "DNS".to_string(),
                passed,
                message: if passed {
                    format!("{} DNS server(s) configured", servers.len())
                } else {
                    "No DNS servers configured".to_string()
                },
                details: Some(serde_json::json!({
                    "servers": servers.iter().map(|s| s.address.to_string()).collect::<Vec<_>>()
                })),
                duration_ms: duration,
            });

            if !passed {
                issues.push("No DNS servers configured".to_string());
                recommendations.push("Configure DNS servers (e.g., 8.8.8.8, 1.1.1.1)".to_string());
            }
        }
        Err(e) => {
            tests.push(DiagnosticTest {
                name: "DNS Configuration".to_string(),
                category: "DNS".to_string(),
                passed: false,
                message: format!("Failed to get DNS servers: {}", e),
                details: None,
                duration_ms: duration,
            });
        }
    }

    // Test 4: DNS Resolution
    let start = Instant::now();
    let resolver = DnsResolver::new().ok();
    let duration_init = start.elapsed().as_millis() as u64;

    if let Some(resolver) = resolver {
        let start = Instant::now();
        let resolve_result = resolver.resolve("google.com").await;
        let duration = start.elapsed().as_millis() as u64 + duration_init;

        match resolve_result {
            Ok(result) if !result.addresses.is_empty() => {
                tests.push(DiagnosticTest {
                    name: "DNS Resolution".to_string(),
                    category: "DNS".to_string(),
                    passed: true,
                    message: format!("Resolved google.com in {:.1}ms", result.duration.as_secs_f64() * 1000.0),
                    details: Some(serde_json::json!({
                        "hostname": "google.com",
                        "addresses": result.addresses.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
                        "resolution_time_ms": result.duration.as_secs_f64() * 1000.0
                    })),
                    duration_ms: duration,
                });
            }
            _ => {
                tests.push(DiagnosticTest {
                    name: "DNS Resolution".to_string(),
                    category: "DNS".to_string(),
                    passed: false,
                    message: "Failed to resolve google.com".to_string(),
                    details: None,
                    duration_ms: duration,
                });
                issues.push("DNS resolution failing".to_string());
                recommendations.push(
                    "Check DNS server configuration or try alternative DNS (8.8.8.8)".to_string(),
                );
            }
        }
    }

    // Test 5: Internet Connectivity (Ping)
    let start = Instant::now();
    let pinger = Pinger::new();
    let ping_config = PingConfig {
        count: if quick_mode { 2 } else { 4 },
        timeout: Duration::from_millis(2000),
        interval: Duration::from_millis(500),
        size: 64,
    };

    // Ping 8.8.8.8 (Google DNS)
    let ping_result = pinger.ping("8.8.8.8".parse().unwrap(), &ping_config).await;
    let duration = start.elapsed().as_millis() as u64;

    match ping_result {
        Ok(stats) if stats.received > 0 => {
            let avg_latency = stats
                .avg_rtt
                .map(|d| d.as_secs_f64() * 1000.0)
                .unwrap_or(0.0);
            tests.push(DiagnosticTest {
                name: "Internet Connectivity".to_string(),
                category: "Connectivity".to_string(),
                passed: true,
                message: format!(
                    "Ping to 8.8.8.8: {:.1}ms avg, {:.0}% loss",
                    avg_latency, stats.loss_percent
                ),
                details: Some(serde_json::json!({
                    "target": "8.8.8.8",
                    "sent": stats.transmitted,
                    "received": stats.received,
                    "loss_percent": stats.loss_percent,
                    "avg_latency_ms": avg_latency
                })),
                duration_ms: duration,
            });

            if stats.loss_percent > 10.0 {
                issues.push(format!("High packet loss ({:.0}%)", stats.loss_percent));
                recommendations.push("Check for network congestion or interference".to_string());
            }
        }
        _ => {
            tests.push(DiagnosticTest {
                name: "Internet Connectivity".to_string(),
                category: "Connectivity".to_string(),
                passed: false,
                message: "Unable to reach 8.8.8.8".to_string(),
                details: None,
                duration_ms: duration,
            });
            issues.push("No internet connectivity".to_string());
            recommendations.push("Check your internet connection and router".to_string());
        }
    }

    // Test 6: WiFi Status (if enabled)
    if test_wifi {
        let start = Instant::now();
        let wifi_result = state.providers.wifi.list_wifi_interfaces().await;
        let duration = start.elapsed().as_millis() as u64;

        match wifi_result {
            Ok(interfaces) if !interfaces.is_empty() => {
                let iface = &interfaces[0];
                if iface.powered_on {
                    let conn_result = state
                        .providers
                        .wifi
                        .get_current_connection(&iface.name)
                        .await;
                    match conn_result {
                        Ok(Some(conn)) => {
                            let rssi = conn.access_point.rssi;
                            let passed = rssi >= -75;
                            let quality = if rssi >= -50 {
                                "Excellent"
                            } else if rssi >= -60 {
                                "Good"
                            } else if rssi >= -70 {
                                "Fair"
                            } else {
                                "Weak"
                            };

                            tests.push(DiagnosticTest {
                                name: "WiFi Signal".to_string(),
                                category: "WiFi".to_string(),
                                passed,
                                message: format!(
                                    "Connected to {} ({} dBm - {})",
                                    conn.access_point.ssid.as_str(),
                                    rssi,
                                    quality
                                ),
                                details: Some(serde_json::json!({
                                    "ssid": conn.access_point.ssid.as_str(),
                                    "rssi": rssi,
                                    "channel": conn.access_point.channel.number,
                                    "quality": quality
                                })),
                                duration_ms: duration,
                            });

                            if !passed {
                                issues.push("Weak WiFi signal".to_string());
                                recommendations.push(
                                    "Move closer to your router or reduce interference".to_string(),
                                );
                            }
                        }
                        Ok(None) => {
                            tests.push(DiagnosticTest {
                                name: "WiFi Signal".to_string(),
                                category: "WiFi".to_string(),
                                passed: false,
                                message: "WiFi enabled but not connected".to_string(),
                                details: None,
                                duration_ms: duration,
                            });
                            issues.push("WiFi not connected".to_string());
                            recommendations.push("Connect to a WiFi network".to_string());
                        }
                        Err(_) => {}
                    }
                } else {
                    tests.push(DiagnosticTest {
                        name: "WiFi Signal".to_string(),
                        category: "WiFi".to_string(),
                        passed: false,
                        message: "WiFi is disabled".to_string(),
                        details: None,
                        duration_ms: duration,
                    });
                }
            }
            _ => {}
        }
    }

    // Test 7: Speed Test (if enabled and not quick mode)
    if test_speed && !quick_mode {
        let start = Instant::now();
        let tester = SpeedTester::new();
        let config = SpeedTestConfig {
            duration: Duration::from_secs(5),
            connections: 2,
            test_download: true,
            test_upload: false,
            ..Default::default()
        };

        let speed_result = tester.run_test(&config).await;
        let duration = start.elapsed().as_millis() as u64;

        match speed_result {
            Ok(result) => {
                let download_mbps = result.download_mbps().unwrap_or(0.0);
                let passed = download_mbps >= 10.0; // Minimum 10 Mbps

                tests.push(DiagnosticTest {
                    name: "Download Speed".to_string(),
                    category: "Speed".to_string(),
                    passed,
                    message: format!("{:.1} Mbps download", download_mbps),
                    details: Some(serde_json::json!({
                        "download_mbps": download_mbps,
                        "server": result.server.name
                    })),
                    duration_ms: duration,
                });

                if !passed {
                    issues.push("Slow download speed".to_string());
                    recommendations.push(
                        "Check for bandwidth-heavy applications or consider upgrading your plan"
                            .to_string(),
                    );
                }
            }
            Err(e) => {
                tests.push(DiagnosticTest {
                    name: "Download Speed".to_string(),
                    category: "Speed".to_string(),
                    passed: false,
                    message: format!("Speed test failed: {}", e),
                    details: None,
                    duration_ms: duration,
                });
            }
        }
    }

    // Calculate summary
    let total = tests.len();
    let passed = tests.iter().filter(|t| t.passed).count();
    let failed = total - passed;

    let overall_status = if failed == 0 {
        "healthy"
    } else if failed <= 2 {
        "warning"
    } else {
        "critical"
    }
    .to_string();

    let summary = DiagnosticsSummary {
        total_tests: total,
        passed,
        failed,
        warnings: 0,
        overall_status,
    };

    Ok(DiagnosticsResult {
        tests,
        summary,
        issues,
        recommendations,
    })
}

/// Generate a network diagnostic report.
#[tauri::command]
pub async fn generate_report(
    state: State<'_, AppState>,
    format: String,
    include_raw_data: Option<bool>,
) -> Result<GeneratedReport, String> {
    let include_raw = include_raw_data.unwrap_or(false);

    // Collect system info
    let system_info = state.providers.system.get_system_info().await.ok();

    // Collect interfaces
    let interfaces = state
        .providers
        .network
        .list_interfaces()
        .await
        .unwrap_or_default();

    let default_iface = state
        .providers
        .network
        .get_default_interface()
        .await
        .ok()
        .flatten();

    let default_name = default_iface.as_ref().map(|i| i.name.clone());

    // Build interface summaries
    let interface_summaries: Vec<InterfaceSummary> = interfaces
        .iter()
        .map(|iface| InterfaceSummary {
            name: iface.name.clone(),
            interface_type: format!("{:?}", iface.interface_type),
            ipv4_addresses: iface
                .ipv4_addresses
                .iter()
                .map(|a| a.address.to_string())
                .collect(),
            ipv6_addresses: iface
                .ipv6_addresses
                .iter()
                .map(|a| a.address.to_string())
                .collect(),
            mac_address: iface.mac_address.as_ref().map(|m| m.to_string()),
            is_up: iface.is_up(),
            is_default: Some(&iface.name) == default_name.as_ref(),
        })
        .collect();

    // Perform DNS resolution test
    let mut dns_results = Vec::new();
    if let Ok(resolver) = DnsResolver::new() {
        let start = Instant::now();
        match resolver.resolve("google.com").await {
            Ok(result) => {
                dns_results.push(DnsSummary {
                    query: "google.com".to_string(),
                    addresses: result.addresses.iter().map(|a| a.to_string()).collect(),
                    duration_ms: result.duration.as_secs_f64() * 1000.0,
                    success: !result.addresses.is_empty(),
                    error: None,
                });
            }
            Err(e) => {
                dns_results.push(DnsSummary {
                    query: "google.com".to_string(),
                    addresses: Vec::new(),
                    duration_ms: start.elapsed().as_secs_f64() * 1000.0,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // Perform ping test
    let mut ping_results = Vec::new();
    let pinger = Pinger::new();
    let ping_config = PingConfig {
        count: 4,
        timeout: Duration::from_millis(2000),
        interval: Duration::from_millis(500),
        size: 64,
    };

    if let Ok(stats) = pinger.ping("8.8.8.8".parse().unwrap(), &ping_config).await {
        let quality = if stats.loss_percent == 0.0
            && stats.avg_rtt.map(|d| d.as_millis() < 100).unwrap_or(false)
        {
            "Excellent"
        } else if stats.loss_percent < 5.0 {
            "Good"
        } else if stats.loss_percent < 20.0 {
            "Fair"
        } else {
            "Poor"
        };

        ping_results.push(PingSummary {
            target: "8.8.8.8".to_string(),
            transmitted: stats.transmitted,
            received: stats.received,
            loss_percent: stats.loss_percent,
            min_rtt_ms: stats.min_rtt.map(|d| d.as_secs_f64() * 1000.0),
            avg_rtt_ms: stats.avg_rtt.map(|d| d.as_secs_f64() * 1000.0),
            max_rtt_ms: stats.max_rtt.map(|d| d.as_secs_f64() * 1000.0),
            stddev_ms: stats.stddev_rtt.map(|d| d.as_secs_f64() * 1000.0),
            quality: quality.to_string(),
        });
    }

    // Build the report
    let mut builder = ReportBuilder::new().title("Network Diagnostics Report");

    if let Some(info) = &system_info {
        builder = builder
            .hostname(&info.hostname)
            .os_info(format!("{:?} {}", info.os_type, info.os_version));
    }

    for iface in interface_summaries {
        builder = builder.add_interface(iface);
    }

    for dns in dns_results {
        builder = builder.add_dns_result(dns);
    }

    for ping in ping_results {
        builder = builder.add_ping_result(ping);
    }

    if include_raw {
        builder = builder.raw_data(serde_json::json!({
            "system_info": system_info,
            "interface_count": interfaces.len(),
        }));
    }

    let report = builder.build();

    // Get health assessment
    let (health_score, health_status) = report
        .health
        .as_ref()
        .map(|h| (h.score, h.status.clone()))
        .unwrap_or((0, "unknown".to_string()));

    // Format the report based on requested format
    let (content, mime_type, file_extension, is_binary) = match format.to_lowercase().as_str() {
        "json" => {
            let formatter = JsonFormatter::new();
            let content = formatter.format(&report).map_err(|e| e.to_string())?;
            (
                content,
                "application/json".to_string(),
                "json".to_string(),
                false,
            )
        }
        "text" | "txt" => {
            let formatter = TextFormatter::new();
            let content = formatter.format(&report).map_err(|e| e.to_string())?;
            (content, "text/plain".to_string(), "txt".to_string(), false)
        }
        "markdown" | "md" => {
            let formatter = MarkdownFormatter::new();
            let content = formatter.format(&report).map_err(|e| e.to_string())?;
            (
                content,
                "text/markdown".to_string(),
                "md".to_string(),
                false,
            )
        }
        "html" => {
            let formatter = HtmlFormatter::new();
            let content = formatter.format(&report).map_err(|e| e.to_string())?;
            (content, "text/html".to_string(), "html".to_string(), false)
        }
        _ => {
            return Err(format!(
                "Unsupported format: {}. Supported formats: json, text, markdown, html",
                format
            ));
        }
    };

    Ok(GeneratedReport {
        content,
        mime_type,
        file_extension,
        is_binary,
        health_score,
        health_status,
    })
}

// ============================================================================
// Auto-Fix Commands (Desktop Only)
// ============================================================================

/// Get available fix actions (desktop only).
#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[tauri::command]
pub async fn get_available_fixes() -> Result<Vec<FixActionInfo>, String> {
    use netdiag_autofix::FixAction;

    // Return a list of commonly available fixes
    let fixes = vec![
        FixAction::flush_dns_cache(),
        FixAction::reset_tcp_ip(),
        FixAction::restart_network_service(),
    ];

    Ok(fixes
        .into_iter()
        .map(|action| FixActionInfo {
            id: action.id.to_string(),
            name: action.name,
            description: action.description,
            severity: format!("{:?}", action.severity).to_lowercase(),
            category: format!("{:?}", action.category).to_lowercase(),
            reversible: action.reversible,
            estimated_time_secs: action.estimated_time_secs,
            prerequisites: action
                .prerequisites
                .iter()
                .map(|p| format!("{:?}", p))
                .collect(),
        })
        .collect())
}

/// Apply a specific fix (desktop only).
/// Note: This uses direct command execution rather than the autofix engine
/// since PlatformProviders doesn't implement Clone.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[tauri::command]
pub async fn apply_fix(
    fix_type: String,
    interface: Option<String>,
    dry_run: Option<bool>,
) -> Result<FixResultInfo, String> {
    use netdiag_autofix::FixAction;
    use std::process::Command;

    let is_dry_run = dry_run.unwrap_or(false);
    let start = Instant::now();

    // Create the appropriate fix action based on type
    let action = match fix_type.as_str() {
        "flush_dns_cache" => FixAction::flush_dns_cache(),
        "reset_tcp_ip" => FixAction::reset_tcp_ip(),
        "restart_network_service" => FixAction::restart_network_service(),
        "reset_adapter" => {
            let iface = interface
                .clone()
                .ok_or("Interface required for reset_adapter")?;
            FixAction::reset_adapter(iface)
        }
        "renew_dhcp" => {
            let iface = interface
                .clone()
                .ok_or("Interface required for renew_dhcp")?;
            FixAction::renew_dhcp(iface)
        }
        "reconnect_wifi" => {
            let iface = interface
                .clone()
                .ok_or("Interface required for reconnect_wifi")?;
            FixAction::reconnect_wifi(iface)
        }
        _ => return Err(format!("Unknown fix type: {}", fix_type)),
    };

    let action_id = action.id.to_string();

    if is_dry_run {
        return Ok(FixResultInfo {
            action_id,
            success: true,
            message: Some("Dry run - not applied".to_string()),
            error: None,
            duration_ms: start.elapsed().as_millis() as u64,
            rollback_id: None,
        });
    }

    // Execute the fix directly based on type
    let result: Result<String, String> = match fix_type.as_str() {
        "flush_dns_cache" => {
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("dscacheutil").args(["-flushcache"]).output();
                let _ = Command::new("killall")
                    .args(["-HUP", "mDNSResponder"])
                    .output();
                Ok("DNS cache flushed".to_string())
            }
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("systemd-resolve")
                    .args(["--flush-caches"])
                    .output();
                let _ = Command::new("resolvectl").args(["flush-caches"]).output();
                Ok("DNS cache flushed".to_string())
            }
            #[cfg(target_os = "windows")]
            {
                Command::new("ipconfig")
                    .args(["/flushdns"])
                    .output()
                    .map(|_| "DNS cache flushed".to_string())
                    .map_err(|e| e.to_string())
            }
        }
        "reset_tcp_ip" => {
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("netsh").args(["int", "ip", "reset"]).output();
                let _ = Command::new("netsh").args(["winsock", "reset"]).output();
                Ok("TCP/IP stack reset".to_string())
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok("TCP/IP reset not needed on this platform".to_string())
            }
        }
        "restart_network_service" => {
            #[cfg(target_os = "macos")]
            {
                Command::new("launchctl")
                    .args([
                        "kickstart",
                        "-k",
                        "system/com.apple.networking.discoveryengine",
                    ])
                    .output()
                    .map(|_| "Network service restarted".to_string())
                    .map_err(|e| e.to_string())
            }
            #[cfg(target_os = "linux")]
            {
                let nm = Command::new("systemctl")
                    .args(["restart", "NetworkManager"])
                    .output();
                if nm.is_err() {
                    Command::new("systemctl")
                        .args(["restart", "networking"])
                        .output()
                        .map(|_| "Network service restarted".to_string())
                        .map_err(|e| e.to_string())
                } else {
                    Ok("Network service restarted".to_string())
                }
            }
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("net").args(["stop", "netman"]).output();
                Command::new("net")
                    .args(["start", "netman"])
                    .output()
                    .map(|_| "Network service restarted".to_string())
                    .map_err(|e| e.to_string())
            }
        }
        "reset_adapter" => {
            let iface = interface.as_ref().ok_or("Interface required")?;
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("ifconfig")
                    .args([iface.as_str(), "down"])
                    .output();
                tokio::time::sleep(Duration::from_secs(1)).await;
                Command::new("ifconfig")
                    .args([iface.as_str(), "up"])
                    .output()
                    .map(|_| format!("Adapter {} reset", iface))
                    .map_err(|e| e.to_string())
            }
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("ip")
                    .args(["link", "set", iface.as_str(), "down"])
                    .output();
                tokio::time::sleep(Duration::from_secs(1)).await;
                Command::new("ip")
                    .args(["link", "set", iface.as_str(), "up"])
                    .output()
                    .map(|_| format!("Adapter {} reset", iface))
                    .map_err(|e| e.to_string())
            }
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("netsh")
                    .args(["interface", "set", "interface", iface.as_str(), "disable"])
                    .output();
                tokio::time::sleep(Duration::from_secs(2)).await;
                Command::new("netsh")
                    .args(["interface", "set", "interface", iface.as_str(), "enable"])
                    .output()
                    .map(|_| format!("Adapter {} reset", iface))
                    .map_err(|e| e.to_string())
            }
        }
        "renew_dhcp" => {
            let iface = interface.as_ref().ok_or("Interface required")?;
            #[cfg(target_os = "macos")]
            {
                Command::new("ipconfig")
                    .args(["set", iface.as_str(), "DHCP"])
                    .output()
                    .map(|_| format!("DHCP renewed on {}", iface))
                    .map_err(|e| e.to_string())
            }
            #[cfg(target_os = "linux")]
            {
                let _ = Command::new("dhclient")
                    .args(["-r", iface.as_str()])
                    .output();
                Command::new("dhclient")
                    .args([iface.as_str()])
                    .output()
                    .map(|_| format!("DHCP renewed on {}", iface))
                    .map_err(|e| e.to_string())
            }
            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("ipconfig")
                    .args(["/release", iface.as_str()])
                    .output();
                Command::new("ipconfig")
                    .args(["/renew", iface.as_str()])
                    .output()
                    .map(|_| format!("DHCP renewed on {}", iface))
                    .map_err(|e| e.to_string())
            }
        }
        _ => Err(format!("Unknown fix type: {}", fix_type)),
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(message) => Ok(FixResultInfo {
            action_id,
            success: true,
            message: Some(message),
            error: None,
            duration_ms,
            rollback_id: None,
        }),
        Err(error) => Ok(FixResultInfo {
            action_id,
            success: false,
            message: None,
            error: Some(error),
            duration_ms,
            rollback_id: None,
        }),
    }
}

/// List available rollback points (desktop only).
/// Note: Rollback points are currently not persisted between sessions
/// in this simplified implementation.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[tauri::command]
pub async fn list_rollback_points() -> Result<Vec<RollbackPointInfo>, String> {
    // In a full implementation, this would load from persistent storage
    // For now, return empty list as we don't persist rollback state
    Ok(Vec::new())
}

/// Perform a rollback (desktop only).
/// Note: Rollback functionality is limited in this simplified implementation.
#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[tauri::command]
pub async fn perform_rollback(_rollback_id: String) -> Result<bool, String> {
    // In a full implementation, this would restore from persistent state
    Err("Rollback functionality is not yet available in this version".to_string())
}

/// Check if autofix is available on this platform.
#[tauri::command]
pub async fn is_autofix_available() -> bool {
    cfg!(not(any(target_os = "ios", target_os = "android")))
}

// Mobile stubs for autofix commands
#[cfg(any(target_os = "ios", target_os = "android"))]
#[tauri::command]
pub async fn get_available_fixes() -> Result<Vec<FixActionInfo>, String> {
    Err("Auto-fix is not available on mobile platforms".to_string())
}

#[cfg(any(target_os = "ios", target_os = "android"))]
#[tauri::command]
pub async fn apply_fix(
    _fix_type: String,
    _interface: Option<String>,
    _dry_run: Option<bool>,
) -> Result<FixResultInfo, String> {
    Err("Auto-fix is not available on mobile platforms".to_string())
}

#[cfg(any(target_os = "ios", target_os = "android"))]
#[tauri::command]
pub async fn list_rollback_points() -> Result<Vec<RollbackPointInfo>, String> {
    Err("Auto-fix is not available on mobile platforms".to_string())
}

#[cfg(any(target_os = "ios", target_os = "android"))]
#[tauri::command]
pub async fn perform_rollback(_rollback_id: String) -> Result<bool, String> {
    Err("Auto-fix is not available on mobile platforms".to_string())
}

// ============================================================================
// Packet Capture Commands (Desktop Only)
// ============================================================================

/// List available capture devices (desktop only).
#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[tauri::command]
pub async fn list_capture_devices() -> Result<Vec<CaptureDeviceInfo>, String> {
    use netdiag_capture::list_devices;

    let devices = list_devices().map_err(|e| e.to_string())?;

    Ok(devices
        .into_iter()
        .map(|d| CaptureDeviceInfo {
            name: d.name,
            description: d.description,
            addresses: d.addresses,
            is_loopback: d.is_loopback,
            is_up: d.is_up,
        })
        .collect())
}

/// Capture packets (desktop only).
#[cfg(not(any(target_os = "ios", target_os = "android")))]
#[tauri::command]
pub async fn capture_packets(
    device: Option<String>,
    count: Option<usize>,
    filter: Option<String>,
) -> Result<(Vec<CapturedPacketInfo>, CaptureStatsInfo), String> {
    use netdiag_capture::{CaptureConfig, CaptureFilter, PacketCapture};

    let packet_count = count.unwrap_or(50).min(500); // Limit to 500 packets max

    let device_name = match device {
        Some(name) => name,
        None => {
            let default = netdiag_capture::default_device().map_err(|e| e.to_string())?;
            default.name
        }
    };

    let mut config = CaptureConfig::for_device(&device_name).max_packets(packet_count);

    if let Some(f) = filter {
        config = config.with_filter(CaptureFilter::new(&f));
    }

    let capture = PacketCapture::new(config);

    let mut captured_packets = Vec::new();

    let stats = capture
        .capture_sync(|packet| {
            let proto_name = format!("{:?}", packet.protocol);

            let summary = match packet.protocol {
                netdiag_capture::Protocol::Tcp => {
                    format!(
                        "TCP {}:{} -> {}:{} len={}",
                        packet.src_ip.map(|ip| ip.to_string()).unwrap_or_default(),
                        packet.src_port.unwrap_or(0),
                        packet.dst_ip.map(|ip| ip.to_string()).unwrap_or_default(),
                        packet.dst_port.unwrap_or(0),
                        packet.length
                    )
                }
                netdiag_capture::Protocol::Udp => {
                    format!(
                        "UDP {}:{} -> {}:{} len={}",
                        packet.src_ip.map(|ip| ip.to_string()).unwrap_or_default(),
                        packet.src_port.unwrap_or(0),
                        packet.dst_ip.map(|ip| ip.to_string()).unwrap_or_default(),
                        packet.dst_port.unwrap_or(0),
                        packet.length
                    )
                }
                netdiag_capture::Protocol::Icmp | netdiag_capture::Protocol::Icmpv6 => {
                    format!(
                        "ICMP {} -> {}",
                        packet.src_ip.map(|ip| ip.to_string()).unwrap_or_default(),
                        packet.dst_ip.map(|ip| ip.to_string()).unwrap_or_default()
                    )
                }
                _ => format!(
                    "{} {} -> {}",
                    proto_name,
                    packet.src_ip.map(|ip| ip.to_string()).unwrap_or_default(),
                    packet.dst_ip.map(|ip| ip.to_string()).unwrap_or_default()
                ),
            };

            captured_packets.push(CapturedPacketInfo {
                timestamp: packet.timestamp.to_rfc3339(),
                protocol: proto_name,
                src_ip: packet.src_ip.map(|ip| ip.to_string()),
                dst_ip: packet.dst_ip.map(|ip| ip.to_string()),
                src_port: packet.src_port,
                dst_port: packet.dst_port,
                length: packet.length,
                summary,
            });

            captured_packets.len() < packet_count
        })
        .map_err(|e| e.to_string())?;

    let protocol_breakdown: Vec<(String, u64)> = stats
        .protocols
        .iter()
        .map(|(k, v)| (k.clone(), v.packets))
        .collect();

    let stats_info = CaptureStatsInfo {
        total_packets: stats.packets_captured,
        total_bytes: stats.bytes_captured,
        packets_dropped: stats.packets_dropped,
        protocol_breakdown,
    };

    Ok((captured_packets, stats_info))
}

/// Check if packet capture is available on this platform.
#[tauri::command]
pub async fn is_capture_available() -> bool {
    cfg!(not(any(target_os = "ios", target_os = "android")))
}

// Mobile stubs for capture commands
#[cfg(any(target_os = "ios", target_os = "android"))]
#[tauri::command]
pub async fn list_capture_devices() -> Result<Vec<CaptureDeviceInfo>, String> {
    Err("Packet capture is not available on mobile platforms".to_string())
}

#[cfg(any(target_os = "ios", target_os = "android"))]
#[tauri::command]
pub async fn capture_packets(
    _device: Option<String>,
    _count: Option<usize>,
    _filter: Option<String>,
) -> Result<(Vec<CapturedPacketInfo>, CaptureStatsInfo), String> {
    Err("Packet capture is not available on mobile platforms".to_string())
}
