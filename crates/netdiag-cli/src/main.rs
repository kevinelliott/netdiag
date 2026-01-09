//! netdiag - Comprehensive Network Diagnostics Tool
//!
//! A command-line tool for diagnosing network issues across all platforms.

use clap::Parser;
use color_eyre::eyre::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod app;
mod commands;

use app::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logging
    init_logging(cli.verbose, cli.quiet);

    // Run the appropriate command
    match cli.command {
        Some(cmd) => run_command(cmd, &cli.format, cli.verbose).await,
        None => run_interactive().await,
    }
}

/// Initialize the logging/tracing system.
fn init_logging(verbose: u8, quiet: bool) {
    let filter = if quiet {
        "error"
    } else {
        match verbose {
            0 => "warn",
            1 => "info",
            2 => "debug",
            _ => "trace",
        }
    };

    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();
}

/// Run a specific command.
async fn run_command(command: Commands, format: &app::OutputFormat, verbose: u8) -> Result<()> {
    match command {
        Commands::Info => commands::info::run().await,
        Commands::Diagnose(mut args) => {
            args.verbose = verbose;
            commands::diagnose::run(args).await
        }
        Commands::Ping(args) => commands::ping::run(args).await,
        Commands::Traceroute(args) => commands::traceroute::run(args).await,
        Commands::Speed(args) => commands::speed::run(args).await,
        Commands::Wifi(args) => commands::wifi::run(args).await,
        Commands::Report(args) => commands::report::run(args).await,
        Commands::Capture(args) => commands::capture::run(args).await,
        Commands::Tui => commands::tui::run().await,
        Commands::Config(args) => commands::config::run(args).await,
        Commands::Completions(args) => commands::completions::run(args),
        Commands::Daemon(args) => commands::daemon::run(&args, format).await,
        Commands::Fix(args) => commands::fix::run(&args, format).await,
    }
}

/// Run in interactive mode.
async fn run_interactive() -> Result<()> {
    println!(
        "netdiag - Network Diagnostics Tool v{}",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("Use 'netdiag --help' for available commands.");
    println!("Use 'netdiag diagnose' for a full diagnostic run.");
    println!();

    // For now, run a quick info command
    commands::info::run().await
}
