//! Auto-fix command for network issues.

use crate::app::{FixArgs, FixCommands, OutputFormat};
use color_eyre::eyre::{eyre, Result};
use dialoguer::Confirm;
use netdiag_autofix::{
    actions::{FixAction, FixPlan, FixSeverity},
    engine::{AutofixConfig, AutofixEngine, NetworkIssue},
};
use std::sync::Arc;

#[cfg(target_os = "macos")]
use netdiag_platform_macos::create_providers;

#[cfg(target_os = "linux")]
use netdiag_platform_linux::create_providers;

#[cfg(target_os = "windows")]
use netdiag_platform_windows::create_providers;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn create_providers() -> netdiag_platform::PlatformProviders {
    netdiag_platform::PlatformProviders::new()
}

/// Runs the fix command.
pub async fn run(args: &FixArgs, format: &OutputFormat) -> Result<()> {
    let providers = Arc::new(create_providers());
    let config = AutofixConfig::default();
    let engine = AutofixEngine::new(providers.clone(), config);
    engine.init().await?;

    match &args.command {
        Some(FixCommands::Analyze { verbose }) => analyze_issues(&engine, *verbose, format).await,
        Some(FixCommands::Apply {
            dry_run,
            safe_only,
            yes,
        }) => apply_fixes(&engine, *dry_run, *safe_only, *yes, format).await,
        Some(FixCommands::FlushDns) => flush_dns(&engine, format).await,
        Some(FixCommands::ResetAdapter { interface }) => {
            reset_adapter(&engine, interface.as_deref(), format).await
        }
        Some(FixCommands::RenewDhcp { interface }) => {
            renew_dhcp(&engine, interface.as_deref(), format).await
        }
        Some(FixCommands::Rollbacks) => show_rollbacks(&engine, format).await,
        Some(FixCommands::Rollback { id }) => rollback(&engine, id, format).await,
        None => analyze_issues(&engine, false, format).await,
    }
}

/// Analyzes network issues and shows available fixes.
async fn analyze_issues(
    engine: &AutofixEngine,
    verbose: bool,
    format: &OutputFormat,
) -> Result<()> {
    println!("Analyzing network issues...\n");

    // Detect issues (in a real implementation, this would run actual diagnostics)
    let issues = detect_issues().await?;

    if issues.is_empty() {
        match format {
            OutputFormat::Json => {
                println!(r#"{{"status": "ok", "issues": [], "fixes": []}}"#);
            }
            _ => {
                println!("No network issues detected.");
            }
        }
        return Ok(());
    }

    // Plan fixes
    let plan = engine.plan_fixes(&issues);

    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&plan)?;
            println!("{}", json);
        }
        _ => {
            println!("Found {} potential issue(s):\n", issues.len());
            for (i, issue) in issues.iter().enumerate() {
                println!("  {}. {:?}", i + 1, issue);
            }

            println!("\nRecommended fixes ({}):\n", plan.actions.len());
            for action in &plan.actions {
                print_action(action, verbose);
            }

            println!("\nRun 'netdiag fix apply' to apply these fixes.");
            println!("Run 'netdiag fix apply --dry-run' to preview without applying.");
        }
    }

    Ok(())
}

/// Applies automatic fixes.
async fn apply_fixes(
    engine: &AutofixEngine,
    dry_run: bool,
    safe_only: bool,
    yes: bool,
    format: &OutputFormat,
) -> Result<()> {
    // Detect issues
    let issues = detect_issues().await?;

    if issues.is_empty() {
        println!("No network issues detected. Nothing to fix.");
        return Ok(());
    }

    // Plan fixes
    let mut plan = engine.plan_fixes(&issues);

    // Filter to safe-only if requested
    if safe_only {
        plan.actions
            .retain(|a| a.severity <= FixSeverity::Low);
    }

    if plan.is_empty() {
        println!("No applicable fixes found.");
        return Ok(());
    }

    // Show plan
    println!("Fix Plan ({} action(s)):\n", plan.actions.len());
    for action in &plan.actions {
        print_action(action, false);
    }

    // Confirm unless --yes or --dry-run
    if !dry_run && !yes {
        println!();
        let confirmed = Confirm::new()
            .with_prompt("Apply these fixes?")
            .default(false)
            .interact()?;

        if !confirmed {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Update dry_run flag
    let plan = FixPlan::new(plan.actions, dry_run);

    // Execute
    if dry_run {
        println!("\n[DRY RUN] - No changes will be made\n");
    }

    let results = engine.execute_with_rollback(&plan).await?;

    // Show results
    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&results)?;
            println!("{}", json);
        }
        _ => {
            println!("\nResults:\n");
            for result in &results {
                if result.success {
                    println!(
                        "  [OK] {} ({}ms)",
                        result
                            .message
                            .as_deref()
                            .unwrap_or("Success"),
                        result.duration_ms
                    );
                } else {
                    println!(
                        "  [FAILED] {}",
                        result.error.as_deref().unwrap_or("Unknown error")
                    );
                }
            }

            let success_count = results.iter().filter(|r| r.success).count();
            println!(
                "\n{}/{} fixes applied successfully.",
                success_count,
                results.len()
            );
        }
    }

    Ok(())
}

