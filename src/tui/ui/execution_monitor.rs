//! Execution monitor view component
//!
//! Provides a consistent component wrapper around the execution monitor functionality.
//! The actual implementation is in `crate::tui::views::execution_monitor`.

use crate::tui::theme::Theme;
use ratatui::Frame;

// Re-export types from the views module
pub use crate::tui::views::execution_monitor::{
    ExecutionMonitorState, ExecutionStatistics, ExecutionStatus, LogEntry, LogLevel, MonitorPanel,
    TaskExecutionState, TaskStatus, TokenUsage,
};

/// Execution monitor view component
pub struct ExecutionMonitorView;

impl ExecutionMonitorView {
    /// Create new execution monitor view
    pub fn new() -> Self {
        Self
    }

    /// Render the execution monitor
    pub fn render(
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        state: &ExecutionMonitorState,
        theme: &Theme,
    ) {
        crate::tui::views::execution_monitor::render(frame, area, state, theme);
    }
}

impl Default for ExecutionMonitorView {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy render function for backward compatibility
pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &ExecutionMonitorState,
    theme: &Theme,
) {
    crate::tui::views::execution_monitor::render(frame, area, state, theme);
}
