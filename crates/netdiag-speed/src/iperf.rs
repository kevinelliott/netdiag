//! iPerf3 speed test implementation.

#![allow(dead_code)]

use crate::{
    BandwidthMeasurement, BandwidthSample, SpeedError, SpeedResult, SpeedTestConfig,
    SpeedTestProvider, SpeedTestResult, SpeedTestServer,
};
use async_trait::async_trait;
use serde::Deserialize;
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, info};

/// iPerf3 configuration.
#[derive(Debug, Clone)]
pub struct IperfConfig {
    /// iPerf3 binary path.
    pub binary: String,

    /// Default port.
    pub port: u16,

    /// Enable JSON output.
    pub json_output: bool,

    /// Reverse mode (server sends, client receives).
    pub reverse: bool,

    /// UDP mode instead of TCP.
    pub udp: bool,

    /// Bandwidth limit (bits/sec, 0 = unlimited).
    pub bandwidth: u64,

    /// Buffer length.
    pub buffer_length: Option<usize>,
}

impl Default for IperfConfig {
    fn default() -> Self {
        Self {
            binary: "iperf3".to_string(),
            port: 5201,
            json_output: true,
            reverse: false,
            udp: false,
            bandwidth: 0,
            buffer_length: None,
        }
    }
}

/// iPerf3 JSON output structures.
#[derive(Debug, Deserialize)]
struct IperfOutput {
    start: Option<IperfStart>,
    intervals: Option<Vec<IperfInterval>>,
    end: Option<IperfEnd>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct IperfStart {
    connected: Option<Vec<IperfConnection>>,
    timestamp: Option<IperfTimestamp>,
}

#[derive(Debug, Deserialize)]
struct IperfConnection {
    socket: Option<i32>,
    local_host: Option<String>,
    local_port: Option<u16>,
    remote_host: Option<String>,
    remote_port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct IperfTimestamp {
    time: Option<String>,
    timesecs: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct IperfInterval {
    streams: Option<Vec<IperfStream>>,
    sum: Option<IperfSum>,
}

#[derive(Debug, Deserialize)]
struct IperfStream {
    socket: Option<i32>,
    start: Option<f64>,
    end: Option<f64>,
    seconds: Option<f64>,
    bytes: Option<u64>,
    bits_per_second: Option<f64>,
    retransmits: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct IperfSum {
    start: Option<f64>,
    end: Option<f64>,
    seconds: Option<f64>,
    bytes: Option<u64>,
    bits_per_second: Option<f64>,
    retransmits: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct IperfEnd {
    streams: Option<Vec<IperfEndStream>>,
    sum_sent: Option<IperfSumSent>,
    sum_received: Option<IperfSumReceived>,
    cpu_utilization_percent: Option<IperfCpu>,
}

#[derive(Debug, Deserialize)]
struct IperfEndStream {
    sender: Option<IperfSender>,
    receiver: Option<IperfReceiver>,
}

#[derive(Debug, Deserialize)]
struct IperfSender {
    socket: Option<i32>,
    start: Option<f64>,
    end: Option<f64>,
    seconds: Option<f64>,
    bytes: Option<u64>,
    bits_per_second: Option<f64>,
    retransmits: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct IperfReceiver {
    socket: Option<i32>,
    start: Option<f64>,
    end: Option<f64>,
    seconds: Option<f64>,
    bytes: Option<u64>,
    bits_per_second: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct IperfSumSent {
    start: Option<f64>,
    end: Option<f64>,
    seconds: Option<f64>,
    bytes: Option<u64>,
    bits_per_second: Option<f64>,
    retransmits: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct IperfSumReceived {
    start: Option<f64>,
    end: Option<f64>,
    seconds: Option<f64>,
    bytes: Option<u64>,
    bits_per_second: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct IperfCpu {
    host_total: Option<f64>,
    host_user: Option<f64>,
    host_system: Option<f64>,
    remote_total: Option<f64>,
    remote_user: Option<f64>,
    remote_system: Option<f64>,
}

/// iPerf3 client for speed testing.
pub struct IperfClient {
    server: String,
    config: IperfConfig,
}

impl IperfClient {
    /// Create a new iPerf3 client.
    pub fn new(server: &str) -> Self {
        Self {
            server: server.to_string(),
            config: IperfConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(server: &str, config: IperfConfig) -> Self {
        Self {
            server: server.to_string(),
            config,
        }
    }

    /// Build the command arguments.
    fn build_args(&self, duration: Duration, connections: usize, reverse: bool) -> Vec<String> {
        let mut args = vec![
            "-c".to_string(),
            self.server.clone(),
            "-p".to_string(),
            self.config.port.to_string(),
            "-t".to_string(),
            duration.as_secs().to_string(),
            "-P".to_string(),
            connections.to_string(),
        ];

        if self.config.json_output {
            args.push("-J".to_string());
        }

        if reverse {
            args.push("-R".to_string());
        }

        if self.config.udp {
            args.push("-u".to_string());
        }

        if self.config.bandwidth > 0 {
            args.push("-b".to_string());
            args.push(self.config.bandwidth.to_string());
        }

        if let Some(len) = self.config.buffer_length {
            args.push("-l".to_string());
            args.push(len.to_string());
        }

        args
    }

    /// Run iPerf3 and parse output.
    async fn run_iperf(&self, args: &[String]) -> SpeedResult<IperfOutput> {
        debug!("Running iperf3 with args: {:?}", args);

        let mut cmd = Command::new(&self.config.binary);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd
            .spawn()
            .map_err(|e| SpeedError::Iperf(format!("Failed to spawn iperf3: {}", e)))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| SpeedError::Iperf("Failed to capture stdout".to_string()))?;

        let mut output = String::new();
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap_or(0) > 0 {
            output.push_str(&line);
            line.clear();
        }

        let status = child
            .wait()
            .await
            .map_err(|e| SpeedError::Iperf(format!("Failed to wait for iperf3: {}", e)))?;

        if !status.success() && output.is_empty() {
            return Err(SpeedError::Iperf(format!(
                "iperf3 failed with status: {}",
                status
            )));
        }

        // Parse JSON output
        let result: IperfOutput = serde_json::from_str(&output)
            .map_err(|e| SpeedError::Parse(format!("Failed to parse iperf3 output: {}", e)))?;

        // Check for error in output
        if let Some(error) = &result.error {
            return Err(SpeedError::Iperf(error.clone()));
        }

        Ok(result)
    }

    /// Convert iPerf output to bandwidth measurement.
    fn to_bandwidth_measurement(
        &self,
        output: &IperfOutput,
        connections: usize,
        is_receive: bool,
    ) -> BandwidthMeasurement {
        let mut samples = Vec::new();

        // Extract samples from intervals
        if let Some(intervals) = &output.intervals {
            for interval in intervals {
                if let Some(sum) = &interval.sum {
                    let sample = BandwidthSample {
                        elapsed: Duration::from_secs_f64(sum.end.unwrap_or(0.0)),
                        bytes: sum.bytes.unwrap_or(0),
                        duration: Duration::from_secs_f64(sum.seconds.unwrap_or(1.0)),
                    };
                    samples.push(sample);
                }
            }
        }

        // Get final totals
        let (bytes, duration) = if let Some(end) = &output.end {
            if is_receive {
                if let Some(sum) = &end.sum_received {
                    (
                        sum.bytes.unwrap_or(0),
                        Duration::from_secs_f64(sum.seconds.unwrap_or(1.0)),
                    )
                } else {
                    (0, Duration::from_secs(1))
                }
            } else {
                if let Some(sum) = &end.sum_sent {
                    (
                        sum.bytes.unwrap_or(0),
                        Duration::from_secs_f64(sum.seconds.unwrap_or(1.0)),
                    )
                } else {
                    (0, Duration::from_secs(1))
                }
            }
        } else {
            // Calculate from samples
            let total_bytes: u64 = samples.iter().map(|s| s.bytes).sum();
            let total_duration = samples.iter().map(|s| s.duration).sum();
            (total_bytes, total_duration)
        };

        BandwidthMeasurement {
            bytes,
            duration,
            connections,
            samples,
        }
    }
}

#[async_trait]
impl SpeedTestProvider for IperfClient {
    fn name(&self) -> &str {
        "iPerf3"
    }

    async fn is_available(&self) -> bool {
        // Check if iperf3 binary exists
        let result = Command::new(&self.config.binary)
            .arg("--version")
            .output()
            .await;

        if result.is_err() {
            debug!("iperf3 binary not found");
            return false;
        }

        // Try to connect to server
        let args = self.build_args(Duration::from_secs(1), 1, false);
        let mut test_args = args.clone();
        // Just test connection, no actual test
        test_args.push("--connect-timeout".to_string());
        test_args.push("3000".to_string());

        // For availability check, we just verify the binary exists
        // Full server check would require actual connection
        true
    }

    async fn test_download(&self, config: &SpeedTestConfig) -> SpeedResult<BandwidthMeasurement> {
        info!("Starting iPerf3 download test (reverse mode)");

        let args = self.build_args(config.duration, config.connections, true);
        let output = self.run_iperf(&args).await?;

        Ok(self.to_bandwidth_measurement(&output, config.connections, true))
    }

    async fn test_upload(&self, config: &SpeedTestConfig) -> SpeedResult<BandwidthMeasurement> {
        info!("Starting iPerf3 upload test");

        let args = self.build_args(config.duration, config.connections, false);
        let output = self.run_iperf(&args).await?;

        Ok(self.to_bandwidth_measurement(&output, config.connections, false))
    }

    async fn run_full_test(&self, config: &SpeedTestConfig) -> SpeedResult<SpeedTestResult> {
        let start = Instant::now();

        let server = SpeedTestServer::new("iPerf3 Server", &self.server);
        let mut result = SpeedTestResult::new(server, self.name());

        // Measure latency first
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
        // Parse host from server string
        let host = if self.server.contains(':') {
            self.server.split(':').next().unwrap_or(&self.server)
        } else {
            &self.server
        };

        // Use ping to measure latency
        let output = Command::new("ping")
            .args(["-c", "3", host])
            .output()
            .await
            .map_err(|e| SpeedError::Iperf(format!("Failed to ping server: {}", e)))?;

        if !output.status.success() {
            return Err(SpeedError::ConnectionFailed("Ping failed".into()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Parse average latency from ping output
        // Format varies by OS, but usually contains "avg" or "average"
        for line in stdout.lines() {
            if line.contains("avg") || line.contains("average") {
                // macOS/Linux format: "round-trip min/avg/max/stddev = x/y/z/w ms"
                if let Some(stats) = line.split('=').nth(1) {
                    let parts: Vec<&str> = stats.trim().split('/').collect();
                    if parts.len() >= 2 {
                        if let Ok(avg_ms) = parts[1].trim().parse::<f64>() {
                            return Ok(Duration::from_secs_f64(avg_ms / 1000.0));
                        }
                    }
                }
            }
        }

        Err(SpeedError::Parse("Could not parse ping output".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iperf_config_default() {
        let config = IperfConfig::default();
        assert_eq!(config.port, 5201);
        assert!(config.json_output);
        assert!(!config.reverse);
    }

    #[test]
    fn test_build_args() {
        let client = IperfClient::new("localhost");
        let args = client.build_args(Duration::from_secs(10), 4, false);

        assert!(args.contains(&"-c".to_string()));
        assert!(args.contains(&"localhost".to_string()));
        assert!(args.contains(&"-t".to_string()));
        assert!(args.contains(&"10".to_string()));
        assert!(args.contains(&"-P".to_string()));
        assert!(args.contains(&"4".to_string()));
        assert!(args.contains(&"-J".to_string()));
    }

    #[test]
    fn test_build_args_reverse() {
        let client = IperfClient::new("localhost");
        let args = client.build_args(Duration::from_secs(10), 1, true);

        assert!(args.contains(&"-R".to_string()));
    }
}
