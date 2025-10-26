//! Integration tests for TUI components
//!
//! Tests end-to-end functionality of the DSL TUI including workflow
//! management, editing, execution, and state persistence.

#![cfg(feature = "tui")]

use periplon_sdk::dsl::{parse_workflow_file, validate_workflow};
use periplon_sdk::tui::app::AppConfig;
use periplon_sdk::tui::state::{
    AppState, ConfirmAction, EditorError, ErrorSeverity, ExecutionState, ExecutionStatus,
    InputAction, Modal, ViewMode, ViewerSection, WorkflowEntry,
};
use periplon_sdk::tui::theme::Theme;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_app_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.workflow_dir, PathBuf::from("."));
    assert!(config.workflow.is_none());
    assert!(!config.readonly);
    assert_eq!(config.theme, "dark");
    assert!(!config.debug);
    assert_eq!(config.tick_rate, 250);
}

#[test]
fn test_app_config_custom() {
    let config = AppConfig {
        workflow_dir: PathBuf::from("/tmp/workflows"),
        workflow: Some(PathBuf::from("test.yaml")),
        readonly: true,
        theme: "light".to_string(),
        state_dir: Some(PathBuf::from("/tmp/state")),
        debug: true,
        tick_rate: 100,
    };

    assert_eq!(config.workflow_dir, PathBuf::from("/tmp/workflows"));
    assert_eq!(config.workflow, Some(PathBuf::from("test.yaml")));
    assert!(config.readonly);
    assert_eq!(config.theme, "light");
    assert!(config.debug);
    assert_eq!(config.tick_rate, 100);
}

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
    assert_eq!(entry.path, PathBuf::from("/workflows/test.yaml"));
    assert_eq!(entry.description, Some("Test workflow".to_string()));
    assert_eq!(entry.version, Some("1.0.0".to_string()));
    assert!(entry.valid);
    assert!(entry.errors.is_empty());
}

#[test]
fn test_workflow_entry_with_errors() {
    let entry = WorkflowEntry {
        name: "invalid.yaml".to_string(),
        path: PathBuf::from("/workflows/invalid.yaml"),
        description: None,
        version: None,
        valid: false,
        errors: vec![
            "Missing required field: agents".to_string(),
            "Invalid task reference".to_string(),
        ],
    };

    assert!(!entry.valid);
    assert_eq!(entry.errors.len(), 2);
    assert!(entry
        .errors
        .contains(&"Missing required field: agents".to_string()));
}

#[test]
fn test_execution_state_creation() {
    let state = ExecutionState {
        workflow_path: PathBuf::from("test.yaml"),
        status: ExecutionStatus::Running,
        current_agent: Some("researcher".to_string()),
        current_task: Some("analyze".to_string()),
        progress: 0.5,
        log: vec!["Starting execution".to_string()],
        completed_tasks: vec![],
        failed_tasks: vec![],
        started_at: std::time::Instant::now(),
    };

    assert_eq!(state.workflow_path, PathBuf::from("test.yaml"));
    assert_eq!(state.status, ExecutionStatus::Running);
    assert_eq!(state.current_agent, Some("researcher".to_string()));
    assert_eq!(state.progress, 0.5);
    assert_eq!(state.log.len(), 1);
    assert!(state.completed_tasks.is_empty());
    assert!(state.failed_tasks.is_empty());
}

#[test]
fn test_execution_status_transitions() {
    let statuses = [ExecutionStatus::Preparing,
        ExecutionStatus::Running,
        ExecutionStatus::Paused,
        ExecutionStatus::Completed,
        ExecutionStatus::Failed];

    // Verify all statuses are distinct from each other
    assert_eq!(statuses.len(), 5);
    assert_ne!(ExecutionStatus::Preparing, ExecutionStatus::Running);
    assert_ne!(ExecutionStatus::Running, ExecutionStatus::Completed);
    assert_ne!(ExecutionStatus::Completed, ExecutionStatus::Failed);
}

