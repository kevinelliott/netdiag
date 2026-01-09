//! Application state management.

#[cfg(target_os = "macos")]
use netdiag_platform_macos::create_providers;

#[cfg(target_os = "linux")]
use netdiag_platform_linux::create_providers;

#[cfg(target_os = "ios")]
use netdiag_platform_ios::create_providers;

#[cfg(target_os = "android")]
use netdiag_platform_android::create_providers;

#[cfg(target_os = "windows")]
use netdiag_platform_windows::create_providers;

#[cfg(not(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "ios",
    target_os = "android",
    target_os = "windows"
)))]
fn create_providers() -> netdiag_platform::PlatformProviders {
    netdiag_platform::PlatformProviders::new()
}

/// Application state shared across Tauri commands.
pub struct AppState {
    /// Platform providers for network operations.
    pub providers: netdiag_platform::PlatformProviders,
}

impl AppState {
    /// Creates a new application state.
    pub fn new() -> Self {
        Self {
            providers: create_providers(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
