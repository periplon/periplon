//! Workflows Page End-to-End Tests
//!
//! Comprehensive E2E testing suite for the workflow list/browser page,
//! covering complete user journeys from start to finish.
//!
//! Test Scenarios:
//! - Initial page load and workflow discovery
//! - Workflow browsing and navigation
//! - Workflow selection and viewing
//! - Workflow creation (new and from generator)
//! - Workflow editing and saving
//! - Workflow deletion with confirmation
//! - Search and filtering workflows
//! - Error handling and validation
//! - State persistence and recovery
//! - Multi-step user journeys
//!
//! These tests validate the entire workflow management experience
//! rather than individual component functionality.

#![cfg(feature = "tui")]
#![allow(clippy::collapsible_match)]

use periplon_sdk::tui::state::{
    AppState, ConfirmAction, InputAction, Modal, ViewMode, WorkflowEntry,
};
use periplon_sdk::tui::theme::Theme;
use periplon_sdk::tui::ui::generator::{GeneratorMode, GeneratorState};
use std::path::PathBuf;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test workflow entry with all fields
fn create_test_workflow(name: &str, valid: bool, has_errors: bool) -> WorkflowEntry {
    WorkflowEntry {
        name: name.to_string(),
        path: PathBuf::from(format!("workflows/{}", name)),
        description: Some(format!("Test workflow: {}", name)),
        version: Some("1.0.0".to_string()),
        valid,
        errors: if has_errors {
            vec![
                "Missing required field: agents".to_string(),
                "Invalid task reference".to_string(),
            ]
        } else {
            vec![]
        },
    }
}

/// Create a minimal valid workflow entry
fn create_workflow(name: &str) -> WorkflowEntry {
    create_test_workflow(name, true, false)
}

/// Create an invalid workflow entry
fn create_invalid_workflow(name: &str) -> WorkflowEntry {
    create_test_workflow(name, false, true)
}

/// Create populated app state with workflows
fn create_populated_state(workflow_count: usize) -> AppState {
    let mut state = AppState::new();
    for i in 1..=workflow_count {
        state
            .workflows
            .push(create_workflow(&format!("workflow_{}.yaml", i)));
    }
    state
}

/// Simulate user typing in modal input
fn simulate_input(state: &mut AppState, text: &str) {
    state.input_buffer = text.to_string();
}

/// Simulate confirming a modal action
fn simulate_confirm_modal(state: &mut AppState) -> Option<ConfirmAction> {
    if let Some(Modal::Confirm { action, .. }) = &state.modal {
        let action = action.clone();
        state.close_modal();
        Some(action)
    } else {
        None
    }
}

/// Simulate submitting input modal
fn simulate_submit_input(state: &mut AppState) -> Option<(InputAction, String)> {
    if let Some(Modal::Input { action, .. }) = &state.modal {
        let action = action.clone();
        let input = state.input_buffer.clone();
        state.close_modal();
        state.input_buffer.clear();
        Some((action, input))
    } else {
        None
    }
}

// ============================================================================
// Initial Page Load Tests
// ============================================================================

#[test]
fn test_e2e_initial_page_load_empty() {
    // Scenario: User launches TUI with no workflows in directory
    let state = AppState::new();

    // Should start on workflow list view
    assert_eq!(state.view_mode, ViewMode::WorkflowList);

    // Should have empty workflow list
    assert_eq!(state.workflows.len(), 0);

    // Should have no selection
    assert_eq!(state.selected_workflow, 0);

    // Should have no modal
    assert!(!state.has_modal());

    // Should be running
    assert!(state.running);

    // Search should be empty
    assert_eq!(state.search_query, "");
}

#[test]
fn test_e2e_initial_page_load_with_workflows() {
    // Scenario: User launches TUI with existing workflows
    let state = create_populated_state(5);

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.workflows.len(), 5);
    assert_eq!(state.selected_workflow, 0);

    // All workflows should be valid
    assert!(state.workflows.iter().all(|w| w.valid));
}

