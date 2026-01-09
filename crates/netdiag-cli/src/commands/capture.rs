//! Packet capture command.

use crate::app::CaptureArgs;
use color_eyre::eyre::Result;
use comfy_table::{presets::UTF8_FULL, Table};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use netdiag_capture::{list_devices, CaptureConfig, CaptureFilter, PacketCapture, Protocol};
use std::time::Duration;

/// Run the capture command.
pub async fn run(args: CaptureArgs) -> Result<()> {
    // List interfaces mode
    if args.list_interfaces {
        return list_interfaces();
    }

    println!("{}", style("Packet Capture").bold().cyan());
    println!();

    // Check for root/admin privileges
    let has_privilege = check_privilege();
    if !has_privilege {
        println!(
            "{}",
            style("Warning: Packet capture typically requires elevated privileges (root/admin)")
                .yellow()
        );
        println!(
            "{}",
            style("Try running with 'sudo netdiag capture' on macOS/Linux").dim()
        );
        println!();
    }

    // Build capture configuration
    let mut config = if let Some(ref iface) = args.interface {
        CaptureConfig::for_device(iface)
    } else {
        // Use default device
        match netdiag_capture::default_device() {
            Ok(dev) => {
                println!("Using default interface: {}", style(&dev.name).green());
                CaptureConfig::for_device(&dev.name)
            }
            Err(e) => {
                eprintln!(
                    "{}",
                    style(format!("Failed to find default device: {}", e)).red()
                );
                return Ok(());
            }
        }
    };

    // Apply filter
    if let Some(ref filter_str) = args.filter {
        config = config.with_filter(CaptureFilter::new(filter_str));
        println!("Filter: {}", style(filter_str).yellow());
    }

    // Configure capture
    config = config.promiscuous(args.promiscuous);

    if args.count > 0 {
        config = config.max_packets(args.count);
    }

    if args.duration > 0 {
        config = config.max_duration(Duration::from_secs(args.duration));
    }

    println!();

    // Create capture
    let capture = PacketCapture::new(config);

    // Setup progress bar
    let pb = if args.count > 0 {
        let pb = ProgressBar::new(args.count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} packets ({eta})")?
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
        Some(pb)
    };

    // Print header
    println!(
        "{:>6} {:>8} {:>20} {:>20} {:>8} {:>6}",
        style("#").bold(),
        style("Proto").bold(),
        style("Source").bold(),
        style("Destination").bold(),
        style("Flags").bold(),
        style("Len").bold()
    );
    println!("{}", "-".repeat(76));

    // Start capture
    let mut packet_count = 0usize;
    let stats = capture.capture_sync(|packet| {
        packet_count += 1;

        // Format source/destination
        let src = packet
            .src_ip
            .map(|ip| {
                if let Some(port) = packet.src_port {
                    format!("{}:{}", ip, port)
                } else {
                    ip.to_string()
                }
            })
            .unwrap_or_else(|| "?".to_string());

        let dst = packet
            .dst_ip
            .map(|ip| {
                if let Some(port) = packet.dst_port {
                    format!("{}:{}", ip, port)
                } else {
                    ip.to_string()
                }
            })
            .unwrap_or_else(|| "?".to_string());

        let flags = packet
            .tcp_flags
            .map(|f| f.to_string_short())
            .unwrap_or_default();

        // Color code by protocol
        let proto_style = match packet.protocol {
            Protocol::Tcp => style(packet.protocol.name()).cyan(),
            Protocol::Udp => style(packet.protocol.name()).green(),
            Protocol::Icmp | Protocol::Icmpv6 => style(packet.protocol.name()).yellow(),
            Protocol::Dns => style(packet.protocol.name()).magenta(),
            Protocol::Http | Protocol::Https => style(packet.protocol.name()).blue(),
            _ => style(packet.protocol.name()).white(),
        };

        println!(
            "{:>6} {:>8} {:>20} {:>20} {:>8} {:>6}",
            style(packet_count).dim(),
            proto_style,
            truncate(&src, 20),
            truncate(&dst, 20),
            flags,
            packet.length
        );

        // Update progress
        if let Some(ref pb) = pb {
            if args.count > 0 {
                pb.set_position(packet_count as u64);
            } else {
                pb.set_message(format!("{} packets captured", packet_count));
            }
        }

        // Continue capturing
        true
    });

    if let Some(pb) = pb {
        pb.finish_and_clear();
    }

    // Print statistics
    match stats {
        Ok(stats) => {
            println!();
            println!("{}", style("Capture Statistics").bold().cyan());
            println!();

            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec!["Metric", "Value"]);

            table.add_row(vec![
                "Packets captured".to_string(),
                stats.packets_captured.to_string(),
            ]);
            table.add_row(vec![
                "Packets dropped (kernel)".to_string(),
                stats.packets_dropped.to_string(),
            ]);
            table.add_row(vec![
                "Packets dropped (interface)".to_string(),
                stats.packets_dropped_interface.to_string(),
            ]);
            table.add_row(vec![
                "Total bytes".to_string(),
                format_bytes(stats.bytes_captured),
            ]);
            table.add_row(vec![
                "Duration".to_string(),
                format!("{:.2}s", stats.duration.as_secs_f64()),
            ]);
            table.add_row(vec![
                "Packets/sec".to_string(),
                format!("{:.1}", stats.packets_per_second),
            ]);
            table.add_row(vec!["Bandwidth".to_string(), stats.format_bandwidth()]);

            println!("{}", table);

            // Protocol breakdown
            if !stats.protocols.is_empty() {
                println!();
                println!("{}", style("Protocol Breakdown").bold().cyan());
                println!();

                let mut proto_table = Table::new();
                proto_table.load_preset(UTF8_FULL);
                proto_table.set_header(vec!["Protocol", "Packets", "Bytes", "%"]);

                let mut protocols: Vec<_> = stats.protocols.iter().collect();
                protocols.sort_by(|a, b| b.1.packets.cmp(&a.1.packets));

                for (name, proto_stats) in protocols {
                    proto_table.add_row(vec![
                        name.to_string(),
                        proto_stats.packets.to_string(),
                        format_bytes(proto_stats.bytes),
                        format!("{:.1}%", proto_stats.percentage(stats.packets_captured)),
                    ]);
                }

                println!("{}", proto_table);
            }

            // Top talkers
            if !stats.top_talkers.is_empty() {
                println!();
                println!("{}", style("Top Talkers").bold().cyan());
                println!();

                for (i, (ip, count)) in stats.top_talkers.iter().take(5).enumerate() {
                    println!("  {}. {} - {} packets", i + 1, ip, count);
                }
            }
        }
        Err(e) => {
            eprintln!("{}", style(format!("Capture error: {}", e)).red());
        }
    }

    Ok(())
}

