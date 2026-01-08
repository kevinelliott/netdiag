//! UI rendering.

use crate::app::{App, Tab};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table, Tabs},
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
        .map(|t| {
            let style = if *t == app.current_tab {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            Line::from(Span::styled(format!(" {} ", t.name()), style))
        })
        .collect();

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" netdiag - Network Diagnostics ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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

    // Left panel - System info
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // System summary
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    let system_info = vec![
        format!("Hostname: {}", hostname),
        format!("Interfaces: {} total", app.interfaces.len()),
        format!(
            "Active: {} up",
            app.interfaces.iter().filter(|i| i.is_up).count()
        ),
        format!(
            "Default: {}",
            app.interfaces
                .iter()
                .find(|i| i.is_default)
                .map(|i| i.name.as_str())
                .unwrap_or("None")
        ),
    ];

    let system_list: Vec<ListItem> = system_info
        .iter()
        .map(|s| ListItem::new(s.as_str()))
        .collect();

    let system_block = List::new(system_list).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" System Info "),
    );
    frame.render_widget(system_block, left_chunks[0]);

    // Quick stats
    let stats = vec![
        format!("Last Update: {}", app.last_update.format("%H:%M:%S")),
        format!("Status: OK"),
    ];

    let stats_list: Vec<ListItem> = stats.iter().map(|s| ListItem::new(s.as_str())).collect();

    let stats_block = List::new(stats_list).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Status "),
    );
    frame.render_widget(stats_block, left_chunks[1]);

    // Right panel - Interface summary
    let interface_rows: Vec<Row> = app
        .interfaces
        .iter()
        .take(10)
        .map(|iface| {
            let status_style = if iface.is_up {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::Red)
            };
            let default_marker = if iface.is_default { "*" } else { "" };

            Row::new(vec![
                Cell::from(format!("{}{}", iface.name, default_marker)),
                Cell::from(if iface.is_up { "UP" } else { "DOWN" }).style(status_style),
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
        Row::new(vec!["Interface", "Status", "IPv4"])
            .style(Style::default().add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Interfaces "),
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
            .title(" Interfaces (↑↓ to select) "),
    );
    frame.render_widget(list, chunks[0]);

    // Interface details
    if let Some(iface) = app.interfaces.get(app.selected_interface) {
        let details = vec![
            format!("Name: {}", iface.name),
            format!("Type: {}", iface.if_type),
            format!("Status: {}", if iface.is_up { "UP" } else { "DOWN" }),
            format!("Default: {}", if iface.is_default { "Yes" } else { "No" }),
            String::new(),
            "IPv4 Addresses:".to_string(),
        ];

        let mut lines: Vec<ListItem> = details.iter().map(|s| ListItem::new(s.as_str())).collect();

        for ip in &iface.ipv4 {
            lines.push(ListItem::new(format!("  {}", ip)));
        }

        lines.push(ListItem::new(""));
        lines.push(ListItem::new("IPv6 Addresses:"));
        for ip in &iface.ipv6 {
            lines.push(ListItem::new(format!("  {}", ip)));
        }

        if let Some(ref mac) = iface.mac {
            lines.push(ListItem::new(""));
            lines.push(ListItem::new(format!("MAC: {}", mac)));
        }

        let details_list = List::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Details "),
        );
        frame.render_widget(details_list, chunks[1]);
    }
}

/// Draw the ping view.
fn draw_ping(app: &App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Target input
    let input_text = if app.ping_target.is_empty() {
        "Press Enter or 'i' to enter target..."
    } else {
        &app.ping_target
    };

    let input_style = if app.input_mode && app.current_tab == Tab::Ping {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default()
    };

    let input = Paragraph::new(input_text).style(input_style).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Ping Target "),
    );
    frame.render_widget(input, chunks[0]);

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
            .title(" Results "),
    );
    frame.render_widget(results_list, chunks[1]);
}

/// Draw the traceroute view.
fn draw_traceroute(_app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Traceroute ");

    let text = Paragraph::new("Traceroute functionality\n\nPress Enter to trace a route...")
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(text, area);
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
    } else {
        Style::default()
    };

    let input = Paragraph::new(input_text).style(input_style).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" DNS Lookup "),
    );
    frame.render_widget(input, chunks[0]);

    // Results
    let results: Vec<ListItem> = app
        .dns_results
        .iter()
        .map(|r| ListItem::new(r.as_str()))
        .collect();

    let results_list = List::new(results).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Results "),
    );
    frame.render_widget(results_list, chunks[1]);
}

/// Draw the WiFi view.
fn draw_wifi(_app: &App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" WiFi Analysis ");

    let text = Paragraph::new("WiFi analysis functionality\n\nScanning for networks...")
        .block(block)
        .alignment(Alignment::Center);

    frame.render_widget(text, area);
}

/// Draw the footer/status bar.
fn draw_footer(app: &App, frame: &mut Frame, area: Rect) {
    let status = app
        .status_message
        .as_deref()
        .unwrap_or("Ready | q:Quit | Tab:Switch | r:Refresh | 1-6:Tabs");

    let mode_indicator = if app.input_mode {
        " [INPUT MODE] "
    } else {
        ""
    };

    let footer = Paragraph::new(format!("{}{}", mode_indicator, status))
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL));

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
        _ => " Input ",
    };

    let input = match app.current_tab {
        Tab::Ping => &app.ping_target,
        Tab::Dns => &app.dns_target,
        _ => "",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().bg(Color::DarkGray));

    let help = "ESC to cancel | Enter to confirm";
    let text = format!("{}\n\n{}", input, help);

    let popup = Paragraph::new(text)
        .block(block)
        .style(Style::default().fg(Color::White))
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