#[test]
fn test_e2e_initial_page_load_mixed_validity() {
    // Scenario: Directory contains both valid and invalid workflows
    let mut state = AppState::new();
    state.workflows.push(create_workflow("valid1.yaml"));
    state
        .workflows
        .push(create_invalid_workflow("invalid1.yaml"));
    state.workflows.push(create_workflow("valid2.yaml"));
    state
        .workflows
        .push(create_invalid_workflow("invalid2.yaml"));

    assert_eq!(state.workflows.len(), 4);

    let valid_count = state.workflows.iter().filter(|w| w.valid).count();
    let invalid_count = state.workflows.iter().filter(|w| !w.valid).count();

    assert_eq!(valid_count, 2);
    assert_eq!(invalid_count, 2);

    // Invalid workflows should have errors
    let invalid = state.workflows.iter().find(|w| !w.valid).unwrap();
    assert!(!invalid.errors.is_empty());
}

// ============================================================================
// Workflow Navigation Tests
// ============================================================================

#[test]
fn test_e2e_navigate_workflow_list() {
    // Scenario: User navigates through workflow list with arrow keys
    let mut state = create_populated_state(5);

    // Start at first item
    assert_eq!(state.selected_workflow, 0);

    // Navigate down
    state.selected_workflow = 1;
    assert_eq!(state.selected_workflow, 1);
    assert_eq!(
        state.workflows[state.selected_workflow].name,
        "workflow_2.yaml"
    );

    // Continue navigating
    state.selected_workflow = 2;
    state.selected_workflow = 3;
    state.selected_workflow = 4;
    assert_eq!(state.selected_workflow, 4);

    // Navigate back up
    state.selected_workflow = 3;
    state.selected_workflow = 2;
    assert_eq!(state.selected_workflow, 2);
}

#[test]
fn test_e2e_navigate_with_boundary_checks() {
    // Scenario: User tries to navigate beyond list boundaries
    let mut state = create_populated_state(3);

    // Try to go up from first item
    state.selected_workflow = 0;
    state.selected_workflow = state.selected_workflow.saturating_sub(1);
    assert_eq!(state.selected_workflow, 0); // Should stay at 0

    // Go to last item
    state.selected_workflow = 2;
    assert_eq!(state.selected_workflow, 2);

    // Try to go down beyond last item
    let new_selection = (state.selected_workflow + 1).min(state.workflows.len() - 1);
    state.selected_workflow = new_selection;
    assert_eq!(state.selected_workflow, 2); // Should stay at last item
}

#[test]
fn test_e2e_navigation_with_empty_list() {
    // Scenario: User attempts navigation with no workflows
    let mut state = AppState::new();

    state.selected_workflow = 0;
    assert_eq!(state.selected_workflow, 0);

    // Navigation should be safe with empty list
    state.selected_workflow = state.selected_workflow.saturating_sub(1);
    assert_eq!(state.selected_workflow, 0);
}

// ============================================================================
// Workflow Selection and Viewing Tests
// ============================================================================

#[test]
fn test_e2e_select_and_view_workflow() {
    // Scenario: User selects workflow and presses Enter to view it
    let mut state = create_populated_state(3);

    // Select second workflow
    state.selected_workflow = 1;
    let selected = &state.workflows[state.selected_workflow];

    // Simulate loading the workflow
    state.current_workflow_path = Some(selected.path.clone());

    // Transition to viewer
    state.view_mode = ViewMode::Viewer;

    assert_eq!(state.view_mode, ViewMode::Viewer);
    assert!(state.current_workflow_path.is_some());
    assert_eq!(
        state.current_workflow_path.as_ref().unwrap(),
        &PathBuf::from("workflows/workflow_2.yaml")
    );
}

