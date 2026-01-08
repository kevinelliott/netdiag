//! Autofix provider trait.

use async_trait::async_trait;
use netdiag_types::{error::Result, system::RollbackId};
use std::net::IpAddr;

/// Provider for automatic fix operations.
#[async_trait]
pub trait AutofixProvider: Send + Sync {
    /// Checks if autofix is available on this platform.
    fn is_available(&self) -> bool;

    /// Creates a rollback point before making changes.
    async fn create_rollback_point(&self, description: &str) -> Result<RollbackId>;

    /// Rolls back changes to a previous state.
    async fn rollback(&self, id: &RollbackId) -> Result<()>;

    /// Lists available rollback points.
    async fn list_rollback_points(&self) -> Result<Vec<RollbackPoint>>;

    /// Flushes the DNS cache.
    async fn flush_dns_cache(&self) -> Result<()>;

    /// Resets a network adapter.
    async fn reset_adapter(&self, interface: &str) -> Result<()>;

    /// Updates DNS servers for an interface.
    async fn set_dns_servers(&self, interface: &str, servers: &[IpAddr]) -> Result<()>;

    /// Enables or disables a network interface.
    async fn toggle_interface(&self, interface: &str, enable: bool) -> Result<()>;

    /// Resets the TCP/IP stack.
    async fn reset_tcpip_stack(&self) -> Result<()>;

    /// Releases and renews DHCP lease.
    async fn renew_dhcp(&self, interface: &str) -> Result<()>;

    /// Clears the ARP cache.
    async fn clear_arp_cache(&self) -> Result<()>;

    /// Gets available fixes for detected issues.
    async fn get_available_fixes(&self) -> Result<Vec<AutofixAction>>;

    /// Applies a specific fix.
    async fn apply_fix(&self, fix: &AutofixAction) -> Result<FixResult>;
}

/// Rollback point information.
#[derive(Debug, Clone)]
pub struct RollbackPoint {
    /// Rollback ID
    pub id: RollbackId,
    /// Description
    pub description: String,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// What was changed
    pub changes: Vec<String>,
}

/// An autofix action that can be applied.
#[derive(Debug, Clone)]
pub struct AutofixAction {
    /// Action ID
    pub id: String,
    /// Action name
    pub name: String,
    /// Description of what this fix does
    pub description: String,
    /// Category of fix
    pub category: FixCategory,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Whether this fix is reversible
    pub reversible: bool,
    /// Estimated time to apply
    pub estimated_duration: std::time::Duration,
    /// Dependencies (other fix IDs that must run first)
    pub dependencies: Vec<String>,
}

/// Category of autofix action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixCategory {
    /// DNS-related fixes
    Dns,
    /// Network adapter fixes
    Adapter,
    /// TCP/IP stack fixes
    TcpIp,
    /// WiFi-related fixes
    Wifi,
    /// Firewall fixes
    Firewall,
    /// Driver-related fixes
    Driver,
    /// System configuration fixes
    System,
}

/// Risk level of a fix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Safe - no risk of disruption
    Safe,
    /// Low - minor temporary disruption possible
    Low,
    /// Medium - may cause temporary connectivity loss
    Medium,
    /// High - significant changes, may require reboot
    High,
    /// Critical - major system changes
    Critical,
}

/// Result of applying a fix.
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Whether the fix was applied successfully
    pub success: bool,
    /// Message describing the result
    pub message: String,
    /// Rollback ID if applicable
    pub rollback_id: Option<RollbackId>,
    /// Any warnings
    pub warnings: Vec<String>,
    /// Whether a reboot is required
    pub reboot_required: bool,
}

impl FixResult {
    /// Creates a successful result.
    #[must_use]
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            rollback_id: None,
            warnings: Vec::new(),
            reboot_required: false,
        }
    }

    /// Creates a failure result.
    #[must_use]
    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            rollback_id: None,
            warnings: Vec::new(),
            reboot_required: false,
        }
    }

    /// Adds a rollback ID.
    #[must_use]
    pub fn with_rollback(mut self, id: RollbackId) -> Self {
        self.rollback_id = Some(id);
        self
    }
}
