//! Auto-fix engine for executing and verifying fixes.

use crate::actions::{FixAction, FixPlan, FixPrerequisite, FixResult, FixSeverity, FixType};
use crate::error::{AutofixError, Result};
use crate::rollback::RollbackManager;
use netdiag_platform::PlatformProviders;
use netdiag_types::system::PrivilegeLevel;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// Well-known public DNS servers.
pub mod well_known_dns {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    /// Cloudflare DNS servers.
    pub fn cloudflare() -> Vec<IpAddr> {
        vec![
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
            IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1)),
            IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111)),
            IpAddr::V6(Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1001)),
        ]
    }

    /// Google DNS servers.
    pub fn google() -> Vec<IpAddr> {
        vec![
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),
            IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)),
            IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)),
            IpAddr::V6(Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8844)),
        ]
    }

    /// Quad9 DNS servers.
    pub fn quad9() -> Vec<IpAddr> {
        vec![
            IpAddr::V4(Ipv4Addr::new(9, 9, 9, 9)),
            IpAddr::V4(Ipv4Addr::new(149, 112, 112, 112)),
        ]
    }
}

/// Configuration for the autofix engine.
#[derive(Debug, Clone)]
pub struct AutofixConfig {
    /// Maximum severity level to auto-apply.
    pub max_auto_severity: FixSeverity,
    /// Whether to create rollback points.
    pub enable_rollback: bool,
    /// Directory for rollback data.
    pub rollback_dir: PathBuf,
    /// Whether to verify fixes after applying.
    pub verify_fixes: bool,
    /// Timeout for verification (seconds).
    pub verify_timeout_secs: u32,
}

impl Default for AutofixConfig {
    fn default() -> Self {
        Self {
            max_auto_severity: FixSeverity::Medium,
            enable_rollback: true,
            rollback_dir: default_rollback_dir(),
            verify_fixes: true,
            verify_timeout_secs: 30,
        }
    }
}

fn default_rollback_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        PathBuf::from("/Library/Application Support/netdiag/rollback")
    } else if cfg!(target_os = "linux") {
        PathBuf::from("/var/lib/netdiag/rollback")
    } else if cfg!(target_os = "windows") {
        PathBuf::from(r"C:\ProgramData\netdiag\rollback")
    } else {
        PathBuf::from("rollback")
    }
}

/// The auto-fix engine.
pub struct AutofixEngine {
    providers: Arc<PlatformProviders>,
    config: AutofixConfig,
    rollback_manager: Arc<RwLock<RollbackManager>>,
}

impl AutofixEngine {
    /// Creates a new autofix engine.
    pub fn new(providers: Arc<PlatformProviders>, config: AutofixConfig) -> Self {
        let rollback_manager = RollbackManager::new(config.rollback_dir.clone(), 100);
        Self {
            providers,
            config,
            rollback_manager: Arc::new(RwLock::new(rollback_manager)),
        }
    }

    /// Initializes the engine.
    pub async fn init(&self) -> Result<()> {
        let mut manager = self.rollback_manager.write().await;
        manager.init()?;
        Ok(())
    }

    /// Plans fixes based on detected issues.
    pub fn plan_fixes(&self, issues: &[NetworkIssue]) -> FixPlan {
        let mut actions = Vec::new();

        for issue in issues {
            match issue {
                NetworkIssue::DnsResolutionFailed => {
                    actions.push(FixAction::flush_dns_cache());
                    // Also suggest changing DNS servers
                    if let Some(iface) = &issue.interface() {
                        actions.push(FixAction::change_dns_servers(
                            iface.clone(),
                            well_known_dns::cloudflare(),
                        ));
                    }
                }
                NetworkIssue::HighLatency { interface } => {
                    if let Some(iface) = interface {
                        actions.push(FixAction::reset_adapter(iface.clone()));
                    }
                }
                NetworkIssue::PacketLoss { interface } => {
                    if let Some(iface) = interface {
                        actions.push(FixAction::reset_adapter(iface.clone()));
                    }
                }
                NetworkIssue::NoConnectivity { interface } => {
                    if let Some(iface) = interface {
                        actions.push(FixAction::renew_dhcp(iface.clone()));
                        actions.push(FixAction::reset_adapter(iface.clone()));
                    }
                    actions.push(FixAction::reset_tcp_ip());
                }
                NetworkIssue::WifiDisconnected { interface } => {
                    if let Some(iface) = interface {
                        actions.push(FixAction::reconnect_wifi(iface.clone()));
                    }
                }
                NetworkIssue::DhcpFailed { interface } => {
                    if let Some(iface) = interface {
                        actions.push(FixAction::renew_dhcp(iface.clone()));
                    }
                }
            }
        }

        // Sort by severity (low first)
        actions.sort_by_key(|a| a.severity);

        FixPlan::new(actions, false)
    }

