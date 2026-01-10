# NetDiag GUI

Desktop and mobile GUI application for NetDiag, built with Tauri 2.x and SvelteKit with Svelte 5.

## Features

- Real-time network monitoring dashboard
- Network interface viewer
- WiFi network scanner and analysis
- Speed test with download/upload metrics
- Ping tool with statistics
- Traceroute visualization
- DNS lookup tool
- Network diagnostics suite
- Network fix utilities
- Packet capture (desktop)
- Report generation

## Platform-Adaptive UI

NetDiag features a platform-adaptive design that provides native-feeling experiences across all platforms:

### Desktop (macOS, Windows, Linux)
- Sidebar navigation with collapsible sections
- macOS vibrancy/translucency effects
- Windows Fluent Design acrylic backgrounds
- Keyboard shortcuts and hover states

### Mobile (iOS, Android)
- Bottom tab bar navigation (5 tabs + More menu)
- iOS: SF Symbols-style icons, blur effects, action sheets
- Android: Material Design 3 navigation bar
- Touch-optimized controls and gestures

### iOS Native Design
The iOS version follows Apple's Human Interface Guidelines:
- **Typography**: SF Pro font with Dynamic Type scale (34px Large Title to 11px Caption)
- **Colors**: Full iOS semantic color system with light/dark mode
- **Tab Bar**: 49pt height with blur, 28pt icons, 10pt labels
- **Lists**: Grouped inset style with proper separators
- **Forms**: Toggle switches, rounded inputs, segmented controls
- **Animations**: Spring timing curves (0.35s duration)

## Prerequisites

- Node.js 18+ (or pnpm/npm)
- Rust 1.75+
- Tauri CLI: `cargo install tauri-cli`

### Platform-Specific

**macOS:**
- Xcode Command Line Tools

**Linux:**
- GTK development libraries
- WebKit2GTK development libraries

**iOS:**
- Xcode 15+
- CocoaPods
- Apple Developer account (for device testing)

**Android:**
- Android SDK
- Android NDK

## Development

```bash
# Install dependencies
npm install

# Start development server (desktop)
npm run tauri dev

# iOS development
npm run tauri ios dev

# Android development
npm run tauri android dev
```

## Building

```bash
# Build for current platform
npm run tauri build

# Build specific targets (macOS)
npm run tauri build --target universal-apple-darwin

# iOS build
npm run tauri ios build

# Android build
npm run tauri android build
```

## Project Structure

```
apps/tauri/
├── src/                          # SvelteKit frontend
│   ├── routes/                   # Pages
│   │   ├── +page.svelte          # Dashboard
│   │   ├── +layout.svelte        # App shell
│   │   ├── diagnose/             # Network diagnostics
│   │   ├── interfaces/           # Network interfaces
│   │   ├── wifi/                 # WiFi scanner
│   │   ├── speed/                # Speed test
│   │   ├── ping/                 # Ping tool
│   │   ├── traceroute/           # Traceroute tool
│   │   ├── dns/                  # DNS lookup
│   │   ├── capture/              # Packet capture
│   │   ├── report/               # Report generation
│   │   └── fix/                  # Network fixes
│   ├── lib/
│   │   ├── components/           # UI components
│   │   │   ├── icons/            # SVG icon system
│   │   │   ├── navigation/       # Sidebar, TabBar
│   │   │   ├── layout/           # AppShell
│   │   │   └── ui/               # iOS list components
│   │   ├── platform/             # Platform detection
│   │   └── styles/
│   │       └── tokens/           # Design tokens per platform
│   └── app.css                   # Global styles
├── src-tauri/                    # Tauri backend (Rust)
│   ├── src/
│   │   ├── lib.rs                # Main app setup
│   │   ├── commands/             # Tauri commands
│   │   ├── state.rs              # App state
│   │   └── error.rs              # Error types
│   └── Cargo.toml
├── static/                       # Static assets
└── vite.config.ts                # Vite configuration
```

## Component Library

### Icons
```svelte
<script>
  import { Icon } from '$lib/components';
</script>

<Icon name="wifi" size={24} filled={true} />
```

Available icons: `dashboard`, `diagnose`, `wifi`, `speed`, `ping`, `traceroute`, `dns`, `interfaces`, `capture`, `report`, `fix`, `more`, `check`, `xmark`, `chevron`, `refresh`, `warning`, `info`, `settings`

### iOS List Components
```svelte
<script>
  import { IOSList, IOSListSection, IOSListCell } from '$lib/components';
</script>

<IOSList>
  <IOSListSection header="Network">
    <IOSListCell title="WiFi" value="Connected" disclosure />
    <IOSListCell title="Cellular" value="Off" />
  </IOSListSection>
</IOSList>
```

### Platform Detection
```svelte
<script>
  import { getPlatform } from '$lib/platform';

  const platform = getPlatform();
  // platform.platform: 'ios' | 'android' | 'macos' | 'windows' | 'linux'
  // platform.isMobile: boolean
  // platform.isDesktop: boolean
</script>
```

## Design Tokens

Platform-specific design tokens are defined in `src/lib/styles/tokens/`:

| File | Platform |
|------|----------|
| `_base.css` | Shared defaults |
| `_ios.css` | iOS (HIG compliant) |
| `_android.css` | Android (Material 3) |
| `_macos.css` | macOS (vibrancy) |
| `_windows.css` | Windows (Fluent) |
| `_linux.css` | Linux (GTK-inspired) |

## License

MIT OR Apache-2.0
