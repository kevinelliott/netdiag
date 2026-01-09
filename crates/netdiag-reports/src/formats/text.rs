//! Text format implementation.

use crate::error::ReportResult;
use crate::report::DiagnosticReport;
use crate::ReportFormatter;
use std::fmt::Write;

/// Text report formatter.
pub struct TextFormatter {
    /// Line width for separator lines
    pub line_width: usize,
}

impl TextFormatter {
    /// Create a new text formatter.
    pub fn new() -> Self {
        Self { line_width: 80 }
    }

    fn separator(&self) -> String {
        "=".repeat(self.line_width)
    }

    fn thin_separator(&self) -> String {
        "-".repeat(self.line_width)
    }
}

impl Default for TextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportFormatter for TextFormatter {
    fn format(&self, report: &DiagnosticReport) -> ReportResult<String> {
        let mut output = String::new();

        // Header
        writeln!(output, "{}", self.separator()).unwrap();
        writeln!(
            output,
            "{:^width$}",
            report.metadata.title,
            width = self.line_width
        )
        .unwrap();
        writeln!(output, "{}", self.separator()).unwrap();
        writeln!(output).unwrap();

        // Metadata
        writeln!(output, "Report ID: {}", report.metadata.id).unwrap();
        writeln!(
            output,
            "Generated: {}",
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        )
        .unwrap();
        writeln!(output, "Version: {}", report.metadata.version).unwrap();
        if let Some(ref hostname) = report.metadata.hostname {
            writeln!(output, "Hostname: {}", hostname).unwrap();
        }
        if let Some(ref os_info) = report.metadata.os_info {
            writeln!(output, "OS: {}", os_info).unwrap();
        }
        writeln!(output).unwrap();

        // Health Assessment
        if let Some(ref health) = report.health {
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output, "HEALTH ASSESSMENT").unwrap();
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output).unwrap();

            writeln!(
                output,
                "Status: {} (Score: {}/100)",
                health.status.to_uppercase(),
                health.score
            )
            .unwrap();
            writeln!(output).unwrap();

            if !health.issues.is_empty() {
                writeln!(output, "Issues:").unwrap();
                for issue in &health.issues {
                    writeln!(output, "  - {}", issue).unwrap();
                }
                writeln!(output).unwrap();
            }

            if !health.recommendations.is_empty() {
                writeln!(output, "Recommendations:").unwrap();
                for rec in &health.recommendations {
                    writeln!(output, "  * {}", rec).unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        // Network Interfaces
        if !report.interfaces.is_empty() {
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output, "NETWORK INTERFACES").unwrap();
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output).unwrap();

            for iface in &report.interfaces {
                let status = if iface.is_up { "UP" } else { "DOWN" };
                let default_marker = if iface.is_default { " (default)" } else { "" };
                writeln!(
                    output,
                    "{} [{}] - {}{}",
                    iface.name, iface.interface_type, status, default_marker
                )
                .unwrap();

                if let Some(ref mac) = iface.mac_address {
                    writeln!(output, "  MAC: {}", mac).unwrap();
                }

                for ip in &iface.ipv4_addresses {
                    writeln!(output, "  IPv4: {}", ip).unwrap();
                }
                for ip in &iface.ipv6_addresses {
                    writeln!(output, "  IPv6: {}", ip).unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        // DNS Results
        if !report.dns_results.is_empty() {
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output, "DNS RESOLUTION").unwrap();
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output).unwrap();

            for dns in &report.dns_results {
                let status = if dns.success { "OK" } else { "FAILED" };
                writeln!(
                    output,
                    "{} -> {} ({:.1}ms)",
                    dns.query, status, dns.duration_ms
                )
                .unwrap();

                if !dns.addresses.is_empty() {
                    writeln!(output, "  Addresses: {}", dns.addresses.join(", ")).unwrap();
                }

                if let Some(ref error) = dns.error {
                    writeln!(output, "  Error: {}", error).unwrap();
                }
            }
            writeln!(output).unwrap();
        }

        // Ping Results
        if !report.ping_results.is_empty() {
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output, "PING RESULTS").unwrap();
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output).unwrap();

            for ping in &report.ping_results {
                writeln!(output, "Target: {}", ping.target).unwrap();
                writeln!(
                    output,
                    "  Packets: {} transmitted, {} received ({:.1}% loss)",
                    ping.transmitted, ping.received, ping.loss_percent
                )
                .unwrap();

                if let (Some(min), Some(avg), Some(max)) =
                    (ping.min_rtt_ms, ping.avg_rtt_ms, ping.max_rtt_ms)
                {
                    write!(
                        output,
                        "  RTT: min={:.1}ms avg={:.1}ms max={:.1}ms",
                        min, avg, max
                    )
                    .unwrap();
                    if let Some(stddev) = ping.stddev_ms {
                        write!(output, " stddev={:.1}ms", stddev).unwrap();
                    }
                    writeln!(output).unwrap();
                }

                writeln!(output, "  Quality: {}", ping.quality).unwrap();
                writeln!(output).unwrap();
            }
        }

        // Traceroute Results
        if !report.traceroute_results.is_empty() {
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output, "TRACEROUTE RESULTS").unwrap();
            writeln!(output, "{}", self.thin_separator()).unwrap();
            writeln!(output).unwrap();

            for traceroute in &report.traceroute_results {
                let status = if traceroute.reached {
                    "reached"
                } else {
                    "not reached"
                };
                writeln!(
                    output,
                    "Target: {} ({}) - {} hops, {:.0}ms",
                    traceroute.target, status, traceroute.hop_count, traceroute.duration_ms
                )
                .unwrap();
                writeln!(output, "Protocol: {}", traceroute.protocol).unwrap();
                writeln!(output).unwrap();

                for hop in &traceroute.hops {
                    if hop.all_timeout {
                        writeln!(output, "{:>3}  * * *", hop.hop).unwrap();
                    } else {
                        let addr = hop.address.as_deref().unwrap_or("*");
                        let name = hop.hostname.as_deref();

                        let display = match name {
                            Some(n) => format!("{} ({})", n, addr),
                            None => addr.to_string(),
                        };

                        let rtts: Vec<String> = hop
                            .rtt_ms
                            .iter()
                            .map(|rtt| match rtt {
                                Some(ms) => format!("{:.1} ms", ms),
                                None => "*".to_string(),
                            })
                            .collect();

                        writeln!(output, "{:>3}  {}  {}", hop.hop, display, rtts.join("  "))
                            .unwrap();
                    }
                }
                writeln!(output).unwrap();
            }
        }

        // Footer
        writeln!(output, "{}", self.separator()).unwrap();
        writeln!(output, "End of Report").unwrap();
        writeln!(output, "{}", self.separator()).unwrap();

        Ok(output)
    }

    fn extension(&self) -> &'static str {
        "txt"
    }

    fn mime_type(&self) -> &'static str {
        "text/plain"
    }
}
