//! WiFi command implementation.

use crate::app::{WifiArgs, WifiCommands};
use color_eyre::eyre::Result;
use console::style;
use netdiag_platform::PlatformProviders;
use netdiag_types::wifi::{InterferenceLevel, WifiBand};

#[cfg(target_os = "macos")]
use netdiag_platform_macos::create_providers;

#[cfg(target_os = "linux")]
use netdiag_platform_linux::create_providers;

#[cfg(target_os = "windows")]
use netdiag_platform_windows::create_providers;

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn create_providers() -> PlatformProviders {
    PlatformProviders::new()
}

/// Run the WiFi command.
pub async fn run(args: WifiArgs) -> Result<()> {
    let providers = create_providers();

    match args.command {
        Some(WifiCommands::Scan) => run_scan(&providers).await,
        Some(WifiCommands::Status) => run_status(&providers).await,
        Some(WifiCommands::Channels) => run_channels(&providers).await,
        Some(WifiCommands::Interference) => run_interference(&providers).await,
        None => run_status(&providers).await,
    }
}

async fn run_scan(providers: &PlatformProviders) -> Result<()> {
    println!("{}", style("Scanning for WiFi networks...").bold());
    println!();

    if !providers.wifi.is_available() {
        println!("{}", style("WiFi is not available on this system").yellow());
        return Ok(());
    }

    // Get WiFi interface
    let interfaces = providers.wifi.list_wifi_interfaces().await?;
    let interface = match interfaces.first() {
        Some(i) => i,
        None => {
            println!("{}", style("No WiFi interfaces found").yellow());
            return Ok(());
        }
    };

    // Trigger a scan
    if let Err(e) = providers.wifi.trigger_scan(&interface.name).await {
        // Scan may require elevated privileges, continue with cached results
        eprintln!(
            "{}",
            style(format!("Note: Could not trigger active scan: {}", e)).dim()
        );
    }

    // Get access points
    let access_points = match providers.wifi.scan_access_points(&interface.name).await {
        Ok(aps) => aps,
        Err(e) => {
            println!(
                "{}",
                style(format!("Failed to scan access points: {}", e)).red()
            );
            return Ok(());
        }
    };

    if access_points.is_empty() {
        println!("{}", style("No WiFi networks found").yellow());
        return Ok(());
    }

    println!(
        "{:<25} {:>8} {:>8} {:>12} {:>12}",
        style("SSID").bold(),
        style("Signal").bold(),
        style("Channel").bold(),
        style("Security").bold(),
        style("Standard").bold()
    );
    println!("{}", "-".repeat(70));

    for ap in &access_points {
        let ssid_display = if ap.is_hidden {
            "<hidden>".to_string()
        } else {
            let ssid_str = ap.ssid.as_str();
            if ssid_str.len() > 24 {
                format!("{}...", &ssid_str[..21])
            } else {
                ssid_str.to_string()
            }
        };

        let signal = format!("{} dBm", ap.rssi);
        let signal_colored = if ap.rssi > -50 {
            style(signal).green()
        } else if ap.rssi > -70 {
            style(signal).yellow()
        } else {
            style(signal).red()
        };

        let ssid_styled = if ap.is_connected {
            style(format!("* {}", ssid_display)).cyan().bold()
        } else {
            style(ssid_display).cyan()
        };

        let standard_display = ap
            .wifi_standard
            .marketing_name()
            .unwrap_or(&ap.wifi_standard.to_string())
            .to_string();

        println!(
            "{:<25} {:>8} {:>8} {:>12} {:>12}",
            ssid_styled,
            signal_colored,
            ap.channel.number,
            ap.security,
            standard_display
        );
    }

    println!();
    println!("Found {} networks", access_points.len());
    println!("{}", style("* = currently connected").dim());

    Ok(())
}

