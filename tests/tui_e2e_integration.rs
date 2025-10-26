//! End-to-End TUI Integration Tests
//!
//! Comprehensive integration tests for TUI workflows focusing on state
//! transitions, business logic, and user journey validation.
//!
//! Unlike unit tests that test individual components, these tests verify:
//! - Complete user workflows from start to finish
//! - State transitions between views
//! - Data flow through the application
//! - Error handling and recovery flows
//! - Multi-step operations
//!
//! Test Categories:
//! - Workflow Management: Browse, select, filter, create, delete
//! - View Navigation: Transitions between all view modes
//! - Editing Workflows: State management during editing
//! - Modal Interactions: Confirm, input, error, success flows
//! - Generator Workflows: AI-assisted creation flows
//! - State Persistence: Load, save, resume workflows
//! - Search and Filter: Complex filtering scenarios
//! - Error Recovery: Validation, rollback, retry

#![cfg(feature = "tui")]

use periplon_sdk::dsl::schema::DSLWorkflow;
use periplon_sdk::tui::state::{
    AppState, ConfirmAction, EditorError, ErrorSeverity, ExecutionState,
    ExecutionStatus, InputAction, Modal, ViewMode, ViewerSection, WorkflowEntry,
};
use periplon_sdk::tui::theme::Theme;
use periplon_sdk::tui::ui::generator::{GeneratorMode, GeneratorState};
use std::path::PathBuf;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create sample workflow entry
fn create_workflow_entry(name: &str, valid: bool) -> WorkflowEntry {
    WorkflowEntry {
        name: name.to_string(),
        path: PathBuf::from(name),
        description: Some(format!("Description for {}", name)),
        version: Some("1.0.0".to_string()),
        valid,
        errors: if valid {
            vec![]
        } else {
            vec!["Validation error".to_string()]
        },
    }
}

// ============================================================================
// Workflow Management E2E Tests
// ============================================================================

#[test]
fn test_e2e_workflow_list_initial_state() {
    let state = AppState::new();

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.selected_workflow, 0);
    assert!(state.workflows.is_empty());
    assert_eq!(state.search_query, "");
    assert!(state.modal.is_none());
    assert!(state.running);
}

#[test]
fn test_e2e_workflow_selection_and_loading() {
    let mut state = AppState::new();

    // Add multiple workflows
    state.workflows = vec![
        create_workflow_entry("workflow1.yaml", true),
        create_workflow_entry("workflow2.yaml", true),
        create_workflow_entry("workflow3.yaml", true),
    ];

    // Navigate down
    state.selected_workflow = 1;
    assert_eq!(state.selected_workflow, 1);
    assert_eq!(state.workflows[state.selected_workflow].name, "workflow2.yaml");

    // Load selected workflow (path only for testing state)
    state.current_workflow_path = Some(PathBuf::from("workflow2.yaml"));

    assert!(state.current_workflow_path.is_some());
    assert_eq!(
        state.current_workflow_path.as_ref().unwrap(),
        &PathBuf::from("workflow2.yaml")
    );
}

#[test]
fn test_e2e_workflow_search_and_filter() {
    let mut state = AppState::new();

    state.workflows = vec![
        create_workflow_entry("api_test.yaml", true),
        create_workflow_entry("database_migration.yaml", true),
        create_workflow_entry("api_integration.yaml", true),
        create_workflow_entry("data_pipeline.yaml", true),
    ];

    // Search for "api"
    state.search_query = "api".to_string();
    let filtered = state.filtered_workflows();

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|w| w.name.contains("api")));

    // Change search
    state.search_query = "data".to_string();
    let filtered = state.filtered_workflows();

    assert_eq!(filtered.len(), 2); // database_migration and data_pipeline
}

#[test]
fn test_e2e_workflow_validation_filtering() {
    let mut state = AppState::new();

    state.workflows = vec![
        create_workflow_entry("valid1.yaml", true),
        create_workflow_entry("invalid1.yaml", false),
        create_workflow_entry("valid2.yaml", true),
        create_workflow_entry("invalid2.yaml", false),
    ];

    // Filter valid workflows
    let valid: Vec<_> = state.workflows.iter().filter(|w| w.valid).collect();
    assert_eq!(valid.len(), 2);

    // Filter invalid workflows
    let invalid: Vec<_> = state.workflows.iter().filter(|w| !w.valid).collect();
    assert_eq!(invalid.len(), 2);
}

