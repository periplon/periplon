//! State browser view component
//!
//! Provides a consistent component wrapper around the state browser functionality.
//! The actual implementation is in `crate::tui::views::state_browser`.

use crate::tui::theme::Theme;
use ratatui::Frame;

// Re-export types from the views module
pub use crate::tui::views::state_browser::{
    StateBrowserState, StateBrowserViewMode, StateEntry, StateSortMode,
};

/// State browser view component
pub struct StateBrowserView;

impl StateBrowserView {
    /// Create new state browser view
    pub fn new() -> Self {
        Self
    }

    /// Render the state browser
    pub fn render(
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        state: &mut StateBrowserState,
        theme: &Theme,
    ) {
        crate::tui::views::state_browser::render_state_browser(frame, area, state, theme);
    }
}

impl Default for StateBrowserView {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy render function for backward compatibility
pub fn render_state_browser(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &mut StateBrowserState,
    theme: &Theme,
) {
    crate::tui::views::state_browser::render_state_browser(frame, area, state, theme);
}
