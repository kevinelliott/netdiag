//! Android WiFi provider implementation.
//!
//! Uses Android's WifiManager through JNI to access WiFi information.
//! Note: Starting with Android 10, WiFi scanning has restrictions
//! and requires ACCESS_FINE_LOCATION permission.

use async_trait::async_trait;
use netdiag_platform::{WifiInterface, WifiProvider};
use netdiag_types::{
    error::Result,
    wifi::{
        AccessPoint, Channel, ChannelBand, ChannelUtilization, ChannelWidth,
        MacAddress as WifiMacAddress, SecurityType, Ssid, WifiConnection, WifiStandard,
    },
    Error,
};

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JString, JValue},
    JNIEnv,
};

/// Android WiFi provider.
///
/// Uses Android's WifiManager through JNI to access WiFi information.
/// Note: Starting with Android 10, WiFi scanning has restrictions
/// and requires location permission.
pub struct AndroidWifiProvider {
    // No persistent state needed - we get fresh JNI env each call
}

impl AndroidWifiProvider {
    /// Creates a new Android WiFi provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets the current WiFi connection info via JNI.
    #[cfg(target_os = "android")]
    fn get_connection_info_jni(&self) -> Result<Option<WifiConnection>> {
        use ndk_context::android_context;

        let ctx = android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as *mut _) }.map_err(|e| {
            Error::Platform {
                message: format!("Failed to get JavaVM: {}", e),
                source: None,
            }
        })?;

        let mut env = vm.attach_current_thread().map_err(|e| Error::Platform {
            message: format!("Failed to attach thread: {}", e),
            source: None,
        })?;

        self.get_wifi_connection_info(&mut env)
    }

    #[cfg(target_os = "android")]
    fn get_wifi_connection_info(&self, env: &mut JNIEnv) -> Result<Option<WifiConnection>> {
        use ndk_context::android_context;

        // Get the Android context
        let ctx = android_context();
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        // Get WifiManager: context.getSystemService(Context.WIFI_SERVICE)
        let wifi_service = env
            .new_string("wifi")
            .map_err(|e| Error::Platform {
                message: format!("Failed to create wifi string: {}", e),
                source: None,
            })?;

        let wifi_manager = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&wifi_service.into())],
            )
            .map_err(|e| Error::Platform {
                message: format!("Failed to get WifiManager: {}", e),
                source: None,
            })?
            .l()
            .map_err(|e| Error::Platform {
                message: format!("WifiManager not an object: {}", e),
                source: None,
            })?;

        if wifi_manager.is_null() {
            return Ok(None);
        }

        // Check if WiFi is enabled
        let is_enabled = env
            .call_method(&wifi_manager, "isWifiEnabled", "()Z", &[])
            .map_err(|e| Error::Platform {
                message: format!("Failed to check isWifiEnabled: {}", e),
                source: None,
            })?
            .z()
            .unwrap_or(false);

        if !is_enabled {
            return Ok(None);
        }

        // Get connection info: wifiManager.getConnectionInfo()
        let connection_info = env
            .call_method(
                &wifi_manager,
                "getConnectionInfo",
                "()Landroid/net/wifi/WifiInfo;",
                &[],
            )
            .map_err(|e| Error::Platform {
                message: format!("Failed to get connection info: {}", e),
                source: None,
            })?
            .l()
            .map_err(|e| Error::Platform {
                message: format!("Connection info not an object: {}", e),
                source: None,
            })?;

        if connection_info.is_null() {
            return Ok(None);
        }

        // Get SSID
        let ssid_obj = env
            .call_method(&connection_info, "getSSID", "()Ljava/lang/String;", &[])
            .ok()
            .and_then(|v| v.l().ok());

        let ssid = ssid_obj
            .and_then(|obj| {
                if obj.is_null() {
                    None
                } else {
                    env.get_string((&obj).into()).ok().map(|s| {
                        let s: String = s.into();
                        // Android returns SSID wrapped in quotes, remove them
                        s.trim_matches('"').to_string()
                    })
                }
            })
            .unwrap_or_default();

        // Check for unknown/disconnected SSID
        if ssid.is_empty() || ssid == "<unknown ssid>" {
            return Ok(None);
        }

        // Get BSSID
        let bssid_obj = env
            .call_method(&connection_info, "getBSSID", "()Ljava/lang/String;", &[])
            .ok()
            .and_then(|v| v.l().ok());

        let bssid = bssid_obj.and_then(|obj| {
            if obj.is_null() {
                None
            } else {
                env.get_string((&obj).into())
                    .ok()
                    .and_then(|s| parse_mac_address(&String::from(s)))
            }
        });

        // Get RSSI
        let rssi = env
            .call_method(&connection_info, "getRssi", "()I", &[])
            .ok()
            .and_then(|v| v.i().ok())
            .unwrap_or(-127);

        // Get link speed (Mbps)
        let link_speed = env
            .call_method(&connection_info, "getLinkSpeed", "()I", &[])
            .ok()
            .and_then(|v| v.i().ok());

        // Get frequency (MHz) - available on API 21+
        let frequency = env
            .call_method(&connection_info, "getFrequency", "()I", &[])
            .ok()
            .and_then(|v| v.i().ok())
            .unwrap_or(0);

        // Determine channel from frequency
        let (channel_number, band) = frequency_to_channel(frequency);

        // Get TX link speed if available (API 29+)
        let tx_link_speed = env
            .call_method(&connection_info, "getTxLinkSpeedMbps", "()I", &[])
            .ok()
            .and_then(|v| v.i().ok())
            .filter(|&v| v > 0);

        // Get RX link speed if available (API 29+)
        let rx_link_speed = env
            .call_method(&connection_info, "getRxLinkSpeedMbps", "()I", &[])
            .ok()
            .and_then(|v| v.i().ok())
            .filter(|&v| v > 0);

        // Create access point info
        let access_point = AccessPoint {
            ssid: Ssid::new(&ssid).unwrap_or_else(|_| Ssid::new("Unknown").unwrap()),
            bssid: bssid.unwrap_or(WifiMacAddress::from([0, 0, 0, 0, 0, 0])),
            rssi,
            noise: None,
            channel: Channel {
                number: channel_number as u8,
                band,
                width: ChannelWidth::Mhz20, // Not available from WifiInfo
            },
            security: SecurityType::Unknown, // Not easily available from WifiInfo
            wifi_standard: infer_wifi_standard(frequency, link_speed.unwrap_or(0)),
            supported_rates: Vec::new(),
            last_seen: std::time::SystemTime::now(),
        };

        Ok(Some(WifiConnection {
            access_point,
            state: netdiag_types::wifi::ConnectionState::Connected,
            tx_rate: tx_link_speed.or(link_speed).map(|s| s as u32),
            rx_rate: rx_link_speed.map(|s| s as u32),
            mcs_index: None,
            guard_interval: None,
            nss: None,
        }))
    }

    /// Gets scan results via JNI.
    #[cfg(target_os = "android")]
    fn get_scan_results_jni(&self) -> Result<Vec<AccessPoint>> {
        use ndk_context::android_context;

        let ctx = android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as *mut _) }.map_err(|e| {
            Error::Platform {
                message: format!("Failed to get JavaVM: {}", e),
                source: None,
            }
        })?;

        let mut env = vm.attach_current_thread().map_err(|e| Error::Platform {
            message: format!("Failed to attach thread: {}", e),
            source: None,
        })?;

        self.get_scan_results(&mut env)
    }

    #[cfg(target_os = "android")]
    fn get_scan_results(&self, env: &mut JNIEnv) -> Result<Vec<AccessPoint>> {
        use ndk_context::android_context;

        let ctx = android_context();
        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        // Get WifiManager
        let wifi_service = env.new_string("wifi").map_err(|e| Error::Platform {
            message: format!("Failed to create wifi string: {}", e),
            source: None,
        })?;

        let wifi_manager = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&wifi_service.into())],
            )
            .map_err(|e| Error::Platform {
                message: format!("Failed to get WifiManager: {}", e),
                source: None,
            })?
            .l()
            .map_err(|e| Error::Platform {
                message: format!("WifiManager not an object: {}", e),
                source: None,
            })?;

        if wifi_manager.is_null() {
            return Err(Error::Platform {
                message: "WifiManager is null".to_string(),
                source: None,
            });
        }

        // Get scan results: wifiManager.getScanResults()
        let scan_results = env
            .call_method(
                &wifi_manager,
                "getScanResults",
                "()Ljava/util/List;",
                &[],
            )
            .map_err(|e| Error::Platform {
                message: format!("Failed to get scan results: {}", e),
                source: None,
            })?
            .l()
            .map_err(|e| Error::Platform {
                message: format!("Scan results not an object: {}", e),
                source: None,
            })?;

        if scan_results.is_null() {
            return Ok(Vec::new());
        }

        // Get list size
        let size = env
            .call_method(&scan_results, "size", "()I", &[])
            .map_err(|e| Error::Platform {
                message: format!("Failed to get list size: {}", e),
                source: None,
            })?
            .i()
            .unwrap_or(0);

        let mut access_points = Vec::new();

        for i in 0..size {
            let result = env
                .call_method(
                    &scan_results,
                    "get",
                    "(I)Ljava/lang/Object;",
                    &[JValue::Int(i)],
                )
                .ok()
                .and_then(|v| v.l().ok());

            if let Some(scan_result) = result {
                if scan_result.is_null() {
                    continue;
                }

                if let Some(ap) = self.parse_scan_result(env, &scan_result) {
                    access_points.push(ap);
                }
            }
        }

        Ok(access_points)
    }

    #[cfg(target_os = "android")]
    fn parse_scan_result(&self, env: &mut JNIEnv, scan_result: &JObject) -> Option<AccessPoint> {
        // Get SSID field
        let ssid_field = env.get_field(scan_result, "SSID", "Ljava/lang/String;").ok()?;
        let ssid_obj = ssid_field.l().ok()?;
        let ssid = if ssid_obj.is_null() {
            String::new()
        } else {
            env.get_string((&ssid_obj).into())
                .ok()
                .map(|s| String::from(s))
                .unwrap_or_default()
        };

        // Get BSSID field
        let bssid_field = env
            .get_field(scan_result, "BSSID", "Ljava/lang/String;")
            .ok()?;
        let bssid_obj = bssid_field.l().ok()?;
        let bssid = if bssid_obj.is_null() {
            None
        } else {
            env.get_string((&bssid_obj).into())
                .ok()
                .and_then(|s| parse_mac_address(&String::from(s)))
        };

        // Get signal level (RSSI)
        let level = env
            .get_field(scan_result, "level", "I")
            .ok()
            .and_then(|v| v.i().ok())
            .unwrap_or(-127);

        // Get frequency
        let frequency = env
            .get_field(scan_result, "frequency", "I")
            .ok()
            .and_then(|v| v.i().ok())
            .unwrap_or(0);

        // Get capabilities (security info)
        let caps_field = env
            .get_field(scan_result, "capabilities", "Ljava/lang/String;")
            .ok()?;
        let caps_obj = caps_field.l().ok()?;
        let capabilities = if caps_obj.is_null() {
            String::new()
        } else {
            env.get_string((&caps_obj).into())
                .ok()
                .map(|s| String::from(s))
                .unwrap_or_default()
        };

        // Get channel width if available (API 23+)
        let channel_width = env
            .get_field(scan_result, "channelWidth", "I")
            .ok()
            .and_then(|v| v.i().ok())
            .map(|w| match w {
                0 => ChannelWidth::Mhz20,
                1 => ChannelWidth::Mhz40,
                2 => ChannelWidth::Mhz80,
                3 => ChannelWidth::Mhz160,
                4 => ChannelWidth::Mhz80Plus80,
                _ => ChannelWidth::Mhz20,
            })
            .unwrap_or(ChannelWidth::Mhz20);

        let (channel_number, band) = frequency_to_channel(frequency);
        let security = parse_security_capabilities(&capabilities);

        Some(AccessPoint {
            ssid: Ssid::new(&ssid).ok()?,
            bssid: bssid.unwrap_or(WifiMacAddress::from([0, 0, 0, 0, 0, 0])),
            rssi: level,
            noise: None,
            channel: Channel {
                number: channel_number as u8,
                band,
                width: channel_width,
            },
            security,
            wifi_standard: infer_wifi_standard_from_caps(&capabilities, frequency),
            supported_rates: Vec::new(),
            last_seen: std::time::SystemTime::now(),
        })
    }

    /// Checks if WiFi is enabled via JNI.
    #[cfg(target_os = "android")]
    fn is_wifi_enabled_jni(&self) -> bool {
        use ndk_context::android_context;

        let ctx = android_context();
        let vm = match unsafe { jni::JavaVM::from_raw(ctx.vm() as *mut _) } {
            Ok(vm) => vm,
            Err(_) => return false,
        };

        let mut env = match vm.attach_current_thread() {
            Ok(env) => env,
            Err(_) => return false,
        };

        let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

        let wifi_service = match env.new_string("wifi") {
            Ok(s) => s,
            Err(_) => return false,
        };

        let wifi_manager = env
            .call_method(
                &activity,
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[JValue::Object(&wifi_service.into())],
            )
            .ok()
            .and_then(|v| v.l().ok());

        if let Some(wm) = wifi_manager {
            if !wm.is_null() {
                return env
                    .call_method(&wm, "isWifiEnabled", "()Z", &[])
                    .ok()
                    .and_then(|v| v.z().ok())
                    .unwrap_or(false);
            }
        }

        false
    }

    #[cfg(not(target_os = "android"))]
    fn get_connection_info_jni(&self) -> Result<Option<WifiConnection>> {
        Ok(None)
    }

    #[cfg(not(target_os = "android"))]
    fn get_scan_results_jni(&self) -> Result<Vec<AccessPoint>> {
        Ok(Vec::new())
    }

    #[cfg(not(target_os = "android"))]
    fn is_wifi_enabled_jni(&self) -> bool {
        false
    }
}

