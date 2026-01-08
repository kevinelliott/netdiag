//! Windows autofix provider implementation.

use async_trait::async_trait;
use netdiag_platform::AutofixProvider;
use netdiag_types::{
    error::Result,
    system::{FixAction, FixResult, PrivilegeLevel, RollbackId},
    Error,
};
use std::net::IpAddr;
use std::process::Command;
use std::time::Instant;

/// Windows autofix provider using netsh and system commands.
pub struct WindowsAutofixProvider {
    rollback_store: std::sync::Mutex<Vec<RollbackPoint>>,
}

#[derive(Clone, Debug)]
struct RollbackPoint {
    id: RollbackId,
    action: String,
    previous_state: Option<String>,
}

impl WindowsAutofixProvider {
    /// Creates a new Windows autofix provider.
    pub fn new() -> Self {
        Self {
            rollback_store: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Runs a Windows command and returns the output.
    fn run_command(&self, program: &str, args: &[&str]) -> Result<String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| Error::platform(&format!("Failed to run {}: {}", program, e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(Error::platform(&format!(
                "Command {} failed: {}",
                program, stderr
            )))
        }
    }

    /// Gets the current DNS servers for an interface.
    fn get_current_dns(&self, interface: &str) -> Result<Vec<IpAddr>> {
        let output = self.run_command(
            "netsh",
            &["interface", "ip", "show", "dns", interface],
        )?;

        let mut servers = Vec::new();
        for line in output.lines() {
            // Parse lines like "DNS servers configured through DHCP:  192.168.1.1"
            // or "Statically Configured DNS Servers:    8.8.8.8"
            if let Some(ip_str) = line.split_whitespace().last() {
                if let Ok(ip) = ip_str.parse::<IpAddr>() {
                    servers.push(ip);
                }
            }
        }
        Ok(servers)
    }

    /// Stores a rollback point.
    fn store_rollback(&self, action: &str, previous_state: Option<String>) -> RollbackId {
        let id = RollbackId::new();
        let point = RollbackPoint {
            id: id.clone(),
            action: action.to_string(),
            previous_state,
        };
        if let Ok(mut store) = self.rollback_store.lock() {
            store.push(point);
        }
        id
    }
}

impl Default for WindowsAutofixProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AutofixProvider for WindowsAutofixProvider {
    async fn flush_dns_cache(&self) -> Result<()> {
        // Run ipconfig /flushdns
        self.run_command("ipconfig", &["/flushdns"])?;
        Ok(())
    }

    async fn reset_adapter(&self, interface: &str) -> Result<()> {
        // Disable the adapter
        self.run_command(
            "netsh",
            &["interface", "set", "interface", interface, "disable"],
        )?;

        // Small delay
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Re-enable the adapter
        self.run_command(
            "netsh",
            &["interface", "set", "interface", interface, "enable"],
        )?;

        Ok(())
    }

    async fn set_dns_servers(&self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        if servers.is_empty() {
            // Reset to DHCP
            self.run_command(
                "netsh",
                &["interface", "ip", "set", "dns", interface, "dhcp"],
            )?;
            return Ok(());
        }

        // Set primary DNS
        let primary = servers[0].to_string();
        self.run_command(
            "netsh",
            &[
                "interface",
                "ip",
                "set",
                "dns",
                interface,
                "static",
                &primary,
            ],
        )?;

        // Add secondary DNS servers
        for server in servers.iter().skip(1) {
            let addr = server.to_string();
            self.run_command(
                "netsh",
                &[
                    "interface",
                    "ip",
                    "add",
                    "dns",
                    interface,
                    &addr,
                    "index=2",
                ],
            )?;
        }

        Ok(())
    }

    async fn reset_tcpip_stack(&self) -> Result<()> {
        // Reset TCP/IP stack
        self.run_command("netsh", &["int", "ip", "reset"])?;

        // Reset Winsock catalog
        self.run_command("netsh", &["winsock", "reset"])?;

        Ok(())
    }

    async fn create_rollback_point(&self) -> Result<RollbackId> {
        // For Windows, we could export current network configuration
        // For now, just create an ID
        Ok(self.store_rollback("snapshot", None))
    }

    async fn rollback(&self, id: RollbackId) -> Result<()> {
        let point = {
            let store = self
                .rollback_store
                .lock()
                .map_err(|_| Error::platform("Failed to acquire rollback store lock"))?;
            store.iter().find(|p| p.id == id).cloned()
        };

        if let Some(point) = point {
            // Restore based on action type
            match point.action.as_str() {
                "dns_change" => {
                    if let Some(prev) = &point.previous_state {
                        // Parse and restore previous DNS
                        let servers: Vec<IpAddr> = prev
                            .split(',')
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        // Would need interface name stored
                        tracing::warn!("DNS rollback not fully implemented: {}", prev);
                    }
                }
                _ => {
                    tracing::info!("Rollback for action '{}' not implemented", point.action);
                }
            }
            Ok(())
        } else {
            Err(Error::platform("Rollback point not found"))
        }
    }

    async fn renew_dhcp(&self, interface: &str) -> Result<()> {
        // Release current lease
        self.run_command("ipconfig", &["/release", interface])?;

        // Small delay
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Renew lease
        self.run_command("ipconfig", &["/renew", interface])?;

        Ok(())
    }

    fn required_privilege_level(&self, action: &FixAction) -> PrivilegeLevel {
        match action {
            FixAction::FlushDns => PrivilegeLevel::Elevated,
            FixAction::ResetAdapter { .. } => PrivilegeLevel::Elevated,
            FixAction::SetDnsServers { .. } => PrivilegeLevel::Elevated,
            FixAction::ResetTcpIp => PrivilegeLevel::Elevated,
            FixAction::RenewDhcp { .. } => PrivilegeLevel::Elevated,
            FixAction::RestartService { .. } => PrivilegeLevel::Elevated,
            FixAction::Custom { .. } => PrivilegeLevel::Elevated,
        }
    }

    async fn execute_fix(&self, action: &FixAction) -> Result<FixResult> {
        let start = Instant::now();

        let result = match action {
            FixAction::FlushDns => self.flush_dns_cache().await,
            FixAction::ResetAdapter { interface } => self.reset_adapter(interface).await,
            FixAction::SetDnsServers { interface, servers } => {
                self.set_dns_servers(interface, servers).await
            }
            FixAction::ResetTcpIp => self.reset_tcpip_stack().await,
            FixAction::RenewDhcp { interface } => self.renew_dhcp(interface).await,
            FixAction::RestartService { service } => self.restart_service(service).await,
            FixAction::Custom { command, .. } => {
                // Parse and execute custom command
                let parts: Vec<&str> = command.split_whitespace().collect();
                if parts.is_empty() {
                    Err(Error::platform("Empty custom command"))
                } else {
                    self.run_command(parts[0], &parts[1..]).map(|_| ())
                }
            }
        };

        let duration = start.elapsed();

        match result {
            Ok(()) => Ok(FixResult {
                action: action.clone(),
                success: true,
                message: "Fix applied successfully".to_string(),
                duration,
                rollback_id: None,
            }),
            Err(e) => Ok(FixResult {
                action: action.clone(),
                success: false,
                message: format!("Fix failed: {}", e),
                duration,
                rollback_id: None,
            }),
        }
    }

    async fn restart_service(&self, service: &str) -> Result<()> {
        // Stop the service
        let _ = self.run_command("net", &["stop", service]);

        // Small delay
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        // Start the service
        self.run_command("net", &["start", service])?;

        Ok(())
    }

    async fn verify_fix(&self, action: &FixAction) -> Result<bool> {
        match action {
            FixAction::FlushDns => {
                // DNS cache flush is fire-and-forget, assume success
                Ok(true)
            }
            FixAction::ResetAdapter { interface } => {
                // Check if interface is up
                let output = self.run_command(
                    "netsh",
                    &["interface", "show", "interface", interface],
                )?;
                Ok(output.contains("Connected") || output.contains("Enabled"))
            }
            FixAction::SetDnsServers { interface, servers } => {
                // Verify DNS servers are set
                let current = self.get_current_dns(interface)?;
                Ok(servers.iter().all(|s| current.contains(s)))
            }
            FixAction::ResetTcpIp => {
                // TCP/IP reset requires reboot, assume success
                Ok(true)
            }
            FixAction::RenewDhcp { interface } => {
                // Check if we have an IP address
                let output = self.run_command(
                    "netsh",
                    &["interface", "ip", "show", "addresses", interface],
                )?;
                Ok(output.contains("DHCP") && !output.contains("0.0.0.0"))
            }
            FixAction::RestartService { service } => {
                // Check if service is running
                let output = self.run_command("sc", &["query", service])?;
                Ok(output.contains("RUNNING"))
            }
            FixAction::Custom { .. } => {
                // Can't verify custom commands
                Ok(true)
            }
        }
    }
}
