//! Daemon service management command.

use crate::app::{DaemonArgs, DaemonCommands, OutputFormat};
use color_eyre::eyre::{eyre, Result};
use netdiag_daemon::{
    config::DaemonConfig,
    ipc::{IpcClient, IpcRequest, IpcResponse},
    service::DaemonService,
};
use std::path::PathBuf;

/// Runs the daemon command.
pub async fn run(args: &DaemonArgs, format: &OutputFormat) -> Result<()> {
    match &args.command {
        Some(DaemonCommands::Start { foreground, config }) => {
            start_daemon(*foreground, config.as_ref(), format).await
        }
        Some(DaemonCommands::Stop) => stop_daemon(format).await,
        Some(DaemonCommands::Restart) => {
            stop_daemon(format).await?;
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            start_daemon(false, None, format).await
        }
        Some(DaemonCommands::Status) => show_status(format).await,
        Some(DaemonCommands::Install) => install_service(format).await,
        Some(DaemonCommands::Uninstall) => uninstall_service(format).await,
        Some(DaemonCommands::Logs { lines, follow }) => show_logs(*lines, *follow, format).await,
        None => show_status(format).await,
    }
}

/// Starts the daemon.
async fn start_daemon(
    foreground: bool,
    config_path: Option<&PathBuf>,
    _format: &OutputFormat,
) -> Result<()> {
    // Load configuration
    let config = if let Some(path) = config_path {
        DaemonConfig::load(path)?
    } else {
        // Try user config, then system config, then default
        let user_path = DaemonConfig::user_path();
        let system_path = DaemonConfig::default_path();

        if let Some(path) = user_path.filter(|p| p.exists()) {
            DaemonConfig::load(&path)?
        } else if system_path.exists() {
            DaemonConfig::load(&system_path)?
        } else {
            DaemonConfig::default()
        }
    };

    // Check if already running
    if let Some(pid_file) = &config.general.pid_file {
        if let Some(pid) = DaemonService::is_already_running(pid_file) {
            return Err(eyre!("Daemon is already running (PID: {})", pid));
        }
    }

    // Daemonize if not running in foreground
    if !foreground {
        #[cfg(unix)]
        {
            println!("Starting daemon...");
            netdiag_daemon::service::daemonize()?;
        }
        #[cfg(windows)]
        {
            return Err(eyre!(
                "On Windows, use 'netdiag daemon install' to run as a service"
            ));
        }
    }

    // Create and start the service
    let mut service = DaemonService::new(config);
    service.start().await?;

    if foreground {
        println!("Daemon running in foreground (Ctrl+C to stop)");
        service.run().await?;
        service.stop().await?;
    }

    Ok(())
}

/// Stops the daemon.
async fn stop_daemon(_format: &OutputFormat) -> Result<()> {
    let config = DaemonConfig::default();
    let mut client = IpcClient::new(config.ipc.socket_path.clone());

    if client.connect().await.is_err() {
        return Err(eyre!("Daemon is not running"));
    }

    let response = client.request(&IpcRequest::Stop).await?;
    match response {
        IpcResponse::Ok { message } => {
            println!("Daemon stopping: {}", message.unwrap_or_default());
            Ok(())
        }
        IpcResponse::Error { message } => Err(eyre!("Failed to stop daemon: {}", message)),
        _ => Err(eyre!("Unexpected response from daemon")),
    }
}

/// Shows daemon status.
async fn show_status(format: &OutputFormat) -> Result<()> {
    let config = DaemonConfig::default();
    let mut client = IpcClient::new(config.ipc.socket_path.clone());

    if client.connect().await.is_err() {
        match format {
            OutputFormat::Json => {
                println!(r#"{{"status": "stopped", "running": false}}"#);
            }
            _ => {
                println!("Daemon Status: Stopped");
                println!("  The daemon is not currently running.");
            }
        }
        return Ok(());
    }

    let response = client.request(&IpcRequest::Status).await?;
    match response {
        IpcResponse::Status {
            state,
            uptime_secs,
            diagnostics_run,
            monitoring_active,
        } => {
            match format {
                OutputFormat::Json => {
                    println!(
                        r#"{{"status": "{:?}", "running": true, "uptime_secs": {}, "diagnostics_run": {}, "monitoring_active": {}}}"#,
                        state, uptime_secs, diagnostics_run, monitoring_active
                    );
                }
                _ => {
                    println!("Daemon Status: {:?}", state);
                    println!("  Uptime: {}s", uptime_secs);
                    println!("  Diagnostics Run: {}", diagnostics_run);
                    println!(
                        "  Monitoring: {}",
                        if monitoring_active {
                            "Active"
                        } else {
                            "Paused"
                        }
                    );
                }
            }
            Ok(())
        }
        IpcResponse::Error { message } => Err(eyre!("Failed to get status: {}", message)),
        _ => Err(eyre!("Unexpected response from daemon")),
    }
}

/// Installs the daemon as a system service.
async fn install_service(_format: &OutputFormat) -> Result<()> {
    let config = DaemonConfig::default();

    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        netdiag_daemon::service::install_service(&config)?;
        println!("Service installed successfully.");
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(eyre!("Service installation not supported on this platform"))
    }
}

/// Uninstalls the daemon system service.
async fn uninstall_service(_format: &OutputFormat) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        let plist_path = "/Library/LaunchDaemons/net.netdiag.daemon.plist";
        if std::path::Path::new(plist_path).exists() {
            std::process::Command::new("launchctl")
                .args(["unload", plist_path])
                .output()?;
            std::fs::remove_file(plist_path)?;
            println!("Service uninstalled successfully.");
        } else {
            println!("Service is not installed.");
        }
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        let service_path = "/etc/systemd/system/netdiag.service";
        if std::path::Path::new(service_path).exists() {
            std::process::Command::new("systemctl")
                .args(["stop", "netdiag"])
                .output()?;
            std::process::Command::new("systemctl")
                .args(["disable", "netdiag"])
                .output()?;
            std::fs::remove_file(service_path)?;
            std::process::Command::new("systemctl")
                .args(["daemon-reload"])
                .output()?;
            println!("Service uninstalled successfully.");
        } else {
            println!("Service is not installed.");
        }
        Ok(())
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(eyre!(
            "Service uninstallation not supported on this platform"
        ))
    }
}

/// Shows daemon logs.
async fn show_logs(lines: usize, follow: bool, _format: &OutputFormat) -> Result<()> {
    let config = DaemonConfig::default();
    let log_file = config
        .general
        .log_file
        .as_ref()
        .ok_or_else(|| eyre!("No log file configured"))?;

    if !log_file.exists() {
        return Err(eyre!("Log file does not exist: {:?}", log_file));
    }

    if follow {
        // Use tail -f
        let mut cmd = std::process::Command::new("tail")
            .args(["-f", "-n", &lines.to_string()])
            .arg(log_file)
            .spawn()?;
        cmd.wait()?;
    } else {
        // Use tail
        let output = std::process::Command::new("tail")
            .args(["-n", &lines.to_string()])
            .arg(log_file)
            .output()?;
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }

    Ok(())
}