impl Default for AndroidWifiProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WifiProvider for AndroidWifiProvider {
    fn is_available(&self) -> bool {
        self.is_wifi_enabled_jni()
    }

    async fn list_wifi_interfaces(&self) -> Result<Vec<WifiInterface>> {
        // Android typically has one WiFi interface (wlan0)
        let is_enabled = self.is_wifi_enabled_jni();
        Ok(vec![WifiInterface {
            name: "wlan0".to_string(),
            display_name: Some("Wi-Fi".to_string()),
            hardware_address: None,
            is_powered_on: is_enabled,
        }])
    }

    async fn scan_access_points(&self, _interface: &str) -> Result<Vec<AccessPoint>> {
        // Note: This requires ACCESS_FINE_LOCATION permission on Android 8.0+
        // and the app must have location services enabled
        self.get_scan_results_jni()
    }

    async fn get_current_connection(&self, _interface: &str) -> Result<Option<WifiConnection>> {
        // Note: This requires ACCESS_FINE_LOCATION on Android 8.0+
        self.get_connection_info_jni()
    }

    async fn get_signal_strength(&self, _interface: &str) -> Result<Option<i32>> {
        // Get RSSI from connection info
        if let Ok(Some(connection)) = self.get_connection_info_jni() {
            Ok(Some(connection.access_point.rssi))
        } else {
            Ok(None)
        }
    }

