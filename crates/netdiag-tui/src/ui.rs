//! UI rendering.

use crate::app::{App, Tab};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Clear, Gauge, List, ListItem, Paragraph, Row, Sparkline, Table, Tabs,
    },
    Frame,
};

/// Draw the UI.
pub fn draw(app: &App, frame: &mut Frame) {
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/Tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer/Status
        ])
        .split(frame.area());

    // Draw tabs
    draw_tabs(app, frame, chunks[0]);

    // Draw main content based on current tab
    match app.current_tab {
        Tab::Dashboard => draw_dashboard(app, frame, chunks[1]),
        Tab::Interfaces => draw_interfaces(app, frame, chunks[1]),
        Tab::Ping => draw_ping(app, frame, chunks[1]),
        Tab::Traceroute => draw_traceroute(app, frame, chunks[1]),
        Tab::Dns => draw_dns(app, frame, chunks[1]),
        Tab::Wifi => draw_wifi(app, frame, chunks[1]),
    }

    // Draw footer/status
    draw_footer(app, frame, chunks[2]);

    // Draw input popup if in input mode
    if app.input_mode {
        draw_input_popup(app, frame);
    }
}

/// Draw the tab bar.
fn draw_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let titles: Vec<Line> = Tab::all()
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let style = if *t == app.current_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(format!(" {}:{} ", i + 1, t.name()), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" netdiag - Network Diagnostics ")
                .title_style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .select(app.current_tab.index())
        .highlight_style(Style::default().bg(Color::DarkGray));

    frame.render_widget(tabs, area);
}