#[test]
fn test_modal_variants() {
    let confirm = Modal::Confirm {
        title: "Delete".to_string(),
        message: "Are you sure?".to_string(),
        action: ConfirmAction::Exit,
    };

    if let Modal::Confirm { action, .. } = confirm {
        assert_eq!(action, ConfirmAction::Exit);
    } else {
        panic!("Expected Confirm modal");
    }

    let input = Modal::Input {
        title: "New Workflow".to_string(),
        prompt: "Enter name:".to_string(),
        default: "workflow.yaml".to_string(),
        action: InputAction::CreateWorkflow,
    };

    if let Modal::Input { action, .. } = input {
        assert_eq!(action, InputAction::CreateWorkflow);
    } else {
        panic!("Expected Input modal");
    }

    let error = Modal::Error {
        title: "Error".to_string(),
        message: "File not found".to_string(),
    };

    assert!(matches!(error, Modal::Error { .. }));
}

#[test]
fn test_editor_error_creation() {
    let error = EditorError {
        line: 42,
        column: Some(10),
        message: "Unexpected token".to_string(),
        severity: ErrorSeverity::Error,
    };

    assert_eq!(error.line, 42);
    assert_eq!(error.column, Some(10));
    assert_eq!(error.severity, ErrorSeverity::Error);
}

#[test]
fn test_error_severity_levels() {
    let severities = [ErrorSeverity::Error,
        ErrorSeverity::Warning,
        ErrorSeverity::Info];

    assert_eq!(severities.len(), 3);
    assert_ne!(ErrorSeverity::Error, ErrorSeverity::Warning);
}

#[test]
fn test_viewer_section_navigation() {
    let sections = [ViewerSection::Overview,
        ViewerSection::Agents,
        ViewerSection::Tasks,
        ViewerSection::Variables];

    assert_eq!(sections[0], ViewerSection::Overview);
    assert_eq!(sections[1], ViewerSection::Agents);
}

#[test]
fn test_theme_consistency() {
    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        // All themes should have all colors defined
        let _ = theme.primary;
        let _ = theme.secondary;
        let _ = theme.accent;
        let _ = theme.bg;
        let _ = theme.fg;
        let _ = theme.success;
        let _ = theme.warning;
        let _ = theme.error;
        let _ = theme.muted;
        let _ = theme.border;
        let _ = theme.highlight;
    }
}

#[test]
fn test_state_workflow_list_to_viewer_transition() {
    let mut state = AppState::new();
    assert_eq!(state.view_mode, ViewMode::WorkflowList);

    state.view_mode = ViewMode::Viewer;
    assert_eq!(state.view_mode, ViewMode::Viewer);
}

#[test]
fn test_state_workflow_selection() {
    let mut state = AppState::new();
    state.workflows = vec![
        WorkflowEntry {
            name: "a.yaml".to_string(),
            path: PathBuf::from("a.yaml"),
            description: None,
            version: None,
            valid: true,
            errors: vec![],
        },
        WorkflowEntry {
            name: "b.yaml".to_string(),
            path: PathBuf::from("b.yaml"),
            description: None,
            version: None,
            valid: true,
            errors: vec![],
        },
    ];

    state.selected_workflow = 0;
    assert_eq!(state.workflows[state.selected_workflow].name, "a.yaml");

    state.selected_workflow = 1;
    assert_eq!(state.workflows[state.selected_workflow].name, "b.yaml");
}

#[test]
fn test_confirm_action_variants() {
    let actions = [ConfirmAction::DeleteWorkflow(PathBuf::from("test.yaml")),
        ConfirmAction::ExecuteWorkflow(PathBuf::from("test.yaml")),
        ConfirmAction::DiscardChanges,
        ConfirmAction::Exit];

    assert_eq!(actions.len(), 4);
}

#[test]
fn test_input_action_variants() {
    let actions = [InputAction::CreateWorkflow,
        InputAction::RenameWorkflow(PathBuf::from("old.yaml")),
        InputAction::GenerateWorkflow];

    assert_eq!(actions.len(), 3);
}