async fn run_status(providers: &PlatformProviders) -> Result<()> {
    println!("{}", style("WiFi Status").bold().underlined());
    println!();

    if !providers.wifi.is_available() {
        println!("{}", style("WiFi is not available on this system").yellow());
        return Ok(());
    }

    // Get WiFi interface
    let interfaces = providers.wifi.list_wifi_interfaces().await?;
    let interface = match interfaces.first() {
        Some(i) => i,
        None => {
            println!("{}", style("No WiFi interfaces found").yellow());
            return Ok(());
        }
    };

    println!(
        "  {} {}",
        style("Interface:").bold(),
        style(&interface.name).cyan()
    );
    println!(
        "  {} {}",
        style("Power:").bold(),
        if interface.powered_on {
            style("On").green()
        } else {
            style("Off").red()
        }
    );

    if let Some(ref mac) = interface.mac_address {
        println!("  {} {}", style("MAC Address:").bold(), mac);
    }

    if let Some(ref country) = interface.country_code {
        println!("  {} {}", style("Country:").bold(), country);
    }

    // Get current connection
    match providers.wifi.get_current_connection(&interface.name).await {
        Ok(Some(conn)) => {
            println!();
            println!("{}", style("Connection:").bold().underlined());
            println!(
                "  {} {}",
                style("Status:").bold(),
                style("Connected").green()
            );
            println!(
                "  {} {}",
                style("SSID:").bold(),
                style(conn.access_point.ssid.as_str()).cyan()
            );
            println!("  {} {}", style("BSSID:").bold(), conn.access_point.bssid);

            // Signal strength
            let signal_quality = match conn.access_point.rssi {
                r if r >= -50 => "Excellent",
                r if r >= -60 => "Good",
                r if r >= -70 => "Fair",
                _ => "Poor",
            };
            let signal_style = match conn.access_point.rssi {
                r if r >= -60 => style(signal_quality).green(),
                r if r >= -70 => style(signal_quality).yellow(),
                _ => style(signal_quality).red(),
            };
            println!(
                "  {} {} dBm ({})",
                style("Signal:").bold(),
                conn.access_point.rssi,
                signal_style
            );

            if let Some(noise) = conn.access_point.noise {
                let snr = conn.access_point.rssi - noise;
                println!("  {} {} dBm", style("Noise:").bold(), noise);
                println!("  {} {} dB", style("SNR:").bold(), snr);
            }

            // Channel
            let band_str = match conn.access_point.channel.band {
                WifiBand::Band2_4GHz => "2.4 GHz",
                WifiBand::Band5GHz => "5 GHz",
                WifiBand::Band6GHz => "6 GHz",
            };
            println!(
                "  {} {} ({})",
                style("Channel:").bold(),
                conn.access_point.channel.number,
                band_str
            );
            println!(
                "  {} {}",
                style("Width:").bold(),
                conn.access_point.channel.width
            );

            // Security
            println!(
                "  {} {}",
                style("Security:").bold(),
                conn.access_point.security
            );

            // WiFi standard
            let standard_display = conn
                .access_point
                .wifi_standard
                .marketing_name()
                .map(|m| format!("{} ({})", conn.access_point.wifi_standard, m))
                .unwrap_or_else(|| conn.access_point.wifi_standard.to_string());
            println!("  {} {}", style("Standard:").bold(), standard_display);

            // Data rates
            if let Some(tx_rate) = conn.tx_rate {
                println!("  {} {:.0} Mbps", style("TX Rate:").bold(), tx_rate);
            }
            if let Some(rx_rate) = conn.rx_rate {
                println!("  {} {:.0} Mbps", style("RX Rate:").bold(), rx_rate);
            }

            // Connection duration
            if let Some(duration) = conn.connected_duration {
                let hours = duration.as_secs() / 3600;
                let mins = (duration.as_secs() % 3600) / 60;
                let secs = duration.as_secs() % 60;
                println!(
                    "  {} {}h {}m {}s",
                    style("Connected for:").bold(),
                    hours,
                    mins,
                    secs
                );
            }
        }
        Ok(None) => {
            println!();
            println!(
                "  {} {}",
                style("Status:").bold(),
                style("Not connected").yellow()
            );
        }
        Err(e) => {
            println!();
            println!(
                "  {} {}",
                style("Status:").bold(),
                style(format!("Error: {}", e)).red()
            );
        }
    }

    Ok(())
}

