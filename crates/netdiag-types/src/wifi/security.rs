//! WiFi security types.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

/// WiFi security type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityType {
    /// Authentication method
    pub authentication: WifiAuthentication,
    /// Encryption type
    pub encryption: WifiEncryption,
    /// Key management suite
    pub key_management: KeyManagement,
    /// Is PMF (Protected Management Frames) required?
    pub pmf_required: bool,
    /// Is this a transition mode (e.g., WPA2/WPA3)?
    pub transition_mode: bool,
}

impl SecurityType {
    /// Creates an open (no security) type.
    #[must_use]
    pub fn open() -> Self {
        Self {
            authentication: WifiAuthentication::Open,
            encryption: WifiEncryption::None,
            key_management: KeyManagement::None,
            pmf_required: false,
            transition_mode: false,
        }
    }

    /// Creates a WPA2-Personal security type.
    #[must_use]
    pub fn wpa2_personal() -> Self {
        Self {
            authentication: WifiAuthentication::Wpa2,
            encryption: WifiEncryption::Ccmp,
            key_management: KeyManagement::Psk,
            pmf_required: false,
            transition_mode: false,
        }
    }

    /// Creates a WPA3-Personal security type.
    #[must_use]
    pub fn wpa3_personal() -> Self {
        Self {
            authentication: WifiAuthentication::Wpa3,
            encryption: WifiEncryption::Gcmp256,
            key_management: KeyManagement::Sae,
            pmf_required: true,
            transition_mode: false,
        }
    }

    /// Creates a WPA2-Enterprise security type.
    #[must_use]
    pub fn wpa2_enterprise() -> Self {
        Self {
            authentication: WifiAuthentication::Wpa2,
            encryption: WifiEncryption::Ccmp,
            key_management: KeyManagement::Eap,
            pmf_required: false,
            transition_mode: false,
        }
    }

    /// Returns true if this is an open network.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.authentication == WifiAuthentication::Open && self.encryption == WifiEncryption::None
    }

    /// Returns true if this is enterprise (802.1x) security.
    #[must_use]
    pub fn is_enterprise(&self) -> bool {
        matches!(
            self.key_management,
            KeyManagement::Eap | KeyManagement::EapSuiteB | KeyManagement::EapSuiteB192
        )
    }

    /// Returns the security level (0-4).
    #[must_use]
    pub fn security_level(&self) -> u8 {
        match (&self.authentication, &self.encryption) {
            (WifiAuthentication::Open, WifiEncryption::None) => 0,
            (WifiAuthentication::Wep, _) => 1,
            (WifiAuthentication::Wpa, _) => 2,
            (WifiAuthentication::Wpa2, _) => 3,
            (WifiAuthentication::Wpa3, _) => 4,
            _ => 2,
        }
    }

    /// Returns the security as a human-readable string.
    #[must_use]
    pub fn to_display_string(&self) -> String {
        if self.is_open() {
            return "Open".to_string();
        }

        let auth = match &self.authentication {
            WifiAuthentication::Wep => "WEP",
            WifiAuthentication::Wpa => "WPA",
            WifiAuthentication::Wpa2 => "WPA2",
            WifiAuthentication::Wpa3 => "WPA3",
            WifiAuthentication::Open => "Open",
            WifiAuthentication::Owe => "OWE",
        };

        let mode = if self.is_enterprise() {
            "Enterprise"
        } else {
            "Personal"
        };

        if self.transition_mode {
            format!("{auth}/{}", self.transition_auth_name().unwrap_or(auth))
        } else {
            format!("{auth}-{mode}")
        }
    }

    fn transition_auth_name(&self) -> Option<&'static str> {
        if self.transition_mode {
            match &self.authentication {
                WifiAuthentication::Wpa3 => Some("WPA2"),
                WifiAuthentication::Wpa2 => Some("WPA"),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl std::fmt::Display for SecurityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

/// WiFi authentication method.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum WifiAuthentication {
    /// Open (no authentication)
    #[default]
    Open,
    /// WEP (deprecated, insecure)
    Wep,
    /// WPA (deprecated)
    Wpa,
    /// WPA2
    Wpa2,
    /// WPA3
    Wpa3,
    /// OWE (Opportunistic Wireless Encryption)
    Owe,
}

/// WiFi encryption type.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum WifiEncryption {
    /// No encryption
    None,
    /// WEP (insecure)
    Wep,
    /// TKIP (deprecated)
    Tkip,
    /// CCMP (AES-128)
    #[default]
    Ccmp,
    /// GCMP-128
    Gcmp,
    /// GCMP-256
    Gcmp256,
    /// BIP (for management frames)
    Bip,
}

/// Key management suite.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum KeyManagement {
    /// No key management
    #[default]
    None,
    /// Pre-Shared Key
    Psk,
    /// Simultaneous Authentication of Equals (WPA3)
    Sae,
    /// EAP (802.1X)
    Eap,
    /// EAP Suite B (192-bit)
    EapSuiteB,
    /// EAP Suite B 192-bit
    EapSuiteB192,
    /// OWE (Opportunistic Wireless Encryption)
    Owe,
    /// FT (Fast BSS Transition) with PSK
    FtPsk,
    /// FT with EAP
    FtEap,
    /// FT with SAE
    FtSae,
}
