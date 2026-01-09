//! Android network provider implementation.
//!
//! Uses JNI to access Android's ConnectivityManager and NetworkInterface APIs.

use async_trait::async_trait;
use netdiag_platform::NetworkProvider;
use netdiag_types::{
    error::Result,
    network::{
        DhcpInfo, DnsServer, Gateway, InterfaceFlags, InterfaceType, IpSubnet, Ipv4Info, Ipv6Info,
        Ipv6Scope, IspInfo, MacAddress, NetworkInterface, Route,
    },
    Error,
};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[cfg(target_os = "android")]
use jni::{
    objects::{JObject, JString, JValue},
    JNIEnv,
};

/// Android network provider using JNI to access Android APIs.
pub struct AndroidNetworkProvider {
    // JNI context would be stored here in a production implementation
}

impl AndroidNetworkProvider {
    /// Creates a new Android network provider.
    pub fn new() -> Self {
        Self {}
    }

    /// Gets network interfaces using Java's NetworkInterface.getNetworkInterfaces().
    #[cfg(target_os = "android")]
    fn get_interfaces_jni(&self) -> Result<Vec<NetworkInterface>> {
        use ndk_context::android_context;

        let ctx = android_context();
        let vm =
            unsafe { jni::JavaVM::from_raw(ctx.vm() as *mut _) }.map_err(|e| Error::Platform {
                message: "Failed to get JavaVM".to_string(),
                source: Some(e.to_string()),
            })?;

        let mut env = vm.attach_current_thread().map_err(|e| Error::Platform {
            message: "Failed to attach to JNI thread".to_string(),
            source: Some(e.to_string()),
        })?;

        self.enumerate_interfaces(&mut env)
    }

    #[cfg(target_os = "android")]
    fn enumerate_interfaces(&self, env: &mut JNIEnv) -> Result<Vec<NetworkInterface>> {
        let mut interfaces = Vec::new();

        // Call NetworkInterface.getNetworkInterfaces()
        let net_iface_class =
            env.find_class("java/net/NetworkInterface")
                .map_err(|e| Error::Platform {
                    message: "Failed to find NetworkInterface class".to_string(),
                    source: Some(e.to_string()),
                })?;

        let ifaces_result = env
            .call_static_method(
                &net_iface_class,
                "getNetworkInterfaces",
                "()Ljava/util/Enumeration;",
                &[],
            )
            .map_err(|e| Error::Platform {
                message: "Failed to call getNetworkInterfaces".to_string(),
                source: Some(e.to_string()),
            })?;

        let enumeration = match ifaces_result {
            JValue::Object(obj) => obj,
            _ => return Ok(interfaces),
        };

        if enumeration.is_null() {
            return Ok(interfaces);
        }

        // Iterate through enumeration
        loop {
            let has_more = env
                .call_method(&enumeration, "hasMoreElements", "()Z", &[])
                .map_err(|e| Error::Platform {
                    message: "Failed to call hasMoreElements".to_string(),
                    source: Some(e.to_string()),
                })?
                .z()
                .unwrap_or(false);

            if !has_more {
                break;
            }

            let iface_obj = env
                .call_method(&enumeration, "nextElement", "()Ljava/lang/Object;", &[])
                .map_err(|e| Error::Platform {
                    message: "Failed to call nextElement".to_string(),
                    source: Some(e.to_string()),
                })?;

            if let JValue::Object(iface) = iface_obj {
                if let Some(net_iface) = self.parse_interface(env, &iface)? {
                    interfaces.push(net_iface);
                }
            }
        }

        Ok(interfaces)
    }

