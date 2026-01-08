//! Info command implementation.

use color_eyre::eyre::Result;
use console::style;
use netdiag_platform::{PlatformInfo, PlatformProviders};

/// Creates platform-specific providers.
fn create_providers() -> PlatformProviders {
    #[cfg(target_os = "macos")]
    {
        netdiag_platform_macos::create_providers()
    }
    #[cfg(target_os = "linux")]
    {
        netdiag_platform_linux::create_providers()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        PlatformProviders::new()
    }
}

/// Run the info command.
pub async fn run() -> Result<()> {
    let platform = PlatformInfo::detect();
    let providers = create_providers();

    println!("{}", style("System Information").bold().underlined());
    println!();

    // Platform info
    println!("  {} {}", style("OS:").bold(), platform.os_type);
    println!("  {} {}", style("Version:").bold(), platform.os_version);
    println!("  {} {}", style("Architecture:").bold(), platform.arch);

    if platform.is_container {
        println!("  {} Running in container", style("Note:").yellow());
    }
    if platform.is_vm {
        println!("  {} Running in virtual machine", style("Note:").yellow());
    }

    println!();
    println!("{}", style("Network Information").bold().underlined());
    println!();

    // Network interfaces
    match providers.network.list_interfaces().await {
        Ok(interfaces) if !interfaces.is_empty() => {
            println!("  {} Found {} interface(s)", style("Interfaces:").bold(), interfaces.len());
            for iface in interfaces {
                let status = if iface.is_up() {
                    style("UP").green()
                } else {
                    style("DOWN").red()
                };
                println!("    {} {} ({})", style(&iface.name).cyan(), status, iface.interface_type);

                if let Some(mac) = &iface.mac_address {
                    println!("      MAC: {}", mac);
                }
                for ip in &iface.ipv4_addresses {
                    println!("      IPv4: {}", ip.address);
                }
                for ip in &iface.ipv6_addresses {
                    println!("      IPv6: {}", ip.address);
                }
            }
        }
        Ok(_) => {
            println!("  {} No network interfaces found", style("Interfaces:").bold());
        }
        Err(e) => {
            println!("  {} Error getting interfaces: {}", style("Interfaces:").red(), e);
        }
    }

    println!();

    // Default gateway
    match providers.network.get_default_gateway().await {
        Ok(Some(gateway)) => {
            println!("  {} {}", style("Gateway:").bold(), gateway.address);
            println!("    Interface: {}", gateway.interface);
            if let Some(hostname) = &gateway.hostname {
                println!("    Hostname: {}", hostname);
            }
        }
        Ok(None) => {
            println!("  {} No default gateway", style("Gateway:").yellow());
        }
        Err(e) => {
            println!("  {} Error: {}", style("Gateway:").red(), e);
        }
    }

    // DNS servers
    match providers.network.get_dns_servers().await {
        Ok(servers) if !servers.is_empty() => {
            println!("  {} {} server(s)", style("DNS:").bold(), servers.len());
            for server in servers {
                let protocol = format!("{}", server.protocol);
                println!("    {} ({})", server.address, protocol);
            }
        }
        Ok(_) => {
            println!("  {} No DNS servers configured", style("DNS:").yellow());
        }
        Err(e) => {
            println!("  {} Error: {}", style("DNS:").red(), e);
        }
    }

    // WiFi info
    if providers.wifi.is_available() {
        println!();
        println!("{}", style("WiFi Information").bold().underlined());
        println!();

        match providers.wifi.list_wifi_interfaces().await {
            Ok(interfaces) if !interfaces.is_empty() => {
                for iface in interfaces {
                    let status = if iface.connected {
                        style("Connected").green()
                    } else {
                        style("Disconnected").yellow()
                    };
                    println!("  {} {} ({})", style(&iface.name).cyan(), status,
                        if iface.powered_on { "On" } else { "Off" });
                }
            }
            Ok(_) => {
                println!("  No WiFi interfaces found");
            }
            Err(e) => {
                println!("  {} Error: {}", style("WiFi:").red(), e);
            }
        }
    }

    println!();
    println!("{}", style("Capabilities").bold().underlined());
    println!();

    let privilege = providers.privilege.current_privilege_level();
    println!("  {} {:?}", style("Privilege Level:").bold(), privilege);

    let caps = providers.privilege.available_capabilities();
    if caps.is_empty() {
        println!("  {} None (run with elevated privileges for more)", style("Capabilities:").yellow());
    } else {
        println!("  {} {}", style("Capabilities:").bold(), caps.len());
        for cap in caps {
            println!("    - {}", cap.description());
        }
    }

    Ok(())
}
