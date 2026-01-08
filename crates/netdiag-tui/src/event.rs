//! Event handling for the TUI.

use crate::error::{TuiError, TuiResult};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

/// Application events.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AppEvent {
    /// Terminal tick (for animations/updates).
    Tick,
    /// Key press event.
    Key(KeyEvent),
    /// Resize event.
    Resize(u16, u16),
    /// Quit the application.
    Quit,
}

/// Event handler that runs in a separate task.
pub struct EventHandler {
    /// Event receiver.
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    /// Create a new event handler.
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn event polling task
        tokio::spawn(async move {
            loop {
                // Poll for events
                if event::poll(tick_rate).unwrap_or(false) {
                    if let Ok(evt) = event::read() {
                        let app_event = match evt {
                            Event::Key(key) => {
                                // Handle Ctrl+C and Ctrl+Q as quit
                                if key.modifiers.contains(KeyModifiers::CONTROL) {
                                    match key.code {
                                        KeyCode::Char('c') | KeyCode::Char('q') => AppEvent::Quit,
                                        _ => AppEvent::Key(key),
                                    }
                                } else {
                                    AppEvent::Key(key)
                                }
                            }
                            Event::Resize(w, h) => AppEvent::Resize(w, h),
                            _ => AppEvent::Tick,
                        };

                        if tx.send(app_event).is_err() {
                            break;
                        }
                    }
                } else {
                    // Send tick on timeout
                    if tx.send(AppEvent::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        Self { rx }
    }

    /// Get the next event.
    pub async fn next(&mut self) -> TuiResult<AppEvent> {
        self.rx
            .recv()
            .await
            .ok_or_else(|| TuiError::Channel("Event channel closed".to_string()))
    }
}
