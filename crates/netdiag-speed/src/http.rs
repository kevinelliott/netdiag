//! HTTP-based speed test implementation.

use crate::{
    BandwidthMeasurement, BandwidthSample, SpeedError, SpeedResult, SpeedTestConfig,
    SpeedTestProvider, SpeedTestResult, SpeedTestServer,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::StreamExt;
use rand::Rng;
use reqwest::Client;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Default download test URLs (large files from CDNs).
const DEFAULT_DOWNLOAD_URLS: &[&str] = &[
    "https://speed.cloudflare.com/__down?bytes=100000000",
    "https://proof.ovh.net/files/100Mb.dat",
];

/// Default upload test URL.
const DEFAULT_UPLOAD_URL: &str = "https://speed.cloudflare.com/__up";

/// HTTP-based speed test configuration.
#[derive(Debug, Clone)]
pub struct HttpSpeedConfig {
    /// Download test URLs.
    pub download_urls: Vec<String>,

    /// Upload test URL.
    pub upload_url: String,

    /// Chunk size for downloads (bytes).
    pub chunk_size: usize,

    /// Upload payload size (bytes).
    pub upload_size: usize,
}

impl Default for HttpSpeedConfig {
    fn default() -> Self {
        Self {
            download_urls: DEFAULT_DOWNLOAD_URLS.iter().map(|s| s.to_string()).collect(),
            upload_url: DEFAULT_UPLOAD_URL.to_string(),
            chunk_size: 1024 * 1024, // 1 MB chunks
            upload_size: 25 * 1024 * 1024, // 25 MB upload
        }
    }
}

/// HTTP-based speed test provider.
pub struct HttpSpeedTest {
    client: Client,
    config: HttpSpeedConfig,
}

impl HttpSpeedTest {
    /// Create a new HTTP speed test provider.
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config: HttpSpeedConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: HttpSpeedConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Test download speed from a single URL.
    async fn download_single(
        &self,
        url: &str,
        duration: Duration,
        bytes_counter: Arc<AtomicU64>,
        sample_tx: mpsc::Sender<BandwidthSample>,
    ) -> SpeedResult<()> {
        let start = Instant::now();
        let mut last_sample = Instant::now();
        let mut sample_bytes = 0u64;

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| SpeedError::ConnectionFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(SpeedError::ConnectionFailed(format!(
                "HTTP {}", response.status()
            )));
        }

        let mut stream = response.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            if start.elapsed() > duration {
                break;
            }

            match chunk_result {
                Ok(chunk) => {
                    let chunk_len = chunk.len() as u64;
                    bytes_counter.fetch_add(chunk_len, Ordering::Relaxed);
                    sample_bytes += chunk_len;

                    // Send sample every 500ms
                    if last_sample.elapsed() >= Duration::from_millis(500) {
                        let sample = BandwidthSample {
                            elapsed: start.elapsed(),
                            bytes: sample_bytes,
                            duration: last_sample.elapsed(),
                        };
                        let _ = sample_tx.send(sample).await;
                        sample_bytes = 0;
                        last_sample = Instant::now();
                    }
                }
                Err(e) => {
                    warn!("Chunk download error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Test upload speed.
    async fn upload_single(
        &self,
        url: &str,
        duration: Duration,
        payload_size: usize,
        bytes_counter: Arc<AtomicU64>,
        sample_tx: mpsc::Sender<BandwidthSample>,
    ) -> SpeedResult<()> {
        let start = Instant::now();
        let mut last_sample = Instant::now();
        let mut sample_bytes = 0u64;

        // Generate random payload
        let mut payload = vec![0u8; payload_size];
        rand::thread_rng().fill(&mut payload[..]);
        let payload = Bytes::from(payload);

        while start.elapsed() < duration {
            let chunk_start = Instant::now();

            let result = self
                .client
                .post(url)
                .body(payload.clone())
                .send()
                .await;

            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        let uploaded = payload.len() as u64;
                        bytes_counter.fetch_add(uploaded, Ordering::Relaxed);
                        sample_bytes += uploaded;

                        // Send sample every 500ms
                        if last_sample.elapsed() >= Duration::from_millis(500) {
                            let sample = BandwidthSample {
                                elapsed: start.elapsed(),
                                bytes: sample_bytes,
                                duration: last_sample.elapsed(),
                            };
                            let _ = sample_tx.send(sample).await;
                            sample_bytes = 0;
                            last_sample = Instant::now();
                        }
                    }
                }
                Err(e) => {
                    warn!("Upload error: {}", e);
                    // Continue trying
                    if chunk_start.elapsed() < Duration::from_millis(100) {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }

        Ok(())
    }

    /// Find the best server by latency.
    async fn find_best_server(&self) -> SpeedResult<SpeedTestServer> {
        let mut best_server: Option<SpeedTestServer> = None;
        let mut best_latency = Duration::MAX;

        for url in &self.config.download_urls {
            let start = Instant::now();

            // Try a small HEAD request to measure latency
            let result = self
                .client
                .head(url)
                .timeout(Duration::from_secs(5))
                .send()
                .await;

            if let Ok(response) = result {
                if response.status().is_success() || response.status().is_redirection() {
                    let latency = start.elapsed();
                    if latency < best_latency {
                        best_latency = latency;

                        // Extract server info from URL
                        let server_name = url
                            .split("//")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .unwrap_or("Unknown");

                        best_server = Some(SpeedTestServer {
                            name: server_name.to_string(),
                            url: url.clone(),
                            location: None,
                            country: None,
                            sponsor: None,
                            distance_km: None,
                            latency: Some(latency),
                        });
                    }
                }
            }
        }

        best_server.ok_or(SpeedError::ServerNotFound)
    }
}

impl Default for HttpSpeedTest {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SpeedTestProvider for HttpSpeedTest {
    fn name(&self) -> &str {
        "HTTP"
    }

    async fn is_available(&self) -> bool {
        // Try to reach at least one server
        for url in &self.config.download_urls {
            let result = self
                .client
                .head(url)
                .timeout(Duration::from_secs(5))
                .send()
                .await;

            if result.is_ok() {
                return true;
            }
        }
        false
    }

    async fn test_download(&self, config: &SpeedTestConfig) -> SpeedResult<BandwidthMeasurement> {
        info!("Starting HTTP download test");

        let server = self.find_best_server().await?;
        debug!("Using server: {}", server.name);

        // Warmup
        if config.warmup > Duration::ZERO {
            debug!("Warmup for {:?}", config.warmup);
            let warmup_bytes = Arc::new(AtomicU64::new(0));
            let (tx, _rx) = mpsc::channel(100);

            let _ = self
                .download_single(&server.url, config.warmup, warmup_bytes, tx)
                .await;
        }

        let bytes_counter = Arc::new(AtomicU64::new(0));
        let (sample_tx, mut sample_rx) = mpsc::channel(1000);
        let mut samples = Vec::new();

        let start = Instant::now();

        // Run downloads in parallel connections
        let mut handles = Vec::new();
        for _ in 0..config.connections {
            let url = server.url.clone();
            let duration = config.duration;
            let counter = bytes_counter.clone();
            let tx = sample_tx.clone();
            let client = self.client.clone();

            let handle = tokio::spawn(async move {
                let test = HttpSpeedTest {
                    client,
                    config: HttpSpeedConfig::default(),
                };
                let _ = test.download_single(&url, duration, counter, tx).await;
            });
            handles.push(handle);
        }

        // Drop our sender so the channel closes when all tasks complete
        drop(sample_tx);

        // Collect samples while waiting
        while let Some(sample) = sample_rx.recv().await {
            samples.push(sample);
        }

        // Wait for all downloads to complete
        for handle in handles {
            let _ = handle.await;
        }

        let duration = start.elapsed();
        let total_bytes = bytes_counter.load(Ordering::Relaxed);

        info!(
            "Download complete: {} bytes in {:?}",
            total_bytes, duration
        );

        Ok(BandwidthMeasurement {
            bytes: total_bytes,
            duration,
            connections: config.connections,
            samples,
        })
    }

    async fn test_upload(&self, config: &SpeedTestConfig) -> SpeedResult<BandwidthMeasurement> {
        info!("Starting HTTP upload test");

        let bytes_counter = Arc::new(AtomicU64::new(0));
        let (sample_tx, mut sample_rx) = mpsc::channel(1000);
        let mut samples = Vec::new();

        let start = Instant::now();

        // Run uploads in parallel connections
        let mut handles = Vec::new();
        let upload_size = self.config.upload_size / config.connections;

        for _ in 0..config.connections {
            let url = self.config.upload_url.clone();
            let duration = config.duration;
            let counter = bytes_counter.clone();
            let tx = sample_tx.clone();
            let client = self.client.clone();

            let handle = tokio::spawn(async move {
                let test = HttpSpeedTest {
                    client,
                    config: HttpSpeedConfig::default(),
                };
                let _ = test
                    .upload_single(&url, duration, upload_size, counter, tx)
                    .await;
            });
            handles.push(handle);
        }

        // Drop our sender so the channel closes when all tasks complete
        drop(sample_tx);

        // Collect samples while waiting
        while let Some(sample) = sample_rx.recv().await {
            samples.push(sample);
        }

        // Wait for all uploads to complete
        for handle in handles {
            let _ = handle.await;
        }

        let duration = start.elapsed();
        let total_bytes = bytes_counter.load(Ordering::Relaxed);

        info!("Upload complete: {} bytes in {:?}", total_bytes, duration);

        Ok(BandwidthMeasurement {
            bytes: total_bytes,
            duration,
            connections: config.connections,
            samples,
        })
    }

    async fn run_full_test(&self, config: &SpeedTestConfig) -> SpeedResult<SpeedTestResult> {
        let start = Instant::now();
        let server = self.find_best_server().await?;

        let mut result = SpeedTestResult::new(server, self.name());
        result.latency = self.measure_latency().await.ok();

        if config.test_download {
            result.download = Some(self.test_download(config).await?);
        }

        if config.test_upload {
            result.upload = Some(self.test_upload(config).await?);
        }

        result.test_duration = start.elapsed();

        Ok(result)
    }

    async fn measure_latency(&self) -> SpeedResult<Duration> {
        let server = self.find_best_server().await?;

        let mut latencies = Vec::new();

        for _ in 0..5 {
            let start = Instant::now();
            let result = self
                .client
                .head(&server.url)
                .timeout(Duration::from_secs(5))
                .send()
                .await;

            if result.is_ok() {
                latencies.push(start.elapsed());
            }
        }

        if latencies.is_empty() {
            return Err(SpeedError::ConnectionFailed("No successful latency measurements".into()));
        }

        // Return median latency
        latencies.sort();
        Ok(latencies[latencies.len() / 2])
    }
}
