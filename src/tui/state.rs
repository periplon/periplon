//! Application state management
//!
//! Defines the core state structures for the TUI application, including
//! view modes, modal states, and workflow management state.

use crate::dsl::DSLWorkflow;
use crate::tui::help::{HelpContext, HelpViewState};
use crate::tui::views::editor::EditorMode;
use crate::tui::views::generator::GeneratorState;
use crate::tui::views::state_browser::StateBrowserState;
use std::path::PathBuf;

/// Main application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Current view mode
    pub view_mode: ViewMode,

    /// Active modal (if any)
    pub modal: Option<Modal>,

    /// List of available workflows
    pub workflows: Vec<WorkflowEntry>,

    /// Currently selected workflow index
    pub selected_workflow: usize,

    /// Currently loaded workflow for editing
    pub current_workflow: Option<DSLWorkflow>,

    /// Path to currently loaded workflow file
    pub current_workflow_path: Option<PathBuf>,

    /// Workflow execution state
    pub execution_state: Option<ExecutionState>,

    /// Search query (for workflow filtering)
    pub search_query: String,

    /// Viewer state for workflow visualization
    pub viewer_state: ViewerState,

    /// Editor state for workflow editing
    pub editor_state: EditorState,

    /// Help view state
    pub help_state: HelpViewState,

    /// State browser state
    pub state_browser: StateBrowserState,

    /// Generator state for AI workflow generation
    pub generator_state: GeneratorState,

    /// Application running flag
    pub running: bool,

    /// Input buffer for modal text input
    pub input_buffer: String,
}

impl AppState {
    /// Create new application state
    pub fn new() -> Self {
        Self {
            view_mode: ViewMode::WorkflowList,
            modal: None,
            workflows: Vec::new(),
            selected_workflow: 0,
            current_workflow: None,
            current_workflow_path: None,
            execution_state: None,
            search_query: String::new(),
            viewer_state: ViewerState::new(),
            editor_state: EditorState::new(),
            help_state: HelpViewState::new(HelpContext::General),
            state_browser: StateBrowserState::default(),
            generator_state: GeneratorState::new_create(),
            running: true,
            input_buffer: String::new(),
        }
    }

    /// Reset to initial state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Check if a modal is active
    pub fn has_modal(&self) -> bool {
        self.modal.is_some()
    }

    /// Close current modal
    pub fn close_modal(&mut self) {
        self.modal = None;
    }

    /// Get filtered workflows based on search query
    pub fn filtered_workflows(&self) -> Vec<&WorkflowEntry> {
        if self.search_query.is_empty() {
            self.workflows.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.workflows
                .iter()
                .filter(|w| {
                    w.name.to_lowercase().contains(&query)
                        || w.description
                            .as_ref()
                            .map(|d| d.to_lowercase().contains(&query))
                            .unwrap_or(false)
                })
                .collect()
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// TUI view modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Workflow list/browser
    WorkflowList,
    /// Workflow viewer (read-only visualization)
    Viewer,
    /// Workflow editor (YAML editing)
    Editor,
    /// Workflow generator (AI-assisted creation)
    Generator,
    /// Execution monitor (watch running workflows)
    ExecutionMonitor,
    /// State browser (view persisted states)
    StateBrowser,
    /// Help system
    Help,
}

/// Modal dialog types
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    /// Confirmation dialog
    Confirm {
        title: String,
        message: String,
        action: ConfirmAction,
    },
    /// Input dialog
    Input {
        title: String,
        prompt: String,
        default: String,
        action: InputAction,
    },
    /// Error message
    Error { title: String, message: String },
    /// Success message
    Success { title: String, message: String },
    /// Info message
    Info { title: String, message: String },
}

/// Actions that can be confirmed
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmAction {
    /// Delete workflow file
    DeleteWorkflow(PathBuf),
    /// Execute workflow
    ExecuteWorkflow(PathBuf),
    /// Discard editor changes
    DiscardChanges,
    /// Exit application
    Exit,
    /// Stop execution (alias for compatibility)
    StopExecution,
}

/// Actions that require input
#[derive(Debug, Clone, PartialEq)]
pub enum InputAction {
    /// Create new workflow
    CreateWorkflow,
    /// Rename workflow
    RenameWorkflow(PathBuf),
    /// Generate workflow from description
    GenerateWorkflow,
    /// Set workflow description
    SetWorkflowDescription,
    /// Save workflow as
    SaveWorkflowAs,
}

/// Workflow entry in the list
#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowEntry {
    /// Workflow file name
    pub name: String,

    /// Full path to workflow file
    pub path: PathBuf,

    /// Optional description from workflow
    pub description: Option<String>,

    /// Workflow version
    pub version: Option<String>,

    /// Is this workflow valid?
    pub valid: bool,

    /// Validation errors (if any)
    pub errors: Vec<String>,
}

