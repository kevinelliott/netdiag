//! PDF format implementation.

use crate::error::{ReportError, ReportResult};
use crate::report::DiagnosticReport;
use crate::ReportFormatter;
use genpdf::{
    elements::{Break, Paragraph},
    fonts,
    style::{self, StyledString},
    Alignment, Document, Element, Size,
};

/// PDF report formatter.
pub struct PdfFormatter {
    /// Page size.
    page_size: Size,
}

impl PdfFormatter {
    /// Create a new PDF formatter with default settings.
    pub fn new() -> Self {
        Self {
            page_size: Size::new(210, 297), // A4 in mm
        }
    }

    /// Set page size to US Letter.
    pub fn with_letter_size(mut self) -> Self {
        self.page_size = Size::new(216, 279); // Letter in mm
        self
    }

    /// Generate PDF bytes from a report.
    pub fn generate_pdf(&self, report: &DiagnosticReport) -> ReportResult<Vec<u8>> {
        // Try to load fonts from various locations
        // Liberation fonts are the most cross-platform option
        let home_dir = std::env::var("HOME").unwrap_or_default();
        let user_fonts_path = format!("{}/Library/Fonts", home_dir);

        let font_family = fonts::from_files("./fonts", "LiberationSans", None)
            // User fonts on macOS (Homebrew installs here)
            .or_else(|_| fonts::from_files(&user_fonts_path, "LiberationSans", None))
            // Linux paths
            .or_else(|_| {
                fonts::from_files(
                    "/usr/share/fonts/truetype/liberation",
                    "LiberationSans",
                    None,
                )
            })
            .or_else(|_| {
                fonts::from_files("/usr/share/fonts/liberation-sans", "LiberationSans", None)
            })
            .or_else(|_| fonts::from_files("/usr/share/fonts/TTF", "LiberationSans", None))
            // Homebrew on macOS (alternative paths)
            .or_else(|_| {
                fonts::from_files(
                    "/opt/homebrew/share/fonts/liberation-sans",
                    "LiberationSans",
                    None,
                )
            })
            .or_else(|_| {
                fonts::from_files(
                    "/usr/local/share/fonts/liberation-sans",
                    "LiberationSans",
                    None,
                )
            })
            // DejaVu fonts as fallback (common on many systems)
            .or_else(|_| fonts::from_files("/usr/share/fonts/truetype/dejavu", "DejaVuSans", None))
            .or_else(|_| fonts::from_files("/usr/share/fonts/dejavu", "DejaVuSans", None))
            .map_err(|_| {
                ReportError::Formatting(
                    "Failed to load fonts for PDF generation. \
                 Please install Liberation fonts:\n  \
                 - macOS: brew install font-liberation\n  \
                 - Ubuntu/Debian: apt install fonts-liberation\n  \
                 - Fedora: dnf install liberation-fonts\n  \
                 - Or place LiberationSans-*.ttf files in ./fonts/"
                        .to_string(),
                )
            })?;

        let mut doc = Document::new(font_family);
        doc.set_title(&report.metadata.title);
        doc.set_paper_size(self.page_size);

        // Title
        doc.push(title_element(&report.metadata.title));
        doc.push(Break::new(1.0));

        // Metadata section
        let meta_text = format!(
            "Report ID: {}  |  Generated: {}  |  Version: {}",
            report.metadata.id,
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.metadata.version
        );
        doc.push(centered_text(&meta_text, 9));

        if let Some(ref hostname) = report.metadata.hostname {
            doc.push(centered_text(&format!("Host: {}", hostname), 9));
        }

        if let Some(ref os_info) = report.metadata.os_info {
            doc.push(centered_text(&format!("OS: {}", os_info), 9));
        }

        doc.push(Break::new(2.0));

        // Health Assessment
        if let Some(ref health) = report.health {
            doc.push(section_header("Health Assessment"));

            let status_text = format!(
                "Status: {} (Score: {})",
                health.status.to_uppercase(),
                health.score
            );
            doc.push(body_text(&status_text));

            if !health.issues.is_empty() {
                doc.push(Break::new(1.0));
                doc.push(subsection_header("Issues"));
                for issue in &health.issues {
                    doc.push(bullet_item(issue));
                }
            }

            if !health.recommendations.is_empty() {
                doc.push(Break::new(1.0));
                doc.push(subsection_header("Recommendations"));
                for rec in &health.recommendations {
                    doc.push(bullet_item(rec));
                }
            }

            doc.push(Break::new(2.0));
        }

        // Network Interfaces
        if !report.interfaces.is_empty() {
            doc.push(section_header("Network Interfaces"));

            for iface in &report.interfaces {
                let status = if iface.is_up { "UP" } else { "DOWN" };
                let default = if iface.is_default { " (Default)" } else { "" };
                let header = format!("{}{} - {}", iface.name, default, status);

                doc.push(bold_text(&header, 11));

                let mut details = vec![format!("Type: {}", iface.interface_type)];
                if !iface.ipv4_addresses.is_empty() {
                    details.push(format!("IPv4: {}", iface.ipv4_addresses.join(", ")));
                }
                if !iface.ipv6_addresses.is_empty() {
                    details.push(format!("IPv6: {}", iface.ipv6_addresses.first().unwrap()));
                }
                if let Some(ref mac) = iface.mac_address {
                    details.push(format!("MAC: {}", mac));
                }
                doc.push(body_text(&format!("  {}", details.join("  |  "))));
                doc.push(Break::new(0.5));
            }

            doc.push(Break::new(1.0));
        }

        // DNS Results
        if !report.dns_results.is_empty() {
            doc.push(section_header("DNS Resolution"));

            for dns in &report.dns_results {
                let status = if dns.success { "OK" } else { "FAIL" };
                let addresses = if dns.addresses.is_empty() {
                    dns.error.as_deref().unwrap_or("-").to_string()
                } else {
                    dns.addresses.join(", ")
                };
                let line = format!(
                    "[{}] {} -> {} ({:.1}ms)",
                    status, dns.query, addresses, dns.duration_ms
                );
                doc.push(body_text(&line));
            }

            doc.push(Break::new(2.0));
        }

        // Ping Results
        if !report.ping_results.is_empty() {
            doc.push(section_header("Ping Results"));

            for ping in &report.ping_results {
                doc.push(bold_text(&ping.target, 11));

                let stats = format!(
                    "  Packets: {}/{} ({:.1}% loss)  |  Quality: {}",
                    ping.received, ping.transmitted, ping.loss_percent, ping.quality
                );
                doc.push(body_text(&stats));

                if let (Some(min), Some(avg), Some(max)) =
                    (ping.min_rtt_ms, ping.avg_rtt_ms, ping.max_rtt_ms)
                {
                    let rtt = format!("  RTT: {:.1}/{:.1}/{:.1} ms (min/avg/max)", min, avg, max);
                    doc.push(body_text(&rtt));
                }

                doc.push(Break::new(0.5));
            }

            doc.push(Break::new(1.0));
        }

        // Traceroute Results
        if !report.traceroute_results.is_empty() {
            doc.push(section_header("Traceroute Results"));

            for traceroute in &report.traceroute_results {
                doc.push(bold_text(&traceroute.target, 11));

                let status = if traceroute.reached {
                    "Reached"
                } else {
                    "Not Reached"
                };
                let summary = format!(
                    "  Protocol: {} | Hops: {} | {} | Duration: {:.0}ms",
                    traceroute.protocol, traceroute.hop_count, status, traceroute.duration_ms
                );
                doc.push(body_text(&summary));
                doc.push(Break::new(0.5));

                for hop in &traceroute.hops {
                    let addr = if hop.all_timeout {
                        "* * *".to_string()
                    } else {
                        match (&hop.hostname, &hop.address) {
                            (Some(name), Some(ip)) => format!("{} ({})", name, ip),
                            (None, Some(ip)) => ip.clone(),
                            _ => "*".to_string(),
                        }
                    };

                    let rtts: Vec<String> = hop
                        .rtt_ms
                        .iter()
                        .map(|rtt| match rtt {
                            Some(ms) => format!("{:.1}ms", ms),
                            None => "*".to_string(),
                        })
                        .collect();

                    let hop_line = format!("  {:2}. {}  [{}]", hop.hop, addr, rtts.join(" / "));
                    doc.push(body_text(&hop_line));
                }

                doc.push(Break::new(1.0));
            }
        }

        // Footer
        doc.push(Break::new(2.0));
        doc.push(centered_text(
            &format!("Generated by netdiag v{}", report.metadata.version),
            9,
        ));

        // Render to bytes
        let mut buffer = Vec::new();
        doc.render(&mut buffer)
            .map_err(|e| ReportError::Formatting(format!("PDF render error: {}", e)))?;

        Ok(buffer)
    }
}

