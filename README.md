<p align="center">
  <img src="docs/images/logo.png" alt="NetDiag Logo" width="120" height="120" />
</p>

<h1 align="center">NetDiag</h1>

<p align="center">
  <strong>A comprehensive, cross-platform network diagnostics toolkit</strong>
</p>

<p align="center">
  <a href="#features">Features</a> •
  <a href="#installation">Installation</a> •
  <a href="#quick-start">Quick Start</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#contributing">Contributing</a>
</p>

<!-- CI & Build Status -->
<p align="center">
  <a href="https://github.com/kevinelliott/netdiag/actions/workflows/ci.yml">
    <img src="https://github.com/kevinelliott/netdiag/actions/workflows/ci.yml/badge.svg" alt="CI" />
  </a>
  <a href="https://github.com/kevinelliott/netdiag/actions/workflows/release.yml">
    <img src="https://github.com/kevinelliott/netdiag/actions/workflows/release.yml/badge.svg" alt="Release" />
  </a>
  <a href="https://codecov.io/gh/kevinelliott/netdiag">
    <img src="https://codecov.io/gh/kevinelliott/netdiag/branch/main/graph/badge.svg" alt="Code Coverage" />
  </a>
  <a href="https://github.com/kevinelliott/netdiag/actions/workflows/security.yml">
    <img src="https://github.com/kevinelliott/netdiag/actions/workflows/security.yml/badge.svg" alt="Security Audit" />
  </a>
</p>

<!-- Package & Version -->
<p align="center">
  <a href="https://crates.io/crates/netdiag-cli">
    <img src="https://img.shields.io/crates/v/netdiag-cli.svg" alt="Crates.io" />
  </a>
  <a href="https://crates.io/crates/netdiag-cli">
    <img src="https://img.shields.io/crates/d/netdiag-cli.svg" alt="Downloads" />
  </a>
  <a href="https://github.com/kevinelliott/netdiag/releases/latest">
    <img src="https://img.shields.io/github/v/release/kevinelliott/netdiag?include_prereleases" alt="Latest Release" />
  </a>
  <a href="https://docs.rs/netdiag-cli">
    <img src="https://img.shields.io/docsrs/netdiag-cli" alt="docs.rs" />
  </a>
</p>

<!-- Project Info -->
<p align="center">
  <img src="https://img.shields.io/badge/rust-1.75+-orange.svg" alt="Rust 1.75+" />
  <a href="https://github.com/kevinelliott/netdiag/blob/main/LICENSE-MIT">
    <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License" />
  </a>
  <img src="https://img.shields.io/badge/platforms-macOS%20%7C%20Linux%20%7C%20Windows%20%7C%20iOS%20%7C%20Android-lightgrey.svg" alt="Platforms" />
</p>

<!-- Community & Activity -->
<p align="center">
  <a href="https://github.com/kevinelliott/netdiag/stargazers">
    <img src="https://img.shields.io/github/stars/kevinelliott/netdiag?style=flat" alt="GitHub Stars" />
  </a>
  <a href="https://github.com/kevinelliott/netdiag/network/members">
    <img src="https://img.shields.io/github/forks/kevinelliott/netdiag?style=flat" alt="GitHub Forks" />
  </a>
  <a href="https://github.com/kevinelliott/netdiag/issues">
    <img src="https://img.shields.io/github/issues/kevinelliott/netdiag" alt="GitHub Issues" />
  </a>
  <a href="https://github.com/kevinelliott/netdiag/graphs/contributors">
    <img src="https://img.shields.io/github/contributors/kevinelliott/netdiag" alt="Contributors" />
  </a>
  <a href="https://github.com/kevinelliott/netdiag/commits/main">
    <img src="https://img.shields.io/github/last-commit/kevinelliott/netdiag" alt="Last Commit" />
  </a>
</p>

---

## Overview

NetDiag is a modern, feature-rich network diagnostics tool that provides deep visibility into your network health across all major platforms. Whether you're a network administrator troubleshooting connectivity issues, a developer debugging network problems, or an end user wanting to understand your connection quality, NetDiag offers the tools you need.

**Key Highlights:**
- **Multi-Platform**: Native support for macOS, Linux, Windows, iOS, and Android
- **Multiple Interfaces**: CLI for scripting, TUI for interactive use, GUI for visual analysis
- **Comprehensive**: Ping, traceroute, DNS, WiFi, speed tests, packet capture, and more
- **Intelligent**: Network path analysis, VoIP quality metrics, buffer bloat detection
- **Automated**: Auto-fix capabilities with rollback, continuous monitoring daemon

