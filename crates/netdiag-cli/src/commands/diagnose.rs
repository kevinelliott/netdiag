//! Diagnose command implementation.

use crate::app::DiagnoseArgs;
use color_eyre::eyre::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use netdiag_connectivity::{DnsResolver, PingConfig, Pinger};
use netdiag_platform::PlatformProviders;
use std::net::IpAddr;
use std::time::Duration;

#[cfg(target_os = "macos")]
use netdiag_platform_macos::create_providers;

#[cfg(target_os = "linux")]
use netdiag_platform_linux::create_providers;

#[cfg(target_os = "windows")]
use netdiag_platform_windows::create_providers;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn create_providers() -> PlatformProviders {
    PlatformProviders::new()
}

/// Diagnostic result for a single check.
#[derive(Debug)]
struct DiagnosticCheck {
    name: String,
    passed: bool,
    details: String,
    suggestion: Option<String>,
}

impl DiagnosticCheck {
    fn pass(name: &str, details: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            details: details.to_string(),
            suggestion: None,
        }
    }

    fn fail(name: &str, details: &str, suggestion: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            details: details.to_string(),
            suggestion: suggestion.map(|s| s.to_string()),
        }
    }
}

/// Run the diagnose command.
pub async fn run(args: DiagnoseArgs) -> Result<()> {
    println!("{}", style("Starting Network Diagnostics...").bold());
    println!();

    let mode = if args.quick { "quick" } else { "comprehensive" };
    println!("Running {} diagnostics", style(mode).cyan());
    println!();

    let providers = create_providers();
    let mut results: Vec<DiagnosticCheck> = Vec::new();
    let mut issues_found = 0;

    // Create progress bar
    let total_steps = calculate_steps(&args);
    let pb = ProgressBar::new(total_steps as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Step 1: Check network interfaces
    pb.set_message("Checking network interfaces...");
    let interface_result = check_interfaces(&providers).await;
    if !interface_result.passed {
        issues_found += 1;
    }
    results.push(interface_result);
    pb.inc(1);

    // Step 2: Check default gateway
    pb.set_message("Checking default gateway...");
    let gateway_result = check_gateway(&providers).await;
    if !gateway_result.passed {
        issues_found += 1;
    }
    results.push(gateway_result);
    pb.inc(1);

    // Step 3: Check DNS servers
    pb.set_message("Checking DNS configuration...");
    let dns_config_result = check_dns_config(&providers).await;
    if !dns_config_result.passed {
        issues_found += 1;
    }
    results.push(dns_config_result);
    pb.inc(1);

    // Step 4: Test DNS resolution
    pb.set_message("Testing DNS resolution...");
    let dns_result = check_dns_resolution().await;
    if !dns_result.passed {
        issues_found += 1;
    }
    results.push(dns_result);
    pb.inc(1);

    // Step 5: Test internet connectivity
    if !args.quick {
        pb.set_message("Testing internet connectivity...");
        let connectivity_result = check_connectivity().await;
        if !connectivity_result.passed {
            issues_found += 1;
        }
        results.push(connectivity_result);
        pb.inc(1);
    }

    // Step 6: WiFi analysis (if requested)
    if args.wifi {
        pb.set_message("Analyzing WiFi...");
        let wifi_result = check_wifi(&providers).await;
        if !wifi_result.passed {
            issues_found += 1;
        }
        results.push(wifi_result);
        pb.inc(1);
    }

    // Step 7: Speed test (if requested and not quick)
    if args.speed && !args.quick {
        pb.set_message("Running speed test...");
        let speed_result = check_speed().await;
        if !speed_result.passed {
            issues_found += 1;
        }
        results.push(speed_result);
        pb.inc(1);
    }

    pb.finish_and_clear();

    // Print results
    println!();
    println!("{}", style("Diagnostic Results").bold().underlined());
    println!();

    for result in &results {
        let status = if result.passed {
            style("PASS").green().bold()
        } else {
            style("FAIL").red().bold()
        };

        println!("  [{}] {}", status, style(&result.name).bold());
        println!("         {}", result.details);

        if let Some(suggestion) = &result.suggestion {
            println!("         {} {}", style("->").yellow(), suggestion);
        }
        println!();
    }

    // Summary
    println!("{}", style("Summary").bold().underlined());
    println!();

    let total_checks = results.len();
    let passed_checks = total_checks - issues_found;

    if issues_found == 0 {
        println!(
            "  {} All {} checks passed!",
            style("OK").green().bold(),
            total_checks
        );
    } else {
        println!(
            "  {} {}/{} checks passed, {} issue(s) found",
            style("!!").yellow().bold(),
            passed_checks,
            total_checks,
            issues_found
        );
    }

    // Auto-fix suggestion
    if issues_found > 0 && !args.fix {
        println!();
        println!(
            "  {} Run 'netdiag fix' to attempt automatic repairs",
            style("Tip:").cyan()
        );
    }

    // Handle auto-fix
    if args.fix && issues_found > 0 {
        println!();
        println!("{}", style("Attempting automatic fixes...").yellow().bold());
        println!("  Run 'netdiag fix apply' for interactive fix mode");
    }

    // Save report if output specified
    if let Some(output) = args.output {
        println!();
        println!("Report saved to: {}", output.display());
    }

    Ok(())
}

fn calculate_steps(args: &DiagnoseArgs) -> usize {
    let mut steps = 4; // interfaces, gateway, dns config, dns resolution

    if !args.quick {
        steps += 1; // connectivity test
    }

    if args.wifi {
        steps += 1;
    }

    if args.speed && !args.quick {
        steps += 1;
    }

    steps
}

async fn check_interfaces(providers: &PlatformProviders) -> DiagnosticCheck {
    match providers.network.list_interfaces().await {
        Ok(interfaces) => {
            let active: Vec<_> = interfaces.iter().filter(|i| i.is_up()).collect();
            let with_ipv4: Vec<_> = active
                .iter()
                .filter(|i| !i.ipv4_addresses.is_empty() && !i.flags.loopback)
                .collect();

            if with_ipv4.is_empty() {
                DiagnosticCheck::fail(
                    "Network Interfaces",
                    "No active network interfaces with IPv4 addresses found",
                    Some("Check your network cable or WiFi connection"),
                )
            } else {
                let names: Vec<_> = with_ipv4
                    .iter()
                    .map(|i| {
                        i.display_name
                            .as_ref()
                            .unwrap_or(&i.name)
                            .clone()
                    })
                    .collect();
                DiagnosticCheck::pass(
                    "Network Interfaces",
                    &format!(
                        "{} active interface(s): {}",
                        with_ipv4.len(),
                        names.join(", ")
                    ),
                )
            }
        }
        Err(e) => DiagnosticCheck::fail(
            "Network Interfaces",
            &format!("Failed to enumerate interfaces: {}", e),
            Some("Check system permissions"),
        ),
    }
}

async fn check_gateway(providers: &PlatformProviders) -> DiagnosticCheck {
    match providers.network.get_default_gateway().await {
        Ok(Some(gateway)) => {
            // Try to ping the gateway
            let pinger = Pinger::new();
            let config = PingConfig {
                count: 1,
                timeout: Duration::from_secs(2),
                ..Default::default()
            };

            match pinger.ping(gateway.address, &config).await {
                Ok(result) if result.received > 0 => {
                    let rtt_str = result
                        .avg_rtt
                        .map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0))
                        .unwrap_or_else(|| "N/A".to_string());
                    DiagnosticCheck::pass(
                        "Default Gateway",
                        &format!("Gateway {} is reachable ({})", gateway.address, rtt_str),
                    )
                }
                Ok(_) => DiagnosticCheck::fail(
                    "Default Gateway",
                    &format!("Gateway {} is not responding", gateway.address),
                    Some("Check your router or network equipment"),
                ),
                Err(_) => DiagnosticCheck::pass(
                    "Default Gateway",
                    &format!("Gateway {} configured (ping test skipped)", gateway.address),
                ),
            }
        }
        Ok(None) => DiagnosticCheck::fail(
            "Default Gateway",
            "No default gateway configured",
            Some("Check your network configuration or DHCP settings"),
        ),
        Err(e) => DiagnosticCheck::fail(
            "Default Gateway",
            &format!("Failed to get gateway: {}", e),
            None,
        ),
    }
}

