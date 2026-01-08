//! Fix actions and their definitions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

/// Category of fix action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixCategory {
    /// DNS-related fixes.
    Dns,
    /// Network adapter fixes.
    Adapter,
    /// TCP/IP stack fixes.
    TcpIp,
    /// WiFi-related fixes.
    Wifi,
    /// Routing fixes.
    Routing,
    /// Firewall fixes.
    Firewall,
    /// Service-related fixes.
    Service,
}

/// Severity/impact level of a fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FixSeverity {
    /// Low impact, safe to apply.
    Low,
    /// Medium impact, may briefly disrupt connectivity.
    Medium,
    /// High impact, will disrupt connectivity.
    High,
    /// Critical impact, requires system restart.
    Critical,
}

/// A fix action that can be applied.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixAction {
    /// Unique identifier for this action.
    pub id: Uuid,
    /// Human-readable name.
    pub name: String,
    /// Description of what this fix does.
    pub description: String,
    /// Category of the fix.
    pub category: FixCategory,
    /// Severity/impact level.
    pub severity: FixSeverity,
    /// The specific fix to apply.
    pub fix_type: FixType,
    /// Whether this fix can be rolled back.
    pub reversible: bool,
    /// Estimated time to apply (seconds).
    pub estimated_time_secs: u32,
    /// Prerequisites for this fix.
    pub prerequisites: Vec<FixPrerequisite>,
}

impl FixAction {
    /// Creates a DNS cache flush action.
    pub fn flush_dns_cache() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Flush DNS Cache".to_string(),
            description: "Clears the DNS resolver cache to remove stale entries".to_string(),
            category: FixCategory::Dns,
            severity: FixSeverity::Low,
            fix_type: FixType::FlushDnsCache,
            reversible: false,
            estimated_time_secs: 1,
            prerequisites: vec![],
        }
    }

    /// Creates a DNS server change action.
    pub fn change_dns_servers(interface: String, servers: Vec<IpAddr>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Change DNS Servers".to_string(),
            description: format!("Changes DNS servers to {:?}", servers),
            category: FixCategory::Dns,
            severity: FixSeverity::Low,
            fix_type: FixType::SetDnsServers { interface, servers },
            reversible: true,
            estimated_time_secs: 2,
            prerequisites: vec![FixPrerequisite::AdminPrivileges],
        }
    }

    /// Creates a network adapter reset action.
    pub fn reset_adapter(interface: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: format!("Reset {}", interface),
            description: format!("Disables and re-enables the {} adapter", interface),
            category: FixCategory::Adapter,
            severity: FixSeverity::Medium,
            fix_type: FixType::ResetAdapter { interface },
            reversible: false,
            estimated_time_secs: 10,
            prerequisites: vec![FixPrerequisite::AdminPrivileges],
        }
    }

    /// Creates a TCP/IP stack reset action.
    pub fn reset_tcp_ip() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Reset TCP/IP Stack".to_string(),
            description: "Resets the TCP/IP networking stack to default settings".to_string(),
            category: FixCategory::TcpIp,
            severity: FixSeverity::High,
            fix_type: FixType::ResetTcpIp,
            reversible: false,
            estimated_time_secs: 5,
            prerequisites: vec![
                FixPrerequisite::AdminPrivileges,
                FixPrerequisite::RebootMayBeRequired,
            ],
        }
    }

    /// Creates a WiFi reconnect action.
    pub fn reconnect_wifi(interface: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Reconnect WiFi".to_string(),
            description: "Disconnects and reconnects to the current WiFi network".to_string(),
            category: FixCategory::Wifi,
            severity: FixSeverity::Medium,
            fix_type: FixType::ReconnectWifi { interface },
            reversible: false,
            estimated_time_secs: 15,
            prerequisites: vec![],
        }
    }

    /// Creates a release/renew DHCP action.
    pub fn renew_dhcp(interface: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Renew DHCP Lease".to_string(),
            description: "Releases and renews the DHCP lease to get a fresh IP".to_string(),
            category: FixCategory::Adapter,
            severity: FixSeverity::Medium,
            fix_type: FixType::RenewDhcp { interface },
            reversible: false,
            estimated_time_secs: 5,
            prerequisites: vec![],
        }
    }

    /// Creates a restart network service action.
    pub fn restart_network_service() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "Restart Network Service".to_string(),
            description: "Restarts the system network service".to_string(),
            category: FixCategory::Service,
            severity: FixSeverity::High,
            fix_type: FixType::RestartNetworkService,
            reversible: false,
            estimated_time_secs: 30,
            prerequisites: vec![FixPrerequisite::AdminPrivileges],
        }
    }
}