---

## Features

### Network Diagnostics

| Feature | Description |
|---------|-------------|
| **Ping** | ICMP/UDP/TCP ping with P50/P95/P99 latency percentiles, jitter analysis |
| **Traceroute** | Network path tracing with RTT statistics and hop analysis |
| **DNS** | Multi-resolver DNS lookups with timing and DNSSEC validation |
| **Network Path Analysis** | Segment identification (Local → Router → ISP → Backbone → Destination) |
| **Health Scoring** | Overall network health score based on multiple metrics |

### WiFi Analysis

- **Network Scanning**: Discover nearby networks with signal strength
- **Channel Analysis**: 2.4GHz and 5GHz channel utilization and recommendations
- **Interference Detection**: Identify sources of WiFi interference
- **Signal Quality**: Real-time signal strength monitoring
- **Connection Details**: BSSID, security type, frequency, link speed

### Speed Testing

- **Multi-Provider**: Cloudflare speed test, iPerf3 support
- **Buffer Bloat Detection**: Measures latency under load (grades A-F)
- **Speed Consistency**: Analyzes variance in download/upload speeds
- **Detailed Metrics**: Download, upload, latency, jitter

### VoIP Quality Metrics

- **MOS Score**: Mean Opinion Score (1.0-5.0) for voice quality
- **R-Factor**: E-model R-factor calculation
- **Jitter Analysis**: Packet delay variation measurement
- **Packet Loss**: Real-time loss percentage tracking

### Packet Capture

- **PCAP-Based**: Industry-standard packet capture format
- **BPF Filters**: Berkeley Packet Filter support for targeted capture
- **Protocol Decoding**: Ethernet, IP, TCP, UDP, DNS, HTTP headers
- **Real-Time Display**: Live packet stream viewing
- **Export**: Save captures for analysis in Wireshark

### Auto-Fix & Remediation

- **Issue Detection**: Automatic identification of common network problems
- **Safe Fixes**: Apply remediation with confidence ratings
- **Rollback Support**: Undo any changes if issues occur
- **Available Fixes**:
  - DNS cache flush
  - DHCP lease renewal
  - Network adapter reset
  - DNS server configuration
  - Routing table corrections

### External Integrations

| Service | Capabilities |
|---------|--------------|
| **Shodan** | IP reputation, open ports, vulnerability assessment |
| **IPinfo** | Geolocation, ASN, carrier information |
| **BGP Looking Glass** | Route path analysis, prefix information |
| **SSL Labs** | TLS/SSL certificate and configuration grading |

### Report Generation

Generate detailed reports in multiple formats:
- **Text**: Human-readable plain text
- **JSON**: Machine-parseable structured data
- **Markdown**: Documentation-ready formatting
- **HTML**: Interactive web-based reports with charts
- **PDF**: Professional printable reports

### Background Daemon

- **Continuous Monitoring**: 24/7 network health tracking
- **Scheduled Diagnostics**: Automated periodic tests
- **Alerting**: Notifications on connectivity issues
- **Historical Data**: SQLite-based metric storage
- **Service Integration**: systemd, launchd, Windows Service

---

## Screenshots

<p align="center">
  <em>Screenshots coming soon</em>
</p>

<!--
<p align="center">
  <img src="docs/images/dashboard.png" alt="Dashboard" width="400" />
  <img src="docs/images/traceroute.png" alt="Traceroute" width="400" />
</p>
-->

---

## Installation

### From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/kevinelliott/netdiag.git
cd netdiag

# Build and install CLI
cargo install --path crates/netdiag-cli

# Or build all components
cargo build --release --workspace
```

### Pre-built Binaries

Download platform-specific binaries from the [Releases](https://github.com/kevinelliott/netdiag/releases) page:

| Platform | Architecture | Download |
|----------|--------------|----------|
| macOS | Apple Silicon | `netdiag-darwin-arm64.tar.gz` |
| macOS | Intel | `netdiag-darwin-x64.tar.gz` |
| Linux | x64 | `netdiag-linux-x64.tar.gz` |
| Linux | ARM64 | `netdiag-linux-arm64.tar.gz` |
| Windows | x64 | `netdiag-windows-x64.zip` |

### Package Managers

```bash
# Homebrew (macOS/Linux) - Coming soon
brew install netdiag

# Cargo
cargo install netdiag-cli

# AUR (Arch Linux) - Coming soon
yay -S netdiag
```

### Requirements

- **Rust**: 1.75 or later
- **Optional**: Root/Administrator privileges for raw sockets (ping, traceroute, packet capture)

---

## Quick Start

### Basic Diagnostics

```bash
# Show system and network information
netdiag info

