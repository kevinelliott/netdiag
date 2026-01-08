//! Rollback management for undoing fixes.

use crate::error::{AutofixError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Unique identifier for a rollback point.
pub type RollbackId = String;

/// A point in time that can be rolled back to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPoint {
    /// Unique identifier.
    pub id: RollbackId,
    /// When this point was created.
    pub created_at: DateTime<Utc>,
    /// Description of what was changed.
    pub description: String,
    /// The original state that can be restored.
    pub state: RollbackState,
    /// Whether this rollback point is still valid.
    pub valid: bool,
    /// Associated fix action ID.
    pub action_id: Option<Uuid>,
}

/// State that can be restored during rollback.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RollbackState {
    /// DNS server configuration.
    DnsServers {
        /// Interface name.
        interface: String,
        /// Original DNS servers.
        servers: Vec<IpAddr>,
    },
    /// Network configuration file backup.
    ConfigFile {
        /// Path to the config file.
        path: PathBuf,
        /// Original contents.
        contents: String,
    },
    /// Multiple states combined.
    Multiple(Vec<RollbackState>),
    /// No state to restore (action is not reversible).
    None,
}

/// Manages rollback points and restoration.
pub struct RollbackManager {
    /// Storage directory for rollback data.
    storage_dir: PathBuf,
    /// Active rollback points.
    points: HashMap<RollbackId, RollbackPoint>,
    /// Maximum number of rollback points to keep.
    max_points: usize,
}

impl RollbackManager {
    /// Creates a new rollback manager.
    pub fn new(storage_dir: PathBuf, max_points: usize) -> Self {
        Self {
            storage_dir,
            points: HashMap::new(),
            max_points,
        }
    }

    /// Initializes the rollback manager, loading any persisted points.
    pub fn init(&mut self) -> Result<()> {
        // Ensure storage directory exists
        std::fs::create_dir_all(&self.storage_dir)?;

        // Load existing rollback points
        let points_file = self.storage_dir.join("rollback_points.json");
        if points_file.exists() {
            let content = std::fs::read_to_string(&points_file)?;
            self.points = serde_json::from_str(&content)?;

            // Filter out invalid points
            self.points.retain(|_, p| p.valid);
        }

        tracing::debug!(
            "Rollback manager initialized with {} points",
            self.points.len()
        );
        Ok(())
    }

    /// Creates a rollback point for DNS server changes.
    pub fn create_dns_point(
        &mut self,
        interface: &str,
        current_servers: Vec<IpAddr>,
        action_id: Option<Uuid>,
    ) -> Result<RollbackId> {
        let point = RollbackPoint {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            description: format!("DNS servers for {}: {:?}", interface, current_servers),
            state: RollbackState::DnsServers {
                interface: interface.to_string(),
                servers: current_servers,
            },
            valid: true,
            action_id,
        };

        let id = point.id.clone();
        self.add_point(point)?;
        Ok(id)
    }

    /// Creates a rollback point for a config file.
    pub fn create_config_point(
        &mut self,
        path: &Path,
        action_id: Option<Uuid>,
    ) -> Result<RollbackId> {
        let contents = std::fs::read_to_string(path)?;

        let point = RollbackPoint {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            description: format!("Config file: {}", path.display()),
            state: RollbackState::ConfigFile {
                path: path.to_path_buf(),
                contents,
            },
            valid: true,
            action_id,
        };

        let id = point.id.clone();
        self.add_point(point)?;
        Ok(id)
    }

    /// Creates a rollback point with multiple states.
    pub fn create_multi_point(
        &mut self,
        description: String,
        states: Vec<RollbackState>,
        action_id: Option<Uuid>,
    ) -> Result<RollbackId> {
        let point = RollbackPoint {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            description,
            state: RollbackState::Multiple(states),
            valid: true,
            action_id,
        };

        let id = point.id.clone();
        self.add_point(point)?;
        Ok(id)
    }

    /// Adds a rollback point.
    fn add_point(&mut self, point: RollbackPoint) -> Result<()> {
        // Enforce maximum points
        if self.points.len() >= self.max_points {
            // Remove oldest point
            if let Some(oldest_id) = self
                .points
                .values()
                .min_by_key(|p| p.created_at)
                .map(|p| p.id.clone())
            {
                self.points.remove(&oldest_id);
            }
        }

        let id = point.id.clone();
        self.points.insert(id, point);
        self.persist()?;
        Ok(())
    }

    /// Persists rollback points to disk.
    fn persist(&self) -> Result<()> {
        let points_file = self.storage_dir.join("rollback_points.json");
        let content = serde_json::to_string_pretty(&self.points)?;
        std::fs::write(points_file, content)?;
        Ok(())
    }

    /// Gets a rollback point by ID.
    pub fn get(&self, id: &str) -> Option<&RollbackPoint> {
        self.points.get(id)
    }

    /// Lists all active rollback points.
    pub fn list(&self) -> Vec<&RollbackPoint> {
        let mut points: Vec<_> = self.points.values().filter(|p| p.valid).collect();
        points.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        points
    }

