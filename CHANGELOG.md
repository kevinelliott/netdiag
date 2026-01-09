# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-01-XX

### Added

- Initial release of netdiag
- **CLI Tool** (`netdiag-cli`)
  - `netdiag info` - Display system and network information
  - `netdiag diagnose` - Run comprehensive network diagnostics
  - `netdiag ping <target>` - Ping with detailed statistics
  - `netdiag traceroute <target>` - Trace route with hop analysis
  - `netdiag speed` - Run speed tests (download/upload/latency)
  - `netdiag wifi` - WiFi analysis (scan, status, channels, interference)
  - `netdiag report` - Generate reports (text, JSON, markdown, HTML)
  - `netdiag capture` - Packet capture with BPF filters
  - `netdiag fix` - Auto-fix network issues with rollback
  - `netdiag daemon` - Background monitoring service
  - `netdiag config` - Configuration management
  - `netdiag completions` - Shell completion generation

- **Desktop GUI** (Tauri + SvelteKit)
  - Dashboard with real-time network status
  - Interface browser
  - WiFi scanner
  - Ping and traceroute tools
  - DNS lookup
  - Mobile-responsive design

- **Platform Support**
  - macOS (full support)
  - Linux (full support)
  - Windows (full support)
  - iOS (mobile-ready UI)
  - Android (mobile-ready UI)

- **Core Features**
  - Multi-interface detection and monitoring
  - WiFi scanning and channel analysis
  - ICMP/UDP/TCP ping
  - Traceroute with RTT statistics
  - Speed testing with multiple providers
  - Packet capture with protocol decoding
  - SQLite-based result storage
  - Daemon mode for continuous monitoring
  - Auto-fix with rollback capability

### Security

- Privilege graceful degradation (works without root where possible)
- No sensitive data stored in plaintext
- Secure default configurations

[Unreleased]: https://github.com/kevinelliott/netdiag/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kevinelliott/netdiag/releases/tag/v0.1.0
