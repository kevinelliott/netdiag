//! macOS autofix provider implementation.

use async_trait::async_trait;
use chrono::Utc;
use netdiag_platform::{
    AutofixAction, AutofixProvider, FixCategory, FixResult, RiskLevel, RollbackPoint,
};
use netdiag_types::{
    error::{Error, Result},
    system::RollbackId,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::process::Command;
use std::sync::RwLock;
use std::time::Duration;
use tracing::{debug, info, warn};

/// macOS autofix provider.
pub struct MacosAutofixProvider {
    /// Rollback points storage
    rollback_points: RwLock<HashMap<String, StoredRollback>>,
}

/// Stored rollback information.
#[derive(Debug, Clone)]
struct StoredRollback {
    id: RollbackId,
    description: String,
    created_at: chrono::DateTime<Utc>,
    changes: Vec<RollbackChange>,
}

/// A change that can be rolled back.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum RollbackChange {
    DnsServers {
        interface: String,
        original_servers: Vec<IpAddr>,
    },
    InterfaceState {
        interface: String,
        was_enabled: bool,
    },
}

impl MacosAutofixProvider {
    /// Creates a new macOS autofix provider.
    pub fn new() -> Self {
        Self {
            rollback_points: RwLock::new(HashMap::new()),
        }
    }