    async fn get_noise_level(&self, _interface: &str) -> Result<Option<i32>> {
        // Noise level is not available through standard Android APIs
        Ok(None)
    }

    async fn get_channel_utilization(&self, _channel: Channel) -> Result<ChannelUtilization> {
        Err(Error::unsupported("Channel utilization", "Android"))
    }

    async fn analyze_channels(&self, _interface: &str) -> Result<Vec<ChannelUtilization>> {
        // Analyze channels based on scan results
        let scan_results = self.get_scan_results_jni()?;

        // Group by channel and count APs
        use std::collections::HashMap;
        let mut channel_counts: HashMap<(u8, ChannelBand), (i32, i32)> = HashMap::new();

        for ap in &scan_results {
            let key = (ap.channel.number, ap.channel.band);
            let entry = channel_counts.entry(key).or_insert((0, -100));
            entry.0 += 1; // AP count
            if ap.rssi > entry.1 {
                entry.1 = ap.rssi; // Max RSSI
            }
        }

        let utilizations = channel_counts
            .into_iter()
            .map(|((number, band), (ap_count, max_rssi))| {
                // Estimate utilization based on AP count
                let utilization = (ap_count as f32 * 10.0).min(100.0);
                ChannelUtilization {
                    channel: Channel {
                        number,
                        band,
                        width: ChannelWidth::Mhz20,
                    },
                    utilization_percent: utilization,
                    access_point_count: ap_count as u32,
                    noise_floor: None,
                    interference_level: if ap_count > 5 {
                        Some(netdiag_types::wifi::InterferenceLevel::High)
                    } else if ap_count > 2 {
                        Some(netdiag_types::wifi::InterferenceLevel::Medium)
                    } else {
                        Some(netdiag_types::wifi::InterferenceLevel::Low)
                    },
                    recommended: ap_count <= 2,
                }
            })
            .collect();

        Ok(utilizations)
    }

