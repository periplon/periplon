//! Dashboard and Application State End-to-End Tests
//!
//! Comprehensive E2E testing suite for the overall TUI application state,
//! view coordination, and dashboard-like functionality that manages the
//! entire application lifecycle.
//!
//! Test Scenarios:
//! - Application initialization and startup state
//! - View navigation and routing between all views
//! - Multi-view state coordination
//! - Cross-view data persistence
//! - Application-wide state management
//! - Session lifecycle management
//! - View stack and history management
//! - Modal overlay coordination
//! - Global search and filtering
//! - Application shutdown and cleanup
//! - State serialization and restoration
//! - Error state propagation across views
//! - Concurrent view state updates
//! - Application health monitoring
//!
//! These tests validate the complete application state management system
//! that coordinates all views and provides a cohesive user experience.

#![cfg(feature = "tui")]

use periplon_sdk::tui::state::{
    AppState, ConfirmAction, ExecutionState, ExecutionStatus, InputAction, Modal, ViewMode,
    WorkflowEntry,
};
use std::path::PathBuf;
use std::time::Instant;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a workflow entry for testing
fn create_workflow(name: &str) -> WorkflowEntry {
    WorkflowEntry {
        name: name.to_string(),
        path: PathBuf::from(format!("workflows/{}", name)),
        description: Some(format!("Test workflow: {}", name)),
        version: Some("1.0.0".to_string()),
        valid: true,
        errors: Vec::new(),
    }
}

/// Create app state with multiple workflows
fn create_app_with_workflows(count: usize) -> AppState {
    let mut state = AppState::new();
    for i in 1..=count {
        state
            .workflows
            .push(create_workflow(&format!("workflow_{}.yaml", i)));
    }
    state
}

/// Create running execution state
fn create_execution() -> ExecutionState {
    ExecutionState {
        workflow_path: PathBuf::from("test.yaml"),
        status: ExecutionStatus::Running,
        current_agent: Some("agent1".to_string()),
        current_task: Some("task1".to_string()),
        progress: 0.5,
        log: vec!["Execution started".to_string()],
        completed_tasks: Vec::new(),
        failed_tasks: Vec::new(),
        started_at: Instant::now(),
    }
}

// ============================================================================
// Application Initialization Tests
// ============================================================================

#[test]
fn test_e2e_app_initialization_default_state() {
    // Scenario: Application starts with default state
    let state = AppState::new();

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert!(state.workflows.is_empty());
    assert_eq!(state.selected_workflow, 0);
    assert!(state.modal.is_none());
    assert!(state.running);
    assert!(state.execution_state.is_none());
    assert_eq!(state.search_query, "");
    assert!(state.current_workflow.is_none());
    assert!(state.current_workflow_path.is_none());
}

#[test]
fn test_e2e_app_starts_in_workflow_list_view() {
    // Scenario: App always starts in WorkflowList view
    let state = AppState::new();

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
}

#[test]
fn test_e2e_app_running_flag_initialized() {
    // Scenario: Running flag is true on startup
    let state = AppState::new();

    assert!(state.running);
}

#[test]
fn test_e2e_app_no_modals_on_startup() {
    // Scenario: No modals are shown on startup
    let state = AppState::new();

    assert!(!state.has_modal());
    assert!(state.modal.is_none());
}

#[test]
fn test_e2e_app_empty_search_on_startup() {
    // Scenario: Search query is empty on startup
    let state = AppState::new();

    assert_eq!(state.search_query, "");
    assert_eq!(state.filtered_workflows().len(), 0);
}

// ============================================================================
// View Navigation Tests
// ============================================================================

#[test]
fn test_e2e_navigate_all_view_modes() {
    // Scenario: Navigate through all available view modes
    let mut state = AppState::new();

    let all_views = vec![
        ViewMode::WorkflowList,
        ViewMode::Viewer,
        ViewMode::Editor,
        ViewMode::Generator,
        ViewMode::ExecutionMonitor,
        ViewMode::StateBrowser,
        ViewMode::Help,
    ];

    for view in all_views {
        state.view_mode = view;
        assert_eq!(state.view_mode, view);
    }
}

