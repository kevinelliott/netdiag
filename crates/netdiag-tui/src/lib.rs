//! # netdiag-tui
//!
//! Terminal user interface for netdiag network diagnostics.
//!
//! Provides an interactive terminal interface built with Ratatui featuring:
//! - Dashboard with system overview
//! - Real-time network monitoring
//! - Interactive ping and traceroute
//! - Interface status display
//! - WiFi analysis view

#![warn(missing_docs)]
#![warn(clippy::all)]

mod app;
mod error;
mod event;
mod ui;
mod widgets;

pub use app::App;
pub use error::{TuiError, TuiResult};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io::{self, stdout};

/// Initialize the terminal for TUI mode.
pub fn init_terminal() -> TuiResult<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restore the terminal to normal mode.
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> TuiResult<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Run the TUI application.
pub async fn run() -> TuiResult<()> {
    // Initialize terminal
    let mut terminal = init_terminal()?;

    // Create and run app
    let app = App::new();
    let result = app.run(&mut terminal).await;

    // Restore terminal
    restore_terminal(&mut terminal)?;

    result
}