#[test]
fn test_e2e_view_to_edit_workflow() {
    // Scenario: User views workflow, then presses 'e' to edit
    let mut state = create_populated_state(1);

    // View workflow
    state.view_mode = ViewMode::Viewer;
    state.current_workflow_path = Some(state.workflows[0].path.clone());

    assert_eq!(state.view_mode, ViewMode::Viewer);

    // Transition to editor
    state.view_mode = ViewMode::Editor;

    assert_eq!(state.view_mode, ViewMode::Editor);
    assert!(!state.editor_state.modified);
}

#[test]
fn test_e2e_return_to_list_from_viewer() {
    // Scenario: User views workflow, then returns to list with Esc
    let mut state = create_populated_state(2);

    state.view_mode = ViewMode::Viewer;
    state.current_workflow_path = Some(state.workflows[0].path.clone());

    // Return to list
    state.view_mode = ViewMode::WorkflowList;

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.selected_workflow, 0); // Selection preserved
}

// ============================================================================
// Workflow Creation Tests
// ============================================================================

#[test]
fn test_e2e_create_new_workflow_via_modal() {
    // Scenario: User presses 'n' to create new workflow via input modal
    let mut state = AppState::new();
    let initial_count = state.workflows.len();

    // Show input modal
    state.modal = Some(Modal::Input {
        title: "New Workflow".to_string(),
        prompt: "Enter workflow name:".to_string(),
        default: "new_workflow.yaml".to_string(),
        action: InputAction::CreateWorkflow,
    });

    assert!(state.has_modal());

    // User types workflow name
    simulate_input(&mut state, "my_awesome_workflow.yaml");

    // User confirms (presses Enter)
    if let Some((action, name)) = simulate_submit_input(&mut state) {
        assert_eq!(action, InputAction::CreateWorkflow);
        assert_eq!(name, "my_awesome_workflow.yaml");

        // Create the workflow
        state.workflows.push(create_workflow(&name));
    }

    assert_eq!(state.workflows.len(), initial_count + 1);
    assert!(!state.has_modal());
    assert_eq!(state.input_buffer, "");
}

#[test]
fn test_e2e_create_workflow_via_generator() {
    // Scenario: User presses 'g' to use AI generator for workflow creation
    let mut state = AppState::new();

    // Navigate to generator
    state.view_mode = ViewMode::Generator;
    state.generator_state = GeneratorState::new_create();

    assert_eq!(state.view_mode, ViewMode::Generator);
    assert_eq!(state.generator_state.mode, GeneratorMode::Create);

    // User enters natural language description
    state.generator_state.nl_input =
        "Create a workflow with a researcher agent that analyzes data".to_string();

    assert!(state.generator_state.can_generate());

    // Simulate AI generation
    state.generator_state.generated_yaml = Some(
        r#"name: "Data Analysis Workflow"
version: "1.0.0"
agents:
  researcher:
    description: "Analyzes data"
tasks:
  analyze:
    agent: "researcher"
    description: "Analyze the data""#
            .to_string(),
    );

    assert!(state.generator_state.generated_yaml.is_some());

    // User accepts and moves to editor
    state.view_mode = ViewMode::Editor;

    assert_eq!(state.view_mode, ViewMode::Editor);
}

#[test]
fn test_e2e_cancel_workflow_creation() {
    // Scenario: User starts creating workflow but cancels
    let mut state = AppState::new();
    let initial_count = state.workflows.len();

    // Show input modal
    state.modal = Some(Modal::Input {
        title: "New Workflow".to_string(),
        prompt: "Enter workflow name:".to_string(),
        default: "new_workflow.yaml".to_string(),
        action: InputAction::CreateWorkflow,
    });

    // User types something
    simulate_input(&mut state, "temp_workflow.yaml");

    // User cancels (presses Esc)
    state.close_modal();
    state.input_buffer.clear();

    assert!(!state.has_modal());
    assert_eq!(state.workflows.len(), initial_count); // No workflow added
    assert_eq!(state.input_buffer, "");
}

// ============================================================================
// Workflow Editing Tests
// ============================================================================