/// Flushes DNS cache.
async fn flush_dns(engine: &AutofixEngine, format: &OutputFormat) -> Result<()> {
    let action = FixAction::flush_dns_cache();
    let plan = FixPlan::new(vec![action], false);

    println!("Flushing DNS cache...");
    let results = engine.execute(&plan).await;

    if results.first().map(|r| r.success).unwrap_or(false) {
        match format {
            OutputFormat::Json => {
                println!(r#"{{"success": true, "message": "DNS cache flushed"}}"#);
            }
            _ => {
                println!("DNS cache flushed successfully.");
            }
        }
    } else {
        let error = results
            .first()
            .and_then(|r| r.error.as_ref())
            .map(|e| e.as_str())
            .unwrap_or("Unknown error");
        return Err(eyre!("Failed to flush DNS cache: {}", error));
    }

    Ok(())
}

/// Resets a network adapter.
async fn reset_adapter(
    engine: &AutofixEngine,
    interface: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let interface = interface
        .map(|s| s.to_string())
        .unwrap_or_else(|| get_default_interface());

    let action = FixAction::reset_adapter(interface.clone());
    let plan = FixPlan::new(vec![action], false);

    println!("Resetting adapter {}...", interface);
    let results = engine.execute(&plan).await;

    if results.first().map(|r| r.success).unwrap_or(false) {
        match format {
            OutputFormat::Json => {
                println!(
                    r#"{{"success": true, "message": "Adapter {} reset"}}"#,
                    interface
                );
            }
            _ => {
                println!("Adapter {} reset successfully.", interface);
            }
        }
    } else {
        let error = results
            .first()
            .and_then(|r| r.error.as_ref())
            .map(|e| e.as_str())
            .unwrap_or("Unknown error");
        return Err(eyre!("Failed to reset adapter: {}", error));
    }

    Ok(())
}

/// Renews DHCP lease.
async fn renew_dhcp(
    engine: &AutofixEngine,
    interface: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let interface = interface
        .map(|s| s.to_string())
        .unwrap_or_else(|| get_default_interface());

    let action = FixAction::renew_dhcp(interface.clone());
    let plan = FixPlan::new(vec![action], false);

    println!("Renewing DHCP lease on {}...", interface);
    let results = engine.execute(&plan).await;

    if results.first().map(|r| r.success).unwrap_or(false) {
        match format {
            OutputFormat::Json => {
                println!(
                    r#"{{"success": true, "message": "DHCP renewed on {}"}}"#,
                    interface
                );
            }
            _ => {
                println!("DHCP lease renewed successfully on {}.", interface);
            }
        }
    } else {
        let error = results
            .first()
            .and_then(|r| r.error.as_ref())
            .map(|e| e.as_str())
            .unwrap_or("Unknown error");
        return Err(eyre!("Failed to renew DHCP: {}", error));
    }

    Ok(())
}

/// Shows available rollback points.
async fn show_rollbacks(engine: &AutofixEngine, format: &OutputFormat) -> Result<()> {
    let rollbacks = engine.list_rollback_points().await;

    if rollbacks.is_empty() {
        match format {
            OutputFormat::Json => {
                println!(r#"{{"rollbacks": []}}"#);
            }
            _ => {
                println!("No rollback points available.");
            }
        }
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&rollbacks)?;
            println!("{}", json);
        }
        _ => {
            println!("Available Rollback Points:\n");
            for point in &rollbacks {
                println!("  ID: {}", point.id);
                println!("    Created: {}", point.created_at);
                println!("    Description: {}", point.description);
                println!("    Valid: {}", point.valid);
                println!();
            }
        }
    }

    Ok(())
}

/// Performs a rollback.
async fn rollback(engine: &AutofixEngine, id: &str, format: &OutputFormat) -> Result<()> {
    println!("Rolling back to {}...", id);

    engine.rollback(id).await?;

    match format {
        OutputFormat::Json => {
            println!(r#"{{"success": true, "message": "Rolled back to {}"}}"#, id);
        }
        _ => {
            println!("Successfully rolled back to {}.", id);
        }
    }

    Ok(())
}

/// Detects network issues (simplified implementation).
async fn detect_issues() -> Result<Vec<NetworkIssue>> {
    let mut issues = Vec::new();

    // Check DNS resolution
    if !test_dns_resolution().await {
        issues.push(NetworkIssue::DnsResolutionFailed);
    }

    // Check gateway connectivity
    // In a real implementation, would ping the gateway
    // if !test_gateway().await {
    //     issues.push(NetworkIssue::NoConnectivity { interface: None });
    // }

    Ok(issues)
}

/// Tests DNS resolution.
async fn test_dns_resolution() -> bool {
    use std::net::ToSocketAddrs;
    "google.com:443".to_socket_addrs().is_ok()
}

/// Gets the default network interface.
fn get_default_interface() -> String {
    #[cfg(target_os = "macos")]
    return "en0".to_string();

    #[cfg(target_os = "linux")]
    return "eth0".to_string();

    #[cfg(target_os = "windows")]
    return "Ethernet".to_string();

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    return "eth0".to_string();
}

/// Prints a fix action.
fn print_action(action: &FixAction, verbose: bool) {
    let severity_icon = match action.severity {
        FixSeverity::Low => "🟢",
        FixSeverity::Medium => "🟡",
        FixSeverity::High => "🟠",
        FixSeverity::Critical => "🔴",
    };

    println!(
        "  {} {} ({:?})",
        severity_icon, action.name, action.severity
    );
    if verbose {
        println!("      {}", action.description);
        println!("      Reversible: {}", action.reversible);
        println!("      Est. time: {}s", action.estimated_time_secs);
        if !action.prerequisites.is_empty() {
            println!("      Prerequisites: {:?}", action.prerequisites);
        }
    }
}