impl Default for PdfFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportFormatter for PdfFormatter {
    fn format(&self, _report: &DiagnosticReport) -> ReportResult<String> {
        // For the trait implementation, we return a placeholder message
        // The actual PDF bytes should be obtained via generate_pdf()
        Err(ReportError::Formatting(
            "PDF format produces binary output. Use generate_pdf() method instead.".to_string(),
        ))
    }

    fn extension(&self) -> &'static str {
        "pdf"
    }

    fn mime_type(&self) -> &'static str {
        "application/pdf"
    }
}

// Helper functions for creating styled elements

fn title_element(text: &str) -> impl Element {
    Paragraph::new(StyledString::new(
        text,
        style::Style::new().bold().with_font_size(20),
    ))
    .aligned(Alignment::Center)
}

fn section_header(text: &str) -> impl Element {
    Paragraph::new(StyledString::new(
        text,
        style::Style::new().bold().with_font_size(14),
    ))
}

fn subsection_header(text: &str) -> impl Element {
    Paragraph::new(StyledString::new(
        text,
        style::Style::new().bold().with_font_size(11),
    ))
}

fn body_text(text: &str) -> impl Element {
    Paragraph::new(StyledString::new(
        text,
        style::Style::new().with_font_size(10),
    ))
}

fn bold_text(text: &str, size: u8) -> impl Element {
    Paragraph::new(StyledString::new(
        text,
        style::Style::new().bold().with_font_size(size),
    ))
}

fn centered_text(text: &str, size: u8) -> impl Element {
    Paragraph::new(StyledString::new(
        text,
        style::Style::new().with_font_size(size),
    ))
    .aligned(Alignment::Center)
}

fn bullet_item(text: &str) -> impl Element {
    Paragraph::new(StyledString::new(
        format!("  - {}", text),
        style::Style::new().with_font_size(10),
    ))
}
