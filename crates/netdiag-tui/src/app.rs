//! Application state and logic.

use crate::error::TuiResult;
use crate::event::{AppEvent, EventHandler};
use crate::ui;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use netdiag_connectivity::{DnsResolver, DnsResult, PingConfig, Pinger, Tracer, TracerouteConfig};
use netdiag_types::diagnostics::{PingStats, TracerouteHop, TracerouteResult};
use ratatui::prelude::*;
use std::io;
use std::net::IpAddr;
use std::time::Duration;
use tokio::sync::mpsc;

/// Application tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    /// Dashboard overview.
    Dashboard,
    /// Network interfaces.
    Interfaces,
    /// Ping tool.
    Ping,
    /// Traceroute tool.
    Traceroute,
    /// DNS lookup.
    Dns,
    /// WiFi analysis.
    Wifi,
}

impl Tab {
    /// Get all tabs.
    pub fn all() -> &'static [Tab] {
        &[
            Tab::Dashboard,
            Tab::Interfaces,
            Tab::Ping,
            Tab::Traceroute,
            Tab::Dns,
            Tab::Wifi,
        ]
    }

    /// Get tab name.
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Interfaces => "Interfaces",
            Tab::Ping => "Ping",
            Tab::Traceroute => "Traceroute",
            Tab::Dns => "DNS",
            Tab::Wifi => "WiFi",
        }
    }

    /// Get tab index.
    pub fn index(&self) -> usize {
        match self {
            Tab::Dashboard => 0,
            Tab::Interfaces => 1,
            Tab::Ping => 2,
            Tab::Traceroute => 3,
            Tab::Dns => 4,
            Tab::Wifi => 5,
        }
    }

    /// Get tab from index.
    pub fn from_index(idx: usize) -> Self {
        match idx % 6 {
            0 => Tab::Dashboard,
            1 => Tab::Interfaces,
            2 => Tab::Ping,
            3 => Tab::Traceroute,
            4 => Tab::Dns,
            5 => Tab::Wifi,
            _ => Tab::Dashboard,
        }
    }
}

/// Network interface info for display.
#[derive(Debug, Clone)]
pub struct InterfaceInfo {
    /// Interface name.
    pub name: String,
    /// Interface type.
    pub if_type: String,
    /// Is interface up.
    pub is_up: bool,
    /// Is default interface.
    pub is_default: bool,
    /// IPv4 addresses.
    pub ipv4: Vec<String>,
    /// IPv6 addresses.
    pub ipv6: Vec<String>,
    /// MAC address.
    pub mac: Option<String>,
}

/// Ping result for display.
#[derive(Debug, Clone)]
pub struct PingResultDisplay {
    /// Sequence number.
    pub seq: u16,
    /// Target.
    pub target: String,
    /// RTT in milliseconds.
    pub rtt_ms: Option<f64>,
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Success.
    pub success: bool,
    /// TTL.
    pub ttl: Option<u8>,
}

/// Traceroute hop for display.
#[derive(Debug, Clone)]
pub struct TracerouteHopDisplay {
    /// Hop number.
    pub hop: u8,
    /// IP address.
    pub address: Option<String>,
    /// Hostname.
    pub hostname: Option<String>,
    /// RTT in ms.
    pub rtt_ms: Option<f64>,
    /// Is timeout.
    pub is_timeout: bool,
}

/// WiFi info for display.
#[derive(Debug, Clone, Default)]
pub struct WifiInfo {
    /// Interface name.
    pub interface: String,
    /// Is powered on.
    pub powered_on: bool,
    /// Current SSID.
    pub ssid: Option<String>,
    /// BSSID.
    pub bssid: Option<String>,
    /// Signal strength (dBm).
    pub rssi: Option<i32>,
    /// Noise level (dBm).
    pub noise: Option<i32>,
    /// Channel.
    pub channel: Option<u8>,
    /// Band.
    pub band: Option<String>,
    /// Security type.
    pub security: Option<String>,
    /// TX rate.
    pub tx_rate: Option<f32>,
    /// WiFi standard.
    pub standard: Option<String>,
}