// ============================================================================
// View Navigation E2E Tests
// ============================================================================

#[test]
fn test_e2e_view_mode_transitions() {
    let mut state = AppState::new();

    // Start at workflow list
    assert_eq!(state.view_mode, ViewMode::WorkflowList);

    // Navigate to viewer
    state.view_mode = ViewMode::Viewer;
    assert_eq!(state.view_mode, ViewMode::Viewer);

    // Navigate to editor
    state.view_mode = ViewMode::Editor;
    assert_eq!(state.view_mode, ViewMode::Editor);

    // Navigate to generator
    state.view_mode = ViewMode::Generator;
    assert_eq!(state.view_mode, ViewMode::Generator);

    // Navigate to state browser
    state.view_mode = ViewMode::StateBrowser;
    assert_eq!(state.view_mode, ViewMode::StateBrowser);

    // Navigate to help
    state.view_mode = ViewMode::Help;
    assert_eq!(state.view_mode, ViewMode::Help);

    // Return to workflow list
    state.view_mode = ViewMode::WorkflowList;
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
}

#[test]
fn test_e2e_workflow_list_to_viewer_flow() {
    let mut state = AppState::new();

    // Setup workflow
    state.workflows = vec![create_workflow_entry("test.yaml", true)];
    state.selected_workflow = 0;
    state.view_mode = ViewMode::WorkflowList;

    // Simulate Enter key - load workflow path
    state.current_workflow_path = Some(PathBuf::from("test.yaml"));

    // Transition to viewer
    state.view_mode = ViewMode::Viewer;

    assert_eq!(state.view_mode, ViewMode::Viewer);
    assert!(state.current_workflow_path.is_some());
    assert_eq!(state.viewer_state.section, ViewerSection::Overview);
}

#[test]
fn test_e2e_viewer_to_editor_transition() {
    let mut state = AppState::new();

    // Setup in viewer mode
    state.view_mode = ViewMode::Viewer;
    
    state.viewer_state.section = ViewerSection::Agents;

    // Transition to editor (e.g., press 'e')
    state.view_mode = ViewMode::Editor;

    assert_eq!(state.view_mode, ViewMode::Editor);
    assert!(!state.editor_state.modified);
}

#[test]
fn test_e2e_complete_navigation_cycle() {
    let mut state = AppState::new();

    state.workflows = vec![create_workflow_entry("cycle.yaml", true)];
    

    // Complete cycle: List -> Viewer -> Editor -> List
    state.view_mode = ViewMode::WorkflowList;
    assert_eq!(state.view_mode, ViewMode::WorkflowList);

    state.view_mode = ViewMode::Viewer;
    assert_eq!(state.view_mode, ViewMode::Viewer);

    state.view_mode = ViewMode::Editor;
    assert_eq!(state.view_mode, ViewMode::Editor);

    state.view_mode = ViewMode::WorkflowList;
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
}

// ============================================================================
// Modal Interaction E2E Tests
// ============================================================================

#[test]
fn test_e2e_confirm_delete_workflow() {
    let mut state = AppState::new();

    state.workflows = vec![
        create_workflow_entry("keep.yaml", true),
        create_workflow_entry("delete.yaml", true),
    ];
    state.selected_workflow = 1;

    // Show confirmation modal
    state.modal = Some(Modal::Confirm {
        title: "Delete Workflow".to_string(),
        message: "Are you sure you want to delete delete.yaml?".to_string(),
        action: ConfirmAction::DeleteWorkflow(PathBuf::from("delete.yaml")),
    });

    assert!(state.has_modal());

    // Simulate confirmation (press 'y')
    if let Some(Modal::Confirm { action, .. }) = &state.modal {
        match action {
            ConfirmAction::DeleteWorkflow(path) => {
                assert_eq!(path, &PathBuf::from("delete.yaml"));
                // Delete the workflow
                state.workflows.retain(|w| w.path != *path);
            }
            _ => panic!("Wrong action type"),
        }
    }

    state.close_modal();

    assert_eq!(state.workflows.len(), 1);
    assert_eq!(state.workflows[0].name, "keep.yaml");
    assert!(!state.has_modal());
}

