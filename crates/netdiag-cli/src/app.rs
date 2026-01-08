//! CLI application definition using clap.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// netdiag - Comprehensive Network Diagnostics Tool
#[derive(Parser, Debug)]
#[command(
    name = "netdiag",
    author,
    version,
    about = "Comprehensive network diagnostics tool",
    long_about = "A powerful network diagnostics tool that helps troubleshoot network issues.\n\n\
                  Supports macOS, Linux, Windows, iOS, iPadOS, and Android.\n\
                  Combines ping, traceroute, speed tests, WiFi analysis, and more."
)]
pub struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Output format
    #[arg(long, value_enum, default_value = "text", global = true)]
    pub format: OutputFormat,

    /// Configuration file path
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Show system and network information
    Info,

    /// Run comprehensive network diagnostics
    Diagnose(DiagnoseArgs),

    /// Ping a host
    Ping(PingArgs),

    /// Trace route to a host
    Traceroute(TracerouteArgs),

    /// Run speed test
    Speed(SpeedArgs),

    /// WiFi analysis and diagnostics
    Wifi(WifiArgs),

    /// Generate diagnostic report
    Report(ReportArgs),

    /// Packet capture and analysis
    Capture(CaptureArgs),

    /// Launch terminal user interface
    Tui,

    /// Configuration management
    Config(ConfigArgs),

    /// Generate shell completions
    Completions(CompletionsArgs),

    /// Daemon service management
    Daemon(DaemonArgs),

    /// Auto-fix network issues
    Fix(FixArgs),
}

/// Output format options
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum OutputFormat {
    /// Human-readable text
    #[default]
    Text,
    /// JSON output
    Json,
    /// Compact output
    Compact,
}

/// Arguments for diagnose command
#[derive(Parser, Debug)]
pub struct DiagnoseArgs {
    /// Quick diagnostics (skip slow tests)
    #[arg(short, long)]
    pub quick: bool,

    /// Include speed test
    #[arg(long)]
    pub speed: bool,

    /// Include WiFi analysis
    #[arg(long)]
    pub wifi: bool,

    /// Include packet capture
    #[arg(long)]
    pub capture: bool,

    /// Attempt automatic fixes
    #[arg(long)]
    pub fix: bool,

    /// Save report to file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

/// Arguments for ping command
#[derive(Parser, Debug)]
pub struct PingArgs {
    /// Target host or IP address
    pub target: String,

    /// Number of pings to send
    #[arg(short, long, default_value = "10")]
    pub count: u32,

    /// Interval between pings in seconds
    #[arg(short, long, default_value = "1.0")]
    pub interval: f64,

    /// Timeout per ping in seconds
    #[arg(short, long, default_value = "5.0")]
    pub timeout: f64,

    /// Packet size in bytes
    #[arg(short, long, default_value = "64")]
    pub size: usize,
}

/// Arguments for traceroute command
#[derive(Parser, Debug)]
pub struct TracerouteArgs {
    /// Target host or IP address
    pub target: String,

    /// Maximum number of hops
    #[arg(short, long, default_value = "30")]
    pub max_hops: u8,

    /// Number of probes per hop
    #[arg(short, long, default_value = "3")]
    pub probes: u8,

    /// Timeout per probe in seconds
    #[arg(short, long, default_value = "5.0")]
    pub timeout: f64,

    /// Protocol to use
    #[arg(long, value_enum, default_value = "icmp")]
    pub protocol: TracerouteProtocol,

    /// Resolve hostnames
    #[arg(short = 'n', long)]
    pub no_resolve: bool,
}

/// Traceroute protocol options
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum TracerouteProtocol {
    #[default]
    Icmp,
    Udp,
    Tcp,
}

/// Arguments for speed test command
#[derive(Parser, Debug)]
pub struct SpeedArgs {
    /// Speed test server URL or ID
    #[arg(short, long)]
    pub server: Option<String>,

    /// Test duration in seconds
    #[arg(short, long, default_value = "10")]
    pub duration: u64,

    /// Number of parallel connections
    #[arg(short, long, default_value = "4")]
    pub connections: usize,

    /// Download only
    #[arg(long)]
    pub download_only: bool,

    /// Upload only
    #[arg(long)]
    pub upload_only: bool,

