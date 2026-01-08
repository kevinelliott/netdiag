//! Traceroute command implementation.

use crate::app::TracerouteArgs;
use color_eyre::eyre::Result;
use console::style;
use netdiag_connectivity::{DnsResolver, Tracer, TracerouteConfig};
use netdiag_types::diagnostics::TracerouteProtocol;
use std::time::Duration;

/// Run the traceroute command.
pub async fn run(args: TracerouteArgs) -> Result<()> {
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

    // Convert protocol
    let protocol = match args.protocol {
        crate::app::TracerouteProtocol::Icmp => TracerouteProtocol::Icmp,
        crate::app::TracerouteProtocol::Udp => TracerouteProtocol::Udp,
        crate::app::TracerouteProtocol::Tcp => TracerouteProtocol::Tcp,
    };

    println!(
        "{} to {} ({}), {} hops max, {} byte packets",
        style("traceroute").bold(),
        hostname.as_ref().unwrap_or(&args.target),
        target_ip,
        args.max_hops,
        64
    );
    println!();

    // Create tracer with config
    let tracer = Tracer::new();
    let config = TracerouteConfig {
        max_hops: args.max_hops,
        probes_per_hop: args.probes,
        timeout: Duration::from_secs_f64(args.timeout),
        protocol,
        resolve_hostnames: !args.no_resolve,
    };

    // Run traceroute
    let result = tracer.trace(target_ip, &config).await?;

    // Display results
    for hop in &result.hops {
        if hop.all_timeout {
            println!(
                "{:>3}  {}",
                style(hop.hop).dim(),
                style("* * *").yellow()
            );
        } else {
            // Build address display
            let addr_str = match (&hop.hostname, hop.address) {
                (Some(host), Some(ip)) if !args.no_resolve => {
                    format!("{} ({})", style(host).cyan(), ip)
                }
                (_, Some(ip)) => format!("{}", style(ip).cyan()),
                _ => style("*").yellow().to_string(),
            };

            // Build RTT display
            let rtt_strs: Vec<String> = hop
                .probes
                .iter()
                .map(|probe| {
                    if let Some(rtt) = probe.rtt {
                        format!("{:.3} ms", rtt.as_secs_f64() * 1000.0)
                    } else {
                        "*".to_string()
                    }
                })
                .collect();

            println!(
                "{:>3}  {}  {}",
                style(hop.hop).dim(),
                addr_str,
                rtt_strs.join("  ")
            );
        }
    }

    println!();

    // Show summary
    if result.reached {
        println!(
            "Destination {} reached in {} hops",
            style(&args.target).green().bold(),
            result.hops.len()
        );
    } else {
        println!(
            "Destination {} not reached after {} hops",
            style(&args.target).yellow(),
            result.hops.len()
        );
    }

    println!(
        "Total time: {:.0}ms",
        result.duration.as_millis()
    );

    Ok(())
}
