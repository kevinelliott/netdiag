//! Report command implementation.

use crate::app::ReportArgs;
use color_eyre::eyre::Result;
use console::style;
use netdiag_connectivity::{DnsResolver, PingConfig, Pinger};
use netdiag_reports::{
    DiagnosticReport, DnsSummary, HtmlFormatter, InterfaceSummary, JsonFormatter, MarkdownFormatter,
    ReportBuilder, ReportFormatter, TextFormatter,
};
use std::fs;
use std::time::Duration;

/// Run the report command.
pub async fn run(args: ReportArgs) -> Result<()> {
    println!("{}", style("Generating Diagnostic Report...").bold());
    println!();

    // Build the report
    let report = generate_report().await?;

    // Format the report based on requested format
    let (content, extension) = match args.report_format {
        crate::app::ReportFormat::Text => {
            let formatter = TextFormatter::new();
            (formatter.format(&report)?, formatter.extension())
        }
        crate::app::ReportFormat::Json => {
            let formatter = JsonFormatter::new();
            (formatter.format(&report)?, formatter.extension())
        }
        crate::app::ReportFormat::Markdown => {
            let formatter = MarkdownFormatter::new();
            (formatter.format(&report)?, formatter.extension())
        }
        crate::app::ReportFormat::Html => {
            let formatter = HtmlFormatter::new();
            (formatter.format(&report)?, formatter.extension())
        }
        crate::app::ReportFormat::Pdf => {
            // For now, use Text as a fallback for PDF
            println!(
                "{}",
                style("PDF format not yet implemented, falling back to Text").yellow()
            );
            let formatter = TextFormatter::new();
            (formatter.format(&report)?, "pdf")
        }
    };

    // Output the report
    if let Some(mut output_path) = args.output {
        // Add extension if not present
        if output_path.extension().is_none() {
            output_path.set_extension(extension);
        }

        fs::write(&output_path, &content)?;
        println!();
        println!(
            "Report saved to: {}",
            style(output_path.display()).green().bold()
        );
    } else {
        // Print to stdout
        println!();
        println!("{}", content);
    }

    Ok(())
}

/// Generate a diagnostic report by running tests.
async fn generate_report() -> Result<DiagnosticReport> {
    let mut builder = ReportBuilder::new()
        .title("Network Diagnostics Report")
        .hostname(hostname::get()?.to_string_lossy().to_string());

    // Get OS info
    #[cfg(target_os = "macos")]
    {
        builder = builder.os_info("macOS".to_string());
    }
    #[cfg(target_os = "linux")]
    {
        builder = builder.os_info("Linux".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        builder = builder.os_info("Windows".to_string());
    }

    // Add network interfaces
    println!("  Scanning network interfaces...");
    let interfaces = netdev::get_interfaces();
    let default_iface = netdev::get_default_interface().ok();
    let default_name = default_iface.map(|i| i.name.clone());

    for iface in interfaces {
        let is_default = default_name.as_ref() == Some(&iface.name);
        builder = builder.add_interface(InterfaceSummary {
            name: iface.name.clone(),
            interface_type: format!("{:?}", iface.if_type),
            ipv4_addresses: iface.ipv4.iter().map(|n| n.addr().to_string()).collect(),
            ipv6_addresses: iface.ipv6.iter().map(|n| n.addr().to_string()).collect(),
            mac_address: iface.mac_addr.map(|m| m.to_string()),
            is_up: iface.is_up(),
            is_default,
        });
    }

    // Run DNS tests
    println!("  Testing DNS resolution...");
    let resolver = DnsResolver::new()?;
    let dns_targets = ["google.com", "cloudflare.com", "1.1.1.1"];

    for target in &dns_targets {
        match resolver.resolve(target).await {
            Ok(result) => {
                builder = builder.add_dns_result(DnsSummary {
                    query: result.query,
                    addresses: result.addresses.iter().map(|a| a.to_string()).collect(),
                    duration_ms: result.duration.as_secs_f64() * 1000.0,
                    success: result.success,
                    error: result.error,
                });
            }
            Err(e) => {
                builder = builder.add_dns_result(DnsSummary {
                    query: target.to_string(),
                    addresses: vec![],
                    duration_ms: 0.0,
                    success: false,
                    error: Some(e.to_string()),
                });
            }
        }
    }

    // Run ping tests
    println!("  Running ping tests...");
    let pinger = Pinger::new();
    let ping_config = PingConfig {
        count: 3,
        timeout: Duration::from_secs(2),
        interval: Duration::from_millis(500),
        size: 64,
    };

    let ping_targets = ["8.8.8.8", "1.1.1.1"];
    for target in &ping_targets {
        if let Ok(ip) = target.parse() {
            if let Ok(stats) = pinger.ping(ip, &ping_config).await {
                builder = builder.add_ping_stats(&stats);
            }
        }
    }

    println!("  Finalizing report...");

    Ok(builder.build())
}
