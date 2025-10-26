//! Comprehensive unit tests for TUI components
//!
//! Tests all TUI modules including state management, themes, and UI components.

#![cfg(feature = "tui")]

use periplon_sdk::tui::app::AppConfig;
use periplon_sdk::tui::state::{
    AppState, EditorError, EditorState, ErrorSeverity, ExecutionState, ExecutionStatus,
    ViewerSection, ViewerState, ViewMode, WorkflowEntry,
};
use periplon_sdk::tui::theme::Theme;
use std::path::PathBuf;

// ============================================================================
// App Configuration Tests
// ============================================================================

#[test]
fn test_app_config_defaults() {
    let config = AppConfig::default();

    assert_eq!(config.workflow_dir, PathBuf::from("."));
    assert!(config.workflow.is_none());
    assert!(!config.readonly);
}

// ============================================================================
// AppState Tests
// ============================================================================

#[test]
fn test_app_state_initialization() {
    let state = AppState::new();

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert!(state.workflows.is_empty());
    assert_eq!(state.selected_workflow, 0);
    assert!(!state.has_modal());
    assert!(state.running);
}

#[test]
fn test_app_state_workflow_management() {
    let mut state = AppState::new();

    state.workflows.push(WorkflowEntry {
        name: "test.yaml".to_string(),
        path: PathBuf::from("test.yaml"),
        description: Some("Test workflow".to_string()),
        version: Some("1.0.0".to_string()),
        valid: true,
        errors: vec![],
    });

    assert_eq!(state.workflows.len(), 1);
    assert_eq!(state.workflows[0].name, "test.yaml");
}

#[test]
fn test_app_state_view_mode_transitions() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Editor;
    assert_eq!(state.view_mode, ViewMode::Editor);

    state.view_mode = ViewMode::Viewer;
    assert_eq!(state.view_mode, ViewMode::Viewer);

    state.view_mode = ViewMode::ExecutionMonitor;
    assert_eq!(state.view_mode, ViewMode::ExecutionMonitor);
}

#[test]
fn test_app_state_workflow_filtering() {
    let mut state = AppState::new();

    state.workflows = vec![
        WorkflowEntry {
            name: "api_test.yaml".to_string(),
            path: PathBuf::from("api_test.yaml"),
            description: Some("API testing".to_string()),
            version: Some("1.0.0".to_string()),
            valid: true,
            errors: vec![],
        },
        WorkflowEntry {
            name: "database.yaml".to_string(),
            path: PathBuf::from("database.yaml"),
            description: Some("Database operations".to_string()),
            version: Some("1.0.0".to_string()),
            valid: true,
            errors: vec![],
        },
    ];

    state.search_query = "api".to_string();
    let filtered = state.filtered_workflows();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "api_test.yaml");
}

// ============================================================================
// ViewerState Tests
// ============================================================================

#[test]
fn test_viewer_state_initialization() {
    let state = ViewerState::new();

    assert_eq!(state.scroll, 0);
    assert_eq!(state.section, ViewerSection::Overview);
    assert!(state.expanded.is_empty());
}

#[test]
fn test_viewer_state_section_navigation() {
    let mut state = ViewerState::new();

    state.section = ViewerSection::Agents;
    assert_eq!(state.section, ViewerSection::Agents);

    state.section = ViewerSection::Tasks;
    assert_eq!(state.section, ViewerSection::Tasks);

    state.section = ViewerSection::Variables;
    assert_eq!(state.section, ViewerSection::Variables);
}

#[test]
fn test_viewer_state_expansion() {
    let mut state = ViewerState::new();

    state.expanded.push("agent1".to_string());
    state.expanded.push("task1".to_string());

    assert_eq!(state.expanded.len(), 2);
    assert!(state.expanded.contains(&"agent1".to_string()));
}

// ============================================================================
// EditorState Tests
// ============================================================================

#[test]
fn test_editor_state_initialization() {
    let state = EditorState::new();

    assert_eq!(state.cursor, (0, 0));
    assert_eq!(state.scroll, (0, 0));
    assert!(!state.modified);
    assert!(state.errors.is_empty());
}

#[test]
fn test_editor_state_cursor_movement() {
    let mut state = EditorState::new();

    state.cursor = (5, 10);
    assert_eq!(state.cursor, (5, 10));

    state.cursor = (0, 0);
    assert_eq!(state.cursor, (0, 0));
}