/// Draw the dashboard.
fn draw_dashboard(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel - System info and status
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(chunks[0]);

    // System summary
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let active_interfaces = app.interfaces.iter().filter(|i| i.is_up).count();
    let default_iface = app
        .interfaces
        .iter()
        .find(|i| i.is_default)
        .map(|i| i.name.as_str())
        .unwrap_or("None");

    let system_info = vec![
        Line::from(vec![
            Span::styled("Hostname: ", Style::default().fg(Color::Gray)),
            Span::styled(hostname, Style::default().fg(Color::White).bold()),
        ]),
        Line::from(vec![
            Span::styled("Interfaces: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!(
                    "{} total, {} active",
                    app.interfaces.len(),
                    active_interfaces
                ),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("Default: ", Style::default().fg(Color::Gray)),
            Span::styled(default_iface, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Last Update: ", Style::default().fg(Color::Gray)),
            Span::styled(
                app.last_update.format("%H:%M:%S").to_string(),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    let system_block = Paragraph::new(system_info).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" System Info ")
            .title_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(system_block, left_chunks[0]);

    // WiFi summary
    let wifi_lines = if let Some(ref wifi) = app.wifi_info {
        let signal_color = wifi.rssi.map(App::signal_color).unwrap_or(Color::Gray);
        let signal_quality = wifi.rssi.map(App::signal_quality).unwrap_or("Unknown");

        vec![
            Line::from(vec![
                Span::styled("Interface: ", Style::default().fg(Color::Gray)),
                Span::styled(&wifi.interface, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("SSID: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    wifi.ssid.as_deref().unwrap_or("Not connected"),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Line::from(vec![
                Span::styled("Signal: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "{} dBm ({})",
                        wifi.rssi.map(|r| r.to_string()).unwrap_or("--".to_string()),
                        signal_quality
                    ),
                    Style::default().fg(signal_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("Channel: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "{} ({})",
                        wifi.channel
                            .map(|c| c.to_string())
                            .unwrap_or("--".to_string()),
                        wifi.band.as_deref().unwrap_or("--")
                    ),
                    Style::default().fg(Color::White),
                ),
            ]),
        ]
    } else if app.wifi_running {
        vec![Line::from(Span::styled(
            "Scanning WiFi...",
            Style::default().fg(Color::Yellow),
        ))]
    } else {
        vec![Line::from(Span::styled(
            "No WiFi info available",
            Style::default().fg(Color::Gray),
        ))]
    };

    let wifi_block = Paragraph::new(wifi_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" WiFi Status ")
            .title_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(wifi_block, left_chunks[1]);

    // Quick actions
    let actions = vec![
        Line::from(Span::styled("Quick Actions:", Style::default().bold())),
        Line::from(""),
        Line::from(Span::styled(
            "  r - Refresh all",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled(
            "  1-6 - Switch tabs",
            Style::default().fg(Color::Gray),
        )),
        Line::from(Span::styled("  q - Quit", Style::default().fg(Color::Gray))),
    ];

    let actions_block = Paragraph::new(actions).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Help ")
            .title_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(actions_block, left_chunks[2]);

    // Right panel - Interface summary
    let interface_rows: Vec<Row> = app
        .interfaces
        .iter()
        .filter(|i| i.is_up)
        .take(15)
        .map(|iface| {
            let default_marker = if iface.is_default { "*" } else { "" };

            Row::new(vec![
                Cell::from(format!("{}{}", iface.name, default_marker)).style(Style::default().fg(
                    if iface.is_default {
                        Color::Cyan
                    } else {
                        Color::White
                    },
                )),
                Cell::from("UP").style(Style::default().fg(Color::Green)),
                Cell::from(iface.ipv4.first().cloned().unwrap_or_default()),
            ])
        })
        .collect();

    let interface_table = Table::new(
        interface_rows,
        [
            Constraint::Length(15),
            Constraint::Length(6),
            Constraint::Min(15),
        ],
    )
    .header(
        Row::new(vec!["Interface", "Status", "IPv4"]).style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        ),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Active Interfaces ")
            .title_style(Style::default().fg(Color::Cyan)),
    );

    frame.render_widget(interface_table, chunks[1]);
}

/// Draw the interfaces view.
fn draw_interfaces(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Interface list
    let items: Vec<ListItem> = app
        .interfaces
        .iter()
        .enumerate()
        .map(|(i, iface)| {
            let status = if iface.is_up { "●" } else { "○" };
            let status_style = if iface.is_up {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let default_marker = if iface.is_default { " [default]" } else { "" };

            let style = if i == app.selected_interface {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", status), status_style),
                Span::styled(format!("{}{}", iface.name, default_marker), style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Interfaces (↑↓ to select) ")
            .title_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(list, chunks[0]);

    // Interface details
    if let Some(iface) = app.interfaces.get(app.selected_interface) {
        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Gray)),
                Span::styled(&iface.name, Style::default().fg(Color::Cyan).bold()),
            ]),
            Line::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Gray)),
                Span::styled(&iface.if_type, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if iface.is_up { "UP" } else { "DOWN" },
                    if iface.is_up {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]),
            Line::from(vec![
                Span::styled("Default: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if iface.is_default { "Yes" } else { "No" },
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "IPv4 Addresses:",
                Style::default().fg(Color::Gray).bold(),
            )),
        ];

        let mut lines: Vec<Line> = details;

        for ip in &iface.ipv4 {
            lines.push(Line::from(Span::styled(
                format!("  {}", ip),
                Style::default().fg(Color::Green),
            )));
        }

        if iface.ipv4.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (none)",
                Style::default().fg(Color::Gray),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "IPv6 Addresses:",
            Style::default().fg(Color::Gray).bold(),
        )));

        for ip in &iface.ipv6 {
            lines.push(Line::from(Span::styled(
                format!("  {}", ip),
                Style::default().fg(Color::Blue),
            )));
        }

        if iface.ipv6.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (none)",
                Style::default().fg(Color::Gray),
            )));
        }

        if let Some(ref mac) = iface.mac {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("MAC: ", Style::default().fg(Color::Gray)),
                Span::styled(mac, Style::default().fg(Color::Yellow)),
            ]));
        }

        let details_para = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(details_para, chunks[1]);
    }
}