# Run comprehensive diagnostics
netdiag diagnose

# Quick connectivity check
netdiag diagnose --quick
```

### Connectivity Tests

```bash
# Ping a host
netdiag ping google.com

# Ping with count and interval
netdiag ping -c 20 -i 0.5 8.8.8.8

# Traceroute to destination
netdiag traceroute cloudflare.com

# Traceroute with max hops
netdiag traceroute -m 20 1.1.1.1
```

### DNS Lookups

```bash
# Basic DNS lookup
netdiag dns example.com

# Query specific record type
netdiag dns -t MX gmail.com

# Use specific DNS server
netdiag dns -s 8.8.8.8 example.com
```

### WiFi Analysis

```bash
# Show current WiFi status
netdiag wifi status

# Scan for networks
netdiag wifi scan

# Channel analysis
netdiag wifi channels

# Detailed WiFi information
netdiag wifi info
```

### Speed Testing

```bash
# Run speed test
netdiag speed

# Speed test with specific provider
netdiag speed --provider cloudflare

# Download only
netdiag speed --download-only
```

---

## Interfaces

### CLI (Command Line Interface)

The primary interface for scripting and terminal use. Full command reference:

```
netdiag <COMMAND>

Commands:
  info         Display system and network information
  diagnose     Run comprehensive network diagnostics
  ping         Ping a host with detailed statistics
  traceroute   Trace the route to a destination
  dns          Perform DNS lookups
  wifi         WiFi network analysis
  speed        Run internet speed tests
  capture      Capture network packets
  report       Generate diagnostic reports
  fix          Auto-fix network issues
  daemon       Manage background monitoring service
  config       View and edit configuration
  completions  Generate shell completions
  help         Print help information
```

### TUI (Terminal User Interface)

Interactive terminal interface with real-time updates:

```bash
netdiag tui
```

Features:
- Dashboard with live metrics
- Interactive ping and traceroute
- WiFi network browser
- Keyboard navigation
- Mouse support

### GUI (Graphical User Interface)

Desktop and mobile application built with Tauri 2.x and SvelteKit:

```bash
cd apps/tauri

# Development
npm install
npm run tauri dev

# iOS development (requires Xcode)
npm run tauri ios dev

# Android development (requires Android SDK)
npm run tauri android dev