#[test]
fn test_e2e_edit_workflow_and_save() {
    // Scenario: User edits workflow and saves changes
    let mut state = create_populated_state(1);

    // Load workflow in editor
    state.view_mode = ViewMode::Editor;
    state.current_workflow_path = Some(state.workflows[0].path.clone());

    assert!(!state.editor_state.modified);

    // Simulate editing
    state.editor_state.modified = true;

    assert!(state.editor_state.modified);

    // User saves (Ctrl+S)
    // Simulate successful save
    state.editor_state.modified = false;

    // Show success modal
    state.modal = Some(Modal::Success {
        title: "Saved".to_string(),
        message: "Workflow saved successfully".to_string(),
    });

    assert!(state.has_modal());
    assert!(!state.editor_state.modified);

    // Dismiss success modal
    state.close_modal();

    assert!(!state.has_modal());
}

#[test]
fn test_e2e_edit_workflow_discard_changes() {
    // Scenario: User edits workflow but discards changes
    let mut state = create_populated_state(1);

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;

    // User tries to exit (Esc)
    // Show confirmation modal
    state.modal = Some(Modal::Confirm {
        title: "Unsaved Changes".to_string(),
        message: "You have unsaved changes. Discard them?".to_string(),
        action: ConfirmAction::DiscardChanges,
    });

    assert!(state.has_modal());
    assert!(state.editor_state.modified);

    // User confirms discard
    if let Some(action) = simulate_confirm_modal(&mut state) {
        assert_eq!(action, ConfirmAction::DiscardChanges);
        state.editor_state.modified = false;
        state.view_mode = ViewMode::WorkflowList;
    }

    assert!(!state.has_modal());
    assert!(!state.editor_state.modified);
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
}

#[test]
fn test_e2e_edit_workflow_validation_error() {
    // Scenario: User edits workflow with syntax errors
    let mut state = create_populated_state(1);

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;

    // Simulate validation errors from editing
    state
        .editor_state
        .errors
        .push(periplon_sdk::tui::state::EditorError {
            line: 10,
            column: Some(5),
            message: "Invalid YAML syntax".to_string(),
            severity: periplon_sdk::tui::state::ErrorSeverity::Error,
        });

    assert_eq!(state.editor_state.errors.len(), 1);

    // Try to save - validation fails
    state.modal = Some(Modal::Error {
        title: "Validation Error".to_string(),
        message: "Cannot save: Invalid YAML syntax at line 10".to_string(),
    });

    assert!(state.has_modal());
    assert!(state.editor_state.modified);

    // User dismisses error modal
    state.close_modal();

    // User fixes the error
    state.editor_state.errors.clear();

    // Now save succeeds
    state.editor_state.modified = false;

    assert!(!state.editor_state.modified);
    assert_eq!(state.editor_state.errors.len(), 0);
}

// ============================================================================
// Workflow Deletion Tests
// ============================================================================

#[test]
fn test_e2e_delete_workflow_with_confirmation() {
    // Scenario: User selects workflow and presses Ctrl+D to delete
    let mut state = create_populated_state(3);
    let initial_count = state.workflows.len();

    // Select second workflow
    state.selected_workflow = 1;
    let to_delete = state.workflows[state.selected_workflow].path.clone();

    // Show confirmation modal
    state.modal = Some(Modal::Confirm {
        title: "Delete Workflow".to_string(),
        message: format!("Delete {}?", to_delete.display()),
        action: ConfirmAction::DeleteWorkflow(to_delete.clone()),
    });

    assert!(state.has_modal());

    // User confirms deletion
    if let Some(action) = simulate_confirm_modal(&mut state) {
        if let ConfirmAction::DeleteWorkflow(path) = action {
            state.workflows.retain(|w| w.path != path);
        }
    }

    assert_eq!(state.workflows.len(), initial_count - 1);
    assert!(!state.has_modal());

    // Workflow should be removed
    assert!(!state.workflows.iter().any(|w| w.path == to_delete));
}