    /// Performs a rollback to a specific point.
    pub async fn rollback(&mut self, id: &str) -> Result<()> {
        let point = self
            .points
            .get(id)
            .ok_or_else(|| AutofixError::RollbackNotFound { id: id.to_string() })?
            .clone();

        if !point.valid {
            return Err(AutofixError::rollback_failed("Rollback point is no longer valid"));
        }

        tracing::info!("Rolling back: {}", point.description);

        self.restore_state(&point.state).await?;

        // Mark point as used
        if let Some(p) = self.points.get_mut(id) {
            p.valid = false;
        }
        self.persist()?;

        tracing::info!("Rollback completed successfully");
        Ok(())
    }

    /// Restores a specific state.
    fn restore_state<'a>(
        &'a self,
        state: &'a RollbackState,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            match state {
                RollbackState::DnsServers { interface, servers } => {
                    self.restore_dns_servers(interface, servers).await
                }
                RollbackState::ConfigFile { path, contents } => {
                    self.restore_config_file(path, contents)
                }
                RollbackState::Multiple(states) => {
                    for s in states {
                        self.restore_state(s).await?;
                    }
                    Ok(())
                }
                RollbackState::None => Ok(()),
            }
        })
    }

    /// Restores DNS servers.
    async fn restore_dns_servers(&self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        tracing::debug!(
            "Restoring DNS servers for {}: {:?}",
            interface,
            servers
        );

        // Platform-specific DNS restoration
        #[cfg(target_os = "macos")]
        {
            self.restore_dns_macos(interface, servers)?;
        }

        #[cfg(target_os = "linux")]
        {
            self.restore_dns_linux(interface, servers)?;
        }

        #[cfg(target_os = "windows")]
        {
            self.restore_dns_windows(interface, servers)?;
        }

        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn restore_dns_macos(&self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        use std::process::Command;

        // Get network service name from interface
        // This is simplified - real implementation would map interface to service
        let service = interface;

        if servers.is_empty() {
            // Reset to DHCP
            Command::new("networksetup")
                .args(["-setdnsservers", service, "Empty"])
                .output()?;
        } else {
            let server_strs: Vec<String> = servers.iter().map(|s| s.to_string()).collect();
            let mut args = vec!["-setdnsservers", service];
            args.extend(server_strs.iter().map(|s| s.as_str()));

            Command::new("networksetup").args(&args).output()?;
        }

        // Flush DNS cache
        Command::new("dscacheutil")
            .args(["-flushcache"])
            .output()?;

        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn restore_dns_linux(&self, _interface: &str, servers: &[IpAddr]) -> Result<()> {
        // Write to resolv.conf
        let mut content = String::new();
        for server in servers {
            content.push_str(&format!("nameserver {}\n", server));
        }

        // Try systemd-resolved first
        if Path::new("/run/systemd/resolve/stub-resolv.conf").exists() {
            tracing::debug!("Using systemd-resolved for DNS restoration");
            // Would use resolvectl
        } else {
            std::fs::write("/etc/resolv.conf", content)?;
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn restore_dns_windows(&self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        use std::process::Command;

        if servers.is_empty() {
            // Reset to DHCP
            Command::new("netsh")
                .args([
                    "interface",
                    "ip",
                    "set",
                    "dns",
                    interface,
                    "dhcp",
                ])
                .output()?;
        } else {
            // Set primary DNS
            Command::new("netsh")
                .args([
                    "interface",
                    "ip",
                    "set",
                    "dns",
                    interface,
                    "static",
                    &servers[0].to_string(),
                ])
                .output()?;

            // Add secondary DNS servers
            for server in &servers[1..] {
                Command::new("netsh")
                    .args([
                        "interface",
                        "ip",
                        "add",
                        "dns",
                        interface,
                        &server.to_string(),
                    ])
                    .output()?;
            }
        }

        // Flush DNS cache
        Command::new("ipconfig").args(["/flushdns"]).output()?;

        Ok(())
    }

    /// Restores a config file.
    fn restore_config_file(&self, path: &Path, contents: &str) -> Result<()> {
        tracing::debug!("Restoring config file: {}", path.display());
        std::fs::write(path, contents)?;
        Ok(())
    }

    /// Invalidates a rollback point without restoring.
    pub fn invalidate(&mut self, id: &str) -> Result<()> {
        if let Some(point) = self.points.get_mut(id) {
            point.valid = false;
            self.persist()?;
        }
        Ok(())
    }

    /// Cleans up old rollback points.
    pub fn cleanup(&mut self, max_age_hours: u32) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours as i64);
        let before = self.points.len();

        self.points.retain(|_, p| p.created_at > cutoff || p.valid);

        let removed = before - self.points.len();
        if removed > 0 {
            self.persist()?;
            tracing::info!("Cleaned up {} old rollback points", removed);
        }

        Ok(removed)
    }
}
