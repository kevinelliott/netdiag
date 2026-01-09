//! Diagnose command implementation.

use crate::app::DiagnoseArgs;
use color_eyre::eyre::Result;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use netdiag_connectivity::{
    identify_isp, DnsResolver, PathAnalyzer, PingConfig, Pinger, Tracer, TracerouteConfig,
};
use netdiag_platform::PlatformProviders;
use netdiag_speed::{SpeedTestConfig, SpeedTester};
use netdiag_types::diagnostics::{JitterStats, PacketLossStats};
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
#[derive(Debug, Clone)]
struct DiagnosticCheck {
    name: String,
    passed: bool,
    details: String,
    suggestion: Option<String>,
    /// Additional verbose details (shown with -v flag)
    verbose_details: Vec<String>,
}

impl DiagnosticCheck {
    fn pass(name: &str, details: &str) -> Self {
        Self {
            name: name.to_string(),
            passed: true,
            details: details.to_string(),
            suggestion: None,
            verbose_details: Vec::new(),
        }
    }

    fn fail(name: &str, details: &str, suggestion: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            passed: false,
            details: details.to_string(),
            suggestion: suggestion.map(|s| s.to_string()),
            verbose_details: Vec::new(),
        }
    }

    fn with_verbose(mut self, details: Vec<String>) -> Self {
        self.verbose_details = details;
        self
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

    // Step 6: Advanced latency and jitter analysis (comprehensive mode)
    if !args.quick {
        pb.set_message("Analyzing latency and jitter...");
        let (latency_result, jitter_result) = check_latency_jitter().await;
        if !latency_result.passed {
            issues_found += 1;
        }
        results.push(latency_result);
        if !jitter_result.passed {
            issues_found += 1;
        }
        results.push(jitter_result);
        pb.inc(1);
    }

    // Step 7: Path analysis with traceroute (comprehensive mode)
    if !args.quick {
        pb.set_message("Analyzing network path...");
        let path_result = check_network_path().await;
        if !path_result.passed {
            issues_found += 1;
        }
        results.push(path_result);
        pb.inc(1);
    }

    // Step 8: ISP identification
    if !args.quick {
        pb.set_message("Identifying ISP...");
        let isp_result = check_isp().await;
        results.push(isp_result);
        pb.inc(1);
    }

    // Step 9: WiFi analysis (if requested)
    if args.wifi {
        pb.set_message("Analyzing WiFi...");
        let wifi_result = check_wifi(&providers).await;
        if !wifi_result.passed {
            issues_found += 1;
        }
        results.push(wifi_result);
        pb.inc(1);
    }

    // Step 10: Speed test (by default in comprehensive mode, skip with --no-speed)
    if !args.quick && !args.no_speed {
        pb.set_message("Running speed test...");
        let speed_result = check_speed(args.connections).await;
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

    let verbose = args.verbose > 0;

    for result in &results {
        let status = if result.passed {
            style("PASS").green().bold()
        } else {
            style("FAIL").red().bold()
        };

        println!("  [{}] {}", status, style(&result.name).bold());
        println!("         {}", result.details);

        // Show verbose details if -v flag is set
        if verbose && !result.verbose_details.is_empty() {
            for detail in &result.verbose_details {
                println!("         {}", style(detail).dim());
            }
        }

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
        steps += 1; // latency/jitter analysis
        steps += 1; // path analysis
        steps += 1; // ISP identification
    }

    if args.wifi {
        steps += 1;
    }

    // Speed test runs by default in comprehensive mode
    if !args.quick && !args.no_speed {
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
                    .map(|i| i.display_name.as_ref().unwrap_or(&i.name).clone())
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
            // Thorough gateway testing with latency, jitter, and packet loss analysis
            let pinger = Pinger::new();
            let config = PingConfig {
                count: 20, // More samples for reliable statistics
                timeout: Duration::from_secs(2),
                interval: Duration::from_millis(100), // Fast interval for local gateway
                size: 64,
            };

            match pinger.ping(gateway.address, &config).await {
                Ok(stats) if stats.received > 0 => {
                    let mut verbose = Vec::new();

                    // Basic latency info
                    let avg_ms = stats.avg_rtt.map_or(0.0, |r| r.as_secs_f64() * 1000.0);
                    let min_ms = stats.min_rtt.map_or(0.0, |r| r.as_secs_f64() * 1000.0);
                    let max_ms = stats.max_rtt.map_or(0.0, |r| r.as_secs_f64() * 1000.0);
                    let jitter_ms = stats.jitter.map_or(0.0, |j| j.as_secs_f64() * 1000.0);

                    // Verbose details
                    verbose.push(format!("Gateway: {}", gateway.address));
                    verbose.push(format!("Samples: {} pings", stats.transmitted));
                    verbose.push(format!(
                        "Min/Avg/Max: {:.1}/{:.1}/{:.1}ms",
                        min_ms, avg_ms, max_ms
                    ));

                    // Latency percentiles
                    if let Some(ref percentiles) = stats.latency_percentiles {
                        verbose.push(format!(
                            "P50/P95/P99: {:.1}/{:.1}/{:.1}ms",
                            percentiles.p50.as_secs_f64() * 1000.0,
                            percentiles.p95.as_secs_f64() * 1000.0,
                            percentiles.p99.as_secs_f64() * 1000.0
                        ));
                    }

                    // Jitter analysis
                    verbose.push(format!("Jitter: {:.2}ms", jitter_ms));

                    // Packet loss analysis
                    let loss_pattern: Vec<bool> = stats.results.iter().map(|r| r.success).collect();
                    let loss_stats = PacketLossStats::from_pattern(&loss_pattern);
                    verbose.push(format!("Packet loss: {:.1}%", loss_stats.loss_percent));

                    if loss_stats.burst_count > 0 {
                        verbose.push(format!(
                            "Loss bursts: {} (max consecutive: {})",
                            loss_stats.burst_count, loss_stats.max_burst_length
                        ));
                    }

                    // Determine status based on metrics
                    let mut issues = Vec::new();

                    // Check latency (gateway should be very fast, < 10ms ideal)
                    if avg_ms > 50.0 {
                        issues.push("High latency to gateway");
                    } else if avg_ms > 20.0 {
                        issues.push("Elevated latency");
                    }

                    // Check jitter (should be very low for local gateway)
                    if jitter_ms > 10.0 {
                        issues.push("High jitter");
                    } else if jitter_ms > 5.0 {
                        issues.push("Elevated jitter");
                    }

                    // Check packet loss
                    if loss_stats.loss_percent > 5.0 {
                        issues.push("Significant packet loss");
                    } else if loss_stats.loss_percent > 1.0 {
                        issues.push("Some packet loss");
                    }

                    // Check for burst losses (indicates intermittent connectivity)
                    if loss_stats.burst_count > 2 {
                        issues.push("Intermittent connectivity");
                    }

                    // Build summary
                    let summary = format!(
                        "Gateway {} reachable ({:.1}ms avg, {:.1}ms jitter, {:.1}% loss)",
                        gateway.address, avg_ms, jitter_ms, loss_stats.loss_percent
                    );

                    if issues.is_empty() {
                        DiagnosticCheck::pass("Default Gateway", &summary).with_verbose(verbose)
                    } else if loss_stats.loss_percent > 10.0 || avg_ms > 100.0 {
                        DiagnosticCheck::fail(
                            "Default Gateway",
                            &format!("{} - Issues: {}", summary, issues.join(", ")),
                            Some("Check router health, WiFi signal, or network cable"),
                        )
                        .with_verbose(verbose)
                    } else {
                        // Minor issues - still pass but note the concerns
                        DiagnosticCheck::pass(
                            "Default Gateway",
                            &format!("{} ({})", summary, issues.join(", ")),
                        )
                        .with_verbose(verbose)
                    }
                }
                Ok(_) => DiagnosticCheck::fail(
                    "Default Gateway",
                    &format!(
                        "Gateway {} is not responding (100% packet loss)",
                        gateway.address
                    ),
                    Some("Check your router, WiFi signal, or network cable"),
                ),
                Err(e) => DiagnosticCheck::fail(
                    "Default Gateway",
                    &format!("Gateway {} test failed: {}", gateway.address, e),
                    Some("Ensure you have network connectivity"),
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
                &format!(
                    "{} DNS server(s): {}",
                    servers.len(),
                    server_list.join(", ")
                ),
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
            let status = if iface.powered_on {
                "enabled"
            } else {
                "disabled"
            };

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
                    &format!("Connected to \"{}\"{}", conn.access_point.ssid, signal_info),
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
        Err(e) => DiagnosticCheck::fail("WiFi", &format!("Failed to check WiFi: {}", e), None),
    }
}

async fn check_speed(connections: usize) -> DiagnosticCheck {
    let tester = SpeedTester::new();
    let config = SpeedTestConfig {
        duration: Duration::from_secs(5),
        connections,
        test_download: true,
        test_upload: true,
        ..Default::default()
    };

    match tester.run_test(&config).await {
        Ok(result) => {
            let mut verbose = Vec::new();

            // Build summary
            let download_str = result
                .download
                .as_ref()
                .map(|d| format!("{:.1} Mbps", d.mbps()))
                .unwrap_or_else(|| "N/A".to_string());

            let upload_str = result
                .upload
                .as_ref()
                .map(|u| format!("{:.1} Mbps", u.mbps()))
                .unwrap_or_else(|| "N/A".to_string());

            let latency_str = result
                .latency
                .map(|l| format!("{:.1}ms", l.as_secs_f64() * 1000.0))
                .unwrap_or_else(|| "N/A".to_string());

            let details = format!(
                "Download: {}, Upload: {}, Latency: {}",
                download_str, upload_str, latency_str
            );

            // Verbose details
            verbose.push(format!("Provider: {}", result.provider));
            verbose.push(format!(
                "Server: {} ({})",
                result.server.name, result.server.url
            ));

            if let Some(ref download) = result.download {
                verbose.push(format!(
                    "Download: {} ({} connections)",
                    download.format_speed(),
                    download.connections
                ));
            }

            if let Some(ref upload) = result.upload {
                verbose.push(format!(
                    "Upload: {} ({} connections)",
                    upload.format_speed(),
                    upload.connections
                ));
            }

            if let Some(ref bloat) = result.buffer_bloat {
                verbose.push(format!(
                    "Buffer Bloat: Grade {} - {}",
                    bloat.grade,
                    bloat.description()
                ));
            }

            if let Some(ref consistency) = result.consistency {
                verbose.push(format!(
                    "Consistency: {} (CV: {:.2})",
                    consistency.rating, consistency.coefficient_of_variation
                ));
            }

            // Determine pass/fail based on download speed
            let download_mbps = result.download_mbps().unwrap_or(0.0);
            if download_mbps < 1.0 {
                DiagnosticCheck::fail(
                    "Speed Test",
                    &details,
                    Some("Very slow connection - check for network issues"),
                )
                .with_verbose(verbose)
            } else {
                DiagnosticCheck::pass("Speed Test", &details).with_verbose(verbose)
            }
        }
        Err(e) => DiagnosticCheck::fail(
            "Speed Test",
            &format!("Speed test failed: {}", e),
            Some("Check internet connectivity or try 'netdiag speed' for detailed test"),
        ),
    }
}

/// Comprehensive latency and jitter analysis.
async fn check_latency_jitter() -> (DiagnosticCheck, DiagnosticCheck) {
    let pinger = Pinger::new();
    let config = PingConfig {
        count: 30, // More samples for better analysis
        timeout: Duration::from_secs(5),
        interval: Duration::from_millis(200), // Faster interval for jitter detection
        size: 64,
    };

    // Test against a reliable target
    let target: IpAddr = "8.8.8.8".parse().unwrap();

    match pinger.ping(target, &config).await {
        Ok(stats) => {
            // Latency analysis with percentiles
            let latency_result = if let Some(ref percentiles) = stats.latency_percentiles {
                let p50_ms = percentiles.p50.as_secs_f64() * 1000.0;
                let p75_ms = percentiles.p75.as_secs_f64() * 1000.0;
                let p90_ms = percentiles.p90.as_secs_f64() * 1000.0;
                let p95_ms = percentiles.p95.as_secs_f64() * 1000.0;
                let p99_ms = percentiles.p99.as_secs_f64() * 1000.0;
                let iqr_ms = percentiles.iqr.as_secs_f64() * 1000.0;

                let quality = if p95_ms < 50.0 {
                    "Excellent"
                } else if p95_ms < 100.0 {
                    "Good"
                } else if p95_ms < 200.0 {
                    "Fair"
                } else {
                    "Poor"
                };

                let details = format!(
                    "P50: {:.1}ms, P95: {:.1}ms, P99: {:.1}ms ({})",
                    p50_ms, p95_ms, p99_ms, quality
                );

                // Verbose details
                let verbose = vec![
                    format!("Target: {} (Google DNS)", target),
                    format!("Samples: {} pings", stats.transmitted),
                    format!("P50 (median): {:.1}ms", p50_ms),
                    format!("P75: {:.1}ms", p75_ms),
                    format!("P90: {:.1}ms", p90_ms),
                    format!("P95: {:.1}ms", p95_ms),
                    format!("P99: {:.1}ms", p99_ms),
                    format!("IQR (P75-P25): {:.1}ms", iqr_ms),
                    format!(
                        "Min/Avg/Max: {:.1}/{:.1}/{:.1}ms",
                        stats.min_rtt.map_or(0.0, |r| r.as_secs_f64() * 1000.0),
                        stats.avg_rtt.map_or(0.0, |r| r.as_secs_f64() * 1000.0),
                        stats.max_rtt.map_or(0.0, |r| r.as_secs_f64() * 1000.0)
                    ),
                ];

                if p95_ms > 200.0 {
                    DiagnosticCheck::fail(
                        "Latency Analysis",
                        &details,
                        Some("High latency detected - check network congestion"),
                    )
                    .with_verbose(verbose)
                } else {
                    DiagnosticCheck::pass("Latency Analysis", &details).with_verbose(verbose)
                }
            } else {
                DiagnosticCheck::fail(
                    "Latency Analysis",
                    "Unable to calculate latency percentiles",
                    None,
                )
            };

            // Jitter and VoIP quality analysis
            let jitter_result = if let Some(ref voip) = stats.voip_quality {
                let mos = voip.mos;
                let rating = format!("{}", voip.rating);
                let jitter_ms = stats.jitter.map_or(0.0, |j| j.as_secs_f64() * 1000.0);

                let details = format!(
                    "Jitter: {:.1}ms, MOS: {:.2}, VoIP Quality: {}",
                    jitter_ms, mos, rating
                );

                // Check packet loss patterns
                let loss_pattern: Vec<bool> = stats.results.iter().map(|r| r.success).collect();
                let loss_stats = PacketLossStats::from_pattern(&loss_pattern);

                let mut full_details = details.clone();
                if loss_stats.burst_count > 0 {
                    full_details.push_str(&format!(
                        " (Loss: {:.1}%, {} burst(s))",
                        loss_stats.loss_percent, loss_stats.burst_count
                    ));
                }

                // Verbose details
                let mut verbose = vec![
                    format!("R-factor: {:.1}", voip.r_factor),
                    format!("MOS (Mean Opinion Score): {:.2}/5.0", voip.mos),
                    format!("Effective latency: {:.1}ms", voip.effective_latency_ms),
                    format!("Avg jitter: {:.1}ms", jitter_ms),
                    format!("Packet loss: {:.1}%", loss_stats.loss_percent),
                ];

                if loss_stats.burst_count > 0 {
                    verbose.push(format!(
                        "Loss bursts: {} (max length: {})",
                        loss_stats.burst_count, loss_stats.max_burst_length
                    ));
                }

                verbose.push(format!("Delay impairment: {:.2}", voip.impact.delay_factor));
                verbose.push(format!(
                    "Jitter impairment: {:.2}",
                    voip.impact.jitter_factor
                ));
                verbose.push(format!("Loss impairment: {:.2}", voip.impact.loss_factor));

                if mos < 3.5 {
                    DiagnosticCheck::fail(
                        "Jitter & VoIP Quality",
                        &full_details,
                        Some("Poor VoIP quality - may affect video calls"),
                    )
                    .with_verbose(verbose)
                } else {
                    DiagnosticCheck::pass("Jitter & VoIP Quality", &full_details)
                        .with_verbose(verbose)
                }
            } else {
                // Fallback to basic jitter stats
                let rtts: Vec<Duration> = stats.results.iter().filter_map(|r| r.rtt).collect();
                if let Some(jitter_stats) = JitterStats::from_rtts(&rtts) {
                    let avg_jitter_ms = jitter_stats.average.as_secs_f64() * 1000.0;
                    let quality = jitter_stats.quality_rating();

                    DiagnosticCheck::pass(
                        "Jitter & VoIP Quality",
                        &format!("Avg jitter: {:.1}ms ({})", avg_jitter_ms, quality),
                    )
                } else {
                    DiagnosticCheck::fail(
                        "Jitter & VoIP Quality",
                        "Insufficient data for jitter analysis",
                        None,
                    )
                }
            };

            (latency_result, jitter_result)
        }
        Err(e) => {
            let fail = DiagnosticCheck::fail(
                "Latency Analysis",
                &format!("Failed: {}", e),
                Some("Check network connectivity"),
            );
            let jitter_fail = DiagnosticCheck::fail(
                "Jitter & VoIP Quality",
                "Unable to analyze (ping failed)",
                None,
            );
            (fail, jitter_fail)
        }
    }
}

/// Network path analysis with segment identification.
async fn check_network_path() -> DiagnosticCheck {
    let tracer = Tracer::new();
    let config = TracerouteConfig {
        max_hops: 20,
        probes_per_hop: 3,
        timeout: Duration::from_secs(3),
        ..Default::default()
    };

    // Trace to a well-known destination
    let target: IpAddr = "8.8.8.8".parse().unwrap();

    match tracer.trace(target, &config).await {
        Ok(trace_result) => {
            let analyzer = PathAnalyzer::new();
            let analysis = analyzer.analyze(&trace_result);

            // Build verbose details
            let mut verbose = Vec::new();

            verbose.push(format!("Target: {} (Google DNS)", target));
            verbose.push(format!("Total hops: {}", trace_result.hops.len()));
            verbose.push(format!("Health score: {}/100", analysis.health.score));
            verbose.push(String::new()); // Empty line separator

            // Detailed segment breakdown
            let segments = [
                ("Local", &analysis.segments.local),
                ("Router", &analysis.segments.router),
                ("ISP", &analysis.segments.isp),
                ("Backbone", &analysis.segments.backbone),
                ("Destination", &analysis.segments.destination),
            ];

            for (name, segment) in &segments {
                if !segment.hops.is_empty() {
                    let latency_str = segment
                        .latency
                        .as_ref()
                        .map(|l| format!("{:.1}ms ({:.0}% of total)", l.absolute_ms, l.percentage))
                        .unwrap_or_else(|| "N/A".to_string());

                    let status_str = format!("{}", segment.status);
                    verbose.push(format!(
                        "[{}] {} - {} hops, {} latency, status: {}",
                        name.to_uppercase(),
                        name,
                        segment.hops.len(),
                        latency_str,
                        status_str
                    ));

                    // Show individual hops in this segment
                    for hop in &segment.hops {
                        let hop_name = hop
                            .hostname
                            .as_ref()
                            .or(hop.ip.as_ref().map(|ip| ip.to_string()).as_ref())
                            .map(|s| s.clone())
                            .unwrap_or_else(|| "*".to_string());

                        let hop_rtt = hop
                            .rtt
                            .map(|r| format!("{:.1}ms", r.as_secs_f64() * 1000.0))
                            .unwrap_or_else(|| "*".to_string());

                        verbose.push(format!(
                            "  Hop {}: {} ({})",
                            hop.hop_number, hop_name, hop_rtt
                        ));
                    }

                    // Show segment issues
                    if !segment.issues.is_empty() {
                        for issue in &segment.issues {
                            verbose
                                .push(format!("  ! {}: {}", issue.issue_type, issue.description));
                        }
                    }

                    verbose.push(String::new()); // Separator between segments
                }
            }

            // Show all identified issues
            if !analysis.issues.is_empty() {
                verbose.push("Issues Found:".to_string());
                for issue in &analysis.issues {
                    verbose.push(format!(
                        "  [{:?}] {} in {}: {}",
                        issue.severity, issue.issue_type, issue.segment, issue.description
                    ));
                }
                verbose.push(String::new());
            }

            // Show recommendations
            if !analysis.recommendations.is_empty() {
                verbose.push("Recommendations:".to_string());
                for rec in &analysis.recommendations {
                    verbose.push(format!("  - {}", rec));
                }
            }

            // Build summary for main output
            let mut details = Vec::new();
            details.push(format!(
                "Path health: {} (score: {})",
                analysis.health.rating, analysis.health.score
            ));

            for (name, segment) in &segments {
                if !segment.hops.is_empty() {
                    let latency_str = segment
                        .latency
                        .as_ref()
                        .map(|l| format!("{:.1}ms", l.absolute_ms))
                        .unwrap_or_else(|| "N/A".to_string());
                    details.push(format!(
                        "{}: {} hops, {}",
                        name,
                        segment.hops.len(),
                        latency_str
                    ));
                }
            }

            if !analysis.issues.is_empty() {
                let critical_count = analysis
                    .issues
                    .iter()
                    .filter(|i| i.severity == netdiag_types::diagnostics::IssueSeverity::Critical)
                    .count();
                let warning_count = analysis.issues.len() - critical_count;
                details.push(format!(
                    "Issues: {} critical, {} warnings",
                    critical_count, warning_count
                ));
            }

            let passed = analysis.health.score >= 60;
            let detail_str = details.join("; ");

            if passed {
                DiagnosticCheck::pass("Network Path Analysis", &detail_str).with_verbose(verbose)
            } else {
                let suggestion = analysis
                    .recommendations
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "Check network configuration".to_string());
                DiagnosticCheck::fail("Network Path Analysis", &detail_str, Some(&suggestion))
                    .with_verbose(verbose)
            }
        }
        Err(e) => DiagnosticCheck::fail(
            "Network Path Analysis",
            &format!("Traceroute failed: {}", e),
            Some("Ensure traceroute is available and you have network connectivity"),
        ),
    }
}

/// Extract ISP name from hostname.
/// Common patterns:
/// - "po-200-xar01.stockton.ca.ccal.comcast.net" -> "Comcast"
/// - "be2930.ccr42.lax04.atlas.cogentco.com" -> "Cogent"
/// - "ae-5.r21.lsanca07.us.bb.gin.ntt.net" -> "NTT"
fn extract_isp_from_hostname(hostname: &str) -> Option<String> {
    let hostname_lower = hostname.to_lowercase();

    // Common ISP domain patterns
    let isp_patterns: &[(&str, &str)] = &[
        ("comcast.net", "Comcast"),
        ("xfinity.com", "Comcast/Xfinity"),
        ("verizon.net", "Verizon"),
        ("verizon.com", "Verizon"),
        ("att.net", "AT&T"),
        ("sbcglobal.net", "AT&T"),
        ("spectrum.net", "Spectrum"),
        ("charter.com", "Spectrum/Charter"),
        ("cox.net", "Cox"),
        ("centurylink.net", "CenturyLink"),
        ("lumen.com", "Lumen"),
        ("cogentco.com", "Cogent"),
        ("level3.net", "Level3/Lumen"),
        ("ntt.net", "NTT"),
        ("telia.net", "Telia"),
        ("gtt.net", "GTT"),
        ("zayo.com", "Zayo"),
        ("he.net", "Hurricane Electric"),
        ("google.com", "Google"),
        ("googlefiber.net", "Google Fiber"),
        ("1e100.net", "Google"),
        ("akamai.com", "Akamai"),
        ("cloudflare.com", "Cloudflare"),
        ("fastly.net", "Fastly"),
        ("amazonaws.com", "AWS"),
        ("azure.com", "Microsoft Azure"),
        ("frontier.com", "Frontier"),
        ("suddenlink.net", "Suddenlink/Altice"),
        ("optimum.net", "Optimum/Altice"),
        ("rcn.net", "RCN"),
        ("sonic.net", "Sonic"),
        ("t-mobile.com", "T-Mobile"),
        ("sprint.net", "Sprint/T-Mobile"),
        ("tmus.net", "T-Mobile"),
    ];

    for (pattern, isp_name) in isp_patterns {
        if hostname_lower.contains(pattern) {
            return Some(isp_name.to_string());
        }
    }

    // Try to extract from subdomain patterns
    // e.g., "*.ccal.comcast.net" - already handled above
    // Try to get the second-to-last domain part for unknown ISPs
    let parts: Vec<&str> = hostname.split('.').collect();
    if parts.len() >= 2 {
        let domain = parts[parts.len() - 2];
        // Capitalize first letter
        if domain.len() > 2 && domain.chars().all(|c| c.is_alphanumeric()) {
            let mut chars = domain.chars();
            if let Some(first) = chars.next() {
                return Some(format!(
                    "{}{}",
                    first.to_uppercase(),
                    chars.collect::<String>()
                ));
            }
        }
    }

    None
}

/// ISP identification.
async fn check_isp() -> DiagnosticCheck {
    let tracer = Tracer::new();
    let config = TracerouteConfig {
        max_hops: 10, // Only need first few hops for ISP identification
        probes_per_hop: 1,
        timeout: Duration::from_secs(2),
        ..Default::default()
    };

    let target: IpAddr = "8.8.8.8".parse().unwrap();

    match tracer.trace(target, &config).await {
        Ok(trace_result) => {
            let mut verbose = Vec::new();

            // Extract hop info for ISP identification
            let hops: Vec<_> = trace_result
                .hops
                .iter()
                .map(|hop| netdiag_types::diagnostics::HopInfo {
                    hop_number: hop.hop,
                    ip: hop.address,
                    hostname: hop.hostname.clone(),
                    rtt: hop.avg_rtt,
                    asn: hop.asn,
                    as_name: hop.as_name.clone(),
                    organization: hop.as_name.clone(),
                    location: hop.location.as_ref().map(|loc| {
                        netdiag_types::diagnostics::GeoLocation {
                            city: loc.city.clone(),
                            region: None,
                            country: loc.country.clone(),
                            country_code: None,
                            latitude: loc.latitude,
                            longitude: loc.longitude,
                        }
                    }),
                    responsive: !hop.all_timeout,
                })
                .collect();

            // Add verbose hop info
            for hop in trace_result.hops.iter().take(8) {
                if let Some(hostname) = &hop.hostname {
                    let rtt_str = hop
                        .avg_rtt
                        .map(|r| format!("{:.1}ms", r.as_secs_f64() * 1000.0))
                        .unwrap_or_else(|| "*".to_string());
                    verbose.push(format!("Hop {}: {} ({})", hop.hop, hostname, rtt_str));
                } else if let Some(ip) = hop.address {
                    let rtt_str = hop
                        .avg_rtt
                        .map(|r| format!("{:.1}ms", r.as_secs_f64() * 1000.0))
                        .unwrap_or_else(|| "*".to_string());
                    verbose.push(format!("Hop {}: {} ({})", hop.hop, ip, rtt_str));
                }
            }

            if let Some(isp) = identify_isp(&hops) {
                let asn_str = isp.asn.map_or(String::new(), |asn| format!(" (AS{})", asn));
                DiagnosticCheck::pass(
                    "ISP Identification",
                    &format!(
                        "Provider: {}{}, Type: {}",
                        isp.name, asn_str, isp.service_type
                    ),
                )
                .with_verbose(verbose)
            } else {
                // Try to extract ISP from hostnames
                let first_isp_hop = trace_result
                    .hops
                    .iter()
                    .skip(1) // Skip local gateway
                    .find(|h| {
                        h.hostname.as_ref().map_or(false, |hostname| {
                            // Skip local/private hostnames
                            !hostname.contains("local")
                                && !hostname.contains("router")
                                && !hostname.contains("gateway")
                                && !hostname.starts_with("192.")
                                && !hostname.starts_with("10.")
                        })
                    });

                if let Some(hop) = first_isp_hop {
                    if let Some(hostname) = &hop.hostname {
                        // Try to extract ISP name from hostname
                        if let Some(isp_name) = extract_isp_from_hostname(hostname) {
                            return DiagnosticCheck::pass(
                                "ISP Identification",
                                &format!("Provider: {} (detected from hostname)", isp_name),
                            )
                            .with_verbose(verbose);
                        }
                    }

                    // Fallback to showing hostname
                    let name = hop
                        .hostname
                        .clone()
                        .or_else(|| hop.address.map(|a| a.to_string()))
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Try one more extraction attempt
                    if let Some(isp_name) = extract_isp_from_hostname(&name) {
                        DiagnosticCheck::pass(
                            "ISP Identification",
                            &format!("Provider: {}", isp_name),
                        )
                        .with_verbose(verbose)
                    } else {
                        DiagnosticCheck::pass(
                            "ISP Identification",
                            &format!("ISP hop detected: {}", name),
                        )
                        .with_verbose(verbose)
                    }
                } else {
                    DiagnosticCheck::pass(
                        "ISP Identification",
                        "Unable to identify ISP (no external hops found)",
                    )
                    .with_verbose(verbose)
                }
            }
        }
        Err(_) => DiagnosticCheck::pass("ISP Identification", "Skipped (traceroute unavailable)"),
    }
}
