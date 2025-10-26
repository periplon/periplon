//! Help view component wrapper
//!
//! Provides a consistent component wrapper around the help system functionality.
//! The actual implementation is in `crate::tui::help`.

use crate::tui::theme::Theme;
use ratatui::Frame;

// Re-export types from the help module
pub use crate::tui::help::{HelpContent, HelpContext, HelpSection, HelpTopic, HelpViewState};

/// Help view component
pub struct HelpView;

impl HelpView {
    /// Create new help view
    pub fn new() -> Self {
        Self
    }

    /// Render the help screen
    pub fn render(
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        state: &mut HelpViewState,
        theme: &Theme,
    ) {
        let help_view = crate::tui::help::HelpView::new();
        help_view.render(frame, area, state, theme);
    }
}

impl Default for HelpView {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy render function for backward compatibility
pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &mut HelpViewState,
    theme: &Theme,
) {
    let help_view = crate::tui::help::HelpView::new();
    help_view.render(frame, area, state, theme);
}