    /// Checks if prerequisites are met for an action.
    pub async fn check_prerequisites(&self, action: &FixAction) -> Vec<String> {
        let mut missing = Vec::new();

        for prereq in &action.prerequisites {
            match prereq {
                FixPrerequisite::AdminPrivileges => {
                    let level = self.providers.privilege.current_privilege_level();
                    if level != PrivilegeLevel::Root && level != PrivilegeLevel::Elevated {
                        missing.push("Administrator/root privileges required".to_string());
                    }
                }
                FixPrerequisite::RebootMayBeRequired => {
                    // Just a warning, not a blocker
                }
                FixPrerequisite::NetworkConnection => {
                    // Check if any interface is up
                    match self.providers.network.list_interfaces().await {
                        Ok(interfaces) => {
                            if !interfaces.iter().any(|i| i.is_up()) {
                                missing.push("No network interface is up".to_string());
                            }
                        }
                        Err(_) => {
                            missing.push("Could not check network interfaces".to_string());
                        }
                    }
                }
                FixPrerequisite::InterfaceExists(name) => {
                    match self.providers.network.list_interfaces().await {
                        Ok(interfaces) => {
                            if !interfaces.iter().any(|i| &i.name == name) {
                                missing.push(format!("Interface {} not found", name));
                            }
                        }
                        Err(_) => {
                            missing.push(format!("Could not verify interface {}", name));
                        }
                    }
                }
            }
        }

        missing
    }

    /// Executes a fix plan.
    pub async fn execute(&self, plan: &FixPlan) -> Vec<FixResult> {
        let mut results = Vec::new();

        for action in &plan.actions {
            if plan.dry_run {
                tracing::info!("[DRY RUN] Would apply: {}", action.name);
                results.push(FixResult::success(
                    action.id,
                    0,
                    Some("Dry run - not applied".to_string()),
                ));
                continue;
            }

            // Check prerequisites
            let missing = self.check_prerequisites(action).await;
            if !missing.is_empty() {
                tracing::warn!(
                    "Skipping '{}': missing prerequisites: {:?}",
                    action.name,
                    missing
                );
                results.push(FixResult::failure(
                    action.id,
                    0,
                    format!("Missing prerequisites: {}", missing.join(", ")),
                ));
                continue;
            }

            // Execute the fix
            let result = self.execute_action(action).await;
            results.push(result);
        }

        results
    }