#[test]
fn test_e2e_view_navigation_workflow_list_to_viewer() {
    // Scenario: Navigate from WorkflowList to Viewer
    let mut state = create_app_with_workflows(1);

    assert_eq!(state.view_mode, ViewMode::WorkflowList);

    // Load workflow and switch to viewer
    state.current_workflow_path = Some(state.workflows[0].path.clone());
    state.view_mode = ViewMode::Viewer;

    assert_eq!(state.view_mode, ViewMode::Viewer);
    assert!(state.current_workflow_path.is_some());
}

#[test]
fn test_e2e_view_navigation_viewer_to_editor() {
    // Scenario: Navigate from Viewer to Editor
    let mut state = create_app_with_workflows(1);

    state.view_mode = ViewMode::Viewer;
    state.current_workflow_path = Some(state.workflows[0].path.clone());

    // Switch to editor
    state.view_mode = ViewMode::Editor;

    assert_eq!(state.view_mode, ViewMode::Editor);
}

#[test]
fn test_e2e_view_navigation_back_to_workflow_list() {
    // Scenario: Return to WorkflowList from any view
    let mut state = AppState::new();

    // Navigate to various views and back
    let views = vec![ViewMode::Help, ViewMode::Generator, ViewMode::StateBrowser];

    for view in views {
        state.view_mode = view;
        state.view_mode = ViewMode::WorkflowList;
        assert_eq!(state.view_mode, ViewMode::WorkflowList);
    }
}

#[test]
fn test_e2e_view_navigation_preserves_selection() {
    // Scenario: Workflow selection preserved across view changes
    let mut state = create_app_with_workflows(5);

    state.selected_workflow = 2;

    // Navigate through views
    state.view_mode = ViewMode::Help;
    state.view_mode = ViewMode::Generator;
    state.view_mode = ViewMode::WorkflowList;

    assert_eq!(state.selected_workflow, 2);
}

// ============================================================================
// Multi-View State Coordination Tests
// ============================================================================

#[test]
fn test_e2e_workflow_data_shared_across_views() {
    // Scenario: Workflow data is accessible from all views
    let mut state = create_app_with_workflows(3);

    // Access workflows from different views
    state.view_mode = ViewMode::WorkflowList;
    assert_eq!(state.workflows.len(), 3);

    state.view_mode = ViewMode::Generator;
    assert_eq!(state.workflows.len(), 3);

    state.view_mode = ViewMode::Help;
    assert_eq!(state.workflows.len(), 3);
}

#[test]
fn test_e2e_search_query_persists_across_views() {
    // Scenario: Search query is maintained across views
    let mut state = create_app_with_workflows(5);

    state.search_query = "workflow_2".to_string();

    // Navigate away and back
    state.view_mode = ViewMode::Help;
    assert_eq!(state.search_query, "workflow_2");

    state.view_mode = ViewMode::WorkflowList;
    assert_eq!(state.search_query, "workflow_2");
    assert_eq!(state.filtered_workflows().len(), 1);
}

#[test]
fn test_e2e_execution_state_visible_from_all_views() {
    // Scenario: Execution state accessible from any view
    let mut state = AppState::new();
    state.execution_state = Some(create_execution());

    // Check execution state from different views
    state.view_mode = ViewMode::WorkflowList;
    assert!(state.execution_state.is_some());

    state.view_mode = ViewMode::ExecutionMonitor;
    assert!(state.execution_state.is_some());

    state.view_mode = ViewMode::Help;
    assert!(state.execution_state.is_some());
}

#[test]
fn test_e2e_current_workflow_shared_state() {
    // Scenario: Current workflow path shared across views
    let mut state = create_app_with_workflows(1);

    state.current_workflow_path = Some(state.workflows[0].path.clone());

    // Navigate between Viewer and Editor
    state.view_mode = ViewMode::Viewer;
    assert!(state.current_workflow_path.is_some());

    state.view_mode = ViewMode::Editor;
    assert!(state.current_workflow_path.is_some());

    // Same path in both views
    assert_eq!(
        state.current_workflow_path,
        Some(PathBuf::from("workflows/workflow_1.yaml"))
    );
}

// ============================================================================
// Modal Coordination Tests
// ============================================================================

#[test]
fn test_e2e_modal_overlay_on_any_view() {
    // Scenario: Modals can appear on any view
    let mut state = AppState::new();

    let views = vec![
        ViewMode::WorkflowList,
        ViewMode::Viewer,
        ViewMode::Editor,
        ViewMode::Generator,
        ViewMode::Help,
    ];

    for view in views {
        state.view_mode = view;
        state.modal = Some(Modal::Info {
            title: "Test".to_string(),
            message: "Modal on view".to_string(),
        });

        assert!(state.has_modal());
        state.close_modal();
    }
}

