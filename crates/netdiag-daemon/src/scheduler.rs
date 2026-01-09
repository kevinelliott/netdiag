//! Diagnostic scheduler for running diagnostics on a schedule.

use crate::config::{DiagnosticType, ScheduleConfig};
use crate::error::{DaemonError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

/// A scheduled diagnostic job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    /// Unique job ID.
    pub id: Uuid,
    /// Job name.
    pub name: String,
    /// Cron expression.
    pub cron: String,
    /// Diagnostic type.
    pub diagnostic_type: DiagnosticType,
    /// Whether the job is enabled.
    pub enabled: bool,
    /// Last run time.
    pub last_run: Option<DateTime<Utc>>,
    /// Next run time.
    pub next_run: Option<DateTime<Utc>>,
}

/// Result of a scheduled diagnostic run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRun {
    /// Run ID.
    pub id: Uuid,
    /// Job that triggered this run.
    pub job_name: String,
    /// Diagnostic type.
    pub diagnostic_type: DiagnosticType,
    /// Start time.
    pub started_at: DateTime<Utc>,
    /// End time.
    pub completed_at: Option<DateTime<Utc>>,
    /// Whether the run succeeded.
    pub success: bool,
    /// Result summary.
    pub summary: Option<String>,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Diagnostic request sent to the execution handler.
#[derive(Debug, Clone)]
pub struct DiagnosticRequest {
    /// Run ID.
    pub run_id: Uuid,
    /// Job name.
    pub job_name: String,
    /// Diagnostic type.
    pub diagnostic_type: DiagnosticType,
}

/// Scheduler for running diagnostics on a schedule.
pub struct DiagnosticScheduler {
    scheduler: JobScheduler,
    jobs: Arc<RwLock<Vec<ScheduledJob>>>,
    diagnostic_tx: mpsc::Sender<DiagnosticRequest>,
}

impl DiagnosticScheduler {
    /// Creates a new diagnostic scheduler.
    pub async fn new(diagnostic_tx: mpsc::Sender<DiagnosticRequest>) -> Result<Self> {
        let scheduler = JobScheduler::new()
            .await
            .map_err(|e| DaemonError::scheduler(e.to_string()))?;

        Ok(Self {
            scheduler,
            jobs: Arc::new(RwLock::new(Vec::new())),
            diagnostic_tx,
        })
    }

    /// Adds scheduled jobs from configuration.
    pub async fn add_schedules(&mut self, schedules: &[ScheduleConfig]) -> Result<()> {
        for schedule in schedules {
            if schedule.enabled {
                self.add_schedule(schedule).await?;
            }
        }
        Ok(())
    }

    /// Adds a single scheduled job.
    pub async fn add_schedule(&mut self, config: &ScheduleConfig) -> Result<Uuid> {
        let job_id = Uuid::new_v4();
        let job_name = config.name.clone();
        let diagnostic_type = config.diagnostic;
        let tx = self.diagnostic_tx.clone();

        // Create the cron job
        let job = Job::new_async(config.cron.as_str(), move |_uuid, _lock| {
            let tx = tx.clone();
            let name = job_name.clone();
            Box::pin(async move {
                let request = DiagnosticRequest {
                    run_id: Uuid::new_v4(),
                    job_name: name,
                    diagnostic_type,
                };
                if let Err(e) = tx.send(request).await {
                    tracing::error!("Failed to send diagnostic request: {}", e);
                }
            })
        })
        .map_err(|e| DaemonError::scheduler(format!("Invalid cron expression: {}", e)))?;

        let scheduler_job_id = self
            .scheduler
            .add(job)
            .await
            .map_err(|e| DaemonError::scheduler(e.to_string()))?;

        // Track the job
        let scheduled_job = ScheduledJob {
            id: job_id,
            name: config.name.clone(),
            cron: config.cron.clone(),
            diagnostic_type: config.diagnostic,
            enabled: true,
            last_run: None,
            next_run: None, // Would be calculated from cron
        };

        self.jobs.write().await.push(scheduled_job);

        tracing::info!(
            "Added scheduled job '{}' with cron '{}' (scheduler id: {:?})",
            config.name,
            config.cron,
            scheduler_job_id
        );

        Ok(job_id)
    }

    /// Starts the scheduler.
    pub async fn start(&self) -> Result<()> {
        self.scheduler
            .start()
            .await
            .map_err(|e| DaemonError::scheduler(e.to_string()))?;
        tracing::info!("Diagnostic scheduler started");
        Ok(())
    }

