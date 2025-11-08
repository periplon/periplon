//! UI rendering logic
use super::components;
use super::layout::{Pane, TuiLayout};
use crate::dsl::debugger::{DebuggerState, Inspector};
use ratatui::Frame;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Application UI state
pub struct AppUI {
    /// Current focused pane
    pub focused_pane: Pane,

    /// Show help overlay
    pub show_help: bool,

    /// REPL input buffer
    pub repl_input: String,

    /// REPL cursor position
    pub repl_cursor: usize,
}

impl AppUI {
    /// Create new UI state
    pub fn new() -> Self {
        Self {
            focused_pane: Pane::WorkflowTree,
            show_help: false,
            repl_input: String::new(),
            repl_cursor: 0,
        }
    }

    /// Switch to next pane
    pub fn next_pane(&mut self) {
        self.focused_pane = self.focused_pane.next();
    }

    /// Switch to previous pane
    pub fn prev_pane(&mut self) {
        self.focused_pane = self.focused_pane.prev();
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Add character to REPL input
    pub fn repl_add_char(&mut self, c: char) {
        self.repl_input.insert(self.repl_cursor, c);
        self.repl_cursor += 1;
    }

    /// Remove character from REPL input (backspace)
    pub fn repl_backspace(&mut self) {
        if self.repl_cursor > 0 {
            self.repl_cursor -= 1;
            self.repl_input.remove(self.repl_cursor);
        }
    }

    /// Clear REPL input
    pub fn repl_clear(&mut self) {
        self.repl_input.clear();
        self.repl_cursor = 0;
    }

    /// Get REPL input and clear it
    pub fn repl_take_input(&mut self) -> String {
        let input = self.repl_input.clone();
        self.repl_clear();
        input
    }
}

impl Default for AppUI {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the entire TUI
pub async fn render(
    frame: &mut Frame<'_>,
    ui: &AppUI,
    debugger: Option<&Arc<Mutex<DebuggerState>>>,
    inspector: Option<&Arc<Inspector>>,
) {
    let layout = TuiLayout::new(frame.area());

    // Render status bar
    components::render_status(frame, layout.status_area, debugger).await;

    // Render workflow tree
    components::render_workflow_tree(
        frame,
        layout.workflow_area,
        ui.focused_pane == Pane::WorkflowTree,
        inspector,
    );

    // Render variables
    components::render_variables(
        frame,
        layout.variables_area,
        ui.focused_pane == Pane::Variables,
        inspector,
    )
    .await;

    // Render timeline
    components::render_timeline(
        frame,
        layout.timeline_area,
        ui.focused_pane == Pane::Timeline,
        inspector,
    )
    .await;

    // Render REPL input
    components::render_repl(
        frame,
        layout.repl_area,
        ui.focused_pane == Pane::Repl,
        &ui.repl_input,
        ui.repl_cursor,
    );

    // Render help overlay if active
    if ui.show_help {
        components::render_help(frame, frame.area());
    }
}