/// Workflow execution state
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionState {
    /// Workflow being executed
    pub workflow_path: PathBuf,

    /// Current execution status
    pub status: ExecutionStatus,

    /// Current agent being executed
    pub current_agent: Option<String>,

    /// Current task being executed
    pub current_task: Option<String>,

    /// Execution progress (0.0 - 1.0)
    pub progress: f64,

    /// Execution log messages
    pub log: Vec<String>,

    /// Completed tasks
    pub completed_tasks: Vec<String>,

    /// Failed tasks
    pub failed_tasks: Vec<String>,

    /// Start time
    pub started_at: std::time::Instant,
}

/// Execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Preparing to execute
    Preparing,
    /// Currently running
    Running,
    /// Paused
    Paused,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed,
}

/// Workflow viewer display mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowViewMode {
    /// Condensed summary view
    Condensed,
    /// Full YAML view
    Full,
}

/// Viewer state for workflow visualization
#[derive(Debug, Clone, PartialEq)]
pub struct ViewerState {
    /// Current scroll position (used as scroll_offset in usize calculations)
    pub scroll: u16,

    /// Current section being viewed
    pub section: ViewerSection,

    /// Expanded sections
    pub expanded: Vec<String>,

    /// View mode (condensed vs full)
    pub view_mode: WorkflowViewMode,
}

impl ViewerState {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            section: ViewerSection::Overview,
            expanded: Vec::new(),
            view_mode: WorkflowViewMode::Condensed,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            WorkflowViewMode::Condensed => WorkflowViewMode::Full,
            WorkflowViewMode::Full => WorkflowViewMode::Condensed,
        };
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self, _max_lines: usize) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn page_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(10);
    }

    pub fn page_down(&mut self, _max_lines: usize) {
        self.scroll = self.scroll.saturating_add(10);
    }

    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    pub fn scroll_to_bottom(&mut self, max_lines: usize) {
        self.scroll = max_lines as u16;
    }
}

impl Default for ViewerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Viewer sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewerSection {
    /// Workflow overview
    Overview,
    /// Agents section
    Agents,
    /// Tasks section
    Tasks,
    /// Variables section
    Variables,
}

/// Editor state for workflow editing
#[derive(Debug, Clone, PartialEq)]
pub struct EditorState {
    /// Editor mode
    pub mode: EditorMode,

    /// Current cursor position (line, column)
    pub cursor: (usize, usize),

    /// Scroll offset
    pub scroll: (u16, u16),

    /// Has unsaved changes?
    pub modified: bool,

    /// Validation errors
    pub errors: Vec<EditorError>,

    /// Auto-completion suggestions
    pub suggestions: Vec<String>,

    /// File content being edited
    pub content: String,

    /// File path being edited
    pub file_path: Option<PathBuf>,