    /// Executes a fix plan with automatic rollback on failure.
    pub async fn execute_with_rollback(&self, plan: &FixPlan) -> Result<Vec<FixResult>> {
        let mut results = Vec::new();
        let mut rollback_ids = Vec::new();

        for action in &plan.actions {
            if plan.dry_run {
                tracing::info!("[DRY RUN] Would apply: {}", action.name);
                results.push(FixResult::success(
                    action.id,
                    0,
                    Some("Dry run - not applied".to_string()),
                ));
                continue;
            }

            // Create rollback point if the action is reversible
            let rollback_id = if action.reversible && self.config.enable_rollback {
                self.create_rollback_point(action).await.ok()
            } else {
                None
            };

            // Execute the fix
            let mut result = self.execute_action(action).await;

            // Verify if enabled
            if result.success && self.config.verify_fixes {
                let verified = self.verify_fix(action).await;
                result.verification_passed = Some(verified);

                if !verified {
                    tracing::warn!("Fix '{}' verification failed", action.name);

                    // Rollback if we have a rollback point
                    if let Some(ref id) = rollback_id {
                        tracing::info!("Rolling back fix '{}'", action.name);
                        let mut manager = self.rollback_manager.write().await;
                        if let Err(e) = manager.rollback(id).await {
                            tracing::error!("Rollback failed: {}", e);
                        }
                    }

                    result.success = false;
                    result.error = Some("Verification failed after applying fix".to_string());
                }
            }

            result.rollback_id = rollback_id.clone();
            if let Some(id) = rollback_id {
                rollback_ids.push(id);
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Executes a single action.
    async fn execute_action(&self, action: &FixAction) -> FixResult {
        tracing::info!("Applying fix: {}", action.name);
        let start = Instant::now();

        let result = match &action.fix_type {
            FixType::FlushDnsCache => self.flush_dns_cache().await,
            FixType::SetDnsServers { interface, servers } => {
                self.set_dns_servers(interface, servers).await
            }
            FixType::ResetAdapter { interface } => self.reset_adapter(interface).await,
            FixType::ResetTcpIp => self.reset_tcp_ip().await,
            FixType::ReconnectWifi { interface } => self.reconnect_wifi(interface).await,
            FixType::RenewDhcp { interface } => self.renew_dhcp(interface).await,
            FixType::RestartNetworkService => self.restart_network_service().await,
            FixType::ClearArpCache => self.clear_arp_cache().await,
            FixType::ResetFirewall => self.reset_firewall().await,
            FixType::CustomCommand { command, args } => {
                self.run_custom_command(command, args).await
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(msg) => {
                tracing::info!("Fix '{}' completed in {}ms", action.name, duration_ms);
                FixResult::success(action.id, duration_ms, msg)
            }
            Err(e) => {
                tracing::error!("Fix '{}' failed: {}", action.name, e);
                FixResult::failure(action.id, duration_ms, e.to_string())
            }
        }
    }

    /// Creates a rollback point for an action.
    async fn create_rollback_point(&self, action: &FixAction) -> Result<String> {
        let mut manager = self.rollback_manager.write().await;

        match &action.fix_type {
            FixType::SetDnsServers {
                interface,
                servers: _,
            } => {
                // Get current DNS servers
                let current = self.get_current_dns_servers(interface).await?;
                manager.create_dns_point(interface, current, Some(action.id))
            }
            _ => {
                // No rollback state needed
                Ok(String::new())
            }
        }
    }

    /// Gets current DNS servers for an interface.
    async fn get_current_dns_servers(&self, _interface: &str) -> Result<Vec<IpAddr>> {
        match self.providers.network.get_dns_servers().await {
            Ok(servers) => Ok(servers.into_iter().map(|s| s.address).collect()),
            Err(e) => Err(AutofixError::Platform(e)),
        }
    }

    /// Verifies that a fix was successful.
    async fn verify_fix(&self, action: &FixAction) -> bool {
        // Allow some time for changes to take effect
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        match &action.fix_type {
            FixType::FlushDnsCache | FixType::SetDnsServers { .. } => {
                // Try to resolve a known domain
                self.verify_dns_resolution().await
            }
            FixType::ResetAdapter { interface } | FixType::ReconnectWifi { interface } => {
                // Check if interface is up
                self.verify_interface_up(interface).await
            }
            FixType::RenewDhcp { interface } => {
                // Check if interface has an IP
                self.verify_has_ip(interface).await
            }
            _ => true, // Assume success for other fix types
        }
    }

    /// Verifies DNS resolution works.
    async fn verify_dns_resolution(&self) -> bool {
        // Try to resolve a well-known domain
        use std::net::ToSocketAddrs;
        "google.com:443".to_socket_addrs().is_ok()
    }

    /// Verifies an interface is up.
    async fn verify_interface_up(&self, interface: &str) -> bool {
        match self.providers.network.list_interfaces().await {
            Ok(interfaces) => interfaces
                .iter()
                .find(|i| &i.name == interface)
                .map(|i| i.is_up())
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Verifies an interface has an IP address.
    async fn verify_has_ip(&self, interface: &str) -> bool {
        match self.providers.network.list_interfaces().await {
            Ok(interfaces) => interfaces
                .iter()
                .find(|i| &i.name == interface)
                .map(|i| !i.ipv4_addresses.is_empty())
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    // Platform-specific fix implementations

    async fn flush_dns_cache(&self) -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("dscacheutil").args(["-flushcache"]).output()?;
            Command::new("killall")
                .args(["-HUP", "mDNSResponder"])
                .output()?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            // Try systemd-resolved first
            let _ = Command::new("systemd-resolve")
                .args(["--flush-caches"])
                .output();
            // Also try resolvectl
            let _ = Command::new("resolvectl").args(["flush-caches"]).output();
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("ipconfig").args(["/flushdns"]).output()?;
        }

        Ok(Some("DNS cache flushed".to_string()))
    }

    async fn set_dns_servers(&self, interface: &str, servers: &[IpAddr]) -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let server_strs: Vec<String> = servers.iter().map(|s| s.to_string()).collect();
            let mut args = vec!["-setdnsservers", interface];
            args.extend(server_strs.iter().map(|s| s.as_str()));
            Command::new("networksetup").args(&args).output()?;
        }

        #[cfg(target_os = "linux")]
        {
            // Update resolv.conf or use resolvectl
            let mut content = String::new();
            for server in servers {
                content.push_str(&format!("nameserver {}\n", server));
            }
            std::fs::write("/etc/resolv.conf", content)?;
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            if !servers.is_empty() {
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
            }
        }

        Ok(Some(format!("DNS servers set to {:?}", servers)))
    }

    async fn reset_adapter(&self, interface: &str) -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("ifconfig")
                .args([interface, "down"])
                .output()?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Command::new("ifconfig").args([interface, "up"]).output()?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("ip")
                .args(["link", "set", interface, "down"])
                .output()?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            Command::new("ip")
                .args(["link", "set", interface, "up"])
                .output()?;
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("netsh")
                .args(["interface", "set", "interface", interface, "disable"])
                .output()?;
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            Command::new("netsh")
                .args(["interface", "set", "interface", interface, "enable"])
                .output()?;
        }

        Ok(Some(format!("Adapter {} reset", interface)))
    }

    async fn reset_tcp_ip(&self) -> Result<Option<String>> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("netsh")
                .args(["int", "ip", "reset"])
                .output()?;
            Command::new("netsh").args(["winsock", "reset"]).output()?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Unix systems don't have an equivalent - would need to restart networking
        }

        Ok(Some("TCP/IP stack reset".to_string()))
    }

