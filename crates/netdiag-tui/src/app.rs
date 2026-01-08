//! Application state and logic.

use crate::error::TuiResult;
use crate::event::{AppEvent, EventHandler};
use crate::ui;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::prelude::*;
use std::io;
use std::time::Duration;

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
pub struct PingResult {
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
    pub ping_results: Vec<PingResult>,
    /// Is ping running.
    pub ping_running: bool,
    /// DNS target input.
    pub dns_target: String,
    /// DNS results.
    pub dns_results: Vec<String>,
    /// Is input mode active.
    pub input_mode: bool,
    /// Status message.
    pub status_message: Option<String>,
    /// Last update time.
    pub last_update: DateTime<Utc>,
    /// Selected interface index.
    pub selected_interface: usize,
}

impl App {
    /// Create a new application.
    pub fn new() -> Self {
        Self {
            should_quit: false,
            current_tab: Tab::Dashboard,
            interfaces: Vec::new(),
            ping_target: String::new(),
            ping_results: Vec::new(),
            ping_running: false,
            dns_target: String::new(),
            dns_results: Vec::new(),
            input_mode: false,
            status_message: None,
            last_update: Utc::now(),
            selected_interface: 0,
        }
    }

    /// Run the application.
    pub async fn run(mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> TuiResult<()> {
        // Create event handler
        let mut events = EventHandler::new(Duration::from_millis(250));

        // Initial data load
        self.refresh_interfaces();

        // Main loop
        loop {
            // Draw UI
            terminal.draw(|frame| ui::draw(&self, frame))?;

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

    /// Handle tick event.
    fn on_tick(&mut self) {
        // Update last update time
        self.last_update = Utc::now();

        // Clear old status messages
        if self.status_message.is_some() {
            self.status_message = None;
        }
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
                self.status_message = Some("Refreshed".to_string());
            }

            // Enter input mode
            KeyCode::Enter | KeyCode::Char('i') => {
                if matches!(self.current_tab, Tab::Ping | Tab::Dns) {
                    self.input_mode = true;
                }
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
            KeyCode::Backspace => {
                match self.current_tab {
                    Tab::Ping => {
                        self.ping_target.pop();
                    }
                    Tab::Dns => {
                        self.dns_target.pop();
                    }
                    _ => {}
                }
            }
            KeyCode::Char(c) => {
                match self.current_tab {
                    Tab::Ping => {
                        self.ping_target.push(c);
                    }
                    Tab::Dns => {
                        self.dns_target.push(c);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// Execute a command based on current tab.
    fn execute_command(&mut self) {
        match self.current_tab {
            Tab::Ping => {
                if !self.ping_target.is_empty() {
                    self.status_message = Some(format!("Starting ping to {}...", self.ping_target));
                    // Note: Actual ping would be handled asynchronously
                    // This is a placeholder for the UI framework
                }
            }
            Tab::Dns => {
                if !self.dns_target.is_empty() {
                    self.status_message = Some(format!("Looking up {}...", self.dns_target));
                    // Note: Actual DNS lookup would be handled asynchronously
                }
            }
            _ => {}
        }
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
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
