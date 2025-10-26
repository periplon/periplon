//! AI workflow generator view component
//!
//! Provides a consistent component wrapper around the generator functionality.
//! The actual implementation is in `crate::tui::views::generator`.

use crate::tui::theme::Theme;
use ratatui::Frame;

// Re-export types from the views module
pub use crate::tui::views::generator::{
    FocusPanel, GenerationStatus, GeneratorMode, GeneratorState,
};

/// AI workflow generator view component
pub struct GeneratorView;

impl GeneratorView {
    /// Create new generator view
    pub fn new() -> Self {
        Self
    }

    /// Render the AI workflow generator
    pub fn render(
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        state: &GeneratorState,
        theme: &Theme,
    ) {
        crate::tui::views::generator::render(frame, area, state, theme);
    }
}

impl Default for GeneratorView {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy render function for backward compatibility
pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &GeneratorState,
    theme: &Theme,
) {
    crate::tui::views::generator::render(frame, area, state, theme);
}