/// List available network interfaces.
fn list_interfaces() -> Result<()> {
    println!("{}", style("Available Network Interfaces").bold().cyan());
    println!();

    match list_devices() {
        Ok(devices) => {
            let mut table = Table::new();
            table.load_preset(UTF8_FULL);
            table.set_header(vec!["Name", "Description", "Addresses", "Status"]);

            for device in devices {
                let status = if device.is_up && device.is_running {
                    style("UP").green().to_string()
                } else if device.is_up {
                    style("DOWN").yellow().to_string()
                } else {
                    style("DISABLED").red().to_string()
                };

                let loopback = if device.is_loopback {
                    " (loopback)"
                } else {
                    ""
                };

                let addresses = if device.addresses.is_empty() {
                    "-".to_string()
                } else {
                    device.addresses.join("\n")
                };

                table.add_row(vec![
                    format!("{}{}", device.name, loopback),
                    device.description.unwrap_or_else(|| "-".to_string()),
                    addresses,
                    status,
                ]);
            }

            println!("{}", table);
        }
        Err(e) => {
            eprintln!("{}", style(format!("Failed to list devices: {}", e)).red());
            eprintln!();
            eprintln!("This may require elevated privileges. Try:");
            eprintln!("  sudo netdiag capture --list-interfaces");
        }
    }

    Ok(())
}

/// Check if we have elevated privileges.
fn check_privilege() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(windows)]
    {
        // On Windows, we'd check for admin privileges differently
        true
    }
}

/// Truncate a string to a maximum length.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format bytes as human readable.
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else if bytes >= 1_000_000 {
        format!("{:.2} MB", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.2} KB", bytes as f64 / 1_000.0)
    } else {
        format!("{} B", bytes)
    }
}