    /// Expanded validation view (full error details)
    pub validation_expanded: bool,
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Text,
            cursor: (0, 0),
            scroll: (0, 0),
            modified: false,
            errors: Vec::new(),
            suggestions: Vec::new(),
            content: String::new(),
            file_path: None,
            validation_expanded: false,
        }
    }

    /// Reset editor state to clean state
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Check if editor has a file loaded
    pub fn has_file(&self) -> bool {
        self.file_path.is_some()
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Get cursor line (convenience method)
    pub fn cursor_line(&self) -> usize {
        self.cursor.0
    }

    /// Get scroll offset (convenience method)
    pub fn scroll_offset(&self) -> usize {
        self.scroll.0 as usize
    }

    /// Validate cursor position against content
    pub fn validate_cursor(&mut self) {
        let lines: Vec<&str> = self.content.lines().collect();

        // Ensure line is valid
        if lines.is_empty() {
            self.cursor = (0, 0);
            return;
        }

        if self.cursor.0 >= lines.len() {
            self.cursor.0 = lines.len().saturating_sub(1);
        }

        // Ensure column is valid
        if self.cursor.0 < lines.len() {
            let line_len = lines[self.cursor.0].len();
            if self.cursor.1 > line_len {
                self.cursor.1 = line_len;
            }
        }
    }
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

/// Editor error information
#[derive(Debug, Clone, PartialEq)]
pub struct EditorError {
    /// Line number (0-indexed)
    pub line: usize,

    /// Column number (0-indexed)
    pub column: Option<usize>,

    /// Error message
    pub message: String,

    /// Error severity
    pub severity: ErrorSeverity,
}

/// Error severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    /// Error (prevents execution)
    Error,
    /// Warning (execution possible but not recommended)
    Warning,
    /// Info (helpful suggestion)
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.view_mode, ViewMode::WorkflowList);
        assert!(state.modal.is_none());
        assert!(state.workflows.is_empty());
        assert_eq!(state.selected_workflow, 0);
        assert!(state.current_workflow.is_none());
        assert!(state.execution_state.is_none());
        assert!(state.search_query.is_empty());
        assert!(state.running);
    }

    #[test]
    fn test_app_state_reset() {
        let mut state = AppState::new();
        state.view_mode = ViewMode::Editor;
        state.search_query = "test".to_string();
        state.running = false;

        state.reset();

        assert_eq!(state.view_mode, ViewMode::WorkflowList);
        assert_eq!(state.search_query, "");
        assert!(state.running);
    }

    #[test]
    fn test_modal_management() {
        let mut state = AppState::new();

        assert!(!state.has_modal());

        state.modal = Some(Modal::Success {
            title: "Test".to_string(),
            message: "Success".to_string(),
        });

        assert!(state.has_modal());

        state.close_modal();

        assert!(!state.has_modal());
    }

    #[test]
    fn test_workflow_filtering_empty_query() {
        let mut state = AppState::new();
        state.workflows = vec![
            WorkflowEntry {
                name: "test1.yaml".to_string(),
                path: PathBuf::from("test1.yaml"),
                description: Some("First workflow".to_string()),
                version: None,
                valid: true,
                errors: vec![],
            },
            WorkflowEntry {
                name: "test2.yaml".to_string(),
                path: PathBuf::from("test2.yaml"),
                description: Some("Second workflow".to_string()),
                version: None,
                valid: true,
                errors: vec![],
            },
        ];

        let filtered = state.filtered_workflows();
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_workflow_filtering_by_name() {
        let mut state = AppState::new();
        state.workflows = vec![
            WorkflowEntry {
                name: "test1.yaml".to_string(),
                path: PathBuf::from("test1.yaml"),
                description: Some("First workflow".to_string()),
                version: None,
                valid: true,
                errors: vec![],
            },
            WorkflowEntry {
                name: "demo.yaml".to_string(),
                path: PathBuf::from("demo.yaml"),
                description: Some("Demo workflow".to_string()),
                version: None,
                valid: true,
                errors: vec![],
            },
        ];

        state.search_query = "test".to_string();
        let filtered = state.filtered_workflows();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "test1.yaml");
    }

    #[test]
    fn test_workflow_filtering_by_description() {
        let mut state = AppState::new();
        state.workflows = vec![
            WorkflowEntry {
                name: "a.yaml".to_string(),
                path: PathBuf::from("a.yaml"),
                description: Some("Database migration".to_string()),
                version: None,
                valid: true,
                errors: vec![],
            },
            WorkflowEntry {
                name: "b.yaml".to_string(),
                path: PathBuf::from("b.yaml"),
                description: Some("API testing".to_string()),
                version: None,
                valid: true,
                errors: vec![],
            },
        ];

        state.search_query = "database".to_string();
        let filtered = state.filtered_workflows();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "a.yaml");
    }

    #[test]
    fn test_workflow_filtering_case_insensitive() {
        let mut state = AppState::new();
        state.workflows = vec![WorkflowEntry {
            name: "TEST.yaml".to_string(),
            path: PathBuf::from("TEST.yaml"),
            description: Some("UPPER CASE".to_string()),
            version: None,
            valid: true,
            errors: vec![],
        }];

        state.search_query = "test".to_string();
        let filtered = state.filtered_workflows();
        assert_eq!(filtered.len(), 1);

        state.search_query = "upper".to_string();
        let filtered = state.filtered_workflows();
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_viewer_state_new() {
        let state = ViewerState::new();
        assert_eq!(state.scroll, 0);
        assert_eq!(state.section, ViewerSection::Overview);
        assert!(state.expanded.is_empty());
    }

    #[test]
    fn test_editor_state_new() {
        let state = EditorState::new();
        assert_eq!(state.mode, EditorMode::Text);
        assert_eq!(state.cursor, (0, 0));
        assert_eq!(state.scroll, (0, 0));
        assert!(!state.modified);
        assert!(state.errors.is_empty());
        assert!(state.suggestions.is_empty());
    }

    #[test]
    fn test_view_mode_equality() {
        assert_eq!(ViewMode::WorkflowList, ViewMode::WorkflowList);
        assert_ne!(ViewMode::WorkflowList, ViewMode::Editor);
    }

    #[test]
    fn test_execution_status_equality() {
        assert_eq!(ExecutionStatus::Running, ExecutionStatus::Running);
        assert_ne!(ExecutionStatus::Running, ExecutionStatus::Completed);
    }

    #[test]
    fn test_modal_equality() {
        let modal1 = Modal::Success {
            title: "Test".to_string(),
            message: "Success".to_string(),
        };
        let modal2 = Modal::Success {
            title: "Test".to_string(),
            message: "Success".to_string(),
        };
        let modal3 = Modal::Error {
            title: "Test".to_string(),
            message: "Error".to_string(),
        };

        assert_eq!(modal1, modal2);
        assert_ne!(modal1, modal3);
    }

    #[test]
    fn test_confirm_action_equality() {
        let action1 = ConfirmAction::Exit;
        let action2 = ConfirmAction::Exit;
        let action3 = ConfirmAction::DiscardChanges;

        assert_eq!(action1, action2);
        assert_ne!(action1, action3);
    }

    #[test]
    fn test_error_severity() {
        assert_eq!(ErrorSeverity::Error, ErrorSeverity::Error);
        assert_ne!(ErrorSeverity::Error, ErrorSeverity::Warning);
    }
}
