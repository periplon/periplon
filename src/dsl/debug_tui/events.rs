//! Event handling for TUI
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

/// TUI events
#[derive(Debug, Clone)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),

    /// Terminal resize
    Resize(u16, u16),

    /// Tick for periodic updates
    Tick,

    /// Quit application
    Quit,
}

/// Event handler for TUI
pub struct EventHandler {
    tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx }
    }

    /// Get next event
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    /// Start event loop
    pub fn start(&self) {
        let tx = self.tx.clone();

        tokio::spawn(async move {
            loop {
                // Poll for events with timeout
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    match event::read() {
                        Ok(event::Event::Key(key)) => {
                            // Ctrl+C or Ctrl+D to quit
                            if (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('d'))
                                && key.modifiers.contains(KeyModifiers::CONTROL)
                            {
                                let _ = tx.send(Event::Quit);
                                break;
                            }

                            let _ = tx.send(Event::Key(key));
                        }
                        Ok(event::Event::Resize(w, h)) => {
                            let _ = tx.send(Event::Resize(w, h));
                        }
                        _ => {}
                    }
                } else {
                    // Send tick event
                    let _ = tx.send(Event::Tick);
                }
            }
        });
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