#[test]
fn test_workflow_filtering_with_multiple_criteria() {
    let mut state = AppState::new();
    state.workflows = vec![
        WorkflowEntry {
            name: "database_migration.yaml".to_string(),
            path: PathBuf::from("database_migration.yaml"),
            description: Some("Migrate database schema".to_string()),
            version: None,
            valid: true,
            errors: vec![],
        },
        WorkflowEntry {
            name: "api_test.yaml".to_string(),
            path: PathBuf::from("api_test.yaml"),
            description: Some("Test API endpoints".to_string()),
            version: None,
            valid: true,
            errors: vec![],
        },
        WorkflowEntry {
            name: "data_pipeline.yaml".to_string(),
            path: PathBuf::from("data_pipeline.yaml"),
            description: Some("Process data batches".to_string()),
            version: None,
            valid: true,
            errors: vec![],
        },
    ];

    // Test filtering by name
    state.search_query = "api".to_string();
    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "api_test.yaml");

    // Test filtering by description
    state.search_query = "migrate".to_string();
    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "database_migration.yaml");

    // Test partial match
    state.search_query = "data".to_string();
    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 2); // Matches both "data_pipeline" and description "Process data batches"
}

#[test]
fn test_execution_progress_tracking() {
    let mut exec_state = ExecutionState {
        workflow_path: PathBuf::from("test.yaml"),
        status: ExecutionStatus::Running,
        current_agent: None,
        current_task: None,
        progress: 0.0,
        log: vec![],
        completed_tasks: vec![],
        failed_tasks: vec![],
        started_at: std::time::Instant::now(),
    };

    // Simulate progress
    exec_state.progress = 0.25;
    exec_state.current_agent = Some("researcher".to_string());
    exec_state.log.push("Agent started".to_string());

    assert_eq!(exec_state.progress, 0.25);
    assert_eq!(exec_state.current_agent, Some("researcher".to_string()));
    assert_eq!(exec_state.log.len(), 1);

    // Continue progress
    exec_state.progress = 0.75;
    exec_state.current_task = Some("analyze".to_string());
    exec_state.log.push("Task started".to_string());

    assert_eq!(exec_state.progress, 0.75);
    assert_eq!(exec_state.log.len(), 2);

    // Complete
    exec_state.status = ExecutionStatus::Completed;
    exec_state.progress = 1.0;

    assert_eq!(exec_state.status, ExecutionStatus::Completed);
    assert_eq!(exec_state.progress, 1.0);
}

#[test]
fn test_viewer_state_section_switching() {
    let mut state = AppState::new();

    assert_eq!(state.viewer_state.section, ViewerSection::Overview);

    state.viewer_state.section = ViewerSection::Agents;
    assert_eq!(state.viewer_state.section, ViewerSection::Agents);

    state.viewer_state.section = ViewerSection::Tasks;
    assert_eq!(state.viewer_state.section, ViewerSection::Tasks);

    state.viewer_state.section = ViewerSection::Variables;
    assert_eq!(state.viewer_state.section, ViewerSection::Variables);
}

#[test]
fn test_viewer_state_expansion() {
    let mut state = AppState::new();

    assert!(state.viewer_state.expanded.is_empty());

    state.viewer_state.expanded.push("agent1".to_string());
    state.viewer_state.expanded.push("task1".to_string());

    assert_eq!(state.viewer_state.expanded.len(), 2);
    assert!(state.viewer_state.expanded.contains(&"agent1".to_string()));
}

#[test]
fn test_editor_state_modifications() {
    let mut state = AppState::new();

    assert!(!state.editor_state.modified);

    state.editor_state.modified = true;
    assert!(state.editor_state.modified);

    state.editor_state.cursor = (10, 5);
    assert_eq!(state.editor_state.cursor, (10, 5));

    state.editor_state.scroll = (0, 10);
    assert_eq!(state.editor_state.scroll, (0, 10));
}