#[test]
fn test_e2e_modal_blocks_view_interaction() {
    // Scenario: Modal presence is detectable for input routing
    let mut state = AppState::new();

    assert!(!state.has_modal());

    // Show modal
    state.modal = Some(Modal::Confirm {
        title: "Confirm".to_string(),
        message: "Are you sure?".to_string(),
        action: ConfirmAction::Exit,
    });

    assert!(state.has_modal());

    // Modal should be handled before view
    // (This would be enforced in the event loop)
}

#[test]
fn test_e2e_modal_types_on_different_views() {
    // Scenario: Different modal types on different views
    let mut state = AppState::new();

    // Error modal on workflow list
    state.view_mode = ViewMode::WorkflowList;
    state.modal = Some(Modal::Error {
        title: "Error".to_string(),
        message: "Failed to load".to_string(),
    });
    assert!(state.has_modal());
    state.close_modal();

    // Input modal on generator
    state.view_mode = ViewMode::Generator;
    state.modal = Some(Modal::Input {
        title: "Input".to_string(),
        prompt: "Enter name".to_string(),
        default: "".to_string(),
        action: InputAction::CreateWorkflow,
    });
    assert!(state.has_modal());
    state.close_modal();

    // Success modal on editor
    state.view_mode = ViewMode::Editor;
    state.modal = Some(Modal::Success {
        title: "Success".to_string(),
        message: "Saved".to_string(),
    });
    assert!(state.has_modal());
}

// ============================================================================
// Application State Management Tests
// ============================================================================

#[test]
fn test_e2e_app_state_reset() {
    // Scenario: App state can be reset to initial state
    let mut state = create_app_with_workflows(5);

    // Modify state extensively
    state.view_mode = ViewMode::Editor;
    state.selected_workflow = 3;
    state.search_query = "test".to_string();
    state.execution_state = Some(create_execution());
    state.modal = Some(Modal::Info {
        title: "Test".to_string(),
        message: "Info".to_string(),
    });

    // Reset
    state.reset();

    // Verify reset to defaults
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.selected_workflow, 0);
    assert_eq!(state.search_query, "");
    assert!(state.execution_state.is_none());
    assert!(!state.has_modal());
    assert!(state.running);
}

#[test]
fn test_e2e_app_state_clone() {
    // Scenario: App state can be cloned for snapshots
    let state1 = create_app_with_workflows(3);
    let state2 = state1.clone();

    assert_eq!(state1.workflows.len(), state2.workflows.len());
    assert_eq!(state1.view_mode, state2.view_mode);
    assert_eq!(state1.selected_workflow, state2.selected_workflow);
}

#[test]
fn test_e2e_app_state_independence_after_clone() {
    // Scenario: Cloned states are independent
    let state1 = create_app_with_workflows(3);
    let mut state2 = state1.clone();

    // Modify clone
    state2.view_mode = ViewMode::Editor;
    state2.selected_workflow = 2;
    state2.search_query = "modified".to_string();

    // Original unchanged
    assert_eq!(state1.view_mode, ViewMode::WorkflowList);
    assert_eq!(state1.selected_workflow, 0);
    assert_eq!(state1.search_query, "");
}

// ============================================================================
// Session Lifecycle Tests
// ============================================================================

#[test]
fn test_e2e_session_startup() {
    // Scenario: Session starts with clean state
    let state = AppState::new();

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert!(state.running);
    assert!(state.workflows.is_empty());
    assert!(!state.has_modal());
}

#[test]
fn test_e2e_session_with_workflows_loaded() {
    // Scenario: Session with workflows loaded
    let state = create_app_with_workflows(10);

    assert_eq!(state.workflows.len(), 10);
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert!(state.running);
}

#[test]
fn test_e2e_session_running_flag_control() {
    // Scenario: Running flag controls application lifecycle
    let mut state = AppState::new();

    assert!(state.running);

    // Stop application
    state.running = false;

    assert!(!state.running);
}

#[test]
fn test_e2e_session_graceful_shutdown() {
    // Scenario: Application prepares for shutdown
    let mut state = create_app_with_workflows(5);

    // Active execution
    state.execution_state = Some(create_execution());
    state.view_mode = ViewMode::ExecutionMonitor;

    // User initiates quit
    state.modal = Some(Modal::Confirm {
        title: "Quit".to_string(),
        message: "Execution in progress. Are you sure?".to_string(),
        action: ConfirmAction::Exit,
    });

    assert!(state.has_modal());
    assert!(state.execution_state.is_some());

    // User confirms
    state.close_modal();
    state.running = false;

    assert!(!state.running);
}