/// Specific type of fix to apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixType {
    /// Flush the DNS cache.
    FlushDnsCache,
    /// Set DNS servers for an interface.
    SetDnsServers {
        /// Interface name.
        interface: String,
        /// DNS servers to set.
        servers: Vec<IpAddr>,
    },
    /// Reset a network adapter.
    ResetAdapter {
        /// Interface name.
        interface: String,
    },
    /// Reset the TCP/IP stack.
    ResetTcpIp,
    /// Reconnect WiFi.
    ReconnectWifi {
        /// Interface name.
        interface: String,
    },
    /// Renew DHCP lease.
    RenewDhcp {
        /// Interface name.
        interface: String,
    },
    /// Restart the network service.
    RestartNetworkService,
    /// Clear ARP cache.
    ClearArpCache,
    /// Reset firewall rules.
    ResetFirewall,
    /// Custom command.
    CustomCommand {
        /// Command to execute.
        command: String,
        /// Arguments.
        args: Vec<String>,
    },
}

/// Prerequisite for applying a fix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixPrerequisite {
    /// Requires administrator/root privileges.
    AdminPrivileges,
    /// May require a system reboot to take effect.
    RebootMayBeRequired,
    /// Requires active network connection.
    NetworkConnection,
    /// Requires specific interface to exist.
    InterfaceExists(String),
}

/// Result of applying a fix.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixResult {
    /// The action that was applied.
    pub action_id: Uuid,
    /// Whether the fix was successful.
    pub success: bool,
    /// When the fix was applied.
    pub applied_at: DateTime<Utc>,
    /// Time taken to apply (milliseconds).
    pub duration_ms: u64,
    /// Output message.
    pub message: Option<String>,
    /// Error message if failed.
    pub error: Option<String>,
    /// Whether verification passed.
    pub verification_passed: Option<bool>,
    /// Rollback point ID if created.
    pub rollback_id: Option<String>,
}

impl FixResult {
    /// Creates a successful result.
    pub fn success(action_id: Uuid, duration_ms: u64, message: Option<String>) -> Self {
        Self {
            action_id,
            success: true,
            applied_at: Utc::now(),
            duration_ms,
            message,
            error: None,
            verification_passed: None,
            rollback_id: None,
        }
    }

    /// Creates a failed result.
    pub fn failure(action_id: Uuid, duration_ms: u64, error: String) -> Self {
        Self {
            action_id,
            success: false,
            applied_at: Utc::now(),
            duration_ms,
            message: None,
            error: Some(error),
            verification_passed: None,
            rollback_id: None,
        }
    }
}

/// A plan of fixes to apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixPlan {
    /// Unique plan ID.
    pub id: Uuid,
    /// Actions in the plan, in order.
    pub actions: Vec<FixAction>,
    /// When the plan was created.
    pub created_at: DateTime<Utc>,
    /// Total estimated time (seconds).
    pub estimated_total_time_secs: u32,
    /// Whether this is a dry run.
    pub dry_run: bool,
}

impl FixPlan {
    /// Creates a new fix plan.
    pub fn new(actions: Vec<FixAction>, dry_run: bool) -> Self {
        let estimated_total_time_secs = actions.iter().map(|a| a.estimated_time_secs).sum();
        Self {
            id: Uuid::new_v4(),
            actions,
            created_at: Utc::now(),
            estimated_total_time_secs,
            dry_run,
        }
    }

    /// Returns true if the plan is empty.
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }

    /// Returns the number of actions in the plan.
    pub fn len(&self) -> usize {
        self.actions.len()
    }
}
