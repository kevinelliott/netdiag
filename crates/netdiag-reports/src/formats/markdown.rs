//! Markdown format implementation.

use crate::error::ReportResult;
use crate::report::DiagnosticReport;
use crate::ReportFormatter;
use std::fmt::Write;

/// Markdown report formatter.
pub struct MarkdownFormatter;

impl MarkdownFormatter {
    /// Create a new Markdown formatter.
    pub fn new() -> Self {
        Self
    }
}

impl Default for MarkdownFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportFormatter for MarkdownFormatter {
    fn format(&self, report: &DiagnosticReport) -> ReportResult<String> {
        let mut output = String::new();

        // Title
        writeln!(output, "# {}", report.metadata.title).unwrap();
        writeln!(output).unwrap();

        // Metadata
        writeln!(output, "| Property | Value |").unwrap();
        writeln!(output, "|----------|-------|").unwrap();
        writeln!(output, "| Report ID | `{}` |", report.metadata.id).unwrap();
        writeln!(output, "| Generated | {} |", report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")).unwrap();
        writeln!(output, "| Version | {} |", report.metadata.version).unwrap();
        if let Some(ref hostname) = report.metadata.hostname {
            writeln!(output, "| Hostname | {} |", hostname).unwrap();
        }
        if let Some(ref os_info) = report.metadata.os_info {
            writeln!(output, "| OS | {} |", os_info).unwrap();
        }
        writeln!(output).unwrap();

        // Health Assessment
        if let Some(ref health) = report.health {
            writeln!(output, "## Health Assessment").unwrap();
            writeln!(output).unwrap();

            let status_emoji = match health.status.as_str() {
                "good" => "🟢",
                "warning" => "🟡",
                "critical" => "🔴",
                _ => "⚪",
            };

            writeln!(output, "**Status:** {} {} (Score: {}/100)", status_emoji, health.status.to_uppercase(), health.score).unwrap();
            writeln!(output).unwrap();

            if !health.issues.is_empty() {
                writeln!(output, "### Issues").unwrap();
                writeln!(output).unwrap();
                for issue in &health.issues {
                    writeln!(output, "- ⚠️ {}", issue).unwrap();
                }
                writeln!(output).unwrap();
            }

            if !health.recommendations.is_empty() {
                writeln!(output, "### Recommendations").unwrap();
                writeln!(output).unwrap();
                for rec in &health.recommendations {
                    writeln!(output, "- 💡 {}", rec).unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        // Network Interfaces
        if !report.interfaces.is_empty() {
            writeln!(output, "## Network Interfaces").unwrap();
            writeln!(output).unwrap();

            for iface in &report.interfaces {
                let status = if iface.is_up { "🟢 UP" } else { "🔴 DOWN" };
                let default_marker = if iface.is_default { " ⭐" } else { "" };

                writeln!(output, "### {} {}{}", iface.name, status, default_marker).unwrap();
                writeln!(output).unwrap();
                writeln!(output, "- **Type:** {}", iface.interface_type).unwrap();

                if let Some(ref mac) = iface.mac_address {
                    writeln!(output, "- **MAC:** `{}`", mac).unwrap();
                }

                if !iface.ipv4_addresses.is_empty() {
                    writeln!(output, "- **IPv4:** {}", iface.ipv4_addresses.iter()
                        .map(|ip| format!("`{}`", ip))
                        .collect::<Vec<_>>()
                        .join(", ")).unwrap();
                }

                if !iface.ipv6_addresses.is_empty() {
                    writeln!(output, "- **IPv6:** {}", iface.ipv6_addresses.iter()
                        .map(|ip| format!("`{}`", ip))
                        .collect::<Vec<_>>()
                        .join(", ")).unwrap();
                }

                writeln!(output).unwrap();
            }
        }

        // DNS Results
        if !report.dns_results.is_empty() {
            writeln!(output, "## DNS Resolution").unwrap();
            writeln!(output).unwrap();
            writeln!(output, "| Query | Status | Duration | Addresses |").unwrap();
            writeln!(output, "|-------|--------|----------|-----------|").unwrap();

            for dns in &report.dns_results {
                let status = if dns.success { "✅" } else { "❌" };
                let addresses = if dns.addresses.is_empty() {
                    dns.error.as_deref().unwrap_or("-").to_string()
                } else {
                    dns.addresses.join(", ")
                };

                writeln!(output, "| {} | {} | {:.1}ms | {} |",
                    dns.query, status, dns.duration_ms, addresses).unwrap();
            }
            writeln!(output).unwrap();
        }

        // Ping Results
        if !report.ping_results.is_empty() {
            writeln!(output, "## Ping Results").unwrap();
            writeln!(output).unwrap();

            for ping in &report.ping_results {
                writeln!(output, "### {}", ping.target).unwrap();
                writeln!(output).unwrap();
                writeln!(output, "| Metric | Value |").unwrap();
                writeln!(output, "|--------|-------|").unwrap();
                writeln!(output, "| Transmitted | {} |", ping.transmitted).unwrap();
                writeln!(output, "| Received | {} |", ping.received).unwrap();
                writeln!(output, "| Loss | {:.1}% |", ping.loss_percent).unwrap();

                if let Some(min) = ping.min_rtt_ms {
                    writeln!(output, "| Min RTT | {:.1}ms |", min).unwrap();
                }
                if let Some(avg) = ping.avg_rtt_ms {
                    writeln!(output, "| Avg RTT | {:.1}ms |", avg).unwrap();
                }
                if let Some(max) = ping.max_rtt_ms {
                    writeln!(output, "| Max RTT | {:.1}ms |", max).unwrap();
                }
                if let Some(stddev) = ping.stddev_ms {
                    writeln!(output, "| Std Dev | {:.1}ms |", stddev).unwrap();
                }
                writeln!(output, "| Quality | {} |", ping.quality).unwrap();
                writeln!(output).unwrap();
            }
        }

        // Traceroute Results
        if !report.traceroute_results.is_empty() {
            writeln!(output, "## Traceroute Results").unwrap();
            writeln!(output).unwrap();

            for traceroute in &report.traceroute_results {
                let status = if traceroute.reached { "✅ Reached" } else { "❌ Not reached" };
                writeln!(output, "### {} ({})", traceroute.target, status).unwrap();
                writeln!(output).unwrap();
                writeln!(output, "- **Protocol:** {}", traceroute.protocol).unwrap();
                writeln!(output, "- **Hops:** {}", traceroute.hop_count).unwrap();
                writeln!(output, "- **Duration:** {:.0}ms", traceroute.duration_ms).unwrap();
                writeln!(output).unwrap();

                writeln!(output, "| Hop | Address | RTT |").unwrap();
                writeln!(output, "|-----|---------|-----|").unwrap();

                for hop in &traceroute.hops {
                    let addr = if hop.all_timeout {
                        "* * *".to_string()
                    } else {
                        match (&hop.hostname, &hop.address) {
                            (Some(name), Some(ip)) => format!("{} (`{}`)", name, ip),
                            (None, Some(ip)) => format!("`{}`", ip),
                            _ => "*".to_string(),
                        }
                    };

                    let rtts: Vec<String> = hop.rtt_ms.iter()
                        .map(|rtt| match rtt {
                            Some(ms) => format!("{:.1}ms", ms),
                            None => "*".to_string(),
                        })
                        .collect();

                    writeln!(output, "| {} | {} | {} |", hop.hop, addr, rtts.join(", ")).unwrap();
                }
                writeln!(output).unwrap();
            }
        }

        // Footer
        writeln!(output, "---").unwrap();
        writeln!(output, "*Generated by netdiag v{}*", report.metadata.version).unwrap();

        Ok(output)
    }

    fn extension(&self) -> &'static str {
        "md"
    }

    fn mime_type(&self) -> &'static str {
        "text/markdown"
    }
}