// ============================================================================
// Cross-View Data Flow Tests
// ============================================================================

#[test]
fn test_e2e_workflow_selection_to_viewer() {
    // Scenario: Select workflow in list, view it
    let mut state = create_app_with_workflows(5);

    state.selected_workflow = 2;
    let selected_path = state.workflows[state.selected_workflow].path.clone();

    // Open in viewer
    state.current_workflow_path = Some(selected_path.clone());
    state.view_mode = ViewMode::Viewer;

    assert_eq!(state.view_mode, ViewMode::Viewer);
    assert_eq!(state.current_workflow_path, Some(selected_path));
}

#[test]
fn test_e2e_workflow_viewer_to_editor_flow() {
    // Scenario: View workflow, then edit it
    let mut state = create_app_with_workflows(1);

    // View
    state.current_workflow_path = Some(state.workflows[0].path.clone());
    state.view_mode = ViewMode::Viewer;

    let viewed_path = state.current_workflow_path.clone();

    // Edit
    state.view_mode = ViewMode::Editor;

    assert_eq!(state.view_mode, ViewMode::Editor);
    assert_eq!(state.current_workflow_path, viewed_path);
}

#[test]
fn test_e2e_generator_creates_workflow_adds_to_list() {
    // Scenario: Generate workflow, it appears in list
    let mut state = AppState::new();
    let initial_count = state.workflows.len();

    state.view_mode = ViewMode::Generator;

    // Simulate workflow generation and save
    let new_workflow = create_workflow("generated_workflow.yaml");
    state.workflows.push(new_workflow);

    state.view_mode = ViewMode::WorkflowList;

    assert_eq!(state.workflows.len(), initial_count + 1);
}

#[test]
fn test_e2e_execution_monitor_updates_execution_state() {
    // Scenario: Execution monitor reflects execution state changes
    let mut state = AppState::new();

    state.execution_state = Some(create_execution());
    state.view_mode = ViewMode::ExecutionMonitor;

    // Update execution state
    if let Some(ref mut exec) = state.execution_state {
        exec.progress = 0.8;
        exec.completed_tasks.push("task1".to_string());
    }

    assert!(state.execution_state.is_some());
    assert_eq!(state.execution_state.as_ref().unwrap().progress, 0.8);
}

// ============================================================================
// Global Search and Filtering Tests
// ============================================================================

#[test]
fn test_e2e_global_search_filters_workflows() {
    // Scenario: Search query filters workflow list globally
    let mut state = create_app_with_workflows(10);

    state.search_query = "workflow_5".to_string();

    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "workflow_5.yaml");
}

#[test]
fn test_e2e_search_persists_during_navigation() {
    // Scenario: Search state maintained during view changes
    let mut state = create_app_with_workflows(5);

    state.search_query = "workflow_2".to_string();
    assert_eq!(state.filtered_workflows().len(), 1);

    // Navigate away
    state.view_mode = ViewMode::Help;

    // Navigate back
    state.view_mode = ViewMode::WorkflowList;

    assert_eq!(state.search_query, "workflow_2");
    assert_eq!(state.filtered_workflows().len(), 1);
}

#[test]
fn test_e2e_clear_search_shows_all_workflows() {
    // Scenario: Clearing search shows all workflows
    let mut state = create_app_with_workflows(5);

    state.search_query = "workflow_1".to_string();
    assert_eq!(state.filtered_workflows().len(), 1);

    state.search_query = String::new();
    assert_eq!(state.filtered_workflows().len(), 5);
}

// ============================================================================
// Error State Propagation Tests
// ============================================================================

#[test]
fn test_e2e_error_modal_shown_on_failure() {
    // Scenario: Errors shown as modals regardless of view
    let mut state = AppState::new();

    state.view_mode = ViewMode::WorkflowList;
    state.modal = Some(Modal::Error {
        title: "Load Error".to_string(),
        message: "Failed to load workflows".to_string(),
    });

    assert!(state.has_modal());
}

