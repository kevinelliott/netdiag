# Contributing to netdiag

Thank you for your interest in contributing to netdiag! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and constructive in all interactions. We welcome contributors of all experience levels.

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Node.js 20+ and pnpm (for Tauri GUI)
- Platform-specific dependencies:
  - **macOS**: Xcode command line tools
  - **Linux**: `libpcap-dev`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`
  - **Windows**: Visual Studio Build Tools

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/kevinelliott/netdiag.git
   cd netdiag
   ```

2. Build the workspace:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. For GUI development:
   ```bash
   cd apps/tauri
   pnpm install
   pnpm dev
   ```

## How to Contribute

### Reporting Issues

- Check if the issue already exists
- Include your OS, Rust version, and netdiag version
- Provide steps to reproduce the issue
- Include relevant error messages or logs

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests and linting:
   ```bash
   cargo test
   cargo fmt --check
   cargo clippy
   ```
5. Commit with a clear message
6. Push and create a pull request

### Commit Messages

Follow conventional commits:
- `feat: add new feature`
- `fix: resolve bug`
- `docs: update documentation`
- `refactor: improve code structure`
- `test: add tests`
- `chore: maintenance tasks`

## Project Structure

```
netdiag/
├── crates/
│   ├── netdiag-types/      # Shared types and errors
│   ├── netdiag-core/       # Core functionality
│   ├── netdiag-platform/   # Platform abstraction traits
│   ├── netdiag-platform-*/ # Platform implementations
│   ├── netdiag-cli/        # CLI binary
│   └── ...                 # Other feature crates
├── apps/
│   └── tauri/              # Desktop/mobile GUI
└── docs/                   # Documentation
```

## Development Guidelines

### Code Style

- Follow Rust idioms and conventions
- Use `cargo fmt` for formatting
- Address `cargo clippy` warnings
- Write documentation for public APIs
- Include tests for new functionality

### Platform Support

When adding features:
- Consider all supported platforms (macOS, Linux, Windows)
- Use the platform abstraction traits in `netdiag-platform`
- Implement graceful degradation for platform-specific features
- Test on multiple platforms when possible

### Testing

- Write unit tests for new functions
- Add integration tests in `/tests` for cross-crate functionality
- Test privilege scenarios (with and without root)
- Manual testing checklist for GUI changes

## Areas for Contribution

- Bug fixes and stability improvements
- Platform-specific implementations
- Documentation improvements
- Performance optimizations
- New diagnostic features
- UI/UX improvements
- Internationalization

## Questions?

Open a discussion on GitHub or reach out to the maintainers.

## License

By contributing, you agree that your contributions will be licensed under the same MIT OR Apache-2.0 license as the project.