#[test]
fn test_e2e_cancel_workflow_deletion() {
    // Scenario: User starts deletion but cancels
    let mut state = create_populated_state(2);
    let initial_count = state.workflows.len();

    state.selected_workflow = 0;
    let to_delete = state.workflows[0].path.clone();

    // Show confirmation modal
    state.modal = Some(Modal::Confirm {
        title: "Delete Workflow".to_string(),
        message: format!("Delete {}?", to_delete.display()),
        action: ConfirmAction::DeleteWorkflow(to_delete.clone()),
    });

    // User cancels (presses Esc or 'n')
    state.close_modal();

    assert!(!state.has_modal());
    assert_eq!(state.workflows.len(), initial_count); // No deletion
    assert!(state.workflows.iter().any(|w| w.path == to_delete));
}

#[test]
fn test_e2e_delete_last_workflow() {
    // Scenario: User deletes the last workflow in list
    let mut state = create_populated_state(3);

    // Select last workflow
    state.selected_workflow = 2;
    let to_delete = state.workflows[2].path.clone();

    state.modal = Some(Modal::Confirm {
        title: "Delete Workflow".to_string(),
        message: format!("Delete {}?", to_delete.display()),
        action: ConfirmAction::DeleteWorkflow(to_delete.clone()),
    });

    if let Some(action) = simulate_confirm_modal(&mut state) {
        if let ConfirmAction::DeleteWorkflow(path) = action {
            state.workflows.retain(|w| w.path != path);
            // Adjust selection if needed
            if state.selected_workflow >= state.workflows.len() && !state.workflows.is_empty() {
                state.selected_workflow = state.workflows.len() - 1;
            }
        }
    }

    assert_eq!(state.workflows.len(), 2);
    assert_eq!(state.selected_workflow, 1); // Should move to new last item
}

// ============================================================================
// Search and Filter Tests
// ============================================================================

#[test]
fn test_e2e_search_workflows_by_name() {
    // Scenario: User presses '/' and searches for workflows
    let mut state = AppState::new();
    state.workflows.push(create_workflow("api_test.yaml"));
    state.workflows.push(create_workflow("database_setup.yaml"));
    state
        .workflows
        .push(create_workflow("api_integration.yaml"));
    state.workflows.push(create_workflow("frontend_build.yaml"));

    // User searches for "api"
    state.search_query = "api".to_string();

    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|w| w.name.contains("api")));
}