#[test]
fn test_editor_state_error_tracking() {
    let mut state = AppState::new();

    assert!(state.editor_state.errors.is_empty());

    state.editor_state.errors.push(EditorError {
        line: 5,
        column: Some(10),
        message: "Syntax error".to_string(),
        severity: ErrorSeverity::Error,
    });

    state.editor_state.errors.push(EditorError {
        line: 8,
        column: None,
        message: "Deprecated syntax".to_string(),
        severity: ErrorSeverity::Warning,
    });

    assert_eq!(state.editor_state.errors.len(), 2);
    assert_eq!(state.editor_state.errors[0].severity, ErrorSeverity::Error);
    assert_eq!(
        state.editor_state.errors[1].severity,
        ErrorSeverity::Warning
    );
}

#[test]
fn test_modal_state_transitions() {
    let mut state = AppState::new();

    assert!(!state.has_modal());

    // Show error modal
    state.modal = Some(Modal::Error {
        title: "Error".to_string(),
        message: "Something went wrong".to_string(),
    });

    assert!(state.has_modal());

    // Close modal
    state.close_modal();

    assert!(!state.has_modal());

    // Show confirmation modal
    state.modal = Some(Modal::Confirm {
        title: "Confirm".to_string(),
        message: "Delete workflow?".to_string(),
        action: ConfirmAction::DeleteWorkflow(PathBuf::from("test.yaml")),
    });

    assert!(state.has_modal());
}

#[test]
fn test_state_reset_clears_all_fields() {
    let mut state = AppState::new();

    // Modify state
    state.view_mode = ViewMode::Editor;
    state.search_query = "test".to_string();
    state.selected_workflow = 5;
    state.modal = Some(Modal::Success {
        title: "Done".to_string(),
        message: "Complete".to_string(),
    });
    state.running = false;

    // Reset
    state.reset();

    // Verify all fields are reset
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.search_query, "");
    assert_eq!(state.selected_workflow, 0);
    assert!(state.modal.is_none());
    assert!(state.running);
}

#[test]
fn test_workflow_entry_equality() {
    let entry1 = WorkflowEntry {
        name: "test.yaml".to_string(),
        path: PathBuf::from("test.yaml"),
        description: Some("Test".to_string()),
        version: Some("1.0.0".to_string()),
        valid: true,
        errors: vec![],
    };

    let entry2 = WorkflowEntry {
        name: "test.yaml".to_string(),
        path: PathBuf::from("test.yaml"),
        description: Some("Test".to_string()),
        version: Some("1.0.0".to_string()),
        valid: true,
        errors: vec![],
    };

    assert_eq!(entry1, entry2);
}

#[cfg(feature = "tui")]
#[tokio::test]
async fn test_workflow_parsing_integration() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("test_workflow.yaml");

    let workflow_yaml = r#"
name: "Test Workflow"
version: "1.0.0"
description: "A test workflow for integration testing"

agents:
  test_agent:
    description: "Test agent"
    tools: [Read, Write]
    permissions:
      mode: "acceptEdits"

tasks:
  test_task:
    description: "Test task"
    agent: "test_agent"
"#;

    std::fs::write(&workflow_path, workflow_yaml).unwrap();

    // Parse workflow
    let workflow = parse_workflow_file(&workflow_path).unwrap();

    assert_eq!(workflow.name, "Test Workflow");
    assert_eq!(workflow.version, "1.0.0");
    assert!(workflow.agents.contains_key("test_agent"));
    assert!(workflow.tasks.contains_key("test_task"));

    // Validate workflow
    let validation_result = validate_workflow(&workflow);
    assert!(validation_result.is_ok(), "Workflow should be valid");
}

