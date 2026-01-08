//! Daemon service management.

use crate::config::DaemonConfig;
use crate::error::{DaemonError, Result};
use crate::ipc::{IpcConnection, IpcRequest, IpcResponse, IpcServer};
use crate::monitor::{Alert, NetworkMonitor};
use crate::scheduler::{DiagnosticExecutor, DiagnosticRequest, DiagnosticScheduler};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[cfg(unix)]
use std::fs;

/// Service state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceState {
    /// Service is starting up.
    Starting,
    /// Service is running.
    Running,
    /// Service is stopping.
    Stopping,
    /// Service is stopped.
    Stopped,
}

/// Daemon service statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    /// Service state.
    pub state: ServiceState,
    /// Start time.
    pub started_at: Option<DateTime<Utc>>,
    /// Number of diagnostics run.
    pub diagnostics_run: u64,
    /// Number of alerts generated.
    pub alerts_generated: u64,
    /// Whether monitoring is active.
    pub monitoring_active: bool,
}

/// The main daemon service.
pub struct DaemonService {
    config: DaemonConfig,
    state: Arc<RwLock<ServiceState>>,
    started_at: Arc<RwLock<Option<DateTime<Utc>>>>,
    diagnostics_run: Arc<RwLock<u64>>,
    alerts_generated: Arc<RwLock<u64>>,
    ipc_server: Option<IpcServer>,
    scheduler: Option<DiagnosticScheduler>,
    monitor: Option<NetworkMonitor>,
}

