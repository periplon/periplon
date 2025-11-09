//! Debug TUI (Terminal User Interface)
//!
//! Full-screen interactive debugger with multiple panes for workflow visualization,
//! state inspection, and execution control.
pub mod app;
pub mod components;
pub mod events;
pub mod layout;
pub mod ui;

pub use app::DebugTUI;
pub use events::{Event, EventHandler};

use crate::error::Result;

/// Run the debug TUI
pub async fn run_tui() -> Result<()> {
    let mut tui = DebugTUI::new()?;
    tui.run().await
}