    /// Use iPerf3 server
    #[arg(long)]
    pub iperf: Option<String>,
}

/// Arguments for WiFi command
#[derive(Parser, Debug)]
pub struct WifiArgs {
    /// Subcommand
    #[command(subcommand)]
    pub command: Option<WifiCommands>,
}

/// WiFi subcommands
#[derive(Subcommand, Debug)]
pub enum WifiCommands {
    /// Scan for nearby networks
    Scan,
    /// Show current connection info
    Status,
    /// Analyze WiFi channels
    Channels,
    /// Check for interference
    Interference,
}

/// Arguments for report command
#[derive(Parser, Debug)]
pub struct ReportArgs {
    /// Report format (text, json, markdown, html, pdf)
    #[arg(short = 'F', long = "report-format", value_enum, default_value = "text")]
    pub report_format: ReportFormat,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Include raw data
    #[arg(long)]
    pub raw: bool,
}

/// Report format options
#[derive(ValueEnum, Clone, Debug, Default)]
pub enum ReportFormat {
    #[default]
    Text,
    Json,
    Markdown,
    Html,
    Pdf,
}

/// Arguments for capture command
#[derive(Parser, Debug)]
pub struct CaptureArgs {
    /// Network interface to capture on
    #[arg(short, long)]
    pub interface: Option<String>,

    /// BPF filter expression
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Number of packets to capture (0 = unlimited)
    #[arg(short, long, default_value = "100")]
    pub count: usize,

    /// Capture duration in seconds (0 = unlimited)
    #[arg(short, long, default_value = "0")]
    pub duration: u64,

    /// Enable promiscuous mode
    #[arg(short, long)]
    pub promiscuous: bool,

    /// Write capture to PCAP file
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// List available interfaces
    #[arg(long)]
    pub list_interfaces: bool,
}

/// Arguments for config command
#[derive(Parser, Debug)]
pub struct ConfigArgs {
    /// Subcommand
    #[command(subcommand)]
    pub command: Option<ConfigCommands>,
}

/// Config subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },
    /// Reset configuration to defaults
    Reset,
}

/// Arguments for completions command
#[derive(Parser, Debug)]
pub struct CompletionsArgs {
    /// Shell to generate completions for
    #[arg(value_enum)]
    pub shell: Shell,
}

/// Shell options for completions
#[derive(ValueEnum, Clone, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl From<Shell> for clap_complete::Shell {
    fn from(shell: Shell) -> Self {
        match shell {
            Shell::Bash => clap_complete::Shell::Bash,
            Shell::Zsh => clap_complete::Shell::Zsh,
            Shell::Fish => clap_complete::Shell::Fish,
            Shell::PowerShell => clap_complete::Shell::PowerShell,
            Shell::Elvish => clap_complete::Shell::Elvish,
        }
    }
}

/// Arguments for daemon command
#[derive(Parser, Debug)]
pub struct DaemonArgs {
    /// Subcommand
    #[command(subcommand)]
    pub command: Option<DaemonCommands>,
}

/// Daemon subcommands
#[derive(Subcommand, Debug)]
pub enum DaemonCommands {
    /// Start the daemon
    Start {
        /// Run in foreground (don't daemonize)
        #[arg(short, long)]
        foreground: bool,

        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Stop the daemon
    Stop,
    /// Restart the daemon
    Restart,
    /// Show daemon status
    Status,
    /// Install as system service
    Install,
    /// Uninstall system service
    Uninstall,
    /// Show daemon logs
    Logs {
        /// Number of lines to show
        #[arg(short, long, default_value = "50")]
        lines: usize,

        /// Follow log output
        #[arg(short, long)]
        follow: bool,
    },
}

/// Arguments for fix command
#[derive(Parser, Debug)]
pub struct FixArgs {
    /// Subcommand
    #[command(subcommand)]
    pub command: Option<FixCommands>,
}

/// Fix subcommands
#[derive(Subcommand, Debug)]
pub enum FixCommands {
    /// Analyze issues and show available fixes
    Analyze {
        /// Show detailed analysis
        #[arg(short, long)]
        verbose: bool,
    },
    /// Apply automatic fixes
    Apply {
        /// Dry run (show what would be fixed without applying)
        #[arg(short, long)]
        dry_run: bool,

        /// Only apply low-severity fixes
        #[arg(long)]
        safe_only: bool,

        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Flush DNS cache
    FlushDns,
    /// Reset network adapter
    ResetAdapter {
        /// Network interface name
        interface: Option<String>,
    },
    /// Renew DHCP lease
    RenewDhcp {
        /// Network interface name
        interface: Option<String>,
    },
    /// Show rollback points
    Rollbacks,
    /// Rollback to a previous state
    Rollback {
        /// Rollback point ID
        id: String,
    },
}
