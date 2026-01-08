//! IPC (Inter-Process Communication) for daemon.
//!
//! Provides cross-platform socket-based communication between the daemon and CLI/GUI clients.
//! Uses Unix domain sockets on Unix platforms and named pipes on Windows.

use crate::error::{DaemonError, Result};
use crate::service::ServiceState;
use interprocess::local_socket::{
    tokio::{prelude::*, Stream},
    GenericFilePath, GenericNamespaced, ListenerOptions, ToFsName, ToNsName,
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

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

/// Gets the socket name for the given path.
/// On Unix, uses filesystem path. On Windows, uses named pipe namespace.
fn get_socket_name(path: &str) -> Result<interprocess::local_socket::Name<'static>> {
    let path_owned = path.to_string();

    // Try namespaced name first (works on both platforms)
    if let Ok(name) = path_owned.clone().to_ns_name::<GenericNamespaced>() {
        return Ok(name);
    }

    // Fall back to filesystem path (Unix only)
    path_owned
        .to_fs_name::<GenericFilePath>()
        .map_err(|e| DaemonError::ipc(&format!("Invalid socket path: {}", e)))
}

/// IPC server that listens for client connections.
pub struct IpcServer {
    socket_path: String,
    listener: Option<interprocess::local_socket::tokio::Listener>,
}

impl IpcServer {
    /// Creates a new IPC server.
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            listener: None,
        }
    }

    /// Starts listening for connections.
    pub async fn start(&mut self) -> Result<()> {
        // Clean up existing socket on Unix
        #[cfg(unix)]
        {
            let path = std::path::Path::new(&self.socket_path);
            if path.exists() {
                let _ = std::fs::remove_file(path);
            }
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        let name = get_socket_name(&self.socket_path)?;
        let opts = ListenerOptions::new().name(name);

        let listener = opts
            .create_tokio()
            .map_err(|e| DaemonError::ipc(&format!("Failed to create listener: {}", e)))?;

        tracing::info!("IPC server listening on {}", self.socket_path);
        self.listener = Some(listener);
        Ok(())
    }

    /// Accepts a client connection.
    pub async fn accept(&self) -> Result<IpcConnection> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| DaemonError::ipc("Server not started"))?;

        let stream = listener
            .accept()
            .await
            .map_err(|e| DaemonError::ipc(&format!("Accept failed: {}", e)))?;

        Ok(IpcConnection::new(stream))
    }

    /// Shuts down the server.
    pub async fn shutdown(&mut self) -> Result<()> {
        self.listener.take();

        // Clean up socket file on Unix
        #[cfg(unix)]
        {
            let _ = std::fs::remove_file(&self.socket_path);
        }

        Ok(())
    }
}

/// An IPC connection to a client.
pub struct IpcConnection {
    reader: BufReader<interprocess::local_socket::tokio::RecvHalf>,
    writer: BufWriter<interprocess::local_socket::tokio::SendHalf>,
}

impl IpcConnection {
    /// Creates a new IPC connection from a stream.
    fn new(stream: Stream) -> Self {
        let (recv, send) = stream.split();
        Self {
            reader: BufReader::new(recv),
            writer: BufWriter::new(send),
        }
    }

    /// Reads a request from the client.
    pub async fn read_request(&mut self) -> Result<Option<IpcRequest>> {
        let mut line = String::new();

        match self.reader.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                let request: IpcRequest = serde_json::from_str(line.trim())
                    .map_err(|e| DaemonError::ipc(&format!("Invalid request: {}", e)))?;
                Ok(Some(request))
            }
            Err(e) => Err(DaemonError::Io(e)),
        }
    }

    /// Sends a response to the client.
    pub async fn send_response(&mut self, response: &IpcResponse) -> Result<()> {
        let json = serde_json::to_string(response)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}

/// IPC client for connecting to the daemon.
pub struct IpcClient {
    socket_path: String,
    connection: Option<IpcConnection>,
}

impl IpcClient {
    /// Creates a new IPC client.
    pub fn new(socket_path: String) -> Self {
        Self {
            socket_path,
            connection: None,
        }
    }

    /// Connects to the daemon.
    pub async fn connect(&mut self) -> Result<()> {
        let name = get_socket_name(&self.socket_path)?;
        let stream = Stream::connect(name)
            .await
            .map_err(|e| DaemonError::ipc(&format!("Connection failed: {}", e)))?;
        self.connection = Some(IpcConnection::new(stream));
        Ok(())
    }

    /// Sends a request and waits for response.
    pub async fn request(&mut self, request: &IpcRequest) -> Result<IpcResponse> {
        let conn = self
            .connection
            .as_mut()
            .ok_or_else(|| DaemonError::ipc("Not connected"))?;

        // Send request
        let json = serde_json::to_string(request)?;
        conn.writer.write_all(json.as_bytes()).await?;
        conn.writer.write_all(b"\n").await?;
        conn.writer.flush().await?;

        // Read response
        let mut line = String::new();
        conn.reader.read_line(&mut line).await?;
        let response: IpcResponse = serde_json::from_str(line.trim())
            .map_err(|e| DaemonError::ipc(&format!("Invalid response: {}", e)))?;
        Ok(response)
    }

    /// Checks if the daemon is running.
    pub async fn is_daemon_running(&mut self) -> bool {
        if self.connect().await.is_err() {
            return false;
        }
        matches!(self.request(&IpcRequest::Ping).await, Ok(IpcResponse::Pong))
    }
}