/// Draw the ping view.
fn draw_ping(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Input
            Constraint::Length(5), // Stats
            Constraint::Length(5), // Sparkline
            Constraint::Min(0),    // Results
        ])
        .split(area);

    // Target input
    let input_text = if app.ping_target.is_empty() {
        "Press Enter or 'i' to enter target..."
    } else {
        &app.ping_target
    };

    let input_style = if app.input_mode && app.current_tab == Tab::Ping {
        Style::default().fg(Color::Yellow)
    } else if app.ping_running {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let status_indicator = if app.ping_running {
        " [Running...]"
    } else {
        ""
    };

    let input = Paragraph::new(format!("{}{}", input_text, status_indicator))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Ping Target (Enter to edit, Space to ping) ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(input, chunks[0]);

    // Statistics
    let stats_lines = if let Some(ref stats) = app.ping_stats {
        let loss_pct = if stats.transmitted > 0 {
            ((stats.transmitted - stats.received) as f64 / stats.transmitted as f64) * 100.0
        } else {
            0.0
        };

        let loss_color = if loss_pct == 0.0 {
            Color::Green
        } else if loss_pct < 20.0 {
            Color::Yellow
        } else {
            Color::Red
        };

        vec![
            Line::from(vec![
                Span::styled("Packets: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{} sent, {} received", stats.transmitted, stats.received),
                    Style::default().fg(Color::White),
                ),
                Span::styled(
                    format!(" ({:.1}% loss)", loss_pct),
                    Style::default().fg(loss_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("RTT: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!(
                        "min={:.1}ms avg={:.1}ms max={:.1}ms",
                        stats
                            .min_rtt
                            .map(|d| d.as_secs_f64() * 1000.0)
                            .unwrap_or(0.0),
                        stats
                            .avg_rtt
                            .map(|d| d.as_secs_f64() * 1000.0)
                            .unwrap_or(0.0),
                        stats
                            .max_rtt
                            .map(|d| d.as_secs_f64() * 1000.0)
                            .unwrap_or(0.0),
                    ),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No ping results yet",
            Style::default().fg(Color::Gray),
        ))]
    };

    let stats_block = Paragraph::new(stats_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Statistics ")
            .title_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(stats_block, chunks[1]);

    // RTT Sparkline
    if !app.ping_rtt_history.is_empty() {
        let data: Vec<u64> = app.ping_rtt_history.iter().map(|&ms| ms as u64).collect();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" RTT History ")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .data(&data)
            .style(Style::default().fg(Color::Green));

        frame.render_widget(sparkline, chunks[2]);
    } else {
        let empty = Paragraph::new("No RTT data").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" RTT History ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(empty, chunks[2]);
    }

    // Results
    let results: Vec<ListItem> = app
        .ping_results
        .iter()
        .rev()
        .take(20)
        .map(|r| {
            let style = if r.success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let rtt = r
                .rtt_ms
                .map(|ms| format!("{:.1}ms", ms))
                .unwrap_or_else(|| "timeout".to_string());
            ListItem::new(format!(
                "[{}] seq={} {} {}",
                r.timestamp.format("%H:%M:%S"),
                r.seq,
                r.target,
                rtt
            ))
            .style(style)
        })
        .collect();

    let results_list = List::new(results).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Results (c to clear) ")
            .title_style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(results_list, chunks[3]);
}

/// Draw the traceroute view.
fn draw_traceroute(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Target input
    let input_text = if app.traceroute_target.is_empty() {
        "Press Enter or 'i' to enter target..."
    } else {
        &app.traceroute_target
    };

    let input_style = if app.input_mode && app.current_tab == Tab::Traceroute {
        Style::default().fg(Color::Yellow)
    } else if app.traceroute_running {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let status_indicator = if app.traceroute_running {
        format!(" [Tracing... {} hops]", app.traceroute_hops.len())
    } else {
        String::new()
    };

    let input = Paragraph::new(format!("{}{}", input_text, status_indicator))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Traceroute Target ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(input, chunks[0]);

    // Hops table
    if app.traceroute_hops.is_empty() && !app.traceroute_running {
        let help = Paragraph::new("Enter a target and press Enter to trace route\n\nResults will appear here as each hop responds")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Hops (c to clear) ")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(help, chunks[1]);
    } else {
        let hop_rows: Vec<Row> = app
            .traceroute_hops
            .iter()
            .map(|hop| {
                let style = if hop.is_timeout {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default().fg(Color::Green)
                };

                let address = hop.address.as_deref().unwrap_or("*");
                let hostname = hop.hostname.as_deref().unwrap_or("");
                let rtt = hop
                    .rtt_ms
                    .map(|ms| format!("{:.1}ms", ms))
                    .unwrap_or_else(|| "*".to_string());

                Row::new(vec![
                    Cell::from(format!("{:>2}", hop.hop)).style(Style::default().fg(Color::Cyan)),
                    Cell::from(address).style(style),
                    Cell::from(hostname).style(Style::default().fg(Color::Gray)),
                    Cell::from(rtt).style(style),
                ])
            })
            .collect();

        let table = Table::new(
            hop_rows,
            [
                Constraint::Length(4),
                Constraint::Length(16),
                Constraint::Min(20),
                Constraint::Length(10),
            ],
        )
        .header(
            Row::new(vec!["Hop", "IP Address", "Hostname", "RTT"]).style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            ),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Hops (c to clear) ")
                .title_style(Style::default().fg(Color::Cyan)),
        );

        frame.render_widget(table, chunks[1]);
    }
}

/// Draw the DNS view.
fn draw_dns(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Target input
    let input_text = if app.dns_target.is_empty() {
        "Press Enter or 'i' to enter hostname..."
    } else {
        &app.dns_target
    };

    let input_style = if app.input_mode && app.current_tab == Tab::Dns {
        Style::default().fg(Color::Yellow)
    } else if app.dns_running {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let status_indicator = if app.dns_running {
        " [Looking up...]"
    } else {
        ""
    };

    let input = Paragraph::new(format!("{}{}", input_text, status_indicator))
        .style(input_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" DNS Lookup ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
    frame.render_widget(input, chunks[0]);

    // Results
    if app.dns_results.is_empty() {
        let help = Paragraph::new(
            "Enter a hostname and press Enter to lookup\n\nResults will show resolved IP addresses",
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Results (c to clear) ")
                .title_style(Style::default().fg(Color::Cyan)),
        )
        .alignment(Alignment::Center);
        frame.render_widget(help, chunks[1]);
    } else {
        let mut lines: Vec<Line> = Vec::new();

        for result in &app.dns_results {
            let status_style = if result.success {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };

            lines.push(Line::from(vec![
                Span::styled(&result.query, Style::default().fg(Color::Cyan).bold()),
                Span::styled(
                    format!(" ({:.1}ms)", result.duration.as_secs_f64() * 1000.0),
                    Style::default().fg(Color::Gray),
                ),
            ]));

            if result.success {
                for addr in &result.addresses {
                    let addr_style = if addr.is_ipv4() {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Blue)
                    };
                    lines.push(Line::from(vec![
                        Span::styled("  → ", Style::default().fg(Color::Gray)),
                        Span::styled(addr.to_string(), addr_style),
                    ]));
                }
            } else if let Some(ref err) = result.error {
                lines.push(Line::from(Span::styled(
                    format!("  Error: {}", err),
                    status_style,
                )));
            }

            lines.push(Line::from(""));
        }

        let results_para = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Results (c to clear) ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(results_para, chunks[1]);
    }
}

/// Draw the WiFi view.
fn draw_wifi(app: &App, frame: &mut Frame, area: Rect) {
    if let Some(ref wifi) = app.wifi_info {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        // Left panel - Connection info
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(chunks[0]);

        // Connection details
        let mut conn_lines = vec![
            Line::from(vec![
                Span::styled("Interface: ", Style::default().fg(Color::Gray)),
                Span::styled(&wifi.interface, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Power: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    if wifi.powered_on { "ON" } else { "OFF" },
                    if wifi.powered_on {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::Red)
                    },
                ),
            ]),
        ];

        if let Some(ref ssid) = wifi.ssid {
            conn_lines.push(Line::from(vec![
                Span::styled("SSID: ", Style::default().fg(Color::Gray)),
                Span::styled(ssid, Style::default().fg(Color::Yellow).bold()),
            ]));
        }

        if let Some(ref bssid) = wifi.bssid {
            conn_lines.push(Line::from(vec![
                Span::styled("BSSID: ", Style::default().fg(Color::Gray)),
                Span::styled(bssid, Style::default().fg(Color::White)),
            ]));
        }

        if let Some(ref security) = wifi.security {
            conn_lines.push(Line::from(vec![
                Span::styled("Security: ", Style::default().fg(Color::Gray)),
                Span::styled(security, Style::default().fg(Color::Green)),
            ]));
        }

        if let Some(ref standard) = wifi.standard {
            conn_lines.push(Line::from(vec![
                Span::styled("WiFi Standard: ", Style::default().fg(Color::Gray)),
                Span::styled(standard, Style::default().fg(Color::White)),
            ]));
        }

        let conn_block = Paragraph::new(conn_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Connection ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(conn_block, left_chunks[0]);

        // Channel info
        let channel_lines = vec![
            Line::from(vec![
                Span::styled("Channel: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    wifi.channel
                        .map(|c| c.to_string())
                        .unwrap_or("--".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("Band: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    wifi.band.as_deref().unwrap_or("--"),
                    Style::default().fg(Color::White),
                ),
            ]),
            Line::from(vec![
                Span::styled("TX Rate: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    wifi.tx_rate
                        .map(|r| format!("{:.0} Mbps", r))
                        .unwrap_or("--".to_string()),
                    Style::default().fg(Color::White),
                ),
            ]),
        ];

        let channel_block = Paragraph::new(channel_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Channel ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(channel_block, left_chunks[1]);

        // Help
        let help_lines = vec![
            Line::from(Span::styled(
                "Press 'r' to refresh",
                Style::default().fg(Color::Gray),
            )),
            Line::from(Span::styled(
                "Press 'Enter' to scan",
                Style::default().fg(Color::Gray),
            )),
        ];

        let help_block = Paragraph::new(help_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Help ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(help_block, left_chunks[2]);

        // Right panel - Signal strength
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Length(8),
                Constraint::Min(0),
            ])
            .split(chunks[1]);

        // Signal gauge
        if let Some(rssi) = wifi.rssi {
            // Convert RSSI to percentage (roughly -100 to -30 dBm range)
            let signal_pct = ((rssi + 100).max(0).min(70) as f64 / 70.0 * 100.0) as u16;
            let signal_color = App::signal_color(rssi);
            let signal_quality = App::signal_quality(rssi);

            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(
                            " Signal Strength: {} dBm ({}) ",
                            rssi, signal_quality
                        ))
                        .title_style(Style::default().fg(Color::Cyan)),
                )
                .gauge_style(Style::default().fg(signal_color))
                .percent(signal_pct);

            frame.render_widget(gauge, right_chunks[0]);
        } else {
            let gauge = Gauge::default()
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Signal Strength ")
                        .title_style(Style::default().fg(Color::Cyan)),
                )
                .percent(0);

            frame.render_widget(gauge, right_chunks[0]);
        }

        // SNR info
        let snr_lines = if let (Some(rssi), Some(noise)) = (wifi.rssi, wifi.noise) {
            let snr = rssi - noise;
            let snr_quality = match snr {
                s if s >= 40 => ("Excellent", Color::Green),
                s if s >= 25 => ("Good", Color::LightGreen),
                s if s >= 15 => ("Fair", Color::Yellow),
                s if s >= 10 => ("Poor", Color::LightRed),
                _ => ("Very Poor", Color::Red),
            };

            vec![
                Line::from(vec![
                    Span::styled("RSSI: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{} dBm", rssi),
                        Style::default().fg(App::signal_color(rssi)),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Noise: ", Style::default().fg(Color::Gray)),
                    Span::styled(format!("{} dBm", noise), Style::default().fg(Color::White)),
                ]),
                Line::from(vec![
                    Span::styled("SNR: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("{} dB ({})", snr, snr_quality.0),
                        Style::default().fg(snr_quality.1),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "SNR Guidelines:",
                    Style::default().fg(Color::Gray).bold(),
                )),
                Line::from(Span::styled(
                    "  40+ dB: Excellent",
                    Style::default().fg(Color::Green),
                )),
                Line::from(Span::styled(
                    "  25-40: Good",
                    Style::default().fg(Color::LightGreen),
                )),
                Line::from(Span::styled(
                    "  15-25: Fair",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(Span::styled("  <15: Poor", Style::default().fg(Color::Red))),
            ]
        } else {
            vec![Line::from(Span::styled(
                "SNR data not available",
                Style::default().fg(Color::Gray),
            ))]
        };

        let snr_block = Paragraph::new(snr_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Signal Quality ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(snr_block, right_chunks[1]);

        // Empty space for future WiFi scan results
        let scan_block = Paragraph::new("").block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Nearby Networks ")
                .title_style(Style::default().fg(Color::Cyan)),
        );
        frame.render_widget(scan_block, right_chunks[2]);
    } else if app.wifi_running {
        let loading = Paragraph::new("Scanning WiFi...")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" WiFi Analysis ")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(loading, area);
    } else {
        let no_wifi = Paragraph::new("No WiFi information available\n\nPress 'r' to refresh")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" WiFi Analysis ")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);
        frame.render_widget(no_wifi, area);
    }
}

/// Draw the footer/status bar.
fn draw_footer(app: &App, frame: &mut Frame, area: Rect) {
    let default_help = "q:Quit | Tab/←→:Switch | 1-6:Tabs | r:Refresh | Enter:Input | c:Clear";

    let tab_help = match app.current_tab {
        Tab::Ping => "Space:Ping | Enter:Edit target | c:Clear results",
        Tab::Traceroute => "Enter:Edit target | c:Clear",
        Tab::Dns => "Enter:Edit target | c:Clear",
        Tab::Wifi => "r:Refresh | Enter:Scan",
        Tab::Interfaces => "↑↓/jk:Select interface",
        Tab::Dashboard => "r:Refresh all",
    };

    let status = app.status_message.as_deref().unwrap_or(default_help);

    let mode_indicator = if app.input_mode {
        Span::styled(
            " [INPUT] ",
            Style::default().fg(Color::Black).bg(Color::Yellow),
        )
    } else if app.ping_running || app.traceroute_running || app.dns_running || app.wifi_running {
        Span::styled(
            " [BUSY] ",
            Style::default().fg(Color::Black).bg(Color::Cyan),
        )
    } else {
        Span::raw("")
    };

    let footer_text = if app.status_message.is_some() {
        Line::from(vec![
            mode_indicator,
            Span::styled(status, Style::default().fg(Color::Yellow)),
        ])
    } else {
        Line::from(vec![
            mode_indicator,
            Span::styled(tab_help, Style::default().fg(Color::Cyan)),
        ])
    };

    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));

    frame.render_widget(footer, area);
}

/// Draw the input popup.
fn draw_input_popup(app: &App, frame: &mut Frame) {
    let area = centered_rect(60, 20, frame.area());

    // Clear the background
    frame.render_widget(Clear, area);

    let title = match app.current_tab {
        Tab::Ping => " Enter Ping Target ",
        Tab::Dns => " Enter Hostname ",
        Tab::Traceroute => " Enter Traceroute Target ",
        _ => " Input ",
    };

    let input = match app.current_tab {
        Tab::Ping => &app.ping_target,
        Tab::Dns => &app.dns_target,
        Tab::Traceroute => &app.traceroute_target,
        _ => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .title_style(Style::default().fg(Color::Yellow).bold())
        .style(Style::default().bg(Color::DarkGray));

    let help = "ESC to cancel | Enter to confirm";
    let cursor = "█";

    let text = vec![
        Line::from(Span::styled(
            format!("{}{}", input, cursor),
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from(Span::styled(help, Style::default().fg(Color::Gray))),
    ];

    let popup = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(popup, area);
}

/// Helper function to create a centered rect.
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
