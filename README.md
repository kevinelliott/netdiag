# netdiag

A comprehensive network diagnostics tool supporting macOS, Linux, Windows, iOS, iPadOS, and Android.

## Features

- **Multi-Platform Support**: Native implementations for macOS, Linux, Windows, iOS, and Android
- **Multiple Interfaces**: CLI, TUI (terminal UI), and GUI (desktop/mobile)
- **Network Diagnostics**: Ping, traceroute, DNS lookup, connectivity testing
- **WiFi Analysis**: Signal strength, channel analysis, interference detection
- **Speed Testing**: Multiple providers (Ookla, Cloudflare, iPerf3)
- **Packet Capture**: PCAP-based capture with protocol decoding
- **Auto-Fix**: Automatic remediation with rollback capability
- **Background Daemon**: Continuous monitoring and scheduled diagnostics
- **External Integrations**: Shodan, BGP looking glass, IPinfo, SSL Labs
- **Report Generation**: JSON, HTML, Markdown, PDF, text formats

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/kevinelliott/netdiag.git
cd netdiag

# Build CLI
cargo build --release --package netdiag-cli

# Install to system
cargo install --path crates/netdiag-cli
```

### Pre-built Binaries

Download from [Releases](https://github.com/kevinelliott/netdiag/releases).

## Usage

### CLI

```bash
# Show system and network info
netdiag info

# Run comprehensive diagnostics
netdiag diagnose

# Ping a host
netdiag ping google.com

# Traceroute
netdiag traceroute 8.8.8.8

# Speed test
netdiag speed

# WiFi analysis
netdiag wifi scan
netdiag wifi status

# DNS lookup
netdiag diagnose --quick

# Generate report
netdiag report --format html --output report.html

# Auto-fix issues
netdiag fix analyze
netdiag fix apply --dry-run
netdiag fix flush-dns

# Daemon management
netdiag daemon start
netdiag daemon status
netdiag daemon stop
```

### TUI (Terminal User Interface)

```bash
netdiag tui
```

### GUI (Desktop/Mobile)

The GUI application is built with Tauri and SvelteKit.

```bash
cd apps/tauri

# Development
pnpm install
pnpm tauri dev

# Build
pnpm tauri build
```

## Project Structure

```
netdiag/
├── crates/
│   ├── netdiag-types/          # Shared types and errors
│   ├── netdiag-platform/       # Platform abstraction traits
│   ├── netdiag-platform-macos/ # macOS implementations
│   ├── netdiag-platform-linux/ # Linux implementations
│   ├── netdiag-platform-ios/   # iOS implementations
│   ├── netdiag-platform-android/ # Android implementations
│   ├── netdiag-connectivity/   # Ping, traceroute, DNS
│   ├── netdiag-wifi/           # WiFi analysis
│   ├── netdiag-speed/          # Speed testing
│   ├── netdiag-capture/        # Packet capture
│   ├── netdiag-integrations/   # External API integrations
│   ├── netdiag-reports/        # Report generation
│   ├── netdiag-storage/        # SQLite storage
│   ├── netdiag-daemon/         # Background service
│   ├── netdiag-autofix/        # Auto-remediation
│   ├── netdiag-tui/            # Terminal UI
│   └── netdiag-cli/            # CLI binary
├── apps/
│   └── tauri/                  # Desktop/mobile GUI
└── docs/                       # Documentation
```

## Platform Support

| Platform | CLI | TUI | GUI | Status |
|----------|-----|-----|-----|--------|
| macOS    | Yes | Yes | Yes | Full support |
| Linux    | Yes | Yes | Yes | Full support |
| Windows  | Yes | Yes | Yes | Planned |
| iOS      | -   | -   | Yes | Requires Xcode |
| Android  | -   | -   | Yes | Requires Android SDK |

## Daemon Service

The daemon provides continuous network monitoring:

```bash
# Start daemon
netdiag daemon start

# Start in foreground
netdiag daemon start --foreground

# Install as system service
netdiag daemon install

# View logs
netdiag daemon logs -f
```

Configuration: `/etc/netdiag/daemon.toml` or `~/.config/netdiag/daemon.toml`

## Auto-Fix

Automatic remediation for common network issues:

```bash
# Analyze issues
netdiag fix analyze -v

# Apply fixes (with confirmation)
netdiag fix apply

# Apply safe fixes only
netdiag fix apply --safe-only

# Dry run
netdiag fix apply --dry-run

# Individual fixes
netdiag fix flush-dns
netdiag fix renew-dhcp en0
netdiag fix reset-adapter en0

# Rollback
netdiag fix rollbacks
netdiag fix rollback <id>
```

## Development

### Requirements

- Rust 1.75+
- For GUI: Node.js 18+, pnpm
- For iOS: Xcode, CocoaPods
- For Android: Android SDK, NDK

### Building

```bash
# Check all crates
cargo check --workspace

# Run tests
cargo test --workspace

# Build release
cargo build --release --workspace

# Run CLI
cargo run --package netdiag-cli -- info
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test --package netdiag-connectivity
```

## Configuration

Default configuration is stored in:
- macOS: `~/Library/Application Support/netdiag/config.toml`
- Linux: `~/.config/netdiag/config.toml`
- Windows: `%APPDATA%\netdiag\config.toml`

## License

MIT OR Apache-2.0

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Author

Kevin Elliott <kevin@kevinelliott.net>
