//! Ping command implementation.

use crate::app::PingArgs;
use color_eyre::eyre::Result;
use console::style;
use netdiag_connectivity::{DnsResolver, PingConfig, Pinger};
use std::time::Duration;

/// Run the ping command.
pub async fn run(args: PingArgs) -> Result<()> {
    // First resolve the target
    let resolver = DnsResolver::new()?;
    let dns_result = resolver.resolve(&args.target).await?;

    if dns_result.addresses.is_empty() {
        eprintln!(
            "{} Could not resolve {}",
            style("Error:").red().bold(),
            args.target
        );
        return Ok(());
    }

    let target_ip = dns_result.addresses[0];
    let hostname = if dns_result.addresses[0].to_string() != args.target {
        Some(args.target.clone())
    } else {
        None
    };

    println!(
        "{} {} ({}) with {} bytes of data",
        style("PING").bold(),
        hostname.as_ref().unwrap_or(&args.target),
        target_ip,
        args.size
    );
    println!();

    // Create pinger with config
    let pinger = Pinger::new();
    let config = PingConfig {
        count: args.count,
        timeout: Duration::from_secs(args.timeout as u64),
        interval: Duration::from_secs_f64(args.interval),
        size: args.size as usize,
    };

    // Run ping
    let stats = pinger.ping(target_ip, &config).await?;

    // Display results
    for result in &stats.results {
        if result.success {
            println!(
                "{} bytes from {}: icmp_seq={} ttl={} time={:.3} ms",
                result.size,
                target_ip,
                result.seq,
                result.ttl.unwrap_or(64),
                result.rtt.unwrap_or_default().as_secs_f64() * 1000.0
            );
        } else {
            println!(
                "Request timeout for icmp_seq {}",
                result.seq
            );
        }
    }

    println!();
    println!(
        "--- {} ping statistics ---",
        style(&args.target).cyan()
    );
    println!(
        "{} packets transmitted, {} received, {:.1}% packet loss, time {:.0}ms",
        stats.transmitted,
        stats.received,
        stats.loss_percent,
        stats.duration.as_millis()
    );

    if let (Some(min), Some(avg), Some(max)) = (stats.min_rtt, stats.avg_rtt, stats.max_rtt) {
        let stddev = stats.stddev_rtt.unwrap_or_default();
        println!(
            "rtt min/avg/max/mdev = {:.3}/{:.3}/{:.3}/{:.3} ms",
            min.as_secs_f64() * 1000.0,
            avg.as_secs_f64() * 1000.0,
            max.as_secs_f64() * 1000.0,
            stddev.as_secs_f64() * 1000.0
        );
    }

    // Show quality rating
    let quality = stats.quality_rating();
    let quality_styled = match quality {
        netdiag_types::diagnostics::PingQuality::Excellent => style("Excellent").green().bold(),
        netdiag_types::diagnostics::PingQuality::VeryGood => style("Very Good").green(),
        netdiag_types::diagnostics::PingQuality::Good => style("Good").green(),
        netdiag_types::diagnostics::PingQuality::Fair => style("Fair").yellow(),
        netdiag_types::diagnostics::PingQuality::Poor => style("Poor").red(),
        netdiag_types::diagnostics::PingQuality::VeryPoor => style("Very Poor").red().bold(),
    };
    println!("Connection quality: {}", quality_styled);

    Ok(())
}
