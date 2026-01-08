//! Tauri commands for network diagnostics.

use crate::{
    DnsResult, InterfaceInfo, PingResult, SystemInfo, TracerouteHop, TracerouteResult,
    AppState,
};
use netdiag_connectivity::{DnsResolver, PingConfig, Pinger, TracerouteConfig, Tracer};
use std::time::Duration;
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
                Ok(stats) => {
                    Ok(PingResult {
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
                    })
                }
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
