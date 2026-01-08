//! Continuous network monitoring.

use crate::config::{AlertConfig, MonitorTarget, MonitoringConfig};
use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

/// Network health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// Everything is working normally.
    Healthy,
    /// Some issues detected but connectivity exists.
    Degraded,
    /// Connectivity is failing.
    Unhealthy,
    /// Status unknown (not yet tested).
    Unknown,
}

/// Result of a single monitoring check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorResult {
    /// Target that was checked.
    pub target: String,
    /// Whether the check succeeded.
    pub success: bool,
    /// Latency in milliseconds.
    pub latency_ms: Option<f64>,
    /// Packet loss percentage.
    pub packet_loss: Option<f64>,
    /// Timestamp of the check.
    pub timestamp: DateTime<Utc>,
    /// Error message if check failed.
    pub error: Option<String>,
}

/// Aggregated monitoring data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringData {
    /// Overall health status.
    pub status: HealthStatus,
    /// Gateway status.
    pub gateway: Option<TargetStatus>,
    /// DNS status.
    pub dns: Option<TargetStatus>,
    /// Internet status.
    pub internet: Option<TargetStatus>,
    /// Custom targets.
    pub custom_targets: HashMap<String, TargetStatus>,
    /// WiFi signal strength (dBm).
    pub wifi_signal: Option<i32>,
    /// Last update timestamp.
    pub last_update: DateTime<Utc>,
}

impl Default for MonitoringData {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            gateway: None,
            dns: None,
            internet: None,
            custom_targets: HashMap::new(),
            wifi_signal: None,
            last_update: Utc::now(),
        }
    }
}

/// Status of a monitored target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetStatus {
    /// Target address/name.
    pub target: String,
    /// Current status.
    pub status: HealthStatus,
    /// Average latency over recent checks.
    pub avg_latency_ms: f64,
    /// Packet loss over recent checks.
    pub packet_loss: f64,
    /// Number of consecutive failures.
    pub consecutive_failures: u32,
    /// Last successful check.
    pub last_success: Option<DateTime<Utc>>,
}

/// Alert event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert severity.
    pub severity: AlertSeverity,
    /// Alert message.
    pub message: String,
    /// Target that triggered the alert.
    pub target: Option<String>,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    /// Informational.
    Info,
    /// Warning.
    Warning,
    /// Critical.
    Critical,
}

/// Network monitor that runs continuous checks.
pub struct NetworkMonitor {
    config: MonitoringConfig,
    alert_config: AlertConfig,
    data: Arc<RwLock<MonitoringData>>,
    running: Arc<RwLock<bool>>,
    alert_tx: Option<mpsc::Sender<Alert>>,
}

impl NetworkMonitor {
    /// Creates a new network monitor.
    pub fn new(
        config: MonitoringConfig,
        alert_config: AlertConfig,
        alert_tx: Option<mpsc::Sender<Alert>>,
    ) -> Self {
        Self {
            config,
            alert_config,
            data: Arc::new(RwLock::new(MonitoringData::default())),
            running: Arc::new(RwLock::new(false)),
            alert_tx,
        }
    }

    /// Gets the current monitoring data.
    pub async fn get_data(&self) -> MonitoringData {
        self.data.read().await.clone()
    }

    /// Checks if monitoring is running.
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Pauses monitoring.
    pub async fn pause(&self) {
        *self.running.write().await = false;
    }

    /// Resumes monitoring.
    pub async fn resume(&self) {
        *self.running.write().await = true;
    }

    /// Starts the monitoring loop.
    pub async fn run(&self) -> Result<()> {
        if !self.config.enabled {
            tracing::info!("Monitoring disabled in configuration");
            return Ok(());
        }

        *self.running.write().await = true;
        tracing::info!("Starting network monitoring with interval {:?}", self.config.interval);

        let mut interval = interval(self.config.interval);

        loop {
            interval.tick().await;

            if !*self.running.read().await {
                tracing::debug!("Monitoring paused, waiting...");
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }

            self.run_checks().await;
        }
    }

    /// Runs all configured checks.
    async fn run_checks(&self) {
        let mut results = Vec::new();

        for target in &self.config.targets {
            let result = self.check_target(target).await;
            results.push(result);
        }

        // Update monitoring data
        self.update_data(&results).await;

        // Check for alerts
        self.check_alerts(&results).await;
    }

    /// Checks a single target.
    async fn check_target(&self, target: &MonitorTarget) -> MonitorResult {
        let target_str = match target {
            MonitorTarget::Gateway => "gateway".to_string(),
            MonitorTarget::Dns => "dns".to_string(),
            MonitorTarget::Internet => "internet".to_string(),
            MonitorTarget::Host(h) => h.clone(),
            MonitorTarget::Ip(ip) => ip.to_string(),
        };

        // Perform the actual check
        let (success, latency_ms, error) = match target {
            MonitorTarget::Gateway => self.check_gateway().await,
            MonitorTarget::Dns => self.check_dns().await,
            MonitorTarget::Internet => self.check_internet().await,
            MonitorTarget::Host(host) => self.check_host(host).await,
            MonitorTarget::Ip(ip) => self.check_ip(*ip).await,
        };

        MonitorResult {
            target: target_str,
            success,
            latency_ms,
            packet_loss: None, // Single check doesn't measure packet loss
            timestamp: Utc::now(),
            error,
        }
    }