    #[cfg(target_os = "android")]
    fn parse_interface(
        &self,
        env: &mut JNIEnv,
        iface: &JObject,
    ) -> Result<Option<NetworkInterface>> {
        // Get interface name
        let name_obj = env
            .call_method(iface, "getName", "()Ljava/lang/String;", &[])
            .map_err(|e| Error::Platform {
                message: "Failed to get interface name".to_string(),
                source: Some(e.to_string()),
            })?;

        let name = match name_obj {
            JValue::Object(obj) => {
                let jstr = JString::from(obj);
                env.get_string(&jstr).map(|s| s.into()).unwrap_or_default()
            }
            _ => return Ok(None),
        };

        // Get interface index
        let index = env
            .call_method(iface, "getIndex", "()I", &[])
            .map_err(|e| Error::Platform {
                message: "Failed to get interface index".to_string(),
                source: Some(e.to_string()),
            })?
            .i()
            .unwrap_or(0) as u32;

        // Check if up
        let is_up = env
            .call_method(iface, "isUp", "()Z", &[])
            .map_err(|e| Error::Platform {
                message: "Failed to check if interface is up".to_string(),
                source: Some(e.to_string()),
            })?
            .z()
            .unwrap_or(false);

        // Check if loopback
        let is_loopback = env
            .call_method(iface, "isLoopback", "()Z", &[])
            .map_err(|e| Error::Platform {
                message: "Failed to check if interface is loopback".to_string(),
                source: Some(e.to_string()),
            })?
            .z()
            .unwrap_or(false);

        // Check if point-to-point
        let is_ptp = env
            .call_method(iface, "isPointToPoint", "()Z", &[])
            .map_err(|e| Error::Platform {
                message: "Failed to check if interface is point-to-point".to_string(),
                source: Some(e.to_string()),
            })?
            .z()
            .unwrap_or(false);

        // Check if supports multicast
        let supports_multicast = env
            .call_method(iface, "supportsMulticast", "()Z", &[])
            .map_err(|e| Error::Platform {
                message: "Failed to check if interface supports multicast".to_string(),
                source: Some(e.to_string()),
            })?
            .z()
            .unwrap_or(false);

        // Get MTU
        let mtu = env
            .call_method(iface, "getMTU", "()I", &[])
            .ok()
            .and_then(|v| v.i().ok())
            .map(|m| m as u32);

        // Get hardware address (MAC)
        let mac_address = self.get_mac_address(env, iface)?;

        // Get IP addresses
        let (ipv4_addresses, ipv6_addresses) = self.get_ip_addresses(env, iface)?;

        Ok(Some(NetworkInterface {
            name: name.clone(),
            display_name: Some(get_display_name(&name)),
            index,
            interface_type: detect_interface_type(&name),
            mac_address,
            ipv4_addresses,
            ipv6_addresses,
            flags: InterfaceFlags {
                up: is_up,
                running: is_up,
                broadcast: !is_loopback && !is_ptp,
                loopback: is_loopback,
                point_to_point: is_ptp,
                multicast: supports_multicast,
                promiscuous: false,
            },
            mtu,
            speed_mbps: None,
            is_default: is_default_interface(&name),
        }))
    }

    #[cfg(target_os = "android")]
    fn get_mac_address(&self, env: &mut JNIEnv, iface: &JObject) -> Result<Option<MacAddress>> {
        let mac_result = env.call_method(iface, "getHardwareAddress", "()[B", &[]);

        if let Ok(JValue::Object(mac_obj)) = mac_result {
            if mac_obj.is_null() {
                return Ok(None);
            }

            let mac_array = env.convert_byte_array(mac_obj.into()).ok();
            if let Some(bytes) = mac_array {
                if bytes.len() == 6 {
                    let mut mac = [0u8; 6];
                    mac.copy_from_slice(&bytes);
                    if mac.iter().any(|&b| b != 0) {
                        return Ok(Some(MacAddress::from(mac)));
                    }
                }
            }
        }

        Ok(None)
    }

