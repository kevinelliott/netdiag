//! Linux autofix provider implementation.

use async_trait::async_trait;
use netdiag_platform::AutofixProvider;
use netdiag_types::error::{Error, Result};
use netdiag_types::system::RollbackId;
use std::collections::HashMap;
use std::fs;
use std::net::IpAddr;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tracing::{debug, info, warn};

/// Linux autofix provider.
pub struct LinuxAutofixProvider {
    rollback_counter: AtomicU64,
    rollback_data: Mutex<HashMap<u64, RollbackState>>,
}

/// State saved for rollback.
#[derive(Debug, Clone)]
struct RollbackState {
    resolv_conf_backup: Option<String>,
    interface_states: HashMap<String, bool>,
}

impl LinuxAutofixProvider {
    /// Creates a new Linux autofix provider.
    pub fn new() -> Self {
        Self {
            rollback_counter: AtomicU64::new(0),
            rollback_data: Mutex::new(HashMap::new()),
        }
    }

    /// Check if NetworkManager is running.
    fn is_networkmanager_running(&self) -> bool {
        Command::new("systemctl")
            .args(["is-active", "--quiet", "NetworkManager"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Check if systemd-resolved is running.
    fn is_resolved_running(&self) -> bool {
        Command::new("systemctl")
            .args(["is-active", "--quiet", "systemd-resolved"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Get current DNS servers from /etc/resolv.conf.
    fn get_current_dns(&self) -> Vec<IpAddr> {
        let mut servers = Vec::new();

        if let Ok(content) = fs::read_to_string("/etc/resolv.conf") {
            for line in content.lines() {
                if line.starts_with("nameserver") {
                    if let Some(addr) = line.split_whitespace().nth(1) {
                        if let Ok(ip) = addr.parse() {
                            servers.push(ip);
                        }
                    }
                }
            }
        }

        servers
    }
}

impl Default for LinuxAutofixProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AutofixProvider for LinuxAutofixProvider {
    async fn flush_dns_cache(&self) -> Result<()> {
        debug!("Flushing DNS cache on Linux");

        // Try systemd-resolved first
        if self.is_resolved_running() {
            let output = Command::new("resolvectl")
                .arg("flush-caches")
                .output()
                .map_err(|e| Error::platform("resolvectl", &e.to_string()))?;

            if output.status.success() {
                info!("DNS cache flushed via systemd-resolved");
                return Ok(());
            }

            // Fallback to systemd-resolve
            let output = Command::new("systemd-resolve")
                .arg("--flush-caches")
                .output();

            if let Ok(out) = output {
                if out.status.success() {
                    info!("DNS cache flushed via systemd-resolve");
                    return Ok(());
                }
            }
        }

        // Try nscd if available
        let output = Command::new("nscd").args(["-i", "hosts"]).output();

        if let Ok(out) = output {
            if out.status.success() {
                info!("DNS cache flushed via nscd");
                return Ok(());
            }
        }

        // Try restarting dnsmasq if running
        let output = Command::new("systemctl")
            .args(["is-active", "--quiet", "dnsmasq"])
            .status();

        if let Ok(status) = output {
            if status.success() {
                let restart = Command::new("systemctl")
                    .args(["restart", "dnsmasq"])
                    .output();

                if let Ok(out) = restart {
                    if out.status.success() {
                        info!("DNS cache flushed by restarting dnsmasq");
                        return Ok(());
                    }
                }
            }
        }

        warn!("No DNS cache service found to flush");
        Ok(())
    }

    async fn reset_adapter(&self, interface: &str) -> Result<()> {
        debug!("Resetting network adapter {} on Linux", interface);

        // Use ip command to bring interface down and up
        let down = Command::new("ip")
            .args(["link", "set", interface, "down"])
            .output()
            .map_err(|e| Error::platform("ip link down", &e.to_string()))?;

        if !down.status.success() {
            return Err(Error::platform(
                "ip link down",
                &String::from_utf8_lossy(&down.stderr),
            ));
        }

        // Wait a moment
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let up = Command::new("ip")
            .args(["link", "set", interface, "up"])
            .output()
            .map_err(|e| Error::platform("ip link up", &e.to_string()))?;

        if !up.status.success() {
            return Err(Error::platform(
                "ip link up",
                &String::from_utf8_lossy(&up.stderr),
            ));
        }

        info!("Network adapter {} reset successfully", interface);
        Ok(())
    }

    async fn renew_dhcp(&self, interface: &str) -> Result<()> {
        debug!("Renewing DHCP lease for {} on Linux", interface);

        // Try NetworkManager first
        if self.is_networkmanager_running() {
            let output = Command::new("nmcli")
                .args(["device", "reapply", interface])
                .output()
                .map_err(|e| Error::platform("nmcli reapply", &e.to_string()))?;

            if output.status.success() {
                info!("DHCP renewed via NetworkManager");
                return Ok(());
            }
        }

        // Try dhclient
        let release = Command::new("dhclient").args(["-r", interface]).output();

        if let Ok(out) = release {
            if out.status.success() {
                let renew = Command::new("dhclient")
                    .arg(interface)
                    .output()
                    .map_err(|e| Error::platform("dhclient", &e.to_string()))?;

                if renew.status.success() {
                    info!("DHCP renewed via dhclient");
                    return Ok(());
                }
            }
        }

        // Try dhcpcd
        let output = Command::new("dhcpcd").args(["-n", interface]).output();

        if let Ok(out) = output {
            if out.status.success() {
                info!("DHCP renewed via dhcpcd");
                return Ok(());
            }
        }

        Err(Error::platform(
            "DHCP renew",
            "No DHCP client available to renew lease",
        ))
    }

    async fn set_dns_servers(&self, interface: &str, servers: &[IpAddr]) -> Result<()> {
        debug!(
            "Setting DNS servers for {} on Linux: {:?}",
            interface, servers
        );

        // Try NetworkManager first
        if self.is_networkmanager_running() {
            let dns_str: Vec<String> = servers.iter().map(|s| s.to_string()).collect();

            let output = Command::new("nmcli")
                .args([
                    "connection",
                    "modify",
                    interface,
                    "ipv4.dns",
                    &dns_str.join(","),
                ])
                .output()
                .map_err(|e| Error::platform("nmcli", &e.to_string()))?;

            if output.status.success() {
                // Apply the changes
                let _ = Command::new("nmcli")
                    .args(["connection", "up", interface])
                    .output();

                info!("DNS servers set via NetworkManager");
                return Ok(());
            }
        }

        // Fallback: modify /etc/resolv.conf directly (requires root)
        let mut content = String::new();
        for server in servers {
            content.push_str(&format!("nameserver {}\n", server));
        }

        fs::write("/etc/resolv.conf", content)
            .map_err(|e| Error::platform("write resolv.conf", &e.to_string()))?;

        info!("DNS servers set via /etc/resolv.conf");
        Ok(())
    }

    async fn reset_tcpip_stack(&self) -> Result<()> {
        debug!("Resetting TCP/IP stack on Linux");

        // Flush routing cache
        let _ = Command::new("ip")
            .args(["route", "flush", "cache"])
            .output();

        // Reset connection tracking
        if std::path::Path::new("/proc/net/nf_conntrack").exists() {
            let _ = Command::new("conntrack").args(["-F"]).output();
        }

        // Restart networking service
        let output = Command::new("systemctl")
            .args(["restart", "networking"])
            .output();

        if output.is_err() || !output.as_ref().unwrap().status.success() {
            // Try NetworkManager instead
            let _ = Command::new("systemctl")
                .args(["restart", "NetworkManager"])
                .output();
        }

        info!("TCP/IP stack reset completed");
        Ok(())
    }

    async fn create_rollback_point(&self) -> Result<RollbackId> {
        debug!("Creating rollback point on Linux");

        let id = self.rollback_counter.fetch_add(1, Ordering::SeqCst);

        // Save current resolv.conf
        let resolv_conf_backup = fs::read_to_string("/etc/resolv.conf").ok();

        // Save interface states
        let mut interface_states = HashMap::new();
        if let Ok(entries) = fs::read_dir("/sys/class/net") {
            for entry in entries.flatten() {
                if let Ok(name) = entry.file_name().into_string() {
                    let operstate_path = format!("/sys/class/net/{}/operstate", name);
                    if let Ok(state) = fs::read_to_string(&operstate_path) {
                        interface_states.insert(name, state.trim() == "up");
                    }
                }
            }
        }

        let state = RollbackState {
            resolv_conf_backup,
            interface_states,
        };

        self.rollback_data.lock().unwrap().insert(id, state);

        info!("Rollback point {} created", id);
        Ok(RollbackId(id))
    }

    async fn rollback(&self, id: RollbackId) -> Result<()> {
        debug!("Rolling back to point {} on Linux", id.0);

        let state = self
            .rollback_data
            .lock()
            .unwrap()
            .remove(&id.0)
            .ok_or_else(|| Error::platform("rollback", "Rollback point not found"))?;

        // Restore resolv.conf
        if let Some(content) = state.resolv_conf_backup {
            let _ = fs::write("/etc/resolv.conf", content);
        }

        // Restore interface states
        for (iface, was_up) in state.interface_states {
            let action = if was_up { "up" } else { "down" };
            let _ = Command::new("ip")
                .args(["link", "set", &iface, action])
                .output();
        }

        info!("Rolled back to point {}", id.0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_rollback_point() {
        let provider = LinuxAutofixProvider::new();
        let id = provider.create_rollback_point().await.unwrap();
        assert_eq!(id.0, 0);

        let id2 = provider.create_rollback_point().await.unwrap();
        assert_eq!(id2.0, 1);
    }
}
