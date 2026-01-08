//! # netdiag-speed
//!
//! Speed testing module for netdiag.
//!
//! Provides bandwidth testing capabilities using multiple providers:
//! - HTTP-based speed tests (download/upload)
//! - iPerf3 integration
//! - Custom server support

#![warn(missing_docs)]
#![warn(clippy::all)]

mod error;
mod http;
mod iperf;
mod result;

pub use error::{SpeedError, SpeedResult};
pub use http::{HttpSpeedTest, HttpSpeedConfig};
pub use iperf::{IperfClient, IperfConfig};
pub use result::{SpeedTestResult, BandwidthMeasurement, BandwidthSample, SpeedTestServer};

use async_trait::async_trait;
use std::time::Duration;

/// Speed test provider trait.
#[async_trait]
pub trait SpeedTestProvider: Send + Sync {
    /// Get the provider name.
    fn name(&self) -> &str;

    /// Check if the provider is available.
    async fn is_available(&self) -> bool;

    /// Run a download speed test.
    async fn test_download(&self, config: &SpeedTestConfig) -> SpeedResult<BandwidthMeasurement>;

    /// Run an upload speed test.
    async fn test_upload(&self, config: &SpeedTestConfig) -> SpeedResult<BandwidthMeasurement>;

    /// Run a full speed test (download + upload).
    async fn run_full_test(&self, config: &SpeedTestConfig) -> SpeedResult<SpeedTestResult>;

    /// Get latency to the test server.
    async fn measure_latency(&self) -> SpeedResult<Duration>;
}

/// Speed test configuration.
#[derive(Debug, Clone)]
pub struct SpeedTestConfig {
    /// Test duration for each phase
    pub duration: Duration,
    /// Number of parallel connections
    pub connections: usize,
    /// Server URL or address
    pub server: Option<String>,
    /// Warmup duration before measurement
    pub warmup: Duration,
    /// Whether to test download
    pub test_download: bool,
    /// Whether to test upload
    pub test_upload: bool,
}

impl Default for SpeedTestConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_secs(10),
            connections: 4,
            server: None,
            warmup: Duration::from_secs(2),
            test_download: true,
            test_upload: true,
        }
    }
}

/// Combined speed tester that can use multiple providers.
pub struct SpeedTester {
    providers: Vec<Box<dyn SpeedTestProvider>>,
}

impl SpeedTester {
    /// Create a new speed tester with default providers.
    pub fn new() -> Self {
        Self {
            providers: vec![
                Box::new(HttpSpeedTest::new()),
            ],
        }
    }

    /// Create a speed tester with iPerf support.
    pub fn with_iperf(server: &str) -> Self {
        Self {
            providers: vec![
                Box::new(IperfClient::new(server)),
            ],
        }
    }

    /// Add a provider.
    pub fn add_provider(&mut self, provider: Box<dyn SpeedTestProvider>) {
        self.providers.push(provider);
    }

    /// Get available providers.
    pub async fn available_providers(&self) -> Vec<&str> {
        let mut available = Vec::new();
        for provider in &self.providers {
            if provider.is_available().await {
                available.push(provider.name());
            }
        }
        available
    }

    /// Run speed test with first available provider.
    pub async fn run_test(&self, config: &SpeedTestConfig) -> SpeedResult<SpeedTestResult> {
        for provider in &self.providers {
            if provider.is_available().await {
                return provider.run_full_test(config).await;
            }
        }
        Err(SpeedError::NoProvider)
    }

    /// Run speed test with specific provider.
    pub async fn run_with_provider(
        &self,
        provider_name: &str,
        config: &SpeedTestConfig,
    ) -> SpeedResult<SpeedTestResult> {
        for provider in &self.providers {
            if provider.name() == provider_name {
                return provider.run_full_test(config).await;
            }
        }
        Err(SpeedError::ProviderNotFound(provider_name.to_string()))
    }
}

impl Default for SpeedTester {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick speed test with default settings.
pub async fn quick_speed_test() -> SpeedResult<SpeedTestResult> {
    let tester = SpeedTester::new();
    let config = SpeedTestConfig {
        duration: Duration::from_secs(5),
        connections: 2,
        ..Default::default()
    };
    tester.run_test(&config).await
}