/// Background task messages.
#[derive(Debug)]
pub enum TaskMessage {
    /// Ping result received.
    PingResult(PingStats),
    /// Ping error.
    PingError(String),
    /// DNS result received.
    DnsResult(DnsResult),
    /// DNS error.
    DnsError(String),
    /// Traceroute hop received.
    TracerouteHop(TracerouteHop),
    /// Traceroute complete.
    TracerouteComplete(TracerouteResult),
    /// Traceroute error.
    TracerouteError(String),
    /// WiFi info updated.
    WifiUpdate(WifiInfo),
    /// WiFi error.
    WifiError(String),
}

/// Application state.
pub struct App {
    /// Should the application quit.
    pub should_quit: bool,
    /// Current active tab.
    pub current_tab: Tab,
    /// Network interfaces.
    pub interfaces: Vec<InterfaceInfo>,
    /// Ping target input.
    pub ping_target: String,
    /// Ping results.
    pub ping_results: Vec<PingResultDisplay>,
    /// Is ping running.
    pub ping_running: bool,
    /// Ping statistics.
    pub ping_stats: Option<PingStats>,
    /// Traceroute target input.
    pub traceroute_target: String,
    /// Traceroute hops.
    pub traceroute_hops: Vec<TracerouteHopDisplay>,
    /// Is traceroute running.
    pub traceroute_running: bool,
    /// DNS target input.
    pub dns_target: String,
    /// DNS results.
    pub dns_results: Vec<DnsResult>,
    /// Is DNS lookup running.
    pub dns_running: bool,
    /// WiFi info.
    pub wifi_info: Option<WifiInfo>,
    /// Is WiFi scan running.
    pub wifi_running: bool,
    /// Is input mode active.
    pub input_mode: bool,
    /// Status message.
    pub status_message: Option<String>,
    /// Last update time.
    pub last_update: DateTime<Utc>,
    /// Selected interface index.
    pub selected_interface: usize,
    /// Task message receiver.
    task_rx: mpsc::UnboundedReceiver<TaskMessage>,
    /// Task message sender.
    task_tx: mpsc::UnboundedSender<TaskMessage>,
    /// RTT history for sparkline (last 50 values).
    pub ping_rtt_history: Vec<f64>,
    /// Gateway connectivity status.
    pub gateway_status: Option<bool>,
    /// Internet connectivity status.
    pub internet_status: Option<bool>,
    /// DNS status.
    pub dns_status: Option<bool>,
}

impl App {
    /// Create a new application.
    pub fn new() -> Self {
        let (task_tx, task_rx) = mpsc::unbounded_channel();

        Self {
            should_quit: false,
            current_tab: Tab::Dashboard,
            interfaces: Vec::new(),
            ping_target: String::new(),
            ping_results: Vec::new(),
            ping_running: false,
            ping_stats: None,
            traceroute_target: String::new(),
            traceroute_hops: Vec::new(),
            traceroute_running: false,
            dns_target: String::new(),
            dns_results: Vec::new(),
            dns_running: false,
            wifi_info: None,
            wifi_running: false,
            input_mode: false,
            status_message: None,
            last_update: Utc::now(),
            selected_interface: 0,
            task_rx,
            task_tx,
            ping_rtt_history: Vec::with_capacity(50),
            gateway_status: None,
            internet_status: None,
            dns_status: None,
        }
    }