    /// Checks gateway connectivity.
    async fn check_gateway(&self) -> (bool, Option<f64>, Option<String>) {
        // In a real implementation, this would:
        // 1. Get the default gateway from the network provider
        // 2. Ping it and measure latency
        // For now, simulate success
        tracing::debug!("Checking gateway connectivity");
        (true, Some(1.5), None)
    }

    /// Checks DNS resolution.
    async fn check_dns(&self) -> (bool, Option<f64>, Option<String>) {
        // In a real implementation, this would:
        // 1. Get DNS servers from the network provider
        // 2. Perform a DNS lookup and measure latency
        tracing::debug!("Checking DNS resolution");
        (true, Some(15.0), None)
    }

    /// Checks internet connectivity.
    async fn check_internet(&self) -> (bool, Option<f64>, Option<String>) {
        // In a real implementation, this would:
        // 1. Ping well-known hosts (8.8.8.8, 1.1.1.1)
        // 2. Or make an HTTP request to a known endpoint
        tracing::debug!("Checking internet connectivity");
        (true, Some(25.0), None)
    }

    /// Checks a specific host.
    async fn check_host(&self, host: &str) -> (bool, Option<f64>, Option<String>) {
        tracing::debug!("Checking host: {}", host);
        // Would perform DNS + ping
        (true, Some(50.0), None)
    }

    /// Checks a specific IP.
    async fn check_ip(&self, ip: IpAddr) -> (bool, Option<f64>, Option<String>) {
        tracing::debug!("Checking IP: {}", ip);
        // Would perform ping
        (true, Some(30.0), None)
    }

    /// Updates monitoring data based on check results.
    async fn update_data(&self, results: &[MonitorResult]) {
        let mut data = self.data.write().await;

        for result in results {
            let status = if result.success {
                if let Some(latency) = result.latency_ms {
                    if latency > self.alert_config.latency_threshold_ms as f64 {
                        HealthStatus::Degraded
                    } else {
                        HealthStatus::Healthy
                    }
                } else {
                    HealthStatus::Healthy
                }
            } else {
                HealthStatus::Unhealthy
            };

            let target_status = TargetStatus {
                target: result.target.clone(),
                status,
                avg_latency_ms: result.latency_ms.unwrap_or(0.0),
                packet_loss: result.packet_loss.unwrap_or(0.0),
                consecutive_failures: if result.success { 0 } else { 1 },
                last_success: if result.success { Some(result.timestamp) } else { None },
            };

            match result.target.as_str() {
                "gateway" => data.gateway = Some(target_status),
                "dns" => data.dns = Some(target_status),
                "internet" => data.internet = Some(target_status),
                _ => {
                    data.custom_targets.insert(result.target.clone(), target_status);
                }
            }
        }

        // Update overall status
        data.status = self.calculate_overall_status(&data);
        data.last_update = Utc::now();
    }

    /// Calculates overall health status.
    fn calculate_overall_status(&self, data: &MonitoringData) -> HealthStatus {
        let statuses = [
            data.gateway.as_ref().map(|s| s.status),
            data.dns.as_ref().map(|s| s.status),
            data.internet.as_ref().map(|s| s.status),
        ];

        let mut has_unhealthy = false;
        let mut has_degraded = false;
        let mut has_healthy = false;

        for status in statuses.iter().flatten() {
            match status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => has_healthy = true,
                HealthStatus::Unknown => {}
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else if has_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }

    /// Checks if any alerts should be triggered.
    async fn check_alerts(&self, results: &[MonitorResult]) {
        if !self.alert_config.enabled {
            return;
        }

        let Some(tx) = &self.alert_tx else {
            return;
        };

        for result in results {
            // Check for failures
            if !result.success {
                let alert = Alert {
                    severity: AlertSeverity::Critical,
                    message: format!(
                        "{} check failed: {}",
                        result.target,
                        result.error.as_deref().unwrap_or("Unknown error")
                    ),
                    target: Some(result.target.clone()),
                    timestamp: Utc::now(),
                };
                let _ = tx.send(alert).await;
            }

            // Check latency threshold
            if let Some(latency) = result.latency_ms {
                if latency > self.alert_config.latency_threshold_ms as f64 {
                    let alert = Alert {
                        severity: AlertSeverity::Warning,
                        message: format!(
                            "High latency to {}: {:.1}ms (threshold: {}ms)",
                            result.target, latency, self.alert_config.latency_threshold_ms
                        ),
                        target: Some(result.target.clone()),
                        timestamp: Utc::now(),
                    };
                    let _ = tx.send(alert).await;
                }
            }
        }
    }
}