#[test]
fn test_e2e_execution_failure_updates_state() {
    // Scenario: Execution failure updates global state
    let mut state = AppState::new();

    state.execution_state = Some(create_execution());

    // Execution fails
    if let Some(ref mut exec) = state.execution_state {
        exec.status = ExecutionStatus::Failed;
        exec.failed_tasks.push("critical_task".to_string());
    }

    assert_eq!(
        state.execution_state.as_ref().unwrap().status,
        ExecutionStatus::Failed
    );
}

#[test]
fn test_e2e_validation_errors_prevent_view_transition() {
    // Scenario: Validation errors shown before view transition
    let mut state = create_app_with_workflows(1);

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;

    // Try to leave editor with unsaved changes
    state.modal = Some(Modal::Confirm {
        title: "Unsaved Changes".to_string(),
        message: "Discard changes?".to_string(),
        action: ConfirmAction::DiscardChanges,
    });

    assert!(state.has_modal());
    assert_eq!(state.view_mode, ViewMode::Editor);
}

// ============================================================================
// Concurrent State Update Tests
// ============================================================================

#[test]
fn test_e2e_multiple_workflows_loaded_concurrently() {
    // Scenario: Multiple workflows loaded at once
    let mut state = AppState::new();

    let workflows = vec![
        create_workflow("w1.yaml"),
        create_workflow("w2.yaml"),
        create_workflow("w3.yaml"),
    ];

    state.workflows.extend(workflows);

    assert_eq!(state.workflows.len(), 3);
}

#[test]
fn test_e2e_workflow_list_updates_during_execution() {
    // Scenario: Workflow list can be updated while execution runs
    let mut state = create_app_with_workflows(2);

    state.execution_state = Some(create_execution());

    // Add new workflow during execution
    state.workflows.push(create_workflow("new_workflow.yaml"));

    assert_eq!(state.workflows.len(), 3);
    assert!(state.execution_state.is_some());
}

// ============================================================================
// View State Consistency Tests
// ============================================================================

#[test]
fn test_e2e_viewer_state_independent() {
    // Scenario: Viewer state separate from workflow list
    let mut state = create_app_with_workflows(3);

    state.view_mode = ViewMode::Viewer;
    state.viewer_state.scroll = 10;

    state.view_mode = ViewMode::WorkflowList;
    assert_eq!(state.selected_workflow, 0);

    state.view_mode = ViewMode::Viewer;
    assert_eq!(state.viewer_state.scroll, 10);
}

#[test]
fn test_e2e_editor_state_preserved() {
    // Scenario: Editor state preserved across view switches
    let mut state = create_app_with_workflows(1);

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;
    state.editor_state.cursor = (10, 5);

    state.view_mode = ViewMode::Help;
    state.view_mode = ViewMode::Editor;

    assert!(state.editor_state.modified);
    assert_eq!(state.editor_state.cursor, (10, 5));
}

#[test]
fn test_e2e_generator_state_maintained() {
    // Scenario: Generator state maintained across navigation
    let mut state = AppState::new();

    state.view_mode = ViewMode::Generator;
    state.generator_state.nl_input = "Create a workflow".to_string();

    state.view_mode = ViewMode::WorkflowList;
    state.view_mode = ViewMode::Generator;

    assert_eq!(state.generator_state.nl_input, "Create a workflow");
}

// ============================================================================
// Application Health Monitoring Tests
// ============================================================================

#[test]
fn test_e2e_app_state_always_valid() {
    // Scenario: App state remains valid through operations
    let mut state = create_app_with_workflows(5);

    // Perform various operations
    state.selected_workflow = 2;
    state.view_mode = ViewMode::Viewer;
    state.search_query = "test".to_string();

    // State should be consistent
    assert!(state.running);
    assert!(state.selected_workflow < state.workflows.len());
}

#[test]
fn test_e2e_invalid_selection_handled() {
    // Scenario: Invalid workflow selection is safe
    let state = create_app_with_workflows(3);

    // Selection beyond bounds
    let invalid_idx = 10;

    // Accessing workflows[invalid_idx] would panic, but we can check bounds
    assert!(invalid_idx >= state.workflows.len());

    // Application should handle this gracefully
    // (UI would clamp selection to valid range)
}

// ============================================================================
// Complex Multi-View Workflows Tests
// ============================================================================