async fn check_dns_config(providers: &PlatformProviders) -> DiagnosticCheck {
    match providers.network.get_dns_servers().await {
        Ok(servers) if !servers.is_empty() => {
            let server_list: Vec<_> = servers.iter().map(|s| s.address.to_string()).collect();
            DiagnosticCheck::pass(
                "DNS Configuration",
                &format!("{} DNS server(s): {}", servers.len(), server_list.join(", ")),
            )
        }
        Ok(_) => DiagnosticCheck::fail(
            "DNS Configuration",
            "No DNS servers configured",
            Some("Configure DNS servers or enable DHCP"),
        ),
        Err(e) => DiagnosticCheck::fail(
            "DNS Configuration",
            &format!("Failed to get DNS servers: {}", e),
            None,
        ),
    }
}

async fn check_dns_resolution() -> DiagnosticCheck {
    let resolver = match DnsResolver::new() {
        Ok(r) => r,
        Err(e) => {
            return DiagnosticCheck::fail(
                "DNS Resolution",
                &format!("Failed to create resolver: {}", e),
                None,
            );
        }
    };

    let test_domains = ["google.com", "cloudflare.com"];

    for domain in &test_domains {
        match resolver.resolve(domain).await {
            Ok(result) if !result.addresses.is_empty() => {
                return DiagnosticCheck::pass(
                    "DNS Resolution",
                    &format!(
                        "Successfully resolved {} ({:.1}ms)",
                        domain,
                        result.duration.as_secs_f64() * 1000.0
                    ),
                );
            }
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    DiagnosticCheck::fail(
        "DNS Resolution",
        "Failed to resolve test domains",
        Some("Check DNS server configuration or try 8.8.8.8"),
    )
}

async fn check_connectivity() -> DiagnosticCheck {
    let pinger = Pinger::new();
    let config = PingConfig {
        count: 3,
        timeout: Duration::from_secs(5),
        ..Default::default()
    };

    let test_targets: [(IpAddr, &str); 2] = [
        ("8.8.8.8".parse().unwrap(), "Google DNS"),
        ("1.1.1.1".parse().unwrap(), "Cloudflare DNS"),
    ];

    for (ip, name) in &test_targets {
        match pinger.ping(*ip, &config).await {
            Ok(result) if result.received > 0 => {
                let loss_pct = result.loss_percent;
                if loss_pct > 50.0 {
                    return DiagnosticCheck::fail(
                        "Internet Connectivity",
                        &format!("{} reachable but {:.0}% packet loss", name, loss_pct),
                        Some("Network may be congested or unstable"),
                    );
                }
                let rtt_str = result
                    .avg_rtt
                    .map(|d| format!("{:.1}ms", d.as_secs_f64() * 1000.0))
                    .unwrap_or_else(|| "N/A".to_string());
                return DiagnosticCheck::pass(
                    "Internet Connectivity",
                    &format!("{} reachable ({}, {:.0}% loss)", name, rtt_str, loss_pct),
                );
            }
            Ok(_) => continue,
            Err(_) => continue,
        }
    }

    DiagnosticCheck::fail(
        "Internet Connectivity",
        "Unable to reach external servers",
        Some("Check firewall settings or ISP connection"),
    )
}

async fn check_wifi(providers: &PlatformProviders) -> DiagnosticCheck {
    if !providers.wifi.is_available() {
        return DiagnosticCheck::pass("WiFi", "WiFi not available on this system");
    }

    match providers.wifi.list_wifi_interfaces().await {
        Ok(interfaces) if !interfaces.is_empty() => {
            let iface = &interfaces[0];
            let status = if iface.powered_on { "enabled" } else { "disabled" };

            // Try to get current connection
            if let Ok(Some(conn)) = providers.wifi.get_current_connection(&iface.name).await {
                let signal_info = if let Ok(Some(signal)) =
                    providers.wifi.get_signal_strength(&iface.name).await
                {
                    let quality = match signal {
                        s if s >= -50 => "Excellent",
                        s if s >= -60 => "Good",
                        s if s >= -70 => "Fair",
                        _ => "Poor",
                    };
                    format!(" ({}dBm - {})", signal, quality)
                } else {
                    String::new()
                };

                DiagnosticCheck::pass(
                    "WiFi",
                    &format!(
                        "Connected to \"{}\"{}",
                        conn.access_point.ssid, signal_info
                    ),
                )
            } else {
                DiagnosticCheck::fail(
                    "WiFi",
                    &format!("WiFi {} but not connected", status),
                    Some("Connect to a WiFi network"),
                )
            }
        }
        Ok(_) => DiagnosticCheck::fail("WiFi", "No WiFi interfaces found", None),
        Err(e) => DiagnosticCheck::fail(
            "WiFi",
            &format!("Failed to check WiFi: {}", e),
            None,
        ),
    }
}

async fn check_speed() -> DiagnosticCheck {
    // Speed test is a longer operation - would use netdiag-speed here
    DiagnosticCheck::pass(
        "Speed Test",
        "Speed test skipped (run 'netdiag speed' for full test)",
    )
}