    #[cfg(target_os = "android")]
    fn get_ip_addresses(
        &self,
        env: &mut JNIEnv,
        iface: &JObject,
    ) -> Result<(Vec<Ipv4Info>, Vec<Ipv6Info>)> {
        let mut ipv4_addresses = Vec::new();
        let mut ipv6_addresses = Vec::new();

        // Get interface addresses
        let addrs_result =
            env.call_method(iface, "getInterfaceAddresses", "()Ljava/util/List;", &[]);

        if let Ok(JValue::Object(addrs_list)) = addrs_result {
            if addrs_list.is_null() {
                return Ok((ipv4_addresses, ipv6_addresses));
            }

            let size = env
                .call_method(&addrs_list, "size", "()I", &[])
                .ok()
                .and_then(|v| v.i().ok())
                .unwrap_or(0);

            for i in 0..size {
                let addr_obj = env.call_method(
                    &addrs_list,
                    "get",
                    "(I)Ljava/lang/Object;",
                    &[JValue::Int(i)],
                );

                if let Ok(JValue::Object(iface_addr)) = addr_obj {
                    // Get address
                    let addr_result =
                        env.call_method(&iface_addr, "getAddress", "()Ljava/net/InetAddress;", &[]);

                    if let Ok(JValue::Object(inet_addr)) = addr_result {
                        let host_addr = env
                            .call_method(&inet_addr, "getHostAddress", "()Ljava/lang/String;", &[])
                            .ok();

                        if let Some(JValue::Object(host_str)) = host_addr {
                            let addr_str: String = env
                                .get_string(&JString::from(host_str))
                                .map(|s| s.into())
                                .unwrap_or_default();

                            // Get prefix length
                            let prefix_len = env
                                .call_method(&iface_addr, "getNetworkPrefixLength", "()S", &[])
                                .ok()
                                .and_then(|v| v.s().ok())
                                .unwrap_or(24) as u8;

                            // Parse address
                            if let Ok(ip) = addr_str.parse::<IpAddr>() {
                                match ip {
                                    IpAddr::V4(addr) => {
                                        let mask = !((1u32 << (32 - prefix_len)) - 1);
                                        let addr_u32 = u32::from(addr);
                                        let network = Ipv4Addr::from(addr_u32 & mask);

                                        ipv4_addresses.push(Ipv4Info {
                                            address: addr,
                                            subnet: IpSubnet::V4 {
                                                address: network,
                                                prefix_len,
                                            },
                                            broadcast: None,
                                        });
                                    }
                                    IpAddr::V6(addr) => {
                                        let scope = if addr.is_loopback() {
                                            Ipv6Scope::Loopback
                                        } else if addr.segments()[0] & 0xffc0 == 0xfe80 {
                                            Ipv6Scope::LinkLocal
                                        } else {
                                            Ipv6Scope::Global
                                        };

                                        ipv6_addresses.push(Ipv6Info {
                                            address: addr,
                                            prefix_len,
                                            scope,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((ipv4_addresses, ipv6_addresses))
    }

    /// Fallback implementation when not on Android.
    #[cfg(not(target_os = "android"))]
    fn get_interfaces_jni(&self) -> Result<Vec<NetworkInterface>> {
        // Return minimal stub data for non-Android builds
        Ok(vec![NetworkInterface {
            name: "lo".to_string(),
            display_name: Some("Loopback".to_string()),
            index: 1,
            interface_type: InterfaceType::Loopback,
            mac_address: None,
            ipv4_addresses: vec![Ipv4Info {
                address: "127.0.0.1".parse().unwrap(),
                subnet: IpSubnet::V4 {
                    address: "127.0.0.0".parse().unwrap(),
                    prefix_len: 8,
                },
                broadcast: None,
            }],
            ipv6_addresses: vec![Ipv6Info {
                address: "::1".parse().unwrap(),
                prefix_len: 128,
                scope: Ipv6Scope::Loopback,
            }],
            flags: InterfaceFlags {
                up: true,
                running: true,
                broadcast: false,
                loopback: true,
                point_to_point: false,
                multicast: true,
                promiscuous: false,
            },
            mtu: Some(65536),
            speed_mbps: None,
            is_default: false,
        }])
    }
}

impl Default for AndroidNetworkProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkProvider for AndroidNetworkProvider {
    async fn list_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        self.get_interfaces_jni()
    }

    async fn get_interface(&self, name: &str) -> Result<Option<NetworkInterface>> {
        let interfaces = self.get_interfaces_jni()?;
        Ok(interfaces.into_iter().find(|i| i.name == name))
    }

    async fn get_default_interface(&self) -> Result<Option<NetworkInterface>> {
        let interfaces = self.get_interfaces_jni()?;
        // On Android, check for wlan0 (WiFi) or rmnet (cellular)
        Ok(interfaces
            .into_iter()
            .find(|i| i.is_default || i.name == "wlan0" || i.name.starts_with("rmnet")))
    }

    async fn get_default_route(&self) -> Result<Option<Route>> {
        // Would use ConnectivityManager.getLinkProperties().getRoutes()
        // For now, infer from default interface
        if let Some(iface) = self.get_default_interface().await? {
            if let Some(ipv4) = iface.ipv4_addresses.first() {
                let octets = ipv4.address.octets();
                let gateway_addr = Ipv4Addr::new(octets[0], octets[1], octets[2], 1);
                return Ok(Some(Route {
                    destination: IpSubnet::V4 {
                        address: Ipv4Addr::new(0, 0, 0, 0),
                        prefix_len: 0,
                    },
                    gateway: Some(IpAddr::V4(gateway_addr)),
                    interface: Some(iface.name),
                    metric: None,
                    flags: 0,
                }));
            }
        }
        Ok(None)
    }

    async fn get_routes(&self) -> Result<Vec<Route>> {
        // Routing table access is limited on Android
        Ok(Vec::new())
    }

    async fn get_default_gateway(&self) -> Result<Option<Gateway>> {
        // Infer gateway from default interface
        if let Some(iface) = self.get_default_interface().await? {
            if let Some(ipv4) = iface.ipv4_addresses.first() {
                let octets = ipv4.address.octets();
                let gateway_addr = Ipv4Addr::new(octets[0], octets[1], octets[2], 1);
                return Ok(Some(Gateway {
                    address: IpAddr::V4(gateway_addr),
                    interface: Some(iface.name),
                    is_default: true,
                    metric: None,
                }));
            }
        }
        Ok(None)
    }

    async fn get_dns_servers(&self) -> Result<Vec<DnsServer>> {
        // Would use LinkProperties.getDnsServers() via JNI
        Ok(Vec::new())
    }

    async fn get_dhcp_info(&self, _interface: &str) -> Result<Option<DhcpInfo>> {
        // Would use WifiManager.getDhcpInfo() for WiFi via JNI
        Ok(None)
    }

    async fn detect_isp(&self) -> Result<Option<IspInfo>> {
        // Would need external API call
        Ok(None)
    }

    fn supports_promiscuous(&self, _interface: &str) -> bool {
        // Promiscuous mode requires root on Android
        false
    }

    async fn refresh(&self) -> Result<()> {
        Ok(())
    }
}

/// Gets a human-readable display name for an interface.
fn get_display_name(name: &str) -> String {
    match name {
        "lo" => "Loopback".to_string(),
        "wlan0" => "Wi-Fi".to_string(),
        "eth0" => "Ethernet".to_string(),
        n if n.starts_with("rmnet") => "Cellular".to_string(),
        n if n.starts_with("tun") => "VPN Tunnel".to_string(),
        n if n.starts_with("p2p") => "Wi-Fi Direct".to_string(),
        _ => name.to_string(),
    }
}

/// Detects the interface type from its name.
fn detect_interface_type(name: &str) -> InterfaceType {
    match name {
        "lo" => InterfaceType::Loopback,
        "wlan0" => InterfaceType::Wifi,
        "eth0" => InterfaceType::Ethernet,
        n if n.starts_with("rmnet") => InterfaceType::Cellular,
        n if n.starts_with("tun") => InterfaceType::Vpn,
        n if n.starts_with("p2p") => InterfaceType::Wifi,
        _ => InterfaceType::Other,
    }
}

/// Determines if this is likely the default interface.
fn is_default_interface(name: &str) -> bool {
    // On Android, wlan0 (WiFi) or rmnet0 (cellular) is typically default
    name == "wlan0" || name == "rmnet0"
}
