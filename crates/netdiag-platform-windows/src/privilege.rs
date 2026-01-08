//! Windows privilege provider implementation.

use async_trait::async_trait;
use netdiag_platform::{Capability, PrivilegeProvider};
use netdiag_types::{
    error::Result,
    system::{ElevationRequest, PrivilegeLevel},
};

/// Windows privilege provider.
pub struct WindowsPrivilegeProvider {
    is_elevated: bool,
}

impl WindowsPrivilegeProvider {
    /// Creates a new Windows privilege provider.
    pub fn new() -> Self {
        Self {
            is_elevated: Self::check_elevation(),
        }
    }

    /// Checks if the current process is running elevated (as Administrator).
    #[cfg(windows)]
    fn check_elevation() -> bool {
        use windows::Win32::Security::*;
        use windows::Win32::System::Threading::*;

        unsafe {
            let mut token_handle = std::mem::zeroed();
            if OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY,
                &mut token_handle,
            ).is_ok() {
                let mut elevation = TOKEN_ELEVATION::default();
                let mut size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

                if GetTokenInformation(
                    token_handle,
                    TokenElevation,
                    Some(&mut elevation as *mut _ as *mut _),
                    size,
                    &mut size,
                ).is_ok() {
                    let _ = windows::Win32::Foundation::CloseHandle(token_handle);
                    return elevation.TokenIsElevated != 0;
                }
                let _ = windows::Win32::Foundation::CloseHandle(token_handle);
            }
            false
        }
    }

    #[cfg(not(windows))]
    fn check_elevation() -> bool {
        false
    }
}

impl Default for WindowsPrivilegeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PrivilegeProvider for WindowsPrivilegeProvider {
    fn current_privilege_level(&self) -> PrivilegeLevel {
        if self.is_elevated {
            PrivilegeLevel::Elevated
        } else {
            PrivilegeLevel::User
        }
    }

    async fn request_elevation(&self, _request: &ElevationRequest) -> Result<bool> {
        // Windows elevation requires restarting the process with runas
        // This can't be done in-process
        Ok(false)
    }

    fn has_capability(&self, capability: Capability) -> bool {
        match capability {
            // Basic capabilities always available
            Capability::NetworkRead => true,
            Capability::DnsResolve => true,

            // These work without elevation
            Capability::Ping => true,
            Capability::Traceroute => true,

            // These require elevation
            Capability::RawSocket => self.is_elevated,
            Capability::PacketCapture => self.is_elevated,
            Capability::NetworkWrite => self.is_elevated,
            Capability::WifiScan => true, // Works but may be limited
            Capability::WifiConnect => self.is_elevated,
            Capability::SystemModify => self.is_elevated,
            Capability::ServiceManage => self.is_elevated,
        }
    }

    fn available_capabilities(&self) -> Vec<Capability> {
        let mut caps = vec![
            Capability::NetworkRead,
            Capability::DnsResolve,
            Capability::Ping,
            Capability::Traceroute,
            Capability::WifiScan,
        ];

        if self.is_elevated {
            caps.extend([
                Capability::RawSocket,
                Capability::PacketCapture,
                Capability::NetworkWrite,
                Capability::WifiConnect,
                Capability::SystemModify,
                Capability::ServiceManage,
            ]);
        }

        caps
    }

    fn capabilities_requiring_elevation(&self) -> Vec<Capability> {
        vec![
            Capability::RawSocket,
            Capability::PacketCapture,
            Capability::NetworkWrite,
            Capability::WifiConnect,
            Capability::SystemModify,
            Capability::ServiceManage,
        ]
    }
}