    fn supports_enterprise(&self) -> bool {
        // Android supports WPA2/WPA3 Enterprise
        true
    }

    async fn trigger_scan(&self, _interface: &str) -> Result<()> {
        // Note: startScan() is throttled on Android 8+ (4 scans in 2 minutes for foreground apps)
        // and deprecated in favor of registerScanResultsCallback on Android 10+
        #[cfg(target_os = "android")]
        {
            use ndk_context::android_context;

            let ctx = android_context();
            let vm = unsafe { jni::JavaVM::from_raw(ctx.vm() as *mut _) }.map_err(|e| {
                Error::Platform {
                    message: format!("Failed to get JavaVM: {}", e),
                    source: None,
                }
            })?;

            let mut env = vm.attach_current_thread().map_err(|e| Error::Platform {
                message: format!("Failed to attach thread: {}", e),
                source: None,
            })?;

            let activity = unsafe { JObject::from_raw(ctx.context() as jni::sys::jobject) };

            let wifi_service = env.new_string("wifi").map_err(|e| Error::Platform {
                message: format!("Failed to create wifi string: {}", e),
                source: None,
            })?;

            let wifi_manager = env
                .call_method(
                    &activity,
                    "getSystemService",
                    "(Ljava/lang/String;)Ljava/lang/Object;",
                    &[JValue::Object(&wifi_service.into())],
                )
                .map_err(|e| Error::Platform {
                    message: format!("Failed to get WifiManager: {}", e),
                    source: None,
                })?
                .l()
                .map_err(|e| Error::Platform {
                    message: format!("WifiManager not an object: {}", e),
                    source: None,
                })?;

            if wifi_manager.is_null() {
                return Err(Error::Platform {
                    message: "WifiManager is null".to_string(),
                    source: None,
                });
            }

            // Call startScan()
            let success = env
                .call_method(&wifi_manager, "startScan", "()Z", &[])
                .map_err(|e| Error::Platform {
                    message: format!("Failed to start scan: {}", e),
                    source: None,
                })?
                .z()
                .unwrap_or(false);

            if !success {
                return Err(Error::Platform {
                    message: "WiFi scan was throttled or failed".to_string(),
                    source: None,
                });
            }

            Ok(())
        }

        #[cfg(not(target_os = "android"))]
        Err(Error::unsupported("WiFi scan trigger", "non-Android"))
    }