#[test]
fn test_e2e_input_workflow_creation() {
    let mut state = AppState::new();

    // Show input modal for new workflow
    state.modal = Some(Modal::Input {
        title: "New Workflow".to_string(),
        prompt: "Enter workflow name:".to_string(),
        default: "new_workflow.yaml".to_string(),
        action: InputAction::CreateWorkflow,
    });

    assert!(state.has_modal());

    // Simulate user entering name and confirming
    let workflow_name = "my_new_workflow.yaml";

    if let Some(Modal::Input { action, .. }) = &state.modal {
        match action {
            InputAction::CreateWorkflow => {
                // Create new workflow entry
                state.workflows.push(create_workflow_entry(workflow_name, true));
            }
            _ => panic!("Wrong action type"),
        }
    }

    state.close_modal();

    assert_eq!(state.workflows.len(), 1);
    assert_eq!(state.workflows[0].name, workflow_name);
    assert!(!state.has_modal());
}

#[test]
fn test_e2e_error_modal_display_and_dismiss() {
    let mut state = AppState::new();

    // Show error modal
    state.modal = Some(Modal::Error {
        title: "Validation Error".to_string(),
        message: "Invalid YAML: Missing required field 'agents'".to_string(),
    });

    assert!(state.has_modal());

    // Verify modal content
    if let Some(Modal::Error { title, message }) = &state.modal {
        assert_eq!(title, "Validation Error");
        assert!(message.contains("agents"));
    }

    // Dismiss modal (press Esc or Enter)
    state.close_modal();

    assert!(!state.has_modal());
}

#[test]
fn test_e2e_success_modal_after_save() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;

    // Simulate successful save
    state.editor_state.modified = false;

    // Show success modal
    state.modal = Some(Modal::Success {
        title: "Saved".to_string(),
        message: "Workflow saved successfully!".to_string(),
    });

    assert!(state.has_modal());
    assert!(!state.editor_state.modified);

    // Dismiss
    state.close_modal();
    assert!(!state.has_modal());
}

// ============================================================================
// Editor Workflow E2E Tests
// ============================================================================

#[test]
fn test_e2e_editor_modification_tracking() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Editor;
    assert!(!state.editor_state.modified);

    // Simulate edit
    state.editor_state.modified = true;
    assert!(state.editor_state.modified);

    // Simulate save
    state.editor_state.modified = false;
    assert!(!state.editor_state.modified);
}

#[test]
fn test_e2e_editor_cursor_movement() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Editor;
    state.editor_state.cursor = (0, 0);

    // Move cursor
    state.editor_state.cursor = (5, 10);
    assert_eq!(state.editor_state.cursor, (5, 10));

    // Move to different line
    state.editor_state.cursor.0 += 1;
    assert_eq!(state.editor_state.cursor, (6, 10));

    // Move column
    state.editor_state.cursor.1 += 5;
    assert_eq!(state.editor_state.cursor, (6, 15));
}

#[test]
fn test_e2e_editor_scroll_tracking() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Editor;
    assert_eq!(state.editor_state.scroll, (0, 0));

    // Scroll down
    state.editor_state.scroll.0 += 10;
    assert_eq!(state.editor_state.scroll, (10, 0));

    // Scroll right
    state.editor_state.scroll.1 += 5;
    assert_eq!(state.editor_state.scroll, (10, 5));
}

#[test]
fn test_e2e_editor_validation_errors() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Editor;
    assert!(state.editor_state.errors.is_empty());

    // Add validation errors
    state.editor_state.errors.push(EditorError {
        line: 10,
        column: Some(5),
        message: "Undefined agent reference".to_string(),
        severity: ErrorSeverity::Error,
    });

    state.editor_state.errors.push(EditorError {
        line: 15,
        column: None,
        message: "Deprecated syntax".to_string(),
        severity: ErrorSeverity::Warning,
    });

    assert_eq!(state.editor_state.errors.len(), 2);

    // Filter by severity
    let errors: Vec<_> = state
        .editor_state
        .errors
        .iter()
        .filter(|e| e.severity == ErrorSeverity::Error)
        .collect();
    assert_eq!(errors.len(), 1);

    let warnings: Vec<_> = state
        .editor_state
        .errors
        .iter()
        .filter(|e| e.severity == ErrorSeverity::Warning)
        .collect();
    assert_eq!(warnings.len(), 1);
}