# Build for production
npm run tauri build
```

**Platform-Adaptive Design:**

| Platform | Design Language |
|----------|-----------------|
| **macOS** | Native vibrancy effects, SF-style icons, sidebar navigation |
| **Windows** | Fluent Design System with acrylic backgrounds |
| **Linux** | GTK-inspired styling |
| **iOS** | Apple HIG compliant: SF Symbols icons, grouped lists, toggle switches, spring animations, blur effects |
| **Android** | Material Design 3 with bottom navigation |

---

## Architecture

NetDiag is built as a modular Rust workspace with clear separation of concerns:

```
netdiag/
├── crates/
│   ├── netdiag-types/            # Shared types, errors, traits
│   ├── netdiag-platform/         # Platform abstraction layer
│   ├── netdiag-platform-macos/   # macOS implementations
│   ├── netdiag-platform-linux/   # Linux implementations
│   ├── netdiag-platform-windows/ # Windows implementations
│   ├── netdiag-platform-ios/     # iOS implementations
│   ├── netdiag-platform-android/ # Android implementations
│   ├── netdiag-connectivity/     # Ping, traceroute, DNS
│   ├── netdiag-wifi/             # WiFi scanning and analysis
│   ├── netdiag-speed/            # Speed testing
│   ├── netdiag-capture/          # Packet capture (libpcap)
│   ├── netdiag-integrations/     # External API integrations
│   ├── netdiag-reports/          # Report generation
│   ├── netdiag-storage/          # SQLite data persistence
│   ├── netdiag-autofix/          # Auto-remediation engine
│   ├── netdiag-daemon/           # Background service
│   ├── netdiag-tui/              # Terminal UI (ratatui)
│   └── netdiag-cli/              # CLI binary (clap)
├── apps/
│   └── tauri/                    # Desktop/mobile GUI
│       ├── src/                  # SvelteKit frontend
│       └── src-tauri/            # Tauri backend (Rust)
├── docs/                         # Documentation
├── tests/                        # Integration tests
└── benches/                      # Performance benchmarks
```

### Core Dependencies

| Dependency | Purpose |
|------------|---------|
| `tokio` | Async runtime |
| `clap` | CLI argument parsing |
| `ratatui` | Terminal UI |
| `tauri` | Desktop/mobile GUI framework |
| `sveltekit` | Frontend framework |
| `hickory-resolver` | DNS resolution |
| `socket2` | Raw socket operations |
| `sqlx` | SQLite database |
| `reqwest` | HTTP client |

---

## Platform Support

### Feature Matrix

| Feature | macOS | Linux | Windows | iOS | Android |
|---------|:-----:|:-----:|:-------:|:---:|:-------:|
| **CLI** | ✅ | ✅ | ✅ | - | - |
| **TUI** | ✅ | ✅ | ✅ | - | - |
| **GUI** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Ping (ICMP)** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Traceroute** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **DNS Lookup** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **WiFi Status** | ✅ | ✅ | ✅ | ✅¹ | ✅¹ |
| **WiFi Scan** | ✅ | ✅ | ✅ | ❌² | ✅³ |
| **Speed Test** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Packet Capture** | ✅⁴ | ✅⁴ | ✅⁴ | ❌ | ❌ |
| **Auto-Fix** | ✅ | ✅ | ✅ | ❌ | ❌ |
| **Daemon** | ✅ | ✅ | ✅ | ❌ | ❌ |

¹ Requires entitlements/permissions
² iOS restricts WiFi scanning API to certain app categories
³ Limited to 4 scans per 2 minutes, requires location permission
⁴ Requires root/administrator privileges

### Platform-Specific Notes

**macOS:**
- Full feature support out of the box
- Some features require running with `sudo`
- WiFi uses CoreWLAN framework

**Linux:**
- Requires `libpcap-dev` for packet capture
- WiFi uses NetworkManager or direct netlink
- Systemd service integration available

**Windows:**
- Requires Npcap for packet capture
- WiFi uses Windows WLAN API
- Windows Service integration available

**iOS:**
- See [MOBILE_SETUP.md](apps/tauri/MOBILE_SETUP.md) for detailed setup
- WiFi info requires "Access WiFi Information" entitlement
- Location permission required for network info

**Android:**
- See [MOBILE_SETUP.md](apps/tauri/MOBILE_SETUP.md) for detailed setup
- Location permission required for WiFi features
- WiFi scan throttling by Android OS

---

## Configuration

### Configuration Files

Configuration is stored in platform-specific locations:

| Platform | Location |
|----------|----------|
| macOS | `~/Library/Application Support/netdiag/config.toml` |
| Linux | `~/.config/netdiag/config.toml` |
| Windows | `%APPDATA%\netdiag\config.toml` |

### Example Configuration

```toml
[general]
default_target = "8.8.8.8"
default_dns_server = "1.1.1.1"
timeout_ms = 5000

[ping]
count = 10
interval_ms = 1000
packet_size = 64

[traceroute]
max_hops = 30
timeout_ms = 3000
protocol = "icmp"  # icmp, udp, tcp

[speed]
provider = "cloudflare"
test_duration_secs = 10

[daemon]
enabled = false
check_interval_secs = 300
targets = ["8.8.8.8", "1.1.1.1"]

[integrations]
shodan_api_key = ""
ipinfo_token = ""

[ui]
theme = "auto"  # auto, light, dark
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `NETDIAG_CONFIG` | Override config file path |
| `NETDIAG_LOG` | Log level (error, warn, info, debug, trace) |
| `NETDIAG_DATA_DIR` | Override data directory |

---

## Daemon Service

The daemon provides continuous network monitoring and scheduled diagnostics.

### Managing the Daemon

```bash
# Start daemon (background)
netdiag daemon start

# Start in foreground (for debugging)
netdiag daemon start --foreground

# Check status
netdiag daemon status

# Stop daemon
netdiag daemon stop

# View logs
netdiag daemon logs
netdiag daemon logs -f  # Follow mode

# Install as system service
sudo netdiag daemon install

# Uninstall system service
sudo netdiag daemon uninstall
```

### Daemon Configuration

```toml
# /etc/netdiag/daemon.toml or ~/.config/netdiag/daemon.toml

[daemon]
check_interval_secs = 300
targets = ["8.8.8.8", "1.1.1.1", "cloudflare.com"]

[alerts]
enabled = true
on_connectivity_loss = true
on_high_latency_ms = 200
on_high_packet_loss_percent = 5.0

[storage]
retention_days = 30
database_path = "~/.local/share/netdiag/metrics.db"
```

---

## Auto-Fix

NetDiag can automatically detect and remediate common network issues.

### Usage