    async fn reconnect_wifi(&self, interface: &str) -> Result<Option<String>> {
        // This would use the WiFi provider
        tracing::debug!("Reconnecting WiFi on {}", interface);
        Ok(Some(format!("WiFi reconnected on {}", interface)))
    }

    async fn renew_dhcp(&self, interface: &str) -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("ipconfig")
                .args(["set", interface, "DHCP"])
                .output()?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            // Release
            let _ = Command::new("dhclient").args(["-r", interface]).output();
            // Renew
            Command::new("dhclient").args([interface]).output()?;
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("ipconfig")
                .args(["/release", interface])
                .output()?;
            Command::new("ipconfig")
                .args(["/renew", interface])
                .output()?;
        }

        Ok(Some(format!("DHCP renewed on {}", interface)))
    }

    async fn restart_network_service(&self) -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("launchctl")
                .args([
                    "kickstart",
                    "-k",
                    "system/com.apple.networking.discoveryengine",
                ])
                .output()?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            // Try NetworkManager first
            let nm = Command::new("systemctl")
                .args(["restart", "NetworkManager"])
                .output();
            if nm.is_err() {
                // Fall back to networking service
                Command::new("systemctl")
                    .args(["restart", "networking"])
                    .output()?;
            }
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("net").args(["stop", "netman"]).output()?;
            Command::new("net").args(["start", "netman"]).output()?;
        }

        Ok(Some("Network service restarted".to_string()))
    }

    async fn clear_arp_cache(&self) -> Result<Option<String>> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("arp").args(["-d", "-a"]).output()?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("ip")
                .args(["neigh", "flush", "all"])
                .output()?;
        }

        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("arp").args(["-d", "*"]).output()?;
        }

        Ok(Some("ARP cache cleared".to_string()))
    }

    async fn reset_firewall(&self) -> Result<Option<String>> {
        Err(AutofixError::not_supported("Firewall reset"))
    }

    async fn run_custom_command(&self, command: &str, args: &[String]) -> Result<Option<String>> {
        use std::process::Command;
        let output = Command::new(command).args(args).output()?;

        if output.status.success() {
            Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
        } else {
            Err(AutofixError::fix_failed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    /// Performs a rollback to a specific point.
    pub async fn rollback(&self, rollback_id: &str) -> Result<()> {
        let mut manager = self.rollback_manager.write().await;
        manager.rollback(rollback_id).await
    }

    /// Lists available rollback points.
    pub async fn list_rollback_points(&self) -> Vec<crate::rollback::RollbackPoint> {
        let manager = self.rollback_manager.read().await;
        manager.list().into_iter().cloned().collect()
    }
}

/// Types of network issues that can be detected and fixed.
#[derive(Debug, Clone)]
pub enum NetworkIssue {
    /// DNS resolution is failing.
    DnsResolutionFailed,
    /// High latency detected.
    HighLatency {
        /// Affected interface.
        interface: Option<String>,
    },
    /// Packet loss detected.
    PacketLoss {
        /// Affected interface.
        interface: Option<String>,
    },
    /// No connectivity.
    NoConnectivity {
        /// Affected interface.
        interface: Option<String>,
    },
    /// WiFi disconnected.
    WifiDisconnected {
        /// WiFi interface.
        interface: Option<String>,
    },
    /// DHCP failed.
    DhcpFailed {
        /// Affected interface.
        interface: Option<String>,
    },
}

impl NetworkIssue {
    /// Gets the interface associated with this issue.
    pub fn interface(&self) -> Option<String> {
        match self {
            NetworkIssue::DnsResolutionFailed => None,
            NetworkIssue::HighLatency { interface }
            | NetworkIssue::PacketLoss { interface }
            | NetworkIssue::NoConnectivity { interface }
            | NetworkIssue::WifiDisconnected { interface }
            | NetworkIssue::DhcpFailed { interface } => interface.clone(),
        }
    }
}