#[test]
fn test_multiple_workflows_management() {
    let mut state = AppState::new();

    // Add multiple workflows
    for i in 1..=5 {
        state.workflows.push(WorkflowEntry {
            name: format!("workflow{}.yaml", i),
            path: PathBuf::from(format!("workflow{}.yaml", i)),
            description: Some(format!("Workflow {}", i)),
            version: Some("1.0.0".to_string()),
            valid: true,
            errors: vec![],
        });
    }

    assert_eq!(state.workflows.len(), 5);

    // Test navigation
    state.selected_workflow = 0;
    assert_eq!(
        state.workflows[state.selected_workflow].name,
        "workflow1.yaml"
    );

    state.selected_workflow = 4;
    assert_eq!(
        state.workflows[state.selected_workflow].name,
        "workflow5.yaml"
    );
}

// ============================================================================
// Advanced Integration Tests
// ============================================================================

#[tokio::test]
async fn test_workflow_lifecycle_full_cycle() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("lifecycle.yaml");

    // Create workflow
    let workflow_yaml = r#"
name: "Lifecycle Test"
version: "1.0.0"
agents:
  test_agent:
    description: "Test agent"
    tools: [Read, Write]
tasks:
  test_task:
    description: "Test task"
    agent: "test_agent"
"#;

    std::fs::write(&workflow_path, workflow_yaml).unwrap();

    // Parse
    let workflow = parse_workflow_file(&workflow_path).unwrap();
    assert_eq!(workflow.name, "Lifecycle Test");

    // Validate
    let result = validate_workflow(&workflow);
    assert!(result.is_ok());

    // Create state
    let mut state = AppState::new();
    state.workflows.push(WorkflowEntry {
        name: "lifecycle.yaml".to_string(),
        path: workflow_path.clone(),
        description: Some("Lifecycle Test".to_string()),
        version: Some("1.0.0".to_string()),
        valid: true,
        errors: vec![],
    });

    assert_eq!(state.workflows.len(), 1);
}

#[test]
fn test_view_mode_transitions_all_combinations() {
    let mut state = AppState::new();

    let view_modes = vec![
        ViewMode::WorkflowList,
        ViewMode::Viewer,
        ViewMode::Editor,
        ViewMode::ExecutionMonitor,
        ViewMode::StateBrowser,
        ViewMode::Generator,
        ViewMode::Help,
    ];

    for mode in view_modes {
        state.view_mode = mode;
        assert_eq!(state.view_mode, mode);
    }
}

#[test]
fn test_error_severity_ordering() {
    let errors = [EditorError {
            line: 1,
            column: None,
            message: "Info".to_string(),
            severity: ErrorSeverity::Info,
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
            message: "Error".to_string(),
            severity: ErrorSeverity::Error,
        }];

    // Errors should be prioritized highest
    let critical: Vec<_> = errors
        .iter()
        .filter(|e| e.severity == ErrorSeverity::Error)
        .collect();

    assert_eq!(critical.len(), 1);
    assert_eq!(critical[0].message, "Error");
}

#[tokio::test]
async fn test_workflow_with_multiple_agents() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("multi_agent.yaml");

    let workflow_yaml = r#"
name: "Multi-Agent Workflow"
version: "1.0.0"
agents:
  researcher:
    description: "Research agent"
    tools: [Read, WebSearch]
  writer:
    description: "Writing agent"
    tools: [Write, Edit]
  reviewer:
    description: "Review agent"
    tools: [Read]
tasks:
  research:
    description: "Research topic"
    agent: "researcher"
  write:
    description: "Write content"
    agent: "writer"
    depends_on: [research]
  review:
    description: "Review content"
    agent: "reviewer"
    depends_on: [write]
"#;

    std::fs::write(&workflow_path, workflow_yaml).unwrap();

    let workflow = parse_workflow_file(&workflow_path).unwrap();

    assert_eq!(workflow.agents.len(), 3);
    assert_eq!(workflow.tasks.len(), 3);

    // Verify dependencies
    let write_task = workflow.tasks.get("write").unwrap();
    assert!(!write_task.depends_on.is_empty());
    assert_eq!(write_task.depends_on, vec!["research".to_string()]);
}