    async fn get_supported_standards(&self, _interface: &str) -> Result<Vec<WifiStandard>> {
        // Modern Android devices support up to WiFi 6E/7
        Ok(vec![
            WifiStandard::B,
            WifiStandard::G,
            WifiStandard::N,
            WifiStandard::Ac,
            WifiStandard::Ax,
        ])
    }
}

/// Converts WiFi frequency (MHz) to channel number and band.
fn frequency_to_channel(frequency: i32) -> (i32, ChannelBand) {
    match frequency {
        // 2.4 GHz band (2412 - 2484 MHz)
        2412..=2484 => {
            let channel = if frequency == 2484 {
                14 // Japan only
            } else {
                (frequency - 2407) / 5
            };
            (channel, ChannelBand::TwoGhz)
        }
        // 5 GHz band (5170 - 5825 MHz)
        5170..=5825 => {
            let channel = (frequency - 5000) / 5;
            (channel, ChannelBand::FiveGhz)
        }
        // 6 GHz band (5935 - 7115 MHz)
        5935..=7115 => {
            let channel = (frequency - 5950) / 5;
            (channel, ChannelBand::SixGhz)
        }
        _ => (0, ChannelBand::TwoGhz),
    }
}

/// Parses security capabilities string from Android ScanResult.
fn parse_security_capabilities(capabilities: &str) -> SecurityType {
    let caps = capabilities.to_uppercase();

    if caps.contains("WPA3-SAE") || caps.contains("SAE") {
        SecurityType::Wpa3Personal
    } else if caps.contains("WPA3") {
        if caps.contains("EAP") {
            SecurityType::Wpa3Enterprise
        } else {
            SecurityType::Wpa3Personal
        }
    } else if caps.contains("WPA2") {
        if caps.contains("EAP") || caps.contains("802.1X") {
            SecurityType::Wpa2Enterprise
        } else {
            SecurityType::Wpa2Personal
        }
    } else if caps.contains("WPA") {
        if caps.contains("EAP") || caps.contains("802.1X") {
            SecurityType::WpaEnterprise
        } else {
            SecurityType::WpaPersonal
        }
    } else if caps.contains("WEP") {
        SecurityType::Wep
    } else if caps.contains("ESS") && !caps.contains("WPA") && !caps.contains("WEP") {
        SecurityType::Open
    } else {
        SecurityType::Unknown
    }
}