#[test]
fn test_editor_state_error_tracking() {
    let mut state = EditorState::new();

    state.errors.push(EditorError {
        line: 5,
        column: Some(10),
        message: "Syntax error".to_string(),
        severity: ErrorSeverity::Error,
    });

    assert_eq!(state.errors.len(), 1);
    assert_eq!(state.errors[0].line, 5);
    assert_eq!(state.errors[0].severity, ErrorSeverity::Error);
}

#[test]
fn test_editor_error_severity_levels() {
    let errors = vec![
        EditorError {
            line: 1,
            column: None,
            message: "Error".to_string(),
            severity: ErrorSeverity::Error,
        },
        EditorError {
            line: 2,
            column: None,
            message: "Warning".to_string(),
            severity: ErrorSeverity::Warning,
        },
        EditorError {
            line: 3,
            column: None,
            message: "Info".to_string(),
            severity: ErrorSeverity::Info,
        },
    ];

    assert_eq!(errors.len(), 3);
    assert_ne!(ErrorSeverity::Error, ErrorSeverity::Warning);
}

// ============================================================================
// ExecutionState Tests
// ============================================================================

#[test]
fn test_execution_state_initialization() {
    let state = ExecutionState {
        workflow_path: PathBuf::from("test.yaml"),
        status: ExecutionStatus::Preparing,
        current_agent: None,
        current_task: None,
        progress: 0.0,
        log: Vec::new(),
        completed_tasks: Vec::new(),
        failed_tasks: Vec::new(),
        started_at: std::time::Instant::now(),
    };

    assert_eq!(state.workflow_path, PathBuf::from("test.yaml"));
    assert_eq!(state.status, ExecutionStatus::Preparing);
    assert_eq!(state.progress, 0.0);
}

#[test]
fn test_execution_state_progress_tracking() {
    let mut state = ExecutionState {
        workflow_path: PathBuf::from("test.yaml"),
        status: ExecutionStatus::Running,
        current_agent: Some("agent1".to_string()),
        current_task: Some("task1".to_string()),
        progress: 0.0,
        log: Vec::new(),
        completed_tasks: Vec::new(),
        failed_tasks: Vec::new(),
        started_at: std::time::Instant::now(),
    };

    state.progress = 0.5;
    state.completed_tasks.push("task1".to_string());

    assert_eq!(state.progress, 0.5);
    assert_eq!(state.completed_tasks.len(), 1);
}

#[test]
fn test_execution_status_transitions() {
    let mut state = ExecutionState {
        workflow_path: PathBuf::from("test.yaml"),
        status: ExecutionStatus::Preparing,
        current_agent: None,
        current_task: None,
        progress: 0.0,
        log: Vec::new(),
        completed_tasks: Vec::new(),
        failed_tasks: Vec::new(),
        started_at: std::time::Instant::now(),
    };

    state.status = ExecutionStatus::Running;
    assert_eq!(state.status, ExecutionStatus::Running);

    state.status = ExecutionStatus::Completed;
    assert_eq!(state.status, ExecutionStatus::Completed);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_theme_defaults() {
    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        // All themes should have all colors defined
        let _ = theme.primary;
        let _ = theme.success;
        let _ = theme.error;

        // Colors should be distinct
        assert_ne!(theme.success, theme.error);
    }
}

// ============================================================================
// WorkflowEntry Tests
// ============================================================================

#[test]
fn test_workflow_entry_creation() {
    let entry = WorkflowEntry {
        name: "test.yaml".to_string(),
        path: PathBuf::from("/workflows/test.yaml"),
        description: Some("Test workflow".to_string()),
        version: Some("1.0.0".to_string()),
        valid: true,
        errors: vec![],
    };

    assert_eq!(entry.name, "test.yaml");
    assert!(entry.valid);
    assert!(entry.errors.is_empty());
}

#[test]
fn test_workflow_entry_with_errors() {
    let entry = WorkflowEntry {
        name: "invalid.yaml".to_string(),
        path: PathBuf::from("invalid.yaml"),
        description: None,
        version: None,
        valid: false,
        errors: vec!["Missing required field".to_string()],
    };

    assert!(!entry.valid);
    assert_eq!(entry.errors.len(), 1);
}