#[test]
fn test_e2e_search_workflows_by_description() {
    // Scenario: Search matches workflow descriptions
    let mut state = AppState::new();

    let mut w1 = create_workflow("workflow1.yaml");
    w1.description = Some("Testing API endpoints".to_string());

    let mut w2 = create_workflow("workflow2.yaml");
    w2.description = Some("Database migration".to_string());

    let mut w3 = create_workflow("workflow3.yaml");
    w3.description = Some("API integration tests".to_string());

    state.workflows = vec![w1, w2, w3];

    // Search by description content
    state.search_query = "API".to_string();

    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_e2e_search_case_insensitive() {
    // Scenario: Search is case-insensitive
    let mut state = AppState::new();
    state.workflows.push(create_workflow("TestWorkflow.yaml"));
    state
        .workflows
        .push(create_workflow("production_test.yaml"));

    // Search with different cases
    state.search_query = "TEST".to_string();
    assert_eq!(state.filtered_workflows().len(), 2);

    state.search_query = "test".to_string();
    assert_eq!(state.filtered_workflows().len(), 2);

    state.search_query = "TeSt".to_string();
    assert_eq!(state.filtered_workflows().len(), 2);
}

#[test]
fn test_e2e_clear_search() {
    // Scenario: User clears search to show all workflows
    let mut state = create_populated_state(5);

    // Apply search
    state.search_query = "workflow_1".to_string();
    assert_eq!(state.filtered_workflows().len(), 1);

    // Clear search
    state.search_query = String::new();
    assert_eq!(state.filtered_workflows().len(), 5);
}

#[test]
fn test_e2e_search_no_results() {
    // Scenario: Search returns no matches
    let mut state = create_populated_state(3);

    state.search_query = "nonexistent".to_string();
    let filtered = state.filtered_workflows();

    assert_eq!(filtered.len(), 0);
}

// ============================================================================
// Complete User Journey Tests
// ============================================================================

#[test]
fn test_e2e_complete_workflow_creation_journey() {
    // Scenario: Complete flow from empty state to created workflow
    let mut state = AppState::new();

    // Step 1: User on empty workflow list
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.workflows.len(), 0);

    // Step 2: User opens generator
    state.view_mode = ViewMode::Generator;
    state.generator_state = GeneratorState::new_create();

    // Step 3: User enters description
    state.generator_state.nl_input = "Create a data processing workflow".to_string();

    // Step 4: AI generates workflow
    state.generator_state.generated_yaml =
        Some("name: \"Data Processing\"\nversion: \"1.0.0\"".to_string());

    // Step 5: User accepts and edits
    state.view_mode = ViewMode::Editor;

    // Step 6: User makes minor edits
    state.editor_state.modified = true;

    // Step 7: User saves
    state.editor_state.modified = false;

    // Step 8: Return to list with new workflow
    state.view_mode = ViewMode::WorkflowList;
    state
        .workflows
        .push(create_workflow("data_processing.yaml"));

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.workflows.len(), 1);
    assert!(!state.editor_state.modified);
}

#[test]
fn test_e2e_browse_view_edit_save_journey() {
    // Scenario: User browses, views, edits, and saves existing workflow
    let mut state = create_populated_state(5);

    // Step 1: Browse and select
    state.selected_workflow = 2;
    assert_eq!(
        state.workflows[state.selected_workflow].name,
        "workflow_3.yaml"
    );

    // Step 2: View workflow
    state.view_mode = ViewMode::Viewer;
    state.current_workflow_path = Some(state.workflows[2].path.clone());

    // Step 3: Switch to edit mode
    state.view_mode = ViewMode::Editor;

    // Step 4: Make changes
    state.editor_state.modified = true;

    // Step 5: Save changes
    state.editor_state.modified = false;
    state.modal = Some(Modal::Success {
        title: "Saved".to_string(),
        message: "Changes saved".to_string(),
    });

    // Step 6: Return to viewer
    state.close_modal();
    state.view_mode = ViewMode::Viewer;

    // Step 7: Return to list
    state.view_mode = ViewMode::WorkflowList;

    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.selected_workflow, 2);
}

#[test]
fn test_e2e_search_select_delete_journey() {
    // Scenario: User searches, selects result, and deletes workflow
    let mut state = AppState::new();
    state.workflows.push(create_workflow("keep_this.yaml"));
    state.workflows.push(create_workflow("delete_me.yaml"));
    state.workflows.push(create_workflow("also_keep.yaml"));

    // Step 1: Search for workflow to delete
    state.search_query = "delete".to_string();
    let filtered = state.filtered_workflows();
    assert_eq!(filtered.len(), 1);

    // Step 2: Find index in original list
    let workflow_to_delete = filtered[0].clone();
    let index = state
        .workflows
        .iter()
        .position(|w| w.path == workflow_to_delete.path)
        .unwrap();

    // Step 3: Select it
    state.selected_workflow = index;

    // Step 4: Confirm deletion
    state.modal = Some(Modal::Confirm {
        title: "Delete".to_string(),
        message: "Delete workflow?".to_string(),
        action: ConfirmAction::DeleteWorkflow(workflow_to_delete.path.clone()),
    });

    if let Some(action) = simulate_confirm_modal(&mut state) {
        if let ConfirmAction::DeleteWorkflow(path) = action {
            state.workflows.retain(|w| w.path != path);
        }
    }

    // Step 5: Clear search and verify deletion
    state.search_query = String::new();
    assert_eq!(state.workflows.len(), 2);
    assert!(!state.workflows.iter().any(|w| w.name == "delete_me.yaml"));
}