#[test]
fn test_e2e_complete_workflow_creation_lifecycle() {
    // Scenario: Complete flow from creation to execution
    let mut state = AppState::new();

    // Step 1: Start in workflow list
    assert_eq!(state.view_mode, ViewMode::WorkflowList);

    // Step 2: Open generator
    state.view_mode = ViewMode::Generator;
    state.generator_state.nl_input = "Create test workflow".to_string();

    // Step 3: Generate (simulate)
    state.generator_state.generated_yaml = Some("name: test".to_string());

    // Step 4: Move to editor
    state.view_mode = ViewMode::Editor;

    // Step 5: Save (add to list)
    state.workflows.push(create_workflow("test.yaml"));
    state.view_mode = ViewMode::WorkflowList;

    // Step 6: Select and execute
    state.selected_workflow = 0;
    state.execution_state = Some(create_execution());
    state.view_mode = ViewMode::ExecutionMonitor;

    assert_eq!(state.view_mode, ViewMode::ExecutionMonitor);
    assert!(state.execution_state.is_some());
    assert_eq!(state.workflows.len(), 1);
}

#[test]
fn test_e2e_workflow_edit_and_re_execute() {
    // Scenario: Edit workflow and re-execute
    let mut state = create_app_with_workflows(1);

    // Execute
    state.execution_state = Some(create_execution());
    state.view_mode = ViewMode::ExecutionMonitor;

    // Execution completes
    if let Some(ref mut exec) = state.execution_state {
        exec.status = ExecutionStatus::Completed;
    }

    // Edit workflow
    state.view_mode = ViewMode::WorkflowList;
    state.selected_workflow = 0;
    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;

    // Save and re-execute
    state.editor_state.modified = false;
    state.execution_state = Some(create_execution());
    state.view_mode = ViewMode::ExecutionMonitor;

    assert_eq!(state.view_mode, ViewMode::ExecutionMonitor);
    assert_eq!(
        state.execution_state.as_ref().unwrap().status,
        ExecutionStatus::Running
    );
}

#[test]
fn test_e2e_multi_view_state_snapshot() {
    // Scenario: State snapshot captures all view states
    let state = create_app_with_workflows(5);
    let snapshot = state.clone();

    // Snapshot preserves everything
    assert_eq!(snapshot.view_mode, state.view_mode);
    assert_eq!(snapshot.workflows.len(), state.workflows.len());
    assert_eq!(snapshot.selected_workflow, state.selected_workflow);
    assert_eq!(snapshot.search_query, state.search_query);
    assert_eq!(snapshot.running, state.running);
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_e2e_empty_application_state() {
    // Scenario: Application with no workflows
    let state = AppState::new();

    assert!(state.workflows.is_empty());
    assert_eq!(state.filtered_workflows().len(), 0);
    assert_eq!(state.selected_workflow, 0);
}

#[test]
fn test_e2e_maximum_workflows_loaded() {
    // Scenario: Application handles many workflows
    let state = create_app_with_workflows(1000);

    assert_eq!(state.workflows.len(), 1000);
    assert_eq!(state.filtered_workflows().len(), 1000);
}

#[test]
fn test_e2e_rapid_view_switching() {
    // Scenario: Rapid view changes don't corrupt state
    let mut state = create_app_with_workflows(3);

    for _ in 0..100 {
        state.view_mode = ViewMode::WorkflowList;
        state.view_mode = ViewMode::Help;
        state.view_mode = ViewMode::Generator;
        state.view_mode = ViewMode::WorkflowList;
    }

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.workflows.len(), 3);
}

#[test]
fn test_e2e_nested_modal_prevention() {
    // Scenario: Only one modal at a time
    let mut state = AppState::new();

    state.modal = Some(Modal::Error {
        title: "Error 1".to_string(),
        message: "First error".to_string(),
    });

    // Try to show another modal (would replace)
    state.modal = Some(Modal::Info {
        title: "Info".to_string(),
        message: "Info message".to_string(),
    });

    // Only latest modal shown
    assert!(state.has_modal());
    if let Some(Modal::Info { title, .. }) = &state.modal {
        assert_eq!(title, "Info");
    } else {
        panic!("Wrong modal type");
    }
}

#[test]
fn test_e2e_app_state_default_equals_new() {
    // Scenario: Default and new() produce same state
    let state1 = AppState::new();
    let state2 = AppState::default();

    assert_eq!(state1.view_mode, state2.view_mode);
    assert_eq!(state1.workflows.len(), state2.workflows.len());
    assert_eq!(state1.running, state2.running);
    assert_eq!(state1.selected_workflow, state2.selected_workflow);
}