/// Infers WiFi standard from frequency and link speed.
fn infer_wifi_standard(frequency: i32, link_speed: i32) -> WifiStandard {
    match frequency {
        5935..=7115 => WifiStandard::Ax, // 6 GHz = WiFi 6E
        5170..=5825 => {
            // 5 GHz
            if link_speed > 1000 {
                WifiStandard::Ax // WiFi 6 speeds
            } else if link_speed > 400 {
                WifiStandard::Ac // WiFi 5 speeds
            } else {
                WifiStandard::N // WiFi 4
            }
        }
        2412..=2484 => {
            // 2.4 GHz
            if link_speed > 200 {
                WifiStandard::Ax // WiFi 6
            } else if link_speed > 54 {
                WifiStandard::N // WiFi 4
            } else if link_speed > 11 {
                WifiStandard::G
            } else {
                WifiStandard::B
            }
        }
        _ => WifiStandard::Unknown,
    }
}

/// Infers WiFi standard from capabilities string and frequency.
fn infer_wifi_standard_from_caps(capabilities: &str, frequency: i32) -> WifiStandard {
    let caps = capabilities.to_uppercase();

    // Check for explicit standard indicators
    if caps.contains("[HE]") || caps.contains("[AX]") || (5935..=7115).contains(&frequency) {
        WifiStandard::Ax
    } else if caps.contains("[VHT]") || caps.contains("[AC]") {
        WifiStandard::Ac
    } else if caps.contains("[HT]") {
        WifiStandard::N
    } else if (5170..=5825).contains(&frequency) {
        // 5 GHz without HT/VHT indicators, assume at least 802.11n
        WifiStandard::N
    } else {
        // 2.4 GHz, could be b/g/n
        WifiStandard::G
    }
}

/// Parses a MAC address string (e.g., "AA:BB:CC:DD:EE:FF") into a WifiMacAddress.
fn parse_mac_address(s: &str) -> Option<WifiMacAddress> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 6 {
        return None;
    }

    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        bytes[i] = u8::from_str_radix(part, 16).ok()?;
    }

    Some(WifiMacAddress::from(bytes))
}