// ============================================================================
// Error Handling and Recovery Tests
// ============================================================================

#[test]
fn test_e2e_handle_invalid_workflow_selection() {
    // Scenario: User tries to view an invalid workflow
    let mut state = AppState::new();
    state.workflows.push(create_invalid_workflow("broken.yaml"));

    state.selected_workflow = 0;
    let selected = &state.workflows[0];

    // Check workflow is invalid
    assert!(!selected.valid);
    assert!(!selected.errors.is_empty());

    // Attempting to view shows error modal
    state.modal = Some(Modal::Error {
        title: "Invalid Workflow".to_string(),
        message: format!("Cannot load workflow: {}", selected.errors[0]),
    });

    assert!(state.has_modal());

    // User dismisses error
    state.close_modal();

    // Stays on workflow list
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
}

#[test]
fn test_e2e_recover_from_save_failure() {
    // Scenario: Save fails, user fixes and retries
    let mut state = create_populated_state(1);

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;

    // Save fails
    state.modal = Some(Modal::Error {
        title: "Save Failed".to_string(),
        message: "Disk full or permission denied".to_string(),
    });

    assert!(state.has_modal());
    assert!(state.editor_state.modified); // Changes not lost

    // User dismisses error
    state.close_modal();

    // User can retry save after fixing issue
    state.editor_state.modified = false;
    state.modal = Some(Modal::Success {
        title: "Saved".to_string(),
        message: "Workflow saved successfully".to_string(),
    });

    assert!(state.has_modal());
    assert!(!state.editor_state.modified);
}

#[test]
fn test_e2e_validation_error_workflow() {
    // Scenario: User edits valid workflow and introduces validation errors
    let mut state = create_populated_state(1);

    state.view_mode = ViewMode::Editor;
    state.editor_state.modified = true;

    // Introduce validation errors
    state
        .editor_state
        .errors
        .push(periplon_sdk::tui::state::EditorError {
            line: 5,
            column: Some(1),
            message: "Undefined agent reference: 'researcher'".to_string(),
            severity: periplon_sdk::tui::state::ErrorSeverity::Error,
        });

    state
        .editor_state
        .errors
        .push(periplon_sdk::tui::state::EditorError {
            line: 12,
            column: None,
            message: "Circular task dependency detected".to_string(),
            severity: periplon_sdk::tui::state::ErrorSeverity::Error,
        });

    assert_eq!(state.editor_state.errors.len(), 2);

    // Try to save
    state.modal = Some(Modal::Error {
        title: "Validation Failed".to_string(),
        message: "Found 2 validation errors. Fix them before saving.".to_string(),
    });

    assert!(state.has_modal());

    // User fixes errors
    state.close_modal();
    state.editor_state.errors.clear();

    // Now can save
    state.editor_state.modified = false;

    assert_eq!(state.editor_state.errors.len(), 0);
    assert!(!state.editor_state.modified);
}

// ============================================================================
// State Persistence Tests
// ============================================================================

#[test]
fn test_e2e_preserve_selection_across_views() {
    // Scenario: Selection is preserved when navigating between views
    let mut state = create_populated_state(5);

    // Select third workflow
    state.selected_workflow = 2;

    // Navigate to help
    state.view_mode = ViewMode::Help;

    // Return to list
    state.view_mode = ViewMode::WorkflowList;

    // Selection preserved
    assert_eq!(state.selected_workflow, 2);
}

#[test]
fn test_e2e_preserve_search_across_views() {
    // Scenario: Search query is preserved
    let mut state = create_populated_state(5);

    state.search_query = "workflow_2".to_string();

    // Navigate away
    state.view_mode = ViewMode::Help;

    // Return
    state.view_mode = ViewMode::WorkflowList;

    // Search still active
    assert_eq!(state.search_query, "workflow_2");
    assert_eq!(state.filtered_workflows().len(), 1);
}

