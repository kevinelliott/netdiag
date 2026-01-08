//! HTML format implementation.

use crate::error::ReportResult;
use crate::report::DiagnosticReport;
use crate::ReportFormatter;
use std::fmt::Write;

/// HTML report formatter.
pub struct HtmlFormatter {
    /// Include inline CSS.
    include_css: bool,
    /// Dark mode.
    dark_mode: bool,
}

impl HtmlFormatter {
    /// Create a new HTML formatter.
    pub fn new() -> Self {
        Self {
            include_css: true,
            dark_mode: false,
        }
    }

    /// Enable dark mode styling.
    pub fn with_dark_mode(mut self) -> Self {
        self.dark_mode = true;
        self
    }

    /// Disable inline CSS (for external stylesheet).
    pub fn without_css(mut self) -> Self {
        self.include_css = false;
        self
    }

    fn css_styles(&self) -> &'static str {
        if self.dark_mode {
            include_str!("html_dark.css")
        } else {
            include_str!("html_light.css")
        }
    }
}

impl Default for HtmlFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportFormatter for HtmlFormatter {
    fn format(&self, report: &DiagnosticReport) -> ReportResult<String> {
        let mut output = String::new();

        // HTML header
        writeln!(output, "<!DOCTYPE html>").unwrap();
        writeln!(output, "<html lang=\"en\">").unwrap();
        writeln!(output, "<head>").unwrap();
        writeln!(output, "    <meta charset=\"UTF-8\">").unwrap();
        writeln!(output, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">").unwrap();
        writeln!(output, "    <title>{}</title>", html_escape(&report.metadata.title)).unwrap();

        if self.include_css {
            writeln!(output, "    <style>").unwrap();
            writeln!(output, "{}", self.css_styles()).unwrap();
            writeln!(output, "    </style>").unwrap();
        }

        writeln!(output, "</head>").unwrap();
        writeln!(output, "<body>").unwrap();
        writeln!(output, "<div class=\"container\">").unwrap();

        // Header
        writeln!(output, "<header>").unwrap();
        writeln!(output, "    <h1>{}</h1>", html_escape(&report.metadata.title)).unwrap();
        writeln!(output, "    <div class=\"metadata\">").unwrap();
        writeln!(output, "        <span class=\"tag\">Report ID: {}</span>", report.metadata.id).unwrap();
        writeln!(output, "        <span class=\"tag\">Generated: {}</span>",
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")).unwrap();
        writeln!(output, "        <span class=\"tag\">Version: {}</span>", report.metadata.version).unwrap();
        if let Some(ref hostname) = report.metadata.hostname {
            writeln!(output, "        <span class=\"tag\">Host: {}</span>", html_escape(hostname)).unwrap();
        }
        if let Some(ref os_info) = report.metadata.os_info {
            writeln!(output, "        <span class=\"tag\">OS: {}</span>", html_escape(os_info)).unwrap();
        }
        writeln!(output, "    </div>").unwrap();
        writeln!(output, "</header>").unwrap();

        // Health Assessment
        if let Some(ref health) = report.health {
            writeln!(output, "<section class=\"health-section\">").unwrap();
            writeln!(output, "    <h2>Health Assessment</h2>").unwrap();

            let status_class = match health.status.as_str() {
                "good" => "status-good",
                "warning" => "status-warning",
                "critical" => "status-critical",
                _ => "status-unknown",
            };

            writeln!(output, "    <div class=\"health-card {}\">", status_class).unwrap();
            writeln!(output, "        <div class=\"health-score\">{}</div>", health.score).unwrap();
            writeln!(output, "        <div class=\"health-status\">{}</div>", health.status.to_uppercase()).unwrap();
            writeln!(output, "    </div>").unwrap();

            if !health.issues.is_empty() {
                writeln!(output, "    <div class=\"issues\">").unwrap();
                writeln!(output, "        <h3>Issues</h3>").unwrap();
                writeln!(output, "        <ul>").unwrap();
                for issue in &health.issues {
                    writeln!(output, "            <li class=\"issue\">{}</li>", html_escape(issue)).unwrap();
                }
                writeln!(output, "        </ul>").unwrap();
                writeln!(output, "    </div>").unwrap();
            }

            if !health.recommendations.is_empty() {
                writeln!(output, "    <div class=\"recommendations\">").unwrap();
                writeln!(output, "        <h3>Recommendations</h3>").unwrap();
                writeln!(output, "        <ul>").unwrap();
                for rec in &health.recommendations {
                    writeln!(output, "            <li class=\"recommendation\">{}</li>", html_escape(rec)).unwrap();
                }
                writeln!(output, "        </ul>").unwrap();
                writeln!(output, "    </div>").unwrap();
            }

            writeln!(output, "</section>").unwrap();
        }

        // Network Interfaces
        if !report.interfaces.is_empty() {
            writeln!(output, "<section class=\"interfaces-section\">").unwrap();
            writeln!(output, "    <h2>Network Interfaces</h2>").unwrap();
            writeln!(output, "    <div class=\"cards\">").unwrap();

            for iface in &report.interfaces {
                let status_class = if iface.is_up { "status-up" } else { "status-down" };
                let default_badge = if iface.is_default { "<span class=\"badge default\">Default</span>" } else { "" };

                writeln!(output, "    <div class=\"card interface-card {}\">", status_class).unwrap();
                writeln!(output, "        <div class=\"card-header\">").unwrap();
                writeln!(output, "            <h3>{} {}</h3>", html_escape(&iface.name), default_badge).unwrap();
                writeln!(output, "            <span class=\"status-indicator\">{}</span>",
                    if iface.is_up { "UP" } else { "DOWN" }).unwrap();
                writeln!(output, "        </div>").unwrap();
                writeln!(output, "        <div class=\"card-body\">").unwrap();
                writeln!(output, "            <p><strong>Type:</strong> {}</p>", html_escape(&iface.interface_type)).unwrap();

                if let Some(ref mac) = iface.mac_address {
                    writeln!(output, "            <p><strong>MAC:</strong> <code>{}</code></p>", mac).unwrap();
                }

                if !iface.ipv4_addresses.is_empty() {
                    writeln!(output, "            <p><strong>IPv4:</strong></p>").unwrap();
                    writeln!(output, "            <ul class=\"ip-list\">").unwrap();
                    for ip in &iface.ipv4_addresses {
                        writeln!(output, "                <li><code>{}</code></li>", ip).unwrap();
                    }
                    writeln!(output, "            </ul>").unwrap();
                }

                if !iface.ipv6_addresses.is_empty() {
                    writeln!(output, "            <p><strong>IPv6:</strong></p>").unwrap();
                    writeln!(output, "            <ul class=\"ip-list\">").unwrap();
                    for ip in &iface.ipv6_addresses {
                        writeln!(output, "                <li><code>{}</code></li>", ip).unwrap();
                    }
                    writeln!(output, "            </ul>").unwrap();
                }

                writeln!(output, "        </div>").unwrap();
                writeln!(output, "    </div>").unwrap();
            }

            writeln!(output, "    </div>").unwrap();
            writeln!(output, "</section>").unwrap();
        }

        // DNS Results
        if !report.dns_results.is_empty() {
            writeln!(output, "<section class=\"dns-section\">").unwrap();
            writeln!(output, "    <h2>DNS Resolution</h2>").unwrap();
            writeln!(output, "    <table class=\"results-table\">").unwrap();
            writeln!(output, "        <thead>").unwrap();
            writeln!(output, "            <tr>").unwrap();
            writeln!(output, "                <th>Query</th>").unwrap();
            writeln!(output, "                <th>Status</th>").unwrap();
            writeln!(output, "                <th>Duration</th>").unwrap();
            writeln!(output, "                <th>Addresses</th>").unwrap();
            writeln!(output, "            </tr>").unwrap();
            writeln!(output, "        </thead>").unwrap();
            writeln!(output, "        <tbody>").unwrap();

            for dns in &report.dns_results {
                let status_class = if dns.success { "success" } else { "failure" };
                let status_icon = if dns.success { "✓" } else { "✗" };
                let addresses = if dns.addresses.is_empty() {
                    dns.error.as_deref().unwrap_or("-").to_string()
                } else {
                    dns.addresses.join(", ")
                };

                writeln!(output, "            <tr class=\"{}\">", status_class).unwrap();
                writeln!(output, "                <td><code>{}</code></td>", html_escape(&dns.query)).unwrap();
                writeln!(output, "                <td class=\"status-cell\">{}</td>", status_icon).unwrap();
                writeln!(output, "                <td>{:.1}ms</td>", dns.duration_ms).unwrap();
                writeln!(output, "                <td>{}</td>", html_escape(&addresses)).unwrap();
                writeln!(output, "            </tr>").unwrap();
            }

            writeln!(output, "        </tbody>").unwrap();
            writeln!(output, "    </table>").unwrap();
            writeln!(output, "</section>").unwrap();
        }

        // Ping Results
        if !report.ping_results.is_empty() {
            writeln!(output, "<section class=\"ping-section\">").unwrap();
            writeln!(output, "    <h2>Ping Results</h2>").unwrap();
            writeln!(output, "    <div class=\"cards\">").unwrap();

            for ping in &report.ping_results {
                let quality_class = match ping.quality.as_str() {
                    "excellent" | "good" => "quality-good",
                    "fair" | "acceptable" => "quality-fair",
                    _ => "quality-poor",
                };

                writeln!(output, "    <div class=\"card ping-card {}\">", quality_class).unwrap();
                writeln!(output, "        <div class=\"card-header\">").unwrap();
                writeln!(output, "            <h3>{}</h3>", html_escape(&ping.target)).unwrap();
                writeln!(output, "            <span class=\"quality-badge\">{}</span>", ping.quality.to_uppercase()).unwrap();
                writeln!(output, "        </div>").unwrap();
                writeln!(output, "        <div class=\"card-body\">").unwrap();
                writeln!(output, "            <div class=\"stats-grid\">").unwrap();
                writeln!(output, "                <div class=\"stat\">").unwrap();
                writeln!(output, "                    <span class=\"stat-value\">{}/{}</span>", ping.received, ping.transmitted).unwrap();
                writeln!(output, "                    <span class=\"stat-label\">Packets</span>").unwrap();
                writeln!(output, "                </div>").unwrap();
                writeln!(output, "                <div class=\"stat\">").unwrap();
                writeln!(output, "                    <span class=\"stat-value\">{:.1}%</span>", ping.loss_percent).unwrap();
                writeln!(output, "                    <span class=\"stat-label\">Loss</span>").unwrap();
                writeln!(output, "                </div>").unwrap();

                if let Some(min) = ping.min_rtt_ms {
                    writeln!(output, "                <div class=\"stat\">").unwrap();
                    writeln!(output, "                    <span class=\"stat-value\">{:.1}ms</span>", min).unwrap();
                    writeln!(output, "                    <span class=\"stat-label\">Min RTT</span>").unwrap();
                    writeln!(output, "                </div>").unwrap();
                }

                if let Some(avg) = ping.avg_rtt_ms {
                    writeln!(output, "                <div class=\"stat\">").unwrap();
                    writeln!(output, "                    <span class=\"stat-value\">{:.1}ms</span>", avg).unwrap();
                    writeln!(output, "                    <span class=\"stat-label\">Avg RTT</span>").unwrap();
                    writeln!(output, "                </div>").unwrap();
                }

                if let Some(max) = ping.max_rtt_ms {
                    writeln!(output, "                <div class=\"stat\">").unwrap();
                    writeln!(output, "                    <span class=\"stat-value\">{:.1}ms</span>", max).unwrap();
                    writeln!(output, "                    <span class=\"stat-label\">Max RTT</span>").unwrap();
                    writeln!(output, "                </div>").unwrap();
                }

                writeln!(output, "            </div>").unwrap();
                writeln!(output, "        </div>").unwrap();
                writeln!(output, "    </div>").unwrap();
            }

            writeln!(output, "    </div>").unwrap();
            writeln!(output, "</section>").unwrap();
        }

        // Traceroute Results
        if !report.traceroute_results.is_empty() {
            writeln!(output, "<section class=\"traceroute-section\">").unwrap();
            writeln!(output, "    <h2>Traceroute Results</h2>").unwrap();

            for traceroute in &report.traceroute_results {
                let status_class = if traceroute.reached { "reached" } else { "not-reached" };

                writeln!(output, "    <div class=\"traceroute-result {}\">", status_class).unwrap();
                writeln!(output, "        <h3>{}</h3>", html_escape(&traceroute.target)).unwrap();
                writeln!(output, "        <div class=\"traceroute-meta\">").unwrap();
                writeln!(output, "            <span>Protocol: {}</span>", traceroute.protocol).unwrap();
                writeln!(output, "            <span>Hops: {}</span>", traceroute.hop_count).unwrap();
                writeln!(output, "            <span>Duration: {:.0}ms</span>", traceroute.duration_ms).unwrap();
                writeln!(output, "            <span class=\"status\">{}</span>",
                    if traceroute.reached { "Reached" } else { "Not Reached" }).unwrap();
                writeln!(output, "        </div>").unwrap();

                writeln!(output, "        <table class=\"hops-table\">").unwrap();
                writeln!(output, "            <thead>").unwrap();
                writeln!(output, "                <tr>").unwrap();
                writeln!(output, "                    <th>Hop</th>").unwrap();
                writeln!(output, "                    <th>Address</th>").unwrap();
                writeln!(output, "                    <th>RTT</th>").unwrap();
                writeln!(output, "                </tr>").unwrap();
                writeln!(output, "            </thead>").unwrap();
                writeln!(output, "            <tbody>").unwrap();

                for hop in &traceroute.hops {
                    let addr = if hop.all_timeout {
                        "* * *".to_string()
                    } else {
                        match (&hop.hostname, &hop.address) {
                            (Some(name), Some(ip)) => format!("{} <code>{}</code>", html_escape(name), ip),
                            (None, Some(ip)) => format!("<code>{}</code>", ip),
                            _ => "*".to_string(),
                        }
                    };

                    let rtts: Vec<String> = hop.rtt_ms.iter()
                        .map(|rtt| match rtt {
                            Some(ms) => format!("{:.1}ms", ms),
                            None => "*".to_string(),
                        })
                        .collect();

                    let hop_class = if hop.all_timeout { "timeout" } else { "" };

                    writeln!(output, "                <tr class=\"{}\">", hop_class).unwrap();
                    writeln!(output, "                    <td>{}</td>", hop.hop).unwrap();
                    writeln!(output, "                    <td>{}</td>", addr).unwrap();
                    writeln!(output, "                    <td>{}</td>", rtts.join(" / ")).unwrap();
                    writeln!(output, "                </tr>").unwrap();
                }

                writeln!(output, "            </tbody>").unwrap();
                writeln!(output, "        </table>").unwrap();
                writeln!(output, "    </div>").unwrap();
            }

            writeln!(output, "</section>").unwrap();
        }

        // Footer
        writeln!(output, "<footer>").unwrap();
        writeln!(output, "    <p>Generated by <strong>netdiag</strong> v{}</p>", report.metadata.version).unwrap();
        writeln!(output, "</footer>").unwrap();

        // Close HTML
        writeln!(output, "</div>").unwrap();
        writeln!(output, "</body>").unwrap();
        writeln!(output, "</html>").unwrap();

        Ok(output)
    }

    fn extension(&self) -> &'static str {
        "html"
    }

    fn mime_type(&self) -> &'static str {
        "text/html"
    }
}

/// Escape HTML special characters.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