#[test]
fn test_modal_state_multiple_transitions() {
    let mut state = AppState::new();

    // No modal
    assert!(!state.has_modal());

    // Show error modal
    state.modal = Some(Modal::Error {
        title: "Error".to_string(),
        message: "Test error".to_string(),
    });
    assert!(state.has_modal());

    // Close and show success
    state.close_modal();
    state.modal = Some(Modal::Success {
        title: "Success".to_string(),
        message: "Test success".to_string(),
    });
    assert!(state.has_modal());

    // Close all
    state.close_modal();
    assert!(!state.has_modal());
}

#[test]
fn test_workflow_search_and_filter_complex() {
    let mut state = AppState::new();

    state.workflows = vec![
        WorkflowEntry {
            name: "api_test_v1.yaml".to_string(),
            path: PathBuf::from("api_test_v1.yaml"),
            description: Some("API testing workflow version 1".to_string()),
            version: Some("1.0.0".to_string()),
            valid: true,
            errors: vec![],
        },
        WorkflowEntry {
            name: "api_test_v2.yaml".to_string(),
            path: PathBuf::from("api_test_v2.yaml"),
            description: Some("API testing workflow version 2".to_string()),
            version: Some("2.0.0".to_string()),
            valid: true,
            errors: vec![],
        },
        WorkflowEntry {
            name: "database_migration.yaml".to_string(),
            path: PathBuf::from("database_migration.yaml"),
            description: Some("Database schema migration".to_string()),
            version: Some("1.0.0".to_string()),
            valid: true,
            errors: vec![],
        },
        WorkflowEntry {
            name: "integration_test.yaml".to_string(),
            path: PathBuf::from("integration_test.yaml"),
            description: Some("Integration testing suite".to_string()),
            version: Some("1.0.0".to_string()),
            valid: false,
            errors: vec!["Missing agent reference".to_string()],
        },
    ];

    // Filter by "api"
    state.search_query = "api".to_string();
    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 2);

    // Filter by "test"
    state.search_query = "test".to_string();
    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 3); // api_test_v1, api_test_v2, integration_test

    // Filter by "migration"
    state.search_query = "migration".to_string();
    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 1);

    // Check validity filter
    let valid_workflows: Vec<_> = state.workflows.iter().filter(|w| w.valid).collect();
    let invalid_workflows: Vec<_> = state.workflows.iter().filter(|w| !w.valid).collect();

    assert_eq!(valid_workflows.len(), 3);
    assert_eq!(invalid_workflows.len(), 1);
}

#[test]
fn test_execution_state_detailed_tracking() {
    let mut exec = ExecutionState {
        workflow_path: PathBuf::from("detailed.yaml"),
        status: ExecutionStatus::Running,
        current_agent: None,
        current_task: None,
        progress: 0.0,
        log: Vec::new(),
        completed_tasks: Vec::new(),
        failed_tasks: Vec::new(),
        started_at: std::time::Instant::now(),
    };

    // Simulate detailed execution tracking
    exec.log.push("[00:00] Workflow started".to_string());
    exec.current_agent = Some("agent1".to_string());
    exec.current_task = Some("task1".to_string());
    exec.progress = 0.2;

    exec.log.push("[00:10] Agent1 started task1".to_string());
    exec.progress = 0.4;

    exec.log
        .push("[00:20] Task1 completed successfully".to_string());
    exec.completed_tasks.push("task1".to_string());
    exec.progress = 0.6;

    exec.current_task = Some("task2".to_string());
    exec.log.push("[00:30] Agent1 started task2".to_string());
    exec.progress = 0.8;

    exec.log
        .push("[00:40] Task2 completed successfully".to_string());
    exec.completed_tasks.push("task2".to_string());
    exec.progress = 1.0;

    exec.status = ExecutionStatus::Completed;
    exec.log.push("[00:50] Workflow completed".to_string());

    assert_eq!(exec.status, ExecutionStatus::Completed);
    assert_eq!(exec.completed_tasks.len(), 2);
    assert_eq!(exec.failed_tasks.len(), 0);
    assert_eq!(exec.progress, 1.0);
    assert!(exec.log.len() >= 6);
}