    /// Run the application.
    pub async fn run(
        mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> TuiResult<()> {
        // Create event handler
        let mut events = EventHandler::new(Duration::from_millis(100));

        // Initial data load
        self.refresh_interfaces();
        self.refresh_wifi();
        self.check_connectivity_status();

        // Main loop
        loop {
            // Draw UI
            terminal.draw(|frame| ui::draw(&self, frame))?;

            // Check for task messages (non-blocking)
            while let Ok(msg) = self.task_rx.try_recv() {
                self.handle_task_message(msg);
            }

            // Handle events
            match events.next().await? {
                AppEvent::Tick => {
                    // Periodic updates
                    self.on_tick();
                }
                AppEvent::Key(key) => {
                    self.on_key(key);
                }
                AppEvent::Resize(_, _) => {
                    // Terminal will automatically handle resize
                }
                AppEvent::Quit => {
                    self.should_quit = true;
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle task message.
    fn handle_task_message(&mut self, msg: TaskMessage) {
        match msg {
            TaskMessage::PingResult(stats) => {
                self.ping_running = false;
                self.ping_stats = Some(stats.clone());

                // Add results to display list
                for ping_result in stats.results.iter() {
                    let result = PingResultDisplay {
                        seq: ping_result.seq,
                        target: stats.target.to_string(),
                        rtt_ms: ping_result.rtt.map(|d| d.as_secs_f64() * 1000.0),
                        timestamp: Utc::now(),
                        success: ping_result.success,
                        ttl: ping_result.ttl,
                    };

                    // Add to RTT history for sparkline
                    if let Some(ms) = result.rtt_ms {
                        self.ping_rtt_history.push(ms);
                        if self.ping_rtt_history.len() > 50 {
                            self.ping_rtt_history.remove(0);
                        }
                    }

                    self.ping_results.push(result);
                }

                // Keep only last 100 results
                if self.ping_results.len() > 100 {
                    self.ping_results.drain(0..self.ping_results.len() - 100);
                }

                self.status_message = Some(format!(
                    "Ping complete: {}/{} received",
                    stats.received, stats.transmitted
                ));
            }
            TaskMessage::PingError(err) => {
                self.ping_running = false;
                self.status_message = Some(format!("Ping error: {}", err));
            }
            TaskMessage::DnsResult(result) => {
                self.dns_running = false;
                self.dns_results.insert(0, result);
                if self.dns_results.len() > 20 {
                    self.dns_results.pop();
                }
                self.status_message = Some("DNS lookup complete".to_string());
            }
            TaskMessage::DnsError(err) => {
                self.dns_running = false;
                self.status_message = Some(format!("DNS error: {}", err));
            }
            TaskMessage::TracerouteHop(hop) => {
                let display = TracerouteHopDisplay {
                    hop: hop.hop,
                    address: hop.address.map(|a| a.to_string()),
                    hostname: hop.hostname.clone(),
                    rtt_ms: hop.avg_rtt.map(|d| d.as_secs_f64() * 1000.0),
                    is_timeout: hop.all_timeout,
                };
                self.traceroute_hops.push(display);
            }
            TaskMessage::TracerouteComplete(result) => {
                self.traceroute_running = false;
                self.status_message = Some(format!(
                    "Traceroute complete: {} hops, {}",
                    result.hops.len(),
                    if result.reached {
                        "target reached"
                    } else {
                        "target not reached"
                    }
                ));
            }
            TaskMessage::TracerouteError(err) => {
                self.traceroute_running = false;
                self.status_message = Some(format!("Traceroute error: {}", err));
            }
            TaskMessage::WifiUpdate(info) => {
                self.wifi_running = false;
                self.wifi_info = Some(info);
            }
            TaskMessage::WifiError(err) => {
                self.wifi_running = false;
                self.status_message = Some(format!("WiFi error: {}", err));
            }
        }
    }

    /// Handle tick event.
    fn on_tick(&mut self) {
        // Update last update time
        self.last_update = Utc::now();

        // Clear old status messages after a few seconds
        // (status_message will be cleared on next user action anyway)
    }

    /// Handle key event.
    fn on_key(&mut self, key: KeyEvent) {
        if self.input_mode {
            self.handle_input_key(key);
            return;
        }

        match key.code {
            // Quit
            KeyCode::Char('q') => self.should_quit = true,

            // Tab navigation
            KeyCode::Tab | KeyCode::Right => {
                let idx = (self.current_tab.index() + 1) % Tab::all().len();
                self.current_tab = Tab::from_index(idx);
            }
            KeyCode::BackTab | KeyCode::Left => {
                let idx = if self.current_tab.index() == 0 {
                    Tab::all().len() - 1
                } else {
                    self.current_tab.index() - 1
                };
                self.current_tab = Tab::from_index(idx);
            }

            // Number keys for tab selection
            KeyCode::Char('1') => self.current_tab = Tab::Dashboard,
            KeyCode::Char('2') => self.current_tab = Tab::Interfaces,
            KeyCode::Char('3') => self.current_tab = Tab::Ping,
            KeyCode::Char('4') => self.current_tab = Tab::Traceroute,
            KeyCode::Char('5') => self.current_tab = Tab::Dns,
            KeyCode::Char('6') => self.current_tab = Tab::Wifi,

            // Refresh
            KeyCode::Char('r') => {
                self.refresh_interfaces();
                self.refresh_wifi();
                self.check_connectivity_status();
                self.status_message = Some("Refreshed".to_string());
            }

            // Enter input mode or execute
            KeyCode::Enter | KeyCode::Char('i') => match self.current_tab {
                Tab::Ping | Tab::Dns | Tab::Traceroute => {
                    self.input_mode = true;
                }
                Tab::Wifi => {
                    self.refresh_wifi();
                }
                _ => {}
            },

            // Clear results
            KeyCode::Char('c') => {
                match self.current_tab {
                    Tab::Ping => {
                        self.ping_results.clear();
                        self.ping_stats = None;
                        self.ping_rtt_history.clear();
                    }
                    Tab::Dns => {
                        self.dns_results.clear();
                    }
                    Tab::Traceroute => {
                        self.traceroute_hops.clear();
                    }
                    _ => {}
                }
                self.status_message = Some("Cleared".to_string());
            }

            // Selection
            KeyCode::Up | KeyCode::Char('k') => {
                if self.selected_interface > 0 {
                    self.selected_interface -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_interface < self.interfaces.len().saturating_sub(1) {
                    self.selected_interface += 1;
                }
            }

            // Quick ping (space to start ping to current target)
            KeyCode::Char(' ') => {
                if self.current_tab == Tab::Ping && !self.ping_target.is_empty() {
                    self.start_ping();
                }
            }

            _ => {}
        }
    }

    /// Handle input mode key.
    fn handle_input_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = false;
            }
            KeyCode::Enter => {
                self.input_mode = false;
                self.execute_command();
            }
            KeyCode::Backspace => match self.current_tab {
                Tab::Ping => {
                    self.ping_target.pop();
                }
                Tab::Dns => {
                    self.dns_target.pop();
                }
                Tab::Traceroute => {
                    self.traceroute_target.pop();
                }
                _ => {}
            },
            KeyCode::Char(c) => match self.current_tab {
                Tab::Ping => {
                    self.ping_target.push(c);
                }
                Tab::Dns => {
                    self.dns_target.push(c);
                }
                Tab::Traceroute => {
                    self.traceroute_target.push(c);
                }
                _ => {}
            },
            _ => {}
        }
    }

    /// Execute a command based on current tab.
    fn execute_command(&mut self) {
        match self.current_tab {
            Tab::Ping => {
                if !self.ping_target.is_empty() && !self.ping_running {
                    self.start_ping();
                }
            }
            Tab::Dns => {
                if !self.dns_target.is_empty() && !self.dns_running {
                    self.start_dns_lookup();
                }
            }
            Tab::Traceroute => {
                if !self.traceroute_target.is_empty() && !self.traceroute_running {
                    self.start_traceroute();
                }
            }
            _ => {}
        }
    }

    /// Start a ping operation.
    fn start_ping(&mut self) {
        let target = self.ping_target.clone();
        let tx = self.task_tx.clone();

        self.ping_running = true;
        self.status_message = Some(format!("Pinging {}...", target));

        tokio::spawn(async move {
            // First resolve DNS if needed
            let ip = match target.parse::<IpAddr>() {
                Ok(ip) => ip,
                Err(_) => {
                    // Try DNS resolution
                    match DnsResolver::new() {
                        Ok(resolver) => match resolver.resolve(&target).await {
                            Ok(result) if !result.addresses.is_empty() => result.addresses[0],
                            _ => {
                                let _ = tx.send(TaskMessage::PingError(format!(
                                    "Failed to resolve: {}",
                                    target
                                )));
                                return;
                            }
                        },
                        Err(e) => {
                            let _ = tx.send(TaskMessage::PingError(e.to_string()));
                            return;
                        }
                    }
                }
            };

            let pinger = Pinger::new();
            let config = PingConfig {
                count: 4,
                timeout: Duration::from_secs(2),
                interval: Duration::from_millis(500),
                size: 64,
            };

            match pinger.ping(ip, &config).await {
                Ok(stats) => {
                    let _ = tx.send(TaskMessage::PingResult(stats));
                }
                Err(e) => {
                    let _ = tx.send(TaskMessage::PingError(e.to_string()));
                }
            }
        });
    }

    /// Start a DNS lookup.
    fn start_dns_lookup(&mut self) {
        let target = self.dns_target.clone();
        let tx = self.task_tx.clone();

        self.dns_running = true;
        self.status_message = Some(format!("Looking up {}...", target));

        tokio::spawn(async move {
            match DnsResolver::new() {
                Ok(resolver) => match resolver.resolve(&target).await {
                    Ok(result) => {
                        let _ = tx.send(TaskMessage::DnsResult(result));
                    }
                    Err(e) => {
                        let _ = tx.send(TaskMessage::DnsError(e.to_string()));
                    }
                },
                Err(e) => {
                    let _ = tx.send(TaskMessage::DnsError(e.to_string()));
                }
            }
        });
    }

    /// Start a traceroute.
    fn start_traceroute(&mut self) {
        let target = self.traceroute_target.clone();
        let tx = self.task_tx.clone();

        self.traceroute_running = true;
        self.traceroute_hops.clear();
        self.status_message = Some(format!("Tracing route to {}...", target));

        tokio::spawn(async move {
            // First resolve DNS if needed
            let ip = match target.parse::<IpAddr>() {
                Ok(ip) => ip,
                Err(_) => match DnsResolver::new() {
                    Ok(resolver) => match resolver.resolve(&target).await {
                        Ok(result) if !result.addresses.is_empty() => result.addresses[0],
                        _ => {
                            let _ = tx.send(TaskMessage::TracerouteError(format!(
                                "Failed to resolve: {}",
                                target
                            )));
                            return;
                        }
                    },
                    Err(e) => {
                        let _ = tx.send(TaskMessage::TracerouteError(e.to_string()));
                        return;
                    }
                },
            };

            let tracer = Tracer::new();
            let config = TracerouteConfig {
                max_hops: 30,
                probes_per_hop: 3,
                timeout: Duration::from_secs(2),
                resolve_hostnames: true,
                ..Default::default()
            };

            match tracer.trace(ip, &config).await {
                Ok(result) => {
                    // Send all hops
                    for hop in &result.hops {
                        let _ = tx.send(TaskMessage::TracerouteHop(hop.clone()));
                    }
                    let _ = tx.send(TaskMessage::TracerouteComplete(result));
                }
                Err(e) => {
                    let _ = tx.send(TaskMessage::TracerouteError(e.to_string()));
                }
            }
        });
    }

    /// Refresh network interfaces.
    fn refresh_interfaces(&mut self) {
        let interfaces = netdev::get_interfaces();
        let default_iface = netdev::get_default_interface().ok();
        let default_name = default_iface.map(|i| i.name.clone());

        self.interfaces = interfaces
            .into_iter()
            .map(|iface| InterfaceInfo {
                name: iface.name.clone(),
                if_type: format!("{:?}", iface.if_type),
                is_up: iface.is_up(),
                is_default: default_name.as_ref() == Some(&iface.name),
                ipv4: iface.ipv4.iter().map(|n| n.addr().to_string()).collect(),
                ipv6: iface.ipv6.iter().map(|n| n.addr().to_string()).collect(),
                mac: iface.mac_addr.map(|m| m.to_string()),
            })
            .collect();

        // Reset selection if out of bounds
        if self.selected_interface >= self.interfaces.len() {
            self.selected_interface = 0;
        }
    }

    /// Refresh WiFi info.
    fn refresh_wifi(&mut self) {
        let tx = self.task_tx.clone();
        self.wifi_running = true;

        tokio::spawn(async move {
            // Use platform-specific WiFi provider
            #[cfg(target_os = "macos")]
            {
                use netdiag_platform::WifiProvider;
                use netdiag_platform_macos::MacosWifiProvider;

                let provider = MacosWifiProvider::new();

                match provider.list_wifi_interfaces().await {
                    Ok(interfaces) if !interfaces.is_empty() => {
                        let iface = &interfaces[0];
                        let mut wifi_info = WifiInfo {
                            interface: iface.name.clone(),
                            powered_on: iface.powered_on,
                            ..Default::default()
                        };

                        // Get connection info
                        if let Ok(Some(conn)) = provider.get_current_connection(&iface.name).await {
                            wifi_info.ssid = Some(conn.access_point.ssid.as_str().to_string());
                            wifi_info.bssid = Some(conn.access_point.bssid.to_string());
                            wifi_info.rssi = Some(conn.access_point.rssi);
                            wifi_info.noise = conn.access_point.noise;
                            wifi_info.channel = Some(conn.access_point.channel.number);
                            wifi_info.band = Some(format!("{:?}", conn.access_point.channel.band));
                            wifi_info.security = Some(format!("{:?}", conn.access_point.security));
                            wifi_info.tx_rate = conn.tx_rate;
                            wifi_info.standard =
                                Some(format!("{:?}", conn.access_point.wifi_standard));
                        }

                        let _ = tx.send(TaskMessage::WifiUpdate(wifi_info));
                    }
                    Ok(_) => {
                        let _ = tx.send(TaskMessage::WifiError(
                            "No WiFi interfaces found".to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = tx.send(TaskMessage::WifiError(e.to_string()));
                    }
                }
            }

            #[cfg(target_os = "linux")]
            {
                use netdiag_platform::WifiProvider;
                use netdiag_platform_linux::LinuxWifiProvider;

                let provider = LinuxWifiProvider::new();

                match provider.list_wifi_interfaces().await {
                    Ok(interfaces) if !interfaces.is_empty() => {
                        let iface = &interfaces[0];
                        let wifi_info = WifiInfo {
                            interface: iface.name.clone(),
                            powered_on: iface.powered_on,
                            ..Default::default()
                        };

                        let _ = tx.send(TaskMessage::WifiUpdate(wifi_info));
                    }
                    Ok(_) => {
                        let _ = tx.send(TaskMessage::WifiError(
                            "No WiFi interfaces found".to_string(),
                        ));
                    }
                    Err(e) => {
                        let _ = tx.send(TaskMessage::WifiError(e.to_string()));
                    }
                }
            }

            #[cfg(not(any(target_os = "macos", target_os = "linux")))]
            {
                let _ = tx.send(TaskMessage::WifiError(
                    "WiFi not supported on this platform".to_string(),
                ));
            }
        });
    }

    /// Check connectivity status for dashboard.
    fn check_connectivity_status(&mut self) {
        let _tx = self.task_tx.clone();

        // Check gateway connectivity
        if let Ok(gateway) = netdev::get_default_interface() {
            if let Some(gw) = gateway.gateway {
                // Get gateway IP from ipv4 or ipv6 addresses
                let gw_ip = gw
                    .ipv4
                    .first()
                    .map(|addr| IpAddr::V4(*addr))
                    .or_else(|| gw.ipv6.first().map(|addr| IpAddr::V6(*addr)));

                if let Some(gw_ip) = gw_ip {
                    tokio::spawn(async move {
                        let pinger = Pinger::new();
                        let config = PingConfig {
                            count: 1,
                            timeout: Duration::from_secs(2),
                            interval: Duration::from_millis(500),
                            size: 64,
                        };
                        // Just run the ping - results handled elsewhere
                        let _ = pinger.ping(gw_ip, &config).await;
                    });
                }
            }
        }
    }

    /// Get signal quality string from RSSI.
    pub fn signal_quality(rssi: i32) -> &'static str {
        match rssi {
            r if r >= -50 => "Excellent",
            r if r >= -60 => "Good",
            r if r >= -70 => "Fair",
            r if r >= -80 => "Weak",
            _ => "Very Weak",
        }
    }

    /// Get signal color from RSSI.
    pub fn signal_color(rssi: i32) -> Color {
        match rssi {
            r if r >= -50 => Color::Green,
            r if r >= -60 => Color::LightGreen,
            r if r >= -70 => Color::Yellow,
            r if r >= -80 => Color::LightRed,
            _ => Color::Red,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