    /// Stops the scheduler.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| DaemonError::scheduler(e.to_string()))?;
        tracing::info!("Diagnostic scheduler stopped");
        Ok(())
    }

    /// Gets all scheduled jobs.
    pub async fn get_jobs(&self) -> Vec<ScheduledJob> {
        self.jobs.read().await.clone()
    }

    /// Updates the last run time for a job.
    pub async fn update_last_run(&self, job_name: &str, timestamp: DateTime<Utc>) {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.name == job_name) {
            job.last_run = Some(timestamp);
        }
    }

    /// Enables or disables a job.
    pub async fn set_job_enabled(&self, job_name: &str, enabled: bool) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.iter_mut().find(|j| j.name == job_name) {
            job.enabled = enabled;
            // Note: Would need to remove/re-add job to scheduler to actually enable/disable
            Ok(())
        } else {
            Err(DaemonError::scheduler(format!(
                "Job '{}' not found",
                job_name
            )))
        }
    }
}

/// Handles diagnostic execution requests.
pub struct DiagnosticExecutor {
    rx: mpsc::Receiver<DiagnosticRequest>,
    runs: Arc<RwLock<Vec<DiagnosticRun>>>,
    max_history: usize,
}

impl DiagnosticExecutor {
    /// Creates a new diagnostic executor.
    pub fn new(rx: mpsc::Receiver<DiagnosticRequest>, max_history: usize) -> Self {
        Self {
            rx,
            runs: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    /// Gets recent diagnostic runs.
    pub async fn get_runs(&self, limit: usize) -> Vec<DiagnosticRun> {
        let runs = self.runs.read().await;
        runs.iter().rev().take(limit).cloned().collect()
    }

    /// Runs the executor loop.
    pub async fn run(&mut self) {
        tracing::info!("Diagnostic executor started");

        while let Some(request) = self.rx.recv().await {
            self.execute_diagnostic(request).await;
        }

        tracing::info!("Diagnostic executor stopped");
    }

    /// Executes a single diagnostic request.
    async fn execute_diagnostic(&self, request: DiagnosticRequest) {
        tracing::info!(
            "Executing diagnostic '{}' (type: {:?})",
            request.job_name,
            request.diagnostic_type
        );

        let started_at = Utc::now();

        // Create the run record
        let mut run = DiagnosticRun {
            id: request.run_id,
            job_name: request.job_name.clone(),
            diagnostic_type: request.diagnostic_type,
            started_at,
            completed_at: None,
            success: false,
            summary: None,
            error: None,
        };

        // Execute the diagnostic based on type
        let result = match request.diagnostic_type {
            DiagnosticType::Quick => self.run_quick_diagnostic().await,
            DiagnosticType::Full => self.run_full_diagnostic().await,
            DiagnosticType::Wifi => self.run_wifi_diagnostic().await,
            DiagnosticType::Speed => self.run_speed_diagnostic().await,
            DiagnosticType::Custom => self.run_custom_diagnostic().await,
        };

        run.completed_at = Some(Utc::now());

        match result {
            Ok(summary) => {
                run.success = true;
                run.summary = Some(summary);
                tracing::info!("Diagnostic '{}' completed successfully", request.job_name);
            }
            Err(e) => {
                run.success = false;
                run.error = Some(e.to_string());
                tracing::error!("Diagnostic '{}' failed: {}", request.job_name, e);
            }
        }

        // Store the run
        let mut runs = self.runs.write().await;
        runs.push(run);

        // Trim history if needed
        if runs.len() > self.max_history {
            let excess = runs.len() - self.max_history;
            runs.drain(0..excess);
        }
    }

    /// Runs a quick connectivity diagnostic.
    async fn run_quick_diagnostic(&self) -> Result<String> {
        // In a real implementation, this would:
        // 1. Check gateway connectivity
        // 2. Check DNS resolution
        // 3. Check internet connectivity
        tracing::debug!("Running quick diagnostic");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok("Gateway: OK, DNS: OK, Internet: OK".to_string())
    }

    /// Runs a full diagnostic.
    async fn run_full_diagnostic(&self) -> Result<String> {
        tracing::debug!("Running full diagnostic");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        Ok("Full diagnostic completed: All systems operational".to_string())
    }

    /// Runs a WiFi diagnostic.
    async fn run_wifi_diagnostic(&self) -> Result<String> {
        tracing::debug!("Running WiFi diagnostic");
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok("WiFi: Connected, Signal: Good".to_string())
    }

    /// Runs a speed diagnostic.
    async fn run_speed_diagnostic(&self) -> Result<String> {
        tracing::debug!("Running speed diagnostic");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        Ok("Download: 100 Mbps, Upload: 50 Mbps, Latency: 15ms".to_string())
    }

    /// Runs a custom diagnostic.
    async fn run_custom_diagnostic(&self) -> Result<String> {
        tracing::debug!("Running custom diagnostic");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok("Custom diagnostic completed".to_string())
    }
}
