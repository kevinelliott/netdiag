# NetDiag Mobile Setup Guide

This guide covers the setup required to build NetDiag for iOS and Android.

## Prerequisites

### Common Requirements
- Rust with the appropriate mobile targets installed
- Node.js and pnpm for the SvelteKit frontend
- Tauri CLI v2.x

```bash
# Install Rust mobile targets
rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

## iOS Setup

### Requirements
- macOS with Xcode 15+
- Apple Developer Account
- Xcode Command Line Tools

### Configuration

1. **Set your Apple Development Team ID** in `src-tauri/tauri.conf.json`:
   ```json
   {
     "bundle": {
       "iOS": {
         "developmentTeam": "YOUR_TEAM_ID",
         "minimumSystemVersion": "14.0"
       }
     }
   }
   ```

   Find your Team ID in Xcode: Xcode → Preferences → Accounts → Select your account → View Details

2. **Initialize the iOS project**:
   ```bash
   cargo tauri ios init
   ```

3. **Add required entitlements** for WiFi access. Create/edit `src-tauri/gen/apple/NetDiag/NetDiag.entitlements`:
   ```xml
   <?xml version="1.0" encoding="UTF-8"?>
   <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
   <plist version="1.0">
   <dict>
       <key>com.apple.developer.networking.wifi-info</key>
       <true/>
   </dict>
   </plist>
   ```

4. **Add Info.plist entries** for permissions:
   ```xml
   <key>NSLocationWhenInUseUsageDescription</key>
   <string>NetDiag needs location access to provide WiFi network information.</string>
   ```

### Building

```bash
# Development build
cargo tauri ios dev

# Release build
cargo tauri ios build
```

### Notes on iOS Limitations
- WiFi scanning requires `NEHotspotHelper` entitlement (restricted to certain app categories)
- Current WiFi info requires "Access WiFi Information" entitlement
- Packet capture is not available on iOS
- System modifications (DNS changes, etc.) are not available

## Android Setup

### Requirements
- Android Studio with SDK Manager
- Android NDK (version 25+)
- Android SDK with API level 24+ (Android 7.0+)
- JDK 17+

### Configuration

1. **Install Android Studio** from https://developer.android.com/studio

2. **Install required SDK components** via Android Studio SDK Manager:
   - Android SDK Platform 34 (or latest)
   - Android SDK Build-Tools 34.x
   - NDK (Side by side) version 25+
   - Android SDK Command-line Tools
   - Android Emulator (optional)

3. **Set environment variables**:
   ```bash
   export ANDROID_HOME=$HOME/Library/Android/sdk
   export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653
   export PATH=$PATH:$ANDROID_HOME/cmdline-tools/latest/bin
   export PATH=$PATH:$ANDROID_HOME/platform-tools
   ```

4. **Accept SDK licenses**:
   ```bash
   yes | sdkmanager --licenses
   ```

5. **Initialize the Android project**:
   ```bash
   cargo tauri android init
   ```

6. **Add required permissions** in `src-tauri/gen/android/app/src/main/AndroidManifest.xml`:
   ```xml
   <uses-permission android:name="android.permission.ACCESS_NETWORK_STATE" />
   <uses-permission android:name="android.permission.ACCESS_WIFI_STATE" />
   <uses-permission android:name="android.permission.CHANGE_WIFI_STATE" />
   <uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
   <uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />
   <uses-permission android:name="android.permission.INTERNET" />
   ```

### Building

```bash
# Development build (runs on connected device/emulator)
cargo tauri android dev

# Release build
cargo tauri android build
```

### Notes on Android Limitations
- WiFi scanning requires `ACCESS_FINE_LOCATION` permission (Android 8.0+)
- WiFi scans are throttled: 4 scans per 2 minutes for foreground apps
- Packet capture requires root access
- System modifications require root access

## Platform Feature Matrix

| Feature | iOS | Android |
|---------|-----|---------|
| Network interfaces | ✅ | ✅ |
| Current WiFi connection | ✅* | ✅* |
| WiFi scanning | ❌ | ✅** |
| Signal strength | ✅* | ✅* |
| Packet capture | ❌ | ❌ |
| DNS cache flush | ❌ | ❌ |
| Adapter reset | ❌ | ❌ |
| Ping | ✅ | ✅ |
| Traceroute | ✅ | ✅ |
| DNS lookup | ✅ | ✅ |

\* Requires entitlements/permissions
\** Requires location permission, subject to throttling

## Troubleshooting

### iOS
- **"Access WiFi Information" not working**: Ensure the entitlement is properly configured in your provisioning profile through Apple Developer Portal.
- **Build fails with signing errors**: Verify your Team ID and that you have a valid development certificate.

### Android
- **JNI errors at runtime**: Ensure NDK is properly installed and NDK_HOME is set.
- **WiFi info returns null**: Request location permission at runtime before accessing WiFi info.
- **Scan results empty**: Location services must be enabled on the device.

## Development Tips

1. **Testing on real devices** is essential for WiFi features since emulators have limited network simulation.

2. **Use `tracing` output** to debug platform-specific issues:
   ```bash
   RUST_LOG=debug cargo tauri ios dev
   ```

3. **The mobile Tauri builds** use the same SvelteKit frontend - no separate mobile UI needed.

4. **Hot reloading** works for frontend changes during development.