#[tokio::test]
async fn test_workflow_with_inputs_and_outputs() {
    let temp_dir = TempDir::new().unwrap();
    let workflow_path = temp_dir.path().join("io_workflow.yaml");

    let workflow_yaml = r#"
name: "Input/Output Test"
version: "1.0.0"
inputs:
  project_name:
    type: string
    required: true
    default: "MyProject"
  api_key:
    type: string
    required: true
agents:
  processor:
    description: "Process with inputs"
    tools: [Read, Write]
tasks:
  process:
    description: "Process ${workflow.project_name}"
    agent: "processor"
    outputs:
      result:
        source:
          type: file
          path: "./result.json"
"#;

    std::fs::write(&workflow_path, workflow_yaml).unwrap();

    let workflow = parse_workflow_file(&workflow_path).unwrap();

    assert_eq!(workflow.inputs.len(), 2);
    assert!(workflow.inputs.contains_key("project_name"));
    assert!(workflow.inputs.contains_key("api_key"));

    let task = workflow.tasks.get("process").unwrap();
    assert!(!task.outputs.is_empty());
}

#[test]
fn test_app_state_comprehensive_reset() {
    let mut state = AppState::new();

    // Set up complex state
    state.view_mode = ViewMode::Editor;
    state.search_query = "test".to_string();
    state.selected_workflow = 5;
    state.workflows.push(WorkflowEntry {
        name: "test.yaml".to_string(),
        path: PathBuf::from("test.yaml"),
        description: None,
        version: None,
        valid: true,
        errors: vec![],
    });
    state.modal = Some(Modal::Error {
        title: "Error".to_string(),
        message: "Test".to_string(),
    });
    state.viewer_state.scroll = 100;
    state.editor_state.modified = true;

    // Reset
    state.reset();

    // Verify everything is reset
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.search_query, "");
    assert_eq!(state.selected_workflow, 0);
    assert!(state.modal.is_none());
    assert_eq!(state.viewer_state.scroll, 0);
    assert!(!state.editor_state.modified);
}

#[test]
fn test_theme_color_consistency() {
    let themes = vec![
        ("dark", Theme::default()),
        ("light", Theme::light()),
        ("monokai", Theme::monokai()),
        ("solarized", Theme::solarized()),
    ];

    for (_name, theme) in themes {
        // Verify all required colors are defined (just check they exist)
        let _ = theme.primary;
        let _ = theme.success;
        let _ = theme.error;

        // Colors should be distinct
        assert_ne!(
            theme.success, theme.error,
            "Theme has same success and error colors"
        );
    }
}

#[test]
fn test_concurrent_workflow_execution_states() {
    let workflows = [("workflow1.yaml", ExecutionStatus::Running),
        ("workflow2.yaml", ExecutionStatus::Completed),
        ("workflow3.yaml", ExecutionStatus::Failed)];

    let states: Vec<ExecutionState> = workflows
        .iter()
        .map(|(path, status)| ExecutionState {
            workflow_path: PathBuf::from(path),
            status: *status,
            current_agent: None,
            current_task: None,
            progress: 0.0,
            log: Vec::new(),
            completed_tasks: Vec::new(),
            failed_tasks: Vec::new(),
            started_at: std::time::Instant::now(),
        })
        .collect();

    assert_eq!(states.len(), 3);

    let running = states
        .iter()
        .filter(|s| s.status == ExecutionStatus::Running)
        .count();
    let completed = states
        .iter()
        .filter(|s| s.status == ExecutionStatus::Completed)
        .count();
    let failed = states
        .iter()
        .filter(|s| s.status == ExecutionStatus::Failed)
        .count();

    assert_eq!(running, 1);
    assert_eq!(completed, 1);
    assert_eq!(failed, 1);
}