async fn run_channels(providers: &PlatformProviders) -> Result<()> {
    println!("{}", style("WiFi Channel Analysis").bold().underlined());
    println!();

    if !providers.wifi.is_available() {
        println!("{}", style("WiFi is not available on this system").yellow());
        return Ok(());
    }

    // Get WiFi interface
    let interfaces = providers.wifi.list_wifi_interfaces().await?;
    let interface = match interfaces.first() {
        Some(i) => i,
        None => {
            println!("{}", style("No WiFi interfaces found").yellow());
            return Ok(());
        }
    };

    // Get channel analysis
    let utilizations = match providers.wifi.analyze_channels(&interface.name).await {
        Ok(u) => u,
        Err(e) => {
            println!(
                "{}",
                style(format!("Failed to analyze channels: {}", e)).red()
            );
            return Ok(());
        }
    };

    if utilizations.is_empty() {
        println!("{}", style("No channel data available").yellow());
        return Ok(());
    }

    // Get current channel for highlighting
    let current_channel = providers
        .wifi
        .get_current_connection(&interface.name)
        .await
        .ok()
        .flatten()
        .map(|c| c.access_point.channel.number);

    // Group by band
    let mut band_2_4: Vec<_> = utilizations
        .iter()
        .filter(|u| u.channel.band == WifiBand::Band2_4GHz)
        .collect();
    let mut band_5: Vec<_> = utilizations
        .iter()
        .filter(|u| u.channel.band == WifiBand::Band5GHz)
        .collect();
    let mut band_6: Vec<_> = utilizations
        .iter()
        .filter(|u| u.channel.band == WifiBand::Band6GHz)
        .collect();

    band_2_4.sort_by_key(|u| u.channel.number);
    band_5.sort_by_key(|u| u.channel.number);
    band_6.sort_by_key(|u| u.channel.number);

    // Display 2.4 GHz channels
    if !band_2_4.is_empty() {
        println!("{}", style("2.4 GHz Band:").bold());
        for u in &band_2_4 {
            let is_current = current_channel == Some(u.channel.number);
            let utilization_style = match u.interference_level {
                InterferenceLevel::Low => style("Low").green(),
                InterferenceLevel::Medium => style("Medium").yellow(),
                InterferenceLevel::High => style("High").red(),
                InterferenceLevel::Severe => style("Severe").red().bold(),
            };

            let channel_str = if is_current {
                format!("  Channel {:>2}: ", u.channel.number)
            } else {
                format!("  Channel {:>2}: ", u.channel.number)
            };

            let suffix = if is_current { " (current)" } else { "" };
            let rec = if u.recommended { " *" } else { "" };

            if is_current {
                println!(
                    "{}{} networks, {} utilization{}{}",
                    style(channel_str).cyan().bold(),
                    u.network_count,
                    utilization_style,
                    suffix,
                    rec
                );
            } else {
                println!(
                    "{}{} networks, {} utilization{}",
                    channel_str, u.network_count, utilization_style, rec
                );
            }
        }
        println!();
    }

    // Display 5 GHz channels
    if !band_5.is_empty() {
        println!("{}", style("5 GHz Band:").bold());
        for u in &band_5 {
            let is_current = current_channel == Some(u.channel.number);
            let utilization_style = match u.interference_level {
                InterferenceLevel::Low => style("Low").green(),
                InterferenceLevel::Medium => style("Medium").yellow(),
                InterferenceLevel::High => style("High").red(),
                InterferenceLevel::Severe => style("Severe").red().bold(),
            };

            let dfs_note = if u.channel.is_dfs() { " [DFS]" } else { "" };
            let suffix = if is_current { " (current)" } else { "" };
            let rec = if u.recommended { " *" } else { "" };

            let channel_str = format!("  Channel {:>3}: ", u.channel.number);

            if is_current {
                println!(
                    "{}{} networks, {} utilization{}{}{}",
                    style(channel_str).cyan().bold(),
                    u.network_count,
                    utilization_style,
                    dfs_note,
                    suffix,
                    rec
                );
            } else {
                println!(
                    "{}{} networks, {} utilization{}{}",
                    channel_str, u.network_count, utilization_style, dfs_note, rec
                );
            }
        }
        println!();
    }

    // Display 6 GHz channels (if any)
    if !band_6.is_empty() {
        println!("{}", style("6 GHz Band (WiFi 6E):").bold());
        for u in &band_6 {
            let is_current = current_channel == Some(u.channel.number);
            let utilization_style = match u.interference_level {
                InterferenceLevel::Low => style("Low").green(),
                InterferenceLevel::Medium => style("Medium").yellow(),
                InterferenceLevel::High => style("High").red(),
                InterferenceLevel::Severe => style("Severe").red().bold(),
            };

            let suffix = if is_current { " (current)" } else { "" };
            let rec = if u.recommended { " *" } else { "" };

            let channel_str = format!("  Channel {:>3}: ", u.channel.number);

            if is_current {
                println!(
                    "{}{} networks, {} utilization{}{}",
                    style(channel_str).cyan().bold(),
                    u.network_count,
                    utilization_style,
                    suffix,
                    rec
                );
            } else {
                println!(
                    "{}{} networks, {} utilization{}",
                    channel_str, u.network_count, utilization_style, rec
                );
            }
        }
        println!();
    }

    println!("{}", style("* = recommended channel").dim());

    // Find best recommendation
    let recommended: Vec<_> = utilizations.iter().filter(|u| u.recommended).collect();
    if !recommended.is_empty() {
        println!();
        let best = recommended.iter().min_by(|a, b| {
            a.utilization_percent
                .partial_cmp(&b.utilization_percent)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        if let Some(best) = best {
            let band_str = match best.channel.band {
                WifiBand::Band2_4GHz => "2.4 GHz",
                WifiBand::Band5GHz => "5 GHz",
                WifiBand::Band6GHz => "6 GHz",
            };
            println!(
                "{} Consider channel {} ({}) for best performance",
                style("Recommendation:").cyan().bold(),
                best.channel.number,
                band_str
            );
        }
    }

    Ok(())
}

async fn run_interference(providers: &PlatformProviders) -> Result<()> {
    println!("{}", style("WiFi Interference Analysis").bold().underlined());
    println!();

    if !providers.wifi.is_available() {
        println!("{}", style("WiFi is not available on this system").yellow());
        return Ok(());
    }

    // Get WiFi interface
    let interfaces = providers.wifi.list_wifi_interfaces().await?;
    let interface = match interfaces.first() {
        Some(i) => i,
        None => {
            println!("{}", style("No WiFi interfaces found").yellow());
            return Ok(());
        }
    };

    // Get current connection
    let connection = match providers.wifi.get_current_connection(&interface.name).await {
        Ok(Some(c)) => c,
        Ok(None) => {
            println!(
                "{}",
                style("Not connected to WiFi - cannot analyze interference").yellow()
            );
            return Ok(());
        }
        Err(e) => {
            println!(
                "{}",
                style(format!("Failed to get connection info: {}", e)).red()
            );
            return Ok(());
        }
    };

    // Get noise and signal
    let signal = connection.access_point.rssi;
    let noise = providers
        .wifi
        .get_noise_level(&interface.name)
        .await
        .ok()
        .flatten();

    println!("{}", style("Current Connection:").bold());
    println!(
        "  {} {}",
        style("SSID:").bold(),
        connection.access_point.ssid
    );
    println!(
        "  {} {}",
        style("Channel:").bold(),
        connection.access_point.channel.number
    );

    // Signal quality
    println!();
    println!("{}", style("Signal Quality:").bold());
    println!("  {} {} dBm", style("Signal:").bold(), signal);

    if let Some(noise_level) = noise {
        let snr = signal - noise_level;
        let snr_quality = match snr {
            s if s >= 40 => ("Excellent", style("Excellent").green()),
            s if s >= 25 => ("Good", style("Good").green()),
            s if s >= 15 => ("Fair", style("Fair").yellow()),
            _ => ("Poor", style("Poor").red()),
        };
        println!("  {} {} dBm", style("Noise floor:").bold(), noise_level);
        println!("  {} {} dB ({})", style("SNR:").bold(), snr, snr_quality.1);
    } else if let Some(snr) = connection.access_point.snr {
        let snr_quality = match snr {
            s if s >= 40 => ("Excellent", style("Excellent").green()),
            s if s >= 25 => ("Good", style("Good").green()),
            s if s >= 15 => ("Fair", style("Fair").yellow()),
            _ => ("Poor", style("Poor").red()),
        };
        println!("  {} {} dB ({})", style("SNR:").bold(), snr, snr_quality.1);
    }

    // Get channel utilization for current channel
    let current_channel = connection.access_point.channel;
    match providers.wifi.get_channel_utilization(current_channel).await {
        Ok(util) => {
            println!();
            println!("{}", style("Channel Analysis:").bold());

            let level_style = match util.interference_level {
                InterferenceLevel::Low => style("Low").green(),
                InterferenceLevel::Medium => style("Medium").yellow(),
                InterferenceLevel::High => style("High").red(),
                InterferenceLevel::Severe => style("Severe").red().bold(),
            };

            println!(
                "  {} {}",
                style("Interference level:").bold(),
                level_style
            );
            println!(
                "  {} {}",
                style("Networks on channel:").bold(),
                util.network_count
            );
            println!(
                "  {} {:.0}%",
                style("Utilization:").bold(),
                util.utilization_percent
            );

            if util.network_count > 1 {
                println!();
                println!("{}", style("Overlapping Networks:").bold());

                // Scan for networks on the same channel
                if let Ok(aps) = providers.wifi.scan_access_points(&interface.name).await {
                    let same_channel: Vec<_> = aps
                        .iter()
                        .filter(|ap| {
                            ap.channel.number == current_channel.number
                                && ap.bssid != connection.access_point.bssid
                        })
                        .collect();

                    if same_channel.is_empty() {
                        println!(
                            "  {} No other networks detected on this channel",
                            style("[OK]").green()
                        );
                    } else {
                        println!(
                            "  {} {} other network(s) on channel {}",
                            style("[!]").yellow(),
                            same_channel.len(),
                            current_channel.number
                        );
                        for ap in same_channel.iter().take(5) {
                            println!(
                                "  {} {} ({} dBm)",
                                style("  -").dim(),
                                ap.ssid,
                                ap.rssi
                            );
                        }
                        if same_channel.len() > 5 {
                            println!(
                                "  {} ...and {} more",
                                style("  -").dim(),
                                same_channel.len() - 5
                            );
                        }
                    }
                }
            } else {
                println!();
                println!(
                    "  {} No other networks on this channel",
                    style("[OK]").green()
                );
            }

            // Recommendation
            if util.interference_level == InterferenceLevel::High
                || util.interference_level == InterferenceLevel::Severe
            {
                println!();
                println!(
                    "{} Run 'netdiag wifi channels' to find a better channel",
                    style("Recommendation:").cyan().bold()
                );
            }
        }
        Err(e) => {
            println!();
            println!(
                "{}",
                style(format!("Could not get channel utilization: {}", e)).dim()
            );
        }
    }

    Ok(())
}
