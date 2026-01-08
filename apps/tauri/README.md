# NetDiag GUI

Desktop and mobile GUI application for NetDiag, built with Tauri and SvelteKit.

## Prerequisites

- Node.js 18+
- pnpm
- Rust 1.75+
- Tauri CLI: `cargo install tauri-cli`

### Platform-Specific

**macOS:**
- Xcode Command Line Tools

**Linux:**
- GTK development libraries
- WebKit2GTK development libraries

**iOS:**
- Xcode
- CocoaPods

**Android:**
- Android SDK
- Android NDK

## Development

```bash
# Install dependencies
pnpm install

# Start development server
pnpm tauri dev
```

## Building

```bash
# Build for current platform
pnpm tauri build

# Build specific targets (macOS)
pnpm tauri build --target universal-apple-darwin
```

## Structure

```
apps/tauri/
├── src/                    # SvelteKit frontend
│   ├── routes/             # Pages
│   │   ├── +page.svelte    # Dashboard
│   │   ├── interfaces/     # Network interfaces
│   │   ├── ping/           # Ping tool
│   │   ├── traceroute/     # Traceroute tool
│   │   └── dns/            # DNS lookup
│   └── lib/                # Shared components
├── src-tauri/              # Tauri backend (Rust)
│   ├── src/
│   │   ├── lib.rs          # Main app setup
│   │   ├── commands.rs     # Tauri commands
│   │   ├── state.rs        # App state
│   │   └── error.rs        # Error types
│   └── Cargo.toml
└── static/                 # Static assets
```

## Features

- Real-time network monitoring dashboard
- Network interface viewer
- Ping tool with statistics
- Traceroute visualization
- DNS lookup tool
- Dark/light mode support
- Responsive design for mobile

## Mobile Development

### iOS

```bash
# Initialize iOS project
pnpm tauri ios init

# Run on simulator
pnpm tauri ios dev

# Build for App Store
pnpm tauri ios build
```

### Android

```bash
# Initialize Android project
pnpm tauri android init

# Run on emulator
pnpm tauri android dev

# Build APK
pnpm tauri android build
```

## License

MIT OR Apache-2.0