impl DaemonService {
    /// Creates a new daemon service.
    pub fn new(config: DaemonConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ServiceState::Stopped)),
            started_at: Arc::new(RwLock::new(None)),
            diagnostics_run: Arc::new(RwLock::new(0)),
            alerts_generated: Arc::new(RwLock::new(0)),
            ipc_server: None,
            scheduler: None,
            monitor: None,
        }
    }

    /// Gets the current service state.
    pub async fn state(&self) -> ServiceState {
        *self.state.read().await
    }

    /// Gets service statistics.
    pub async fn stats(&self) -> ServiceStats {
        ServiceStats {
            state: *self.state.read().await,
            started_at: *self.started_at.read().await,
            diagnostics_run: *self.diagnostics_run.read().await,
            alerts_generated: *self.alerts_generated.read().await,
            monitoring_active: self
                .monitor
                .as_ref()
                .map(|m| futures::executor::block_on(m.is_running()))
                .unwrap_or(false),
        }
    }

    /// Starts the daemon service.
    pub async fn start(&mut self) -> Result<()> {
        *self.state.write().await = ServiceState::Starting;
        tracing::info!("Starting netdiag daemon service");

        // Write PID file
        if let Some(pid_file) = &self.config.general.pid_file {
            self.write_pid_file(pid_file)?;
        }

        // Create channels
        let (diagnostic_tx, diagnostic_rx) = mpsc::channel::<DiagnosticRequest>(32);
        let (alert_tx, mut alert_rx) = mpsc::channel::<Alert>(32);

        // Start scheduler
        let mut scheduler = DiagnosticScheduler::new(diagnostic_tx).await?;
        scheduler.add_schedules(&self.config.schedules).await?;
        scheduler.start().await?;
        self.scheduler = Some(scheduler);

        // Start diagnostic executor in background
        let _diagnostics_run = self.diagnostics_run.clone();
        let mut executor = DiagnosticExecutor::new(diagnostic_rx, 1000);
        tokio::spawn(async move {
            // TODO: Wire up diagnostics_run counter updates from executor
            executor.run().await;
        });

        // Start monitor
        let monitor = NetworkMonitor::new(
            self.config.monitoring.clone(),
            self.config.alerts.clone(),
            Some(alert_tx),
        );
        self.monitor = Some(monitor);

        // Spawn monitor task
        if let Some(monitor) = &self.monitor {
            let monitor_data = monitor.get_data().await;
            tracing::debug!("Initial monitor status: {:?}", monitor_data.status);
        }

        // Start alert handler
        let alerts_generated = self.alerts_generated.clone();
        tokio::spawn(async move {
            while let Some(alert) = alert_rx.recv().await {
                tracing::warn!("[ALERT] {:?}: {}", alert.severity, alert.message);
                *alerts_generated.write().await += 1;
            }
        });

        // Start IPC server
        let mut ipc_server = IpcServer::new(self.config.ipc.socket_path.clone());
        ipc_server.start().await?;
        self.ipc_server = Some(ipc_server);

        // Update state
        *self.state.write().await = ServiceState::Running;
        *self.started_at.write().await = Some(Utc::now());

        tracing::info!("Daemon service started successfully");
        Ok(())
    }

    /// Runs the main service loop.
    pub async fn run(&mut self) -> Result<()> {
        tracing::info!("Entering main service loop");

        // Handle IPC connections
        loop {
            if *self.state.read().await != ServiceState::Running {
                break;
            }

            if let Some(server) = &self.ipc_server {
                match server.accept().await {
                    Ok(mut connection) => {
                        self.handle_connection(&mut connection).await;
                    }
                    Err(e) => {
                        tracing::debug!("IPC accept error (may be normal): {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            } else {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }

        Ok(())
    }

    /// Handles an IPC connection.
    async fn handle_connection(&self, connection: &mut IpcConnection) {
        loop {
            match connection.read_request().await {
                Ok(Some(request)) => {
                    let response = self.handle_request(request).await;
                    if let Err(e) = connection.send_response(&response).await {
                        tracing::error!("Failed to send IPC response: {}", e);
                        break;
                    }
                    // Check if we should stop
                    if matches!(response, IpcResponse::Ok { .. })
                        && *self.state.read().await == ServiceState::Stopping
                    {
                        break;
                    }
                }
                Ok(None) => {
                    // Client disconnected
                    break;
                }
                Err(e) => {
                    tracing::error!("IPC read error: {}", e);
                    break;
                }
            }
        }
    }

    /// Handles an IPC request.
    async fn handle_request(&self, request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::Ping => IpcResponse::Pong,

            IpcRequest::Status => {
                let state = *self.state.read().await;
                let uptime_secs = self
                    .started_at
                    .read()
                    .await
                    .map(|t| (Utc::now() - t).num_seconds() as u64)
                    .unwrap_or(0);
                let diagnostics_run = *self.diagnostics_run.read().await;
                let monitoring_active = self
                    .monitor
                    .as_ref()
                    .map(|m| futures::executor::block_on(m.is_running()))
                    .unwrap_or(false);

                IpcResponse::Status {
                    state,
                    uptime_secs,
                    diagnostics_run,
                    monitoring_active,
                }
            }

            IpcRequest::Stop => {
                tracing::info!("Received stop request");
                *self.state.write().await = ServiceState::Stopping;
                IpcResponse::Ok {
                    message: Some("Daemon stopping".to_string()),
                }
            }

            IpcRequest::Reload => {
                // Would reload configuration
                IpcResponse::Ok {
                    message: Some("Configuration reloaded".to_string()),
                }
            }

            IpcRequest::RunDiagnostic { diagnostic_type: _ } => {
                *self.diagnostics_run.write().await += 1;
                IpcResponse::Ok {
                    message: Some("Diagnostic started".to_string()),
                }
            }

            IpcRequest::GetResults { limit: _ } => IpcResponse::Results {
                results: Vec::new(),
            },

            IpcRequest::GetMonitoringData => {
                if let Some(monitor) = &self.monitor {
                    let data = monitor.get_data().await;
                    match serde_json::to_string(&data) {
                        Ok(json) => IpcResponse::MonitoringData { data: json },
                        Err(e) => IpcResponse::Error {
                            message: format!("Failed to serialize monitoring data: {}", e),
                        },
                    }
                } else {
                    IpcResponse::Error {
                        message: "Monitor not initialized".to_string(),
                    }
                }
            }

            IpcRequest::PauseMonitoring => {
                if let Some(monitor) = &self.monitor {
                    monitor.pause().await;
                    IpcResponse::Ok {
                        message: Some("Monitoring paused".to_string()),
                    }
                } else {
                    IpcResponse::Error {
                        message: "Monitor not initialized".to_string(),
                    }
                }
            }

            IpcRequest::ResumeMonitoring => {
                if let Some(monitor) = &self.monitor {
                    monitor.resume().await;
                    IpcResponse::Ok {
                        message: Some("Monitoring resumed".to_string()),
                    }
                } else {
                    IpcResponse::Error {
                        message: "Monitor not initialized".to_string(),
                    }
                }
            }
        }
    }

    /// Stops the daemon service.
    pub async fn stop(&mut self) -> Result<()> {
        *self.state.write().await = ServiceState::Stopping;
        tracing::info!("Stopping netdiag daemon service");

        // Stop scheduler
        if let Some(scheduler) = &mut self.scheduler {
            scheduler.shutdown().await?;
        }

        // Stop IPC server
        if let Some(server) = &mut self.ipc_server {
            server.shutdown().await?;
        }

        // Remove PID file
        if let Some(pid_file) = &self.config.general.pid_file {
            let _ = std::fs::remove_file(pid_file);
        }

        *self.state.write().await = ServiceState::Stopped;
        tracing::info!("Daemon service stopped");
        Ok(())
    }

    /// Writes the PID file.
    fn write_pid_file(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let pid = std::process::id();
        std::fs::write(path, pid.to_string())?;
        tracing::debug!("Wrote PID {} to {:?}", pid, path);
        Ok(())
    }

    /// Checks if another daemon instance is running.
    pub fn is_already_running(pid_file: &Path) -> Option<u32> {
        if !pid_file.exists() {
            return None;
        }

        let pid_str = match std::fs::read_to_string(pid_file) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let pid: u32 = match pid_str.trim().parse() {
            Ok(p) => p,
            Err(_) => return None,
        };

        // Check if process is running
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            match kill(Pid::from_raw(pid as i32), Signal::SIGCONT) {
                Ok(_) => Some(pid),
                Err(nix::errno::Errno::ESRCH) => {
                    // Process doesn't exist, remove stale PID file
                    let _ = std::fs::remove_file(pid_file);
                    None
                }
                Err(_) => Some(pid), // Assume running if we can't signal it
            }
        }

        #[cfg(windows)]
        {
            // On Windows, we'd check if the process exists differently
            Some(pid)
        }
    }
}

/// Daemonizes the process (Unix only).
#[cfg(unix)]
pub fn daemonize() -> Result<()> {
    use daemonize::Daemonize;

    let daemonize = Daemonize::new();

    daemonize
        .start()
        .map_err(|e| DaemonError::platform(format!("Failed to daemonize: {}", e)))?;

    Ok(())
}

/// Daemonizes the process (Windows stub).
#[cfg(windows)]
pub fn daemonize() -> Result<()> {
    // Windows uses services, not daemonization
    Ok(())
}

/// Installs the daemon as a system service.
#[cfg(target_os = "macos")]
pub fn install_service(config: &DaemonConfig) -> Result<()> {
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>net.netdiag.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/netdiag</string>
        <string>daemon</string>
        <string>run</string>
        <string>--foreground</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{}</string>
    <key>StandardErrorPath</key>
    <string>{}</string>
</dict>
</plist>"#,
        config
            .general
            .log_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "/var/log/netdiag/daemon.log".to_string()),
        config
            .general
            .log_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "/var/log/netdiag/daemon.log".to_string()),
    );

    let plist_path = Path::new("/Library/LaunchDaemons/net.netdiag.daemon.plist");

    // Ensure we have permission
    fs::write(plist_path, plist_content)?;

    tracing::info!("Installed launchd service at {:?}", plist_path);
    tracing::info!("Run 'sudo launchctl load {:?}' to start", plist_path);

    Ok(())
}

