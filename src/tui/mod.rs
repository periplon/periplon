//! TUI (Terminal User Interface) for DSL workflow management
//!
//! This module provides an interactive REPL-style terminal interface for creating,
//! editing, and executing DSL workflows. It follows hexagonal architecture patterns,
//! treating the TUI as a primary adapter that drives the DSL application layer.
//!
//! # Architecture
//!
//! - `app.rs`: Core application state, event loop, view router, modal system
//! - `events.rs`: Event handling and input processing
//! - `ui/`: View components (workflow list, editor, execution monitor, help)
//! - `state.rs`: Application state management
//! - `theme.rs`: Color schemes and styling
//! - `help/`: Comprehensive help and documentation system
//!
//! # Usage
//!
//! ```no_run
//! use periplon_sdk::tui::TuiApp;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut app = TuiApp::new()?;
//!     app.run().await?;
//!     Ok(())
//! }
//! ```

pub mod app;
pub mod events;
pub mod help;
pub mod state;
pub mod theme;
pub mod ui;
pub mod views;

pub use app::{App, AppConfig};
pub use state::{AppState, ViewMode, Modal};

/// Type alias for backward compatibility and clearer usage
pub type TuiApp = App;
