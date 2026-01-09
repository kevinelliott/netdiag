//! Integration tests for the netdiag CLI.

use std::process::Command;

/// Helper to run the CLI and capture output.
fn run_netdiag(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--package", "netdiag-cli", "--"])
        .args(args)
        .output()
        .expect("Failed to execute netdiag CLI")
}

#[test]
fn test_cli_help() {
    let output = run_netdiag(&["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("netdiag"));
    assert!(stdout.contains("diagnose"));
    assert!(stdout.contains("ping"));
    assert!(stdout.contains("traceroute"));
}

#[test]
fn test_cli_version() {
    let output = run_netdiag(&["--version"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("netdiag"));
}

#[test]
fn test_cli_info() {
    let output = run_netdiag(&["info"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain system info
    assert!(
        stdout.contains("System") || stdout.contains("Network") || stdout.contains("Interface"),
        "Info output should contain system/network information"
    );
}

#[test]
fn test_cli_diagnose_help() {
    let output = run_netdiag(&["diagnose", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("diagnose"));
    assert!(stdout.contains("quick") || stdout.contains("comprehensive"));
}

#[test]
fn test_cli_ping_help() {
    let output = run_netdiag(&["ping", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ping"));
    assert!(stdout.contains("target") || stdout.contains("TARGET") || stdout.contains("host"));
}

#[test]
fn test_cli_traceroute_help() {
    let output = run_netdiag(&["traceroute", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("traceroute"));
}

#[test]
fn test_cli_speed_help() {
    let output = run_netdiag(&["speed", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("speed"));
}

#[test]
fn test_cli_wifi_help() {
    let output = run_netdiag(&["wifi", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("wifi") || stdout.contains("WiFi"));
}

#[test]
fn test_cli_fix_help() {
    let output = run_netdiag(&["fix", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("fix"));
    assert!(stdout.contains("analyze") || stdout.contains("apply") || stdout.contains("dns"));
}

#[test]
fn test_cli_daemon_help() {
    let output = run_netdiag(&["daemon", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("daemon"));
    assert!(stdout.contains("start") || stdout.contains("stop") || stdout.contains("status"));
}

#[test]
fn test_cli_report_help() {
    let output = run_netdiag(&["report", "--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("report"));
    assert!(stdout.contains("format") || stdout.contains("output"));
}

#[test]
fn test_cli_invalid_command() {
    let output = run_netdiag(&["invalid-command-that-does-not-exist"]);
    // Should fail with non-zero exit code
    assert!(!output.status.success());
}
