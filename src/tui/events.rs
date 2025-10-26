//! Event handling for TUI
//!
//! Manages keyboard input, terminal events, and custom application events.

use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyModifiers};
use std::time::Duration;
use tokio::sync::mpsc;

/// Application event types
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Keyboard input
    Key(KeyEvent),

    /// Terminal resize
    Resize(u16, u16),

    /// Application tick (for animations, async updates)
    Tick,

    /// Workflow execution update
    ExecutionUpdate(ExecutionUpdate),

    /// Error occurred
    Error(String),

    /// Quit application
    Quit,
}

/// Keyboard event wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Create new key event
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Check if Ctrl is pressed
    pub fn is_ctrl(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }

    /// Check if Alt is pressed
    pub fn is_alt(&self) -> bool {
        self.modifiers.contains(KeyModifiers::ALT)
    }

    /// Check if Shift is pressed
    pub fn is_shift(&self) -> bool {
        self.modifiers.contains(KeyModifiers::SHIFT)
    }
}

/// Execution update event
#[derive(Debug, Clone)]
pub enum ExecutionUpdate {
    TaskStarted(String),
    TaskCompleted(String),
    TaskFailed { task: String, error: String },
    LogMessage { level: String, message: String },
    StatusChanged(String),
}

/// Event handler that polls terminal events and forwards them
pub struct EventHandler {
    /// Event sender
    tx: mpsc::UnboundedSender<AppEvent>,

    /// Event receiver
    rx: mpsc::UnboundedReceiver<AppEvent>,

    /// Tick rate for periodic events
    tick_rate: Duration,
}

impl EventHandler {
    /// Create new event handler
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self { tx, rx, tick_rate }
    }

    /// Get a sender for external events
    pub fn sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.tx.clone()
    }

    /// Start event polling loop
    pub async fn start_polling(&self) {
        let tx = self.tx.clone();
        let tick_rate = self.tick_rate;

        tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                tokio::select! {
                    // Poll terminal events
                    _ = tokio::time::sleep(Duration::from_millis(10)) => {
                        if event::poll(Duration::from_millis(0)).unwrap_or(false) {
                            if let Ok(event) = event::read() {
                                match event {
                                    CrosstermEvent::Key(key) => {
                                        let _ = tx.send(AppEvent::Key(KeyEvent::new(
                                            key.code,
                                            key.modifiers,
                                        )));
                                    }
                                    CrosstermEvent::Resize(w, h) => {
                                        let _ = tx.send(AppEvent::Resize(w, h));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    // Periodic tick
                    _ = tick_interval.tick() => {
                        let _ = tx.send(AppEvent::Tick);
                    }
                }
            }
        });
    }

    /// Receive next event
    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }

    /// Try to receive event without blocking
    pub fn try_next(&mut self) -> Option<AppEvent> {
        self.rx.try_recv().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_modifiers() {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(key.is_ctrl());
        assert!(!key.is_alt());
        assert!(!key.is_shift());

        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT | KeyModifiers::SHIFT);
        assert!(!key.is_ctrl());
        assert!(key.is_alt());
        assert!(key.is_shift());
    }

    #[tokio::test]
    async fn test_event_handler_creation() {
        let handler = EventHandler::new(Duration::from_millis(100));
        let sender = handler.sender();

        // Send test event
        sender.send(AppEvent::Quit).unwrap();

        // Should be able to create handler
        assert_eq!(handler.tick_rate, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_event_handler_send_receive() {
        let mut handler = EventHandler::new(Duration::from_millis(100));
        let sender = handler.sender();

        // Send event
        sender.send(AppEvent::Quit).unwrap();

        // Receive event
        let event = handler.next().await;
        assert!(event.is_some());
        assert!(matches!(event.unwrap(), AppEvent::Quit));
    }

    #[test]
    fn test_execution_update_variants() {
        let updates = vec![
            ExecutionUpdate::TaskStarted("test".to_string()),
            ExecutionUpdate::TaskCompleted("test".to_string()),
            ExecutionUpdate::TaskFailed {
                task: "test".to_string(),
                error: "error".to_string(),
            },
            ExecutionUpdate::LogMessage {
                level: "info".to_string(),
                message: "message".to_string(),
            },
            ExecutionUpdate::StatusChanged("running".to_string()),
        ];

        assert_eq!(updates.len(), 5);
    }

    #[test]
    fn test_app_event_variants() {
        let events = vec![
            AppEvent::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE)),
            AppEvent::Resize(80, 24),
            AppEvent::Tick,
            AppEvent::Error("error".to_string()),
            AppEvent::Quit,
        ];

        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_key_event_equality() {
        let key1 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let key2 = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let key3 = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_key_event_multiple_modifiers() {
        let key = KeyEvent::new(
            KeyCode::Char('c'),
            KeyModifiers::CONTROL | KeyModifiers::SHIFT,
        );

        assert!(key.is_ctrl());
        assert!(key.is_shift());
        assert!(!key.is_alt());
    }

    #[test]
    fn test_key_event_no_modifiers() {
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(!key.is_ctrl());
        assert!(!key.is_alt());
        assert!(!key.is_shift());
    }

    #[tokio::test]
    async fn test_event_handler_multiple_events() {
        let mut handler = EventHandler::new(Duration::from_millis(100));
        let sender = handler.sender();

        // Send multiple events
        sender
            .send(AppEvent::Key(KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::NONE,
            )))
            .unwrap();
        sender.send(AppEvent::Resize(100, 50)).unwrap();
        sender.send(AppEvent::Quit).unwrap();

        // Receive all events
        let event1 = handler.next().await;
        assert!(matches!(event1, Some(AppEvent::Key(_))));

        let event2 = handler.next().await;
        assert!(matches!(event2, Some(AppEvent::Resize(100, 50))));

        let event3 = handler.next().await;
        assert!(matches!(event3, Some(AppEvent::Quit)));
    }

    #[tokio::test]
    async fn test_event_handler_try_next() {
        let mut handler = EventHandler::new(Duration::from_millis(100));
        let sender = handler.sender();

        // No events yet
        assert!(handler.try_next().is_none());

        // Send event
        sender.send(AppEvent::Quit).unwrap();

        // Small delay to ensure event is processed
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should now have event
        let event = handler.try_next();
        assert!(event.is_some());
    }

    #[test]
    fn test_execution_update_task_started() {
        let update = ExecutionUpdate::TaskStarted("analyze_data".to_string());
        if let ExecutionUpdate::TaskStarted(task) = update {
            assert_eq!(task, "analyze_data");
        } else {
            panic!("Expected TaskStarted variant");
        }
    }

    #[test]
    fn test_execution_update_task_failed() {
        let update = ExecutionUpdate::TaskFailed {
            task: "process".to_string(),
            error: "timeout".to_string(),
        };

        if let ExecutionUpdate::TaskFailed { task, error } = update {
            assert_eq!(task, "process");
            assert_eq!(error, "timeout");
        } else {
            panic!("Expected TaskFailed variant");
        }
    }

    #[test]
    fn test_execution_update_log_message() {
        let update = ExecutionUpdate::LogMessage {
            level: "warn".to_string(),
            message: "Deprecation warning".to_string(),
        };

        if let ExecutionUpdate::LogMessage { level, message } = update {
            assert_eq!(level, "warn");
            assert_eq!(message, "Deprecation warning");
        } else {
            panic!("Expected LogMessage variant");
        }
    }
}
