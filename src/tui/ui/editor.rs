//! Workflow editor view component
//!
//! Provides a consistent component wrapper around the editor functionality.
//! The actual implementation is in `crate::tui::views::editor`.

use crate::tui::state::EditorState;
use crate::tui::theme::Theme;
use ratatui::Frame;

// Re-export types and functions from the views module
pub use crate::tui::views::editor::{
    get_autocomplete_suggestions, validate_and_get_feedback, AutoCompletionSuggestion, EditorMode,
    ValidationFeedback,
};

/// Workflow editor component
pub struct EditorView;

impl EditorView {
    /// Create new editor view
    pub fn new() -> Self {
        Self
    }

    /// Render the workflow editor
    pub fn render(
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        state: &EditorState,
        feedback: &ValidationFeedback,
        theme: &Theme,
    ) {
        crate::tui::views::editor::render(frame, area, state, feedback, theme);
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy render function for backward compatibility
pub fn render(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &EditorState,
    feedback: &ValidationFeedback,
    theme: &Theme,
) {
    crate::tui::views::editor::render(frame, area, state, feedback, theme);
}
