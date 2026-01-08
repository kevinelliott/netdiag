//! JSON format implementation.

use crate::error::ReportResult;
use crate::report::DiagnosticReport;
use crate::ReportFormatter;

/// JSON report formatter.
pub struct JsonFormatter {
    /// Pretty print the JSON
    pub pretty: bool,
}

impl JsonFormatter {
    /// Create a new JSON formatter.
    pub fn new() -> Self {
        Self { pretty: true }
    }

    /// Create a compact JSON formatter.
    pub fn compact() -> Self {
        Self { pretty: false }
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl ReportFormatter for JsonFormatter {
    fn format(&self, report: &DiagnosticReport) -> ReportResult<String> {
        if self.pretty {
            Ok(serde_json::to_string_pretty(report)?)
        } else {
            Ok(serde_json::to_string(report)?)
        }
    }

    fn extension(&self) -> &'static str {
        "json"
    }

    fn mime_type(&self) -> &'static str {
        "application/json"
    }
}