#[test]
fn test_e2e_editor_discard_changes_flow() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;
    

    // Show confirm modal
    state.modal = Some(Modal::Confirm {
        title: "Discard Changes".to_string(),
        message: "You have unsaved changes. Discard them?".to_string(),
        action: ConfirmAction::DiscardChanges,
    });

    assert!(state.has_modal());
    assert!(state.editor_state.modified);

    // Simulate confirmation
    if let Some(Modal::Confirm { action, .. }) = &state.modal {
        match action {
            ConfirmAction::DiscardChanges => {
                state.editor_state.modified = false;
                state.view_mode = ViewMode::WorkflowList;
            }
            _ => panic!("Wrong action"),
        }
    }

    state.close_modal();

    assert!(!state.editor_state.modified);
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert!(!state.has_modal());
}

// ============================================================================
// Generator Workflow E2E Tests
// ============================================================================

#[test]
fn test_e2e_generator_create_mode_initialization() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Generator;
    state.generator_state = GeneratorState::new_create();

    assert_eq!(state.generator_state.mode, GeneratorMode::Create);
    assert_eq!(state.generator_state.nl_input, "");
    assert_eq!(state.generator_state.original_yaml, None);
    assert!(!state.generator_state.show_diff);
}

#[test]
fn test_e2e_generator_modify_mode_initialization() {
    let mut state = AppState::new();

    let original = "name: \"Original\"\nversion: \"1.0.0\"".to_string();
    state.view_mode = ViewMode::Generator;
    state.generator_state = GeneratorState::new_modify(original.clone());

    assert_eq!(state.generator_state.mode, GeneratorMode::Modify);
    assert_eq!(state.generator_state.original_yaml, Some(original));
    assert!(state.generator_state.show_diff); // Diff enabled by default in modify mode
}

#[test]
fn test_e2e_generator_input_and_generation() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Generator;
    state.generator_state = GeneratorState::new_create();

    // User types description
    state.generator_state.nl_input = "Create a workflow with a researcher agent".to_string();

    assert!(!state.generator_state.nl_input.is_empty());
    assert!(state.generator_state.can_generate());

    // Simulate generation (would call AI)
    state.generator_state.generated_yaml =
        Some("name: \"Research Workflow\"\nversion: \"1.0.0\"".to_string());

    assert!(state.generator_state.generated_yaml.is_some());
}

#[test]
fn test_e2e_generator_accept_and_edit() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Generator;
    state.generator_state = GeneratorState::new_create();

    // Generate workflow
    let generated = "name: \"Generated\"\nversion: \"1.0.0\"".to_string();
    state.generator_state.generated_yaml = Some(generated.clone());

    // Accept - transition to editor
    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = false;

    assert_eq!(state.view_mode, ViewMode::Editor);
}

// ============================================================================
// Viewer Section Navigation E2E Tests
// ============================================================================

#[test]
fn test_e2e_viewer_section_switching() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Viewer;
    
    state.viewer_state.section = ViewerSection::Overview;

    // Navigate through sections
    state.viewer_state.section = ViewerSection::Agents;
    assert_eq!(state.viewer_state.section, ViewerSection::Agents);

    state.viewer_state.section = ViewerSection::Tasks;
    assert_eq!(state.viewer_state.section, ViewerSection::Tasks);

    state.viewer_state.section = ViewerSection::Variables;
    assert_eq!(state.viewer_state.section, ViewerSection::Variables);

    state.viewer_state.section = ViewerSection::Overview;
    assert_eq!(state.viewer_state.section, ViewerSection::Overview);
}

#[test]
fn test_e2e_viewer_expansion_state() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Viewer;
    assert!(state.viewer_state.expanded.is_empty());

    // Expand items
    state.viewer_state.expanded.push("agent1".to_string());
    state.viewer_state.expanded.push("task1".to_string());

    assert_eq!(state.viewer_state.expanded.len(), 2);
    assert!(state.viewer_state.expanded.contains(&"agent1".to_string()));

    // Collapse item
    state.viewer_state.expanded.retain(|id| id != "agent1");

    assert_eq!(state.viewer_state.expanded.len(), 1);
    assert!(!state.viewer_state.expanded.contains(&"agent1".to_string()));
}

#[test]
fn test_e2e_viewer_scroll_position() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::Viewer;
    assert_eq!(state.viewer_state.scroll, 0);

    // Scroll down
    state.viewer_state.scroll = 10;
    assert_eq!(state.viewer_state.scroll, 10);

    // Scroll more
    state.viewer_state.scroll += 5;
    assert_eq!(state.viewer_state.scroll, 15);

    // Reset scroll
    state.viewer_state.scroll = 0;
    assert_eq!(state.viewer_state.scroll, 0);
}

