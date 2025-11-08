//! TUI layout management
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Pane focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    /// Workflow tree pane
    WorkflowTree,

    /// Variables pane
    Variables,

    /// Timeline pane
    Timeline,

    /// REPL command input pane
    Repl,

    /// Help pane
    Help,
}

impl Pane {
    /// Get next pane (Tab navigation)
    pub fn next(self) -> Self {
        match self {
            Pane::WorkflowTree => Pane::Variables,
            Pane::Variables => Pane::Timeline,
            Pane::Timeline => Pane::Repl,
            Pane::Repl => Pane::WorkflowTree,
            Pane::Help => Pane::WorkflowTree,
        }
    }

    /// Get previous pane (Shift+Tab navigation)
    pub fn prev(self) -> Self {
        match self {
            Pane::WorkflowTree => Pane::Repl,
            Pane::Variables => Pane::WorkflowTree,
            Pane::Timeline => Pane::Variables,
            Pane::Repl => Pane::Timeline,
            Pane::Help => Pane::WorkflowTree,
        }
    }
}

/// Layout configuration
pub struct TuiLayout {
    /// Total area
    pub area: Rect,

    /// Workflow tree area
    pub workflow_area: Rect,

    /// Variables area
    pub variables_area: Rect,

    /// Timeline area
    pub timeline_area: Rect,

    /// REPL area
    pub repl_area: Rect,

    /// Status bar area
    pub status_area: Rect,
}

impl TuiLayout {
    /// Calculate layout from terminal size
    pub fn new(area: Rect) -> Self {
        // Main layout: vertical split
        // [Status Bar - 1 line]
        // [Main Content - rest]
        // [REPL Input - 3 lines]
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status bar
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // REPL input
            ])
            .split(area);

        let status_area = main_chunks[0];
        let content_area = main_chunks[1];
        let repl_area = main_chunks[2];

        // Content layout: horizontal split
        // [Workflow Tree - 40%] [Right Panes - 60%]
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Workflow tree
                Constraint::Percentage(60), // Variables + Timeline
            ])
            .split(content_area);

        let workflow_area = content_chunks[0];
        let right_area = content_chunks[1];

        // Right panes: vertical split
        // [Variables - 50%]
        // [Timeline - 50%]
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Variables
                Constraint::Percentage(50), // Timeline
            ])
            .split(right_area);

        let variables_area = right_chunks[0];
        let timeline_area = right_chunks[1];

        Self {
            area,
            workflow_area,
            variables_area,
            timeline_area,
            repl_area,
            status_area,
        }
    }

    /// Get area for specific pane
    pub fn get_pane_area(&self, pane: Pane) -> Rect {
        match pane {
            Pane::WorkflowTree => self.workflow_area,
            Pane::Variables => self.variables_area,
            Pane::Timeline => self.timeline_area,
            Pane::Repl => self.repl_area,
            Pane::Help => self.area,
        }
    }
}