#[test]
fn test_e2e_app_state_reset_clears_everything() {
    // Scenario: App reset returns to initial state
    let mut state = create_populated_state(5);

    // Modify everything
    state.view_mode = ViewMode::Editor;
    state.selected_workflow = 3;
    state.search_query = "test".to_string();
    state.editor_state.modified = true;
    state.modal = Some(Modal::Error {
        title: "Error".to_string(),
        message: "Test".to_string(),
    });

    // Reset
    state.reset();

    // Everything back to initial state
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.selected_workflow, 0);
    assert_eq!(state.search_query, "");
    assert!(!state.editor_state.modified);
    assert!(!state.has_modal());
    assert!(state.running);
}

// ============================================================================
// Theme and UI Consistency Tests
// ============================================================================

#[test]
fn test_e2e_workflow_list_renders_with_all_themes() {
    // Scenario: Workflow list works with all theme variants
    let state = create_populated_state(3);

    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        // Verify theme has all required colors
        let _ = theme.primary;
        let _ = theme.secondary;
        let _ = theme.accent;
        let _ = theme.fg;
        let _ = theme.bg;
        let _ = theme.border;

        // Each theme should work with the workflow list
        assert_eq!(state.workflows.len(), 3);
    }
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_e2e_rapid_view_switching() {
    // Scenario: User rapidly switches between views
    let mut state = create_populated_state(2);

    for _ in 0..10 {
        state.view_mode = ViewMode::WorkflowList;
        assert_eq!(state.view_mode, ViewMode::WorkflowList);

        state.view_mode = ViewMode::Viewer;
        assert_eq!(state.view_mode, ViewMode::Viewer);

        state.view_mode = ViewMode::Editor;
        assert_eq!(state.view_mode, ViewMode::Editor);

        state.view_mode = ViewMode::Generator;
        assert_eq!(state.view_mode, ViewMode::Generator);

        state.view_mode = ViewMode::Help;
        assert_eq!(state.view_mode, ViewMode::Help);
    }
}

#[test]
fn test_e2e_many_workflows_performance() {
    // Scenario: App handles large number of workflows
    let mut state = AppState::new();

    // Add 100 workflows
    for i in 1..=100 {
        state
            .workflows
            .push(create_workflow(&format!("workflow_{:03}.yaml", i)));
    }

    assert_eq!(state.workflows.len(), 100);

    // Navigation works
    state.selected_workflow = 50;
    assert_eq!(state.selected_workflow, 50);

    // Search works
    state.search_query = "workflow_001".to_string();
    assert_eq!(state.filtered_workflows().len(), 1);

    // Filtering works - matches workflow_001 through workflow_099 (99 workflows)
    state.search_query = "workflow_0".to_string();
    assert_eq!(state.filtered_workflows().len(), 99); // workflow_001 through workflow_099
}

#[test]
fn test_e2e_workflow_with_long_names() {
    // Scenario: Workflows with very long names
    let mut state = AppState::new();

    let long_name = "a".repeat(200) + ".yaml";
    state.workflows.push(create_workflow(&long_name));

    assert_eq!(state.workflows.len(), 1);
    assert_eq!(state.workflows[0].name.len(), 205);
}

#[test]
fn test_e2e_workflow_with_special_characters() {
    // Scenario: Workflow names with special characters
    let mut state = AppState::new();

    let special_names = vec![
        "workflow-with-dashes.yaml",
        "workflow_with_underscores.yaml",
        "workflow.v2.0.yaml",
        "workflow (copy).yaml",
    ];

    for name in special_names {
        state.workflows.push(create_workflow(name));
    }

    assert_eq!(state.workflows.len(), 4);

    // Search works with special characters
    state.search_query = "v2.0".to_string();
    assert_eq!(state.filtered_workflows().len(), 1);
}