```bash
# Analyze issues without making changes
netdiag fix analyze
netdiag fix analyze -v  # Verbose output

# Apply recommended fixes
netdiag fix apply

# Apply only safe fixes (confidence > 90%)
netdiag fix apply --safe-only

# Dry run (show what would be done)
netdiag fix apply --dry-run

# Individual fixes
netdiag fix flush-dns
netdiag fix renew-dhcp en0
netdiag fix reset-adapter en0
netdiag fix set-dns 8.8.8.8 8.8.4.4

# View rollback history
netdiag fix rollbacks

# Rollback a specific fix
netdiag fix rollback <rollback-id>
```

### Supported Fixes

| Fix | Confidence | Description |
|-----|------------|-------------|
| DNS Cache Flush | High | Clears local DNS resolver cache |
| DHCP Renewal | High | Releases and renews DHCP lease |
| Adapter Reset | Medium | Disables and re-enables network adapter |
| DNS Reconfiguration | Medium | Updates DNS server settings |
| Route Table Repair | Low | Corrects routing table entries |

---

## Development

### Prerequisites

| Component | Requirement |
|-----------|-------------|
| Rust | 1.75+ |
| Node.js | 18+ (for GUI) |
| npm/pnpm | Latest (for GUI) |
| Xcode | 15+ (for iOS) |
| Android SDK | 34+ (for Android) |
| Android NDK | 25+ (for Android) |

### Building

```bash
# Check all crates
cargo check --workspace

# Build debug
cargo build --workspace

# Build release
cargo build --release --workspace

# Run CLI in development
cargo run --package netdiag-cli -- info

# Run with logging
RUST_LOG=debug cargo run --package netdiag-cli -- diagnose
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test --package netdiag-connectivity

# Run with output
cargo test --workspace -- --nocapture

# Run integration tests
cargo test --test '*'
```

### Code Quality

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run linter
cargo clippy --workspace

# Run all checks
cargo fmt --check && cargo clippy --workspace && cargo test --workspace
```

### GUI Development

```bash
cd apps/tauri

# Install dependencies
npm install

# Development server with hot reload
npm run tauri dev

# Type checking
npm run check

# Build for production
npm run tauri build
```

---

## Roadmap

### Current (v0.1.0)
- [x] Core diagnostics (ping, traceroute, DNS)
- [x] WiFi analysis
- [x] Speed testing
- [x] CLI interface
- [x] TUI interface
- [x] Desktop GUI (macOS, Linux, Windows)
- [x] Mobile GUI (iOS, Android)
- [x] Packet capture
- [x] Auto-fix with rollback
- [x] Background daemon
- [x] Report generation

### Planned (v0.2.0)
- [ ] Network topology mapping
- [ ] Historical trend analysis
- [ ] Alert notifications (email, Slack, webhook)
- [ ] REST API for remote access
- [ ] Plugin system for custom diagnostics
- [ ] Improved VPN detection and analysis

### Future
- [ ] Network performance benchmarking
- [ ] Multi-device fleet management
- [ ] Cloud sync for settings and history
- [ ] AI-powered issue diagnosis
- [ ] Bandwidth usage monitoring

---

## Contributing

We welcome contributions of all kinds! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Quick Contribution Guide

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature/amazing-feature`
3. **Make** your changes
4. **Test** your changes: `cargo test --workspace`
5. **Commit** with a clear message: `git commit -m 'feat: add amazing feature'`
6. **Push** to your fork: `git push origin feature/amazing-feature`
7. **Open** a Pull Request

### Areas for Contribution

- 🐛 Bug fixes and stability improvements
- 🖥️ Platform-specific implementations
- 📖 Documentation improvements
- ⚡ Performance optimizations
- 🆕 New diagnostic features
- 🎨 UI/UX improvements
- 🌍 Internationalization

---

## License

NetDiag is dual-licensed under your choice of:

- **MIT License** ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)

---

## Author

**Kevin Elliott**
Email: kevin@kevinelliott.net
GitHub: [@kevinelliott](https://github.com/kevinelliott)

---

## Acknowledgments

- [Tauri](https://tauri.app/) - Desktop/mobile application framework
- [SvelteKit](https://kit.svelte.dev/) - Frontend framework
- [Ratatui](https://ratatui.rs/) - Terminal UI library
- [Hickory DNS](https://github.com/hickory-dns/hickory-dns) - DNS resolver
- [libpcap](https://www.tcpdump.org/) - Packet capture library

---

<p align="center">
  Made with ❤️ for the network troubleshooting community
</p>
