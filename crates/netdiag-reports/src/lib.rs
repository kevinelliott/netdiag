//! # netdiag-reports
//!
//! Report generation for netdiag network diagnostics.
//!
//! Supports generating reports in multiple formats including JSON, text,
//! Markdown, HTML, and PDF.

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod formats;
mod report;

pub use error::{ReportError, ReportResult};
pub use formats::{HtmlFormatter, JsonFormatter, MarkdownFormatter, PdfFormatter, TextFormatter};
pub use report::{
    DiagnosticReport, DnsSummary, HealthAssessment, InterfaceSummary, PingSummary,
    ReportBuilder, ReportFormat, ReportMetadata, TracerouteHopSummary, TracerouteSummary,
};

/// Generate a report from diagnostic results.
pub trait ReportFormatter: Send + Sync {
    /// Format the report.
    fn format(&self, report: &DiagnosticReport) -> ReportResult<String>;

    /// Get the file extension for this format.
    fn extension(&self) -> &'static str;

    /// Get the MIME type for this format.
    fn mime_type(&self) -> &'static str;
}
