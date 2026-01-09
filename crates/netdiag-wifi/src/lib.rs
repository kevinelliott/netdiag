//! # netdiag-wifi
//!
//! WiFi analysis and diagnostics module for netdiag.
//!
//! Provides comprehensive WiFi analysis capabilities:
//! - Signal quality assessment
//! - Channel optimization recommendations
//! - Interference detection
//! - Security analysis
//! - Site survey capabilities
//! - Enterprise network support

#![warn(missing_docs)]
#![warn(clippy::all)]

mod analysis;
mod channel;
mod error;
mod interference;
mod quality;
mod security;
mod survey;

pub use analysis::{WifiAnalysis, WifiAnalyzer, WifiEnvironment};
pub use channel::{ChannelAnalysis, ChannelRecommendation};
pub use error::{WifiError, WifiResult};
pub use interference::{InterferenceAnalysis, InterferenceSource};
pub use quality::{QualityMetrics, SignalQuality};
pub use security::{SecurityAnalysis, SecurityIssue, SecurityRating};
pub use survey::{SiteSurvey, SurveyPoint, SurveyResult};

use netdiag_platform::WifiProvider;
use std::sync::Arc;

/// WiFi diagnostics engine.
pub struct WifiDiagnostics {
    provider: Arc<dyn WifiProvider>,
}

impl WifiDiagnostics {
    /// Create a new WiFi diagnostics engine.
    pub fn new(provider: Arc<dyn WifiProvider>) -> Self {
        Self { provider }
    }

    /// Get the WiFi provider.
    pub fn provider(&self) -> &dyn WifiProvider {
        self.provider.as_ref()
    }

    /// Check if WiFi is available.
    pub fn is_available(&self) -> bool {
        self.provider.is_available()
    }

    /// Create a WiFi analyzer for deep analysis.
    pub fn analyzer(&self) -> WifiAnalyzer {
        WifiAnalyzer::new(self.provider.clone())
    }

    /// Create a site survey tool.
    pub fn site_survey(&self) -> SiteSurvey {
        SiteSurvey::new(self.provider.clone())
    }

    /// Run a quick WiFi health check.
    pub async fn quick_check(&self, interface: &str) -> WifiResult<WifiAnalysis> {
        self.analyzer().analyze(interface).await
    }
}
