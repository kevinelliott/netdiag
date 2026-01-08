//! IPC (Inter-Process Communication) for daemon.
//!
//! Provides socket-based communication between the daemon and CLI/GUI clients.

use crate::error::{DaemonError, Result};
use crate::service::ServiceState;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};

/// IPC request from client to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcRequest {
    /// Get daemon status.
    Status,
    /// Stop the daemon.
    Stop,
    /// Reload configuration.
    Reload,
    /// Run a diagnostic now.
    RunDiagnostic {
        /// Type of diagnostic to run.
        diagnostic_type: String,
    },
    /// Get recent results.
    GetResults {
        /// Maximum number of results to return.
        limit: usize,
    },
    /// Get current monitoring data.
    GetMonitoringData,
    /// Pause monitoring.
    PauseMonitoring,
    /// Resume monitoring.
    ResumeMonitoring,
    /// Ping to check if daemon is alive.
    Ping,
}

/// IPC response from daemon to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    /// Success with optional message.
    Ok {
        /// Optional message.
        message: Option<String>,
    },
    /// Error response.
    Error {
        /// Error message.
        message: String,
    },
    /// Status response.
    Status {
        /// Current state.
        state: ServiceState,
        /// Uptime in seconds.
        uptime_secs: u64,
        /// Number of completed diagnostics.
        diagnostics_run: u64,
        /// Monitoring status.
        monitoring_active: bool,
    },
    /// Diagnostic results.
    Results {
        /// Serialized results.
        results: Vec<String>,
    },
    /// Monitoring data.
    MonitoringData {
        /// Current monitoring data as JSON.
        data: String,
    },
    /// Pong response.
    Pong,
}

/// IPC server that listens for client connections.
pub struct IpcServer {
    socket_path: String,
    #[cfg(unix)]
    listener: Option<UnixListener>,
}

impl IpcServer {
    /// Creates a new IPC server.
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            #[cfg(unix)]
            listener: None,
        }
    }

    /// Starts listening for connections.
    #[cfg(unix)]
    pub async fn start(&mut self) -> Result<()> {
        // Remove existing socket if it exists
        let path = Path::new(&self.socket_path);
        if path.exists() {
            std::fs::remove_file(path)?;
        }

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        tracing::info!("IPC server listening on {}", self.socket_path);
        self.listener = Some(listener);
        Ok(())
    }

    /// Starts listening for connections (Windows stub).
    #[cfg(windows)]
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("IPC server starting on {}", self.socket_path);
        // Windows would use named pipes
        Ok(())
    }

    /// Accepts a client connection.
    #[cfg(unix)]
    pub async fn accept(&self) -> Result<IpcConnection> {
        let listener = self.listener.as_ref().ok_or_else(|| {
            DaemonError::ipc("Server not started")
        })?;

        let (stream, _addr) = listener.accept().await?;
        Ok(IpcConnection { stream })
    }

    /// Accepts a client connection (Windows stub).
    #[cfg(windows)]
    pub async fn accept(&self) -> Result<IpcConnection> {
        // Windows implementation would use named pipes
        Err(DaemonError::ipc("Windows IPC not yet implemented"))
    }

    /// Shuts down the server.
    #[cfg(unix)]
    pub async fn shutdown(&mut self) -> Result<()> {
        self.listener.take();
        let _ = std::fs::remove_file(&self.socket_path);
        Ok(())
    }

    /// Shuts down the server (Windows stub).
    #[cfg(windows)]
    pub async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// An IPC connection to a client.
#[cfg(unix)]
pub struct IpcConnection {
    stream: UnixStream,
}

#[cfg(unix)]
impl IpcConnection {
    /// Reads a request from the client.
    pub async fn read_request(&mut self) -> Result<Option<IpcRequest>> {
        let mut reader = BufReader::new(&mut self.stream);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                let request: IpcRequest = serde_json::from_str(line.trim())?;
                Ok(Some(request))
            }
            Err(e) => Err(DaemonError::Io(e)),
        }
    }

    /// Sends a response to the client.
    pub async fn send_response(&mut self, response: &IpcResponse) -> Result<()> {
        let json = serde_json::to_string(response)?;
        self.stream.write_all(json.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        self.stream.flush().await?;
        Ok(())
    }
}

/// Dummy IPC connection for Windows.
#[cfg(windows)]
pub struct IpcConnection;

#[cfg(windows)]
impl IpcConnection {
    /// Reads a request from the client.
    pub async fn read_request(&mut self) -> Result<Option<IpcRequest>> {
        Err(DaemonError::ipc("Windows IPC not yet implemented"))
    }

    /// Sends a response to the client.
    pub async fn send_response(&mut self, _response: &IpcResponse) -> Result<()> {
        Err(DaemonError::ipc("Windows IPC not yet implemented"))
    }
}

/// IPC client for connecting to the daemon.
pub struct IpcClient {
    socket_path: String,
    #[cfg(unix)]
    stream: Option<UnixStream>,
}

impl IpcClient {
    /// Creates a new IPC client.
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            #[cfg(unix)]
            stream: None,
        }
    }

    /// Connects to the daemon.
    #[cfg(unix)]
    pub async fn connect(&mut self) -> Result<()> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        self.stream = Some(stream);
        Ok(())
    }

    /// Connects to the daemon (Windows stub).
    #[cfg(windows)]
    pub async fn connect(&mut self) -> Result<()> {
        Err(DaemonError::ipc("Windows IPC not yet implemented"))
    }

    /// Sends a request and waits for response.
    #[cfg(unix)]
    pub async fn request(&mut self, request: &IpcRequest) -> Result<IpcResponse> {
        let stream = self.stream.as_mut().ok_or_else(|| {
            DaemonError::ipc("Not connected")
        })?;

        // Send request
        let json = serde_json::to_string(request)?;
        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Read response
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let response: IpcResponse = serde_json::from_str(line.trim())?;
        Ok(response)
    }

    /// Sends a request and waits for response (Windows stub).
    #[cfg(windows)]
    pub async fn request(&mut self, _request: &IpcRequest) -> Result<IpcResponse> {
        Err(DaemonError::ipc("Windows IPC not yet implemented"))
    }

    /// Checks if the daemon is running.
    pub async fn is_daemon_running(&mut self) -> bool {
        if self.connect().await.is_err() {
            return false;
        }
        matches!(self.request(&IpcRequest::Ping).await, Ok(IpcResponse::Pong))
    }
}