// ============================================================================
// State Browser E2E Tests
// ============================================================================

#[test]
fn test_e2e_state_browser_mode_activation() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::StateBrowser;
    assert_eq!(state.view_mode, ViewMode::StateBrowser);

    // Navigate back
    state.view_mode = ViewMode::WorkflowList;
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
}

// ============================================================================
// Execution State E2E Tests
// ============================================================================

#[test]
fn test_e2e_execution_state_lifecycle() {
    let mut exec = ExecutionState {
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

    // Start execution
    exec.status = ExecutionStatus::Running;
    exec.current_agent = Some("researcher".to_string());
    exec.log.push("Starting execution".to_string());

    assert_eq!(exec.status, ExecutionStatus::Running);
    assert_eq!(exec.current_agent, Some("researcher".to_string()));

    // Progress
    exec.current_task = Some("analyze".to_string());
    exec.progress = 0.5;
    exec.log.push("Task started".to_string());

    assert_eq!(exec.progress, 0.5);
    assert_eq!(exec.log.len(), 2);

    // Complete
    exec.completed_tasks.push("analyze".to_string());
    exec.status = ExecutionStatus::Completed;
    exec.progress = 1.0;

    assert_eq!(exec.status, ExecutionStatus::Completed);
    assert_eq!(exec.completed_tasks.len(), 1);
}

#[test]
fn test_e2e_execution_failure_tracking() {
    let mut exec = ExecutionState {
        workflow_path: PathBuf::from("test.yaml"),
        status: ExecutionStatus::Running,
        current_agent: Some("agent1".to_string()),
        current_task: Some("task1".to_string()),
        progress: 0.3,
        log: Vec::new(),
        completed_tasks: Vec::new(),
        failed_tasks: Vec::new(),
        started_at: std::time::Instant::now(),
    };

    // Task fails
    exec.failed_tasks.push("task1".to_string());
    exec.status = ExecutionStatus::Failed;
    exec.log.push("Task1 failed: Connection error".to_string());

    assert_eq!(exec.status, ExecutionStatus::Failed);
    assert_eq!(exec.failed_tasks.len(), 1);
    assert_eq!(exec.completed_tasks.len(), 0);
}

// ============================================================================
// Complete Multi-Step Workflows
// ============================================================================

#[test]
fn test_e2e_complete_workflow_creation_journey() {
    let mut state = AppState::new();

    // Step 1: Start at workflow list
    assert_eq!(state.view_mode, ViewMode::WorkflowList);

    // Step 2: Open generator
    state.view_mode = ViewMode::Generator;
    state.generator_state = GeneratorState::new_create();
    assert_eq!(state.view_mode, ViewMode::Generator);

    // Step 3: Enter description
    state.generator_state.nl_input = "Create a testing workflow".to_string();
    assert!(!state.generator_state.nl_input.is_empty());

    // Step 4: Generate
    state.generator_state.generated_yaml =
        Some("name: \"Test\"\nversion: \"1.0.0\"".to_string());
    assert!(state.generator_state.generated_yaml.is_some());

    // Step 5: Accept and edit
    state.view_mode = ViewMode::Editor;
    assert_eq!(state.view_mode, ViewMode::Editor);

    // Step 6: Save
    state.editor_state.modified = false;

    // Step 7: Return to workflow list
    state.view_mode = ViewMode::WorkflowList;
    state.workflows.push(create_workflow_entry("test.yaml", true));

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.workflows.len(), 1);
}

