//! File manager view component
//!
//! Provides a consistent component wrapper around the file manager functionality.
//! The actual implementation is in `crate::tui::views::file_manager`.

use crate::tui::theme::Theme;
use ratatui::Frame;

// Re-export types from the views module
pub use crate::tui::views::file_manager::{
    FileActionMode, FileEntry, FileManagerState, FileManagerViewMode, FileSortMode,
};

/// File manager view component
pub struct FileManagerView;

impl FileManagerView {
    /// Create new file manager view
    pub fn new() -> Self {
        Self
    }

    /// Render the file manager
    pub fn render(
        frame: &mut Frame,
        area: ratatui::layout::Rect,
        state: &mut FileManagerState,
        theme: &Theme,
    ) {
        crate::tui::views::file_manager::render_file_manager(frame, area, state, theme);
    }
}

impl Default for FileManagerView {
    fn default() -> Self {
        Self::new()
    }
}

/// Legacy render function for backward compatibility
pub fn render_file_manager(
    frame: &mut Frame,
    area: ratatui::layout::Rect,
    state: &mut FileManagerState,
    theme: &Theme,
) {
    crate::tui::views::file_manager::render_file_manager(frame, area, state, theme);
}