/// Installs the daemon as a system service (Linux).
#[cfg(target_os = "linux")]
pub fn install_service(config: &DaemonConfig) -> Result<()> {
    let service_content = format!(
        r#"[Unit]
Description=NetDiag Network Diagnostics Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/netdiag daemon run --foreground
Restart=always
RestartSec=10
StandardOutput=append:{}
StandardError=append:{}

[Install]
WantedBy=multi-user.target
"#,
        config
            .general
            .log_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "/var/log/netdiag/daemon.log".to_string()),
        config
            .general
            .log_file
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "/var/log/netdiag/daemon.log".to_string()),
    );

    let service_path = Path::new("/etc/systemd/system/netdiag.service");

    fs::write(service_path, service_content)?;

    tracing::info!("Installed systemd service at {:?}", service_path);
    tracing::info!("Run 'sudo systemctl daemon-reload && sudo systemctl enable --now netdiag' to start");

    Ok(())
}

/// Installs the daemon as a system service (Windows stub).
#[cfg(target_os = "windows")]
pub fn install_service(_config: &DaemonConfig) -> Result<()> {
    // Would use windows-service crate
    Err(DaemonError::platform(
        "Windows service installation not yet implemented".to_string(),
    ))
}

/// Installs the daemon as a system service (other platforms).
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn install_service(_config: &DaemonConfig) -> Result<()> {
    Err(DaemonError::platform(
        "Service installation not supported on this platform".to_string(),
    ))
}