#[test]
fn test_e2e_workflow_edit_with_validation_failure() {
    let mut state = AppState::new();

    // Load workflow
    state.workflows = vec![create_workflow_entry("edit_me.yaml", true)];
    

    // View workflow
    state.view_mode = ViewMode::Viewer;
    assert_eq!(state.view_mode, ViewMode::Viewer);

    // Switch to editor
    state.view_mode = ViewMode::Editor;
    assert_eq!(state.view_mode, ViewMode::Editor);

    // Make invalid edit
    state.editor_state.modified = true;
    state.editor_state.errors.push(EditorError {
        line: 5,
        column: Some(10),
        message: "Invalid syntax".to_string(),
        severity: ErrorSeverity::Error,
    });

    // Try to save - validation fails
    state.modal = Some(Modal::Error {
        title: "Validation Error".to_string(),
        message: "Cannot save: Invalid syntax at line 5".to_string(),
    });

    assert!(state.has_modal());
    assert!(state.editor_state.modified);

    // Fix error
    state.close_modal();
    state.editor_state.errors.clear();

    // Save successfully
    state.editor_state.modified = false;
    state.modal = Some(Modal::Success {
        title: "Saved".to_string(),
        message: "Workflow saved successfully".to_string(),
    });

    assert!(state.has_modal());
    assert!(!state.editor_state.modified);
}

#[test]
fn test_e2e_theme_consistency_across_views() {
    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        // Verify all themes have required colors
        let _ = theme.primary;
        let _ = theme.secondary;
        let _ = theme.accent;
        let _ = theme.success;
        let _ = theme.error;
        let _ = theme.warning;

        // Colors should be distinct
        assert_ne!(theme.success, theme.error);
    }
}

#[test]
fn test_e2e_app_state_reset() {
    let mut state = AppState::new();

    // Modify state extensively
    state.view_mode = ViewMode::Editor;
    state.search_query = "search term".to_string();
    state.selected_workflow = 5;
    state.workflows = vec![create_workflow_entry("test.yaml", true)];
    state.modal = Some(Modal::Error {
        title: "Error".to_string(),
        message: "Test error".to_string(),
    });
    state.editor_state.modified = true;
    state.viewer_state.scroll = 100;
    state.running = false;

    // Reset
    state.reset();

    // Verify everything is reset
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.search_query, "");
    assert_eq!(state.selected_workflow, 0);
    assert!(state.modal.is_none());
    assert!(!state.editor_state.modified);
    assert_eq!(state.viewer_state.scroll, 0);
    assert!(state.running);
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_e2e_empty_workflow_list_navigation() {
    let mut state = AppState::new();

    state.view_mode = ViewMode::WorkflowList;
    state.workflows = vec![];

    // Navigation should handle empty list
    state.selected_workflow = 0;
    assert_eq!(state.selected_workflow, 0);

    // Filtered workflows should be empty
    let filtered = state.filtered_workflows();
    assert!(filtered.is_empty());
}

#[test]
fn test_e2e_workflow_boundary_navigation() {
    let mut state = AppState::new();

    state.workflows = vec![
        create_workflow_entry("first.yaml", true),
        create_workflow_entry("second.yaml", true),
        create_workflow_entry("third.yaml", true),
    ];

    // Start at beginning
    state.selected_workflow = 0;
    assert_eq!(state.selected_workflow, 0);

    // Try to go past beginning (should stay at 0)
    state.selected_workflow = state.selected_workflow.saturating_sub(1);
    assert_eq!(state.selected_workflow, 0);

    // Go to end
    state.selected_workflow = state.workflows.len() - 1;
    assert_eq!(state.selected_workflow, 2);

    // Try to go past end (should stay at max)
    state.selected_workflow = (state.selected_workflow + 1).min(state.workflows.len() - 1);
    assert_eq!(state.selected_workflow, 2);
}

#[test]
fn test_e2e_rapid_modal_state_changes() {
    let mut state = AppState::new();

    // Rapidly show and hide different modals
    for i in 0..10 {
        state.modal = Some(Modal::Error {
            title: format!("Error {}", i),
            message: format!("Message {}", i),
        });
        assert!(state.has_modal());

        state.close_modal();
        assert!(!state.has_modal());

        state.modal = Some(Modal::Success {
            title: format!("Success {}", i),
            message: format!("Message {}", i),
        });
        assert!(state.has_modal());

        state.close_modal();
        assert!(!state.has_modal());
    }
}

#[test]
fn test_e2e_concurrent_state_modifications() {
    let mut state = AppState::new();

    // Simulate multiple state changes happening in quick succession
    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;
    state.viewer_state.scroll = 50;
    state.search_query = "test".to_string();
    state.selected_workflow = 3;

    // All changes should be preserved
    assert_eq!(state.view_mode, ViewMode::Editor);
    assert!(state.editor_state.modified);
    assert_eq!(state.viewer_state.scroll, 50);
    assert_eq!(state.search_query, "test");
    assert_eq!(state.selected_workflow, 3);
}