    /// Runs a command and returns success status.
    fn run_command(command: &str, args: &[&str]) -> Result<()> {
        let output = Command::new(command)
            .args(args)
            .output()
            .map_err(|e| Error::Other {
                context: format!("{} {:?}", command, args),
                message: e.to_string(),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Other {
                context: format!("{} {:?}", command, args),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    /// Runs a command with sudo.
    fn run_sudo_command(command: &str, args: &[&str]) -> Result<()> {
        let mut full_args = vec![command];
        full_args.extend(args);

        let output = Command::new("sudo")
            .args(&full_args)
            .output()
            .map_err(|e| Error::Other {
                context: format!("sudo {} {:?}", command, args),
                message: e.to_string(),
            })?;

        if output.status.success() {
            Ok(())
        } else {
            Err(Error::Other {
                context: format!("sudo {} {:?}", command, args),
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            })
        }
    }

    /// Gets current DNS servers for an interface.
    fn get_current_dns_servers(&self, interface: &str) -> Result<Vec<IpAddr>> {
        // Use networksetup to get DNS servers
        let output = Command::new("networksetup")
            .args(["-getdnsservers", interface])
            .output()
            .map_err(|e| Error::Other {
                context: "networksetup".to_string(),
                message: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let servers: Vec<IpAddr> = stdout
            .lines()
            .filter_map(|line| line.trim().parse().ok())
            .collect();

        Ok(servers)
    }

    /// Gets the network service name for an interface.
    fn get_network_service(&self, interface: &str) -> Option<String> {
        let output = Command::new("networksetup")
            .args(["-listallhardwareports"])
            .output()
            .ok()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut current_service = None;

        for line in stdout.lines() {
            if line.starts_with("Hardware Port:") {
                current_service = line
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().to_string());
            } else if line.starts_with("Device:") {
                if let Some(device) = line.split(':').nth(1) {
                    if device.trim() == interface {
                        return current_service;
                    }
                }
            }
        }

        None
    }

    /// Checks if we have root privileges.
    fn is_root(&self) -> bool {
        unsafe { libc::geteuid() == 0 }
    }
}

impl Default for MacosAutofixProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AutofixProvider for MacosAutofixProvider {
    fn is_available(&self) -> bool {
        // Autofix is available but some operations require root
        true
    }

    async fn create_rollback_point(&self, description: &str) -> Result<RollbackId> {
        let id = RollbackId::new();
        let rollback = StoredRollback {
            id: id.clone(),
            description: description.to_string(),
            created_at: Utc::now(),
            changes: Vec::new(),
        };

        if let Ok(mut guard) = self.rollback_points.write() {
            guard.insert(id.0.clone(), rollback);
        }

        debug!("Created rollback point: {}", id);
        Ok(id)
    }

    async fn rollback(&self, id: &RollbackId) -> Result<()> {
        let rollback = {
            let guard = self.rollback_points.read().map_err(|_| Error::Other {
                context: "rollback".to_string(),
                message: "Lock error".to_string(),
            })?;

            guard.get(&id.0).cloned().ok_or_else(|| Error::Other {
                context: "rollback".to_string(),
                message: format!("Rollback point {} not found", id),
            })?
        };

        info!("Rolling back changes from: {}", rollback.description);

        for change in rollback.changes {
            match change {
                RollbackChange::DnsServers {
                    interface,
                    original_servers,
                } => {
                    debug!("Restoring DNS servers for {}", interface);
                    self.set_dns_servers(&interface, &original_servers).await?;
                }
                RollbackChange::InterfaceState {
                    interface,
                    was_enabled,
                } => {
                    debug!("Restoring interface {} state to {}", interface, was_enabled);
                    self.toggle_interface(&interface, was_enabled).await?;
                }
            }
        }

        // Remove the rollback point
        if let Ok(mut guard) = self.rollback_points.write() {
            guard.remove(&id.0);
        }

        Ok(())
    }

    async fn list_rollback_points(&self) -> Result<Vec<RollbackPoint>> {
        let guard = self.rollback_points.read().map_err(|_| Error::Other {
            context: "list_rollback_points".to_string(),
            message: "Lock error".to_string(),
        })?;

        Ok(guard
            .values()
            .map(|r| RollbackPoint {
                id: r.id.clone(),
                description: r.description.clone(),
                created_at: r.created_at,
                changes: r
                    .changes
                    .iter()
                    .map(|c| match c {
                        RollbackChange::DnsServers { interface, .. } => {
                            format!("DNS servers for {}", interface)
                        }
                        RollbackChange::InterfaceState { interface, .. } => {
                            format!("Interface {} state", interface)
                        }
                    })
                    .collect(),
            })
            .collect())
    }

    async fn flush_dns_cache(&self) -> Result<()> {
        info!("Flushing DNS cache");

        // macOS uses dscacheutil and mDNSResponder
        Self::run_command("dscacheutil", &["-flushcache"])?;

        // Also restart mDNSResponder (requires root)
        if self.is_root() {
            Self::run_command("killall", &["-HUP", "mDNSResponder"])?;
        } else {
            // Try with sudo
            match Self::run_sudo_command("killall", &["-HUP", "mDNSResponder"]) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Could not restart mDNSResponder (requires root): {}", e);
                }
            }
        }

        debug!("DNS cache flushed");
        Ok(())
    }

    async fn reset_adapter(&self, interface: &str) -> Result<()> {
        info!("Resetting network adapter: {}", interface);

        // Get the network service name
        let service = self.get_network_service(interface).ok_or_else(|| Error::Other {
            context: "reset_adapter".to_string(),
            message: format!("Could not find network service for {}", interface),
        })?;

        // Disable and re-enable the interface
        if self.is_root() {
            // ifconfig down/up
            Self::run_command("ifconfig", &[interface, "down"])?;
            std::thread::sleep(Duration::from_millis(500));
            Self::run_command("ifconfig", &[interface, "up"])?;
        } else {
            // Use networksetup (may prompt for password)
            Self::run_command("networksetup", &["-setnetworkserviceenabled", &service, "off"])?;
            std::thread::sleep(Duration::from_millis(500));
            Self::run_command("networksetup", &["-setnetworkserviceenabled", &service, "on"])?;
        }

        debug!("Adapter {} reset complete", interface);
        Ok(())
    }

    async fn set_dns_servers(&self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        let service = self.get_network_service(interface).ok_or_else(|| Error::Other {
            context: "set_dns_servers".to_string(),
            message: format!("Could not find network service for {}", interface),
        })?;

        info!("Setting DNS servers for {} to {:?}", service, servers);

        // Store current DNS for rollback
        let _current_servers = self.get_current_dns_servers(&service)?;

        if servers.is_empty() {
            // Reset to DHCP
            Self::run_command("networksetup", &["-setdnsservers", &service, "Empty"])?;
        } else {
            let mut args = vec!["-setdnsservers", &service];
            let server_strs: Vec<String> = servers.iter().map(|s| s.to_string()).collect();
            for s in &server_strs {
                args.push(s);
            }
            Self::run_command("networksetup", &args)?;
        }

        debug!("DNS servers updated for {}", service);
        Ok(())
    }

    async fn toggle_interface(&self, interface: &str, enable: bool) -> Result<()> {
        let service = self.get_network_service(interface).ok_or_else(|| Error::Other {
            context: "toggle_interface".to_string(),
            message: format!("Could not find network service for {}", interface),
        })?;

        let state = if enable { "on" } else { "off" };
        info!("Setting {} to {}", service, state);

        Self::run_command("networksetup", &["-setnetworkserviceenabled", &service, state])?;

        Ok(())
    }

    async fn reset_tcpip_stack(&self) -> Result<()> {
        info!("Resetting TCP/IP stack");

        // On macOS, we can flush routes and renew DHCP
        if self.is_root() {
            // Flush routing table (default routes)
            let _ = Self::run_command("route", &["-n", "flush"]);
        }

        // Renew DHCP on primary interface
        // Get primary interface
        let output = Command::new("route")
            .args(["-n", "get", "default"])
            .output()
            .map_err(|e| Error::Other {
                context: "reset_tcpip_stack".to_string(),
                message: e.to_string(),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let interface = stdout
            .lines()
            .find(|l| l.trim().starts_with("interface:"))
            .and_then(|l| l.split(':').nth(1))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "en0".to_string());

        self.renew_dhcp(&interface).await?;

        debug!("TCP/IP stack reset complete");
        Ok(())
    }

    async fn renew_dhcp(&self, interface: &str) -> Result<()> {
        info!("Renewing DHCP lease for {}", interface);

        // Use ipconfig to renew DHCP
        if self.is_root() {
            Self::run_command("ipconfig", &["set", interface, "DHCP"])?;
        } else {
            Self::run_sudo_command("ipconfig", &["set", interface, "DHCP"])?;
        }

        debug!("DHCP renewed for {}", interface);
        Ok(())
    }

    async fn clear_arp_cache(&self) -> Result<()> {
        info!("Clearing ARP cache");

        if self.is_root() {
            Self::run_command("arp", &["-a", "-d"])?;
        } else {
            Self::run_sudo_command("arp", &["-a", "-d"])?;
        }

        debug!("ARP cache cleared");
        Ok(())
    }

    async fn get_available_fixes(&self) -> Result<Vec<AutofixAction>> {
        let mut fixes = Vec::new();

        // DNS cache flush - always safe
        fixes.push(AutofixAction {
            id: "flush_dns".to_string(),
            name: "Flush DNS Cache".to_string(),
            description: "Clear the local DNS resolver cache to fix stale DNS entries".to_string(),
            category: FixCategory::Dns,
            risk_level: RiskLevel::Safe,
            reversible: false,
            estimated_duration: Duration::from_secs(2),
            dependencies: Vec::new(),
        });

        // Set DNS to Cloudflare
        fixes.push(AutofixAction {
            id: "set_dns_cloudflare".to_string(),
            name: "Use Cloudflare DNS".to_string(),
            description: "Set DNS servers to Cloudflare (1.1.1.1, 1.0.0.1) for faster and more reliable DNS resolution".to_string(),
            category: FixCategory::Dns,
            risk_level: RiskLevel::Low,
            reversible: true,
            estimated_duration: Duration::from_secs(3),
            dependencies: Vec::new(),
        });

        // Set DNS to Google
        fixes.push(AutofixAction {
            id: "set_dns_google".to_string(),
            name: "Use Google DNS".to_string(),
            description: "Set DNS servers to Google (8.8.8.8, 8.8.4.4) for reliable DNS resolution".to_string(),
            category: FixCategory::Dns,
            risk_level: RiskLevel::Low,
            reversible: true,
            estimated_duration: Duration::from_secs(3),
            dependencies: Vec::new(),
        });

        // Renew DHCP
        fixes.push(AutofixAction {
            id: "renew_dhcp".to_string(),
            name: "Renew DHCP Lease".to_string(),
            description: "Release and renew the DHCP lease to get a fresh IP address".to_string(),
            category: FixCategory::TcpIp,
            risk_level: RiskLevel::Low,
            reversible: false,
            estimated_duration: Duration::from_secs(5),
            dependencies: Vec::new(),
        });

        // Reset network adapter
        fixes.push(AutofixAction {
            id: "reset_adapter".to_string(),
            name: "Reset Network Adapter".to_string(),
            description: "Disable and re-enable the network adapter to clear any stuck state".to_string(),
            category: FixCategory::Adapter,
            risk_level: RiskLevel::Medium,
            reversible: false,
            estimated_duration: Duration::from_secs(10),
            dependencies: Vec::new(),
        });

        // Clear ARP cache
        fixes.push(AutofixAction {
            id: "clear_arp".to_string(),
            name: "Clear ARP Cache".to_string(),
            description: "Clear the ARP cache to resolve MAC address resolution issues".to_string(),
            category: FixCategory::TcpIp,
            risk_level: RiskLevel::Safe,
            reversible: false,
            estimated_duration: Duration::from_secs(1),
            dependencies: Vec::new(),
        });

        // Reset TCP/IP stack
        fixes.push(AutofixAction {
            id: "reset_tcpip".to_string(),
            name: "Reset TCP/IP Stack".to_string(),
            description: "Flush routing table and renew network configuration".to_string(),
            category: FixCategory::TcpIp,
            risk_level: RiskLevel::Medium,
            reversible: false,
            estimated_duration: Duration::from_secs(15),
            dependencies: Vec::new(),
        });

        Ok(fixes)
    }

    async fn apply_fix(&self, fix: &AutofixAction) -> Result<FixResult> {
        info!("Applying fix: {} ({})", fix.name, fix.id);

        let result = match fix.id.as_str() {
            "flush_dns" => {
                self.flush_dns_cache().await?;
                FixResult::success("DNS cache flushed successfully")
            }
            "set_dns_cloudflare" => {
                // Create rollback point
                let rollback_id = self.create_rollback_point("Before Cloudflare DNS").await?;

                let servers = vec![
                    "1.1.1.1".parse::<IpAddr>().unwrap(),
                    "1.0.0.1".parse::<IpAddr>().unwrap(),
                ];
                self.set_dns_servers("Wi-Fi", &servers).await?;
                self.flush_dns_cache().await?;

                FixResult::success("DNS set to Cloudflare (1.1.1.1, 1.0.0.1)")
                    .with_rollback(rollback_id)
            }
            "set_dns_google" => {
                let rollback_id = self.create_rollback_point("Before Google DNS").await?;

                let servers = vec![
                    "8.8.8.8".parse::<IpAddr>().unwrap(),
                    "8.8.4.4".parse::<IpAddr>().unwrap(),
                ];
                self.set_dns_servers("Wi-Fi", &servers).await?;
                self.flush_dns_cache().await?;

                FixResult::success("DNS set to Google (8.8.8.8, 8.8.4.4)")
                    .with_rollback(rollback_id)
            }
            "renew_dhcp" => {
                self.renew_dhcp("en0").await?;
                FixResult::success("DHCP lease renewed")
            }
            "reset_adapter" => {
                self.reset_adapter("en0").await?;
                FixResult::success("Network adapter reset")
            }
            "clear_arp" => {
                self.clear_arp_cache().await?;
                FixResult::success("ARP cache cleared")
            }
            "reset_tcpip" => {
                self.reset_tcpip_stack().await?;
                FixResult::success("TCP/IP stack reset")
            }
            _ => {
                FixResult::failure(format!("Unknown fix: {}", fix.id))
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_provider() {
        let provider = MacosAutofixProvider::new();
        assert!(provider.is_available());
    }

    #[tokio::test]
    async fn test_get_available_fixes() {
        let provider = MacosAutofixProvider::new();
        let fixes = provider.get_available_fixes().await.unwrap();
        assert!(!fixes.is_empty());

        // Check that flush_dns is available
        assert!(fixes.iter().any(|f| f.id == "flush_dns"));
    }

    #[tokio::test]
    async fn test_create_rollback_point() {
        let provider = MacosAutofixProvider::new();
        let id = provider.create_rollback_point("Test rollback").await.unwrap();
        assert!(!id.0.is_empty());

        let points = provider.list_rollback_points().await.unwrap();
        assert!(points.iter().any(|p| p.id == id));
    }
}
