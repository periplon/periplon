//! Comprehensive keyboard handling tests for the viewer screen
//!
//! Tests all keyboard interactions including navigation, scrolling, view mode toggle,
//! and mode switching.

#![cfg(feature = "tui")]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use periplon_sdk::dsl::DSLWorkflow;
use periplon_sdk::tui::state::{AppState, ViewMode, WorkflowViewMode};
use std::collections::HashMap;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a simple KeyEvent for testing
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Create a KeyEvent with Control modifier
fn ctrl_key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::CONTROL)
}

/// Simulate viewer keyboard handling
/// Returns the new view mode and viewer state
async fn handle_viewer_key_simulation(state: &mut AppState, key_event: KeyEvent) -> ViewMode {
    // Simulate the logic from handle_viewer_key in app.rs
    let viewer_state = &mut state.viewer_state;

    match key_event.code {
        KeyCode::Esc => {
            state.view_mode = ViewMode::WorkflowList;
            viewer_state.reset();
        }

        KeyCode::Tab => {
            viewer_state.toggle_view_mode();
        }

        KeyCode::Up | KeyCode::Char('k') => {
            viewer_state.scroll_up();
        }

        KeyCode::Down | KeyCode::Char('j') => {
            let max_lines = 100;
            viewer_state.scroll_down(max_lines);
        }

        KeyCode::PageUp => {
            viewer_state.page_up();
        }

        KeyCode::PageDown => {
            let max_lines = 100;
            viewer_state.page_down(max_lines);
        }

        KeyCode::Home => {
            viewer_state.scroll_to_top();
        }

        KeyCode::End => {
            let max_lines = 100;
            viewer_state.scroll_to_bottom(max_lines);
        }

        KeyCode::Char('e') => {
            state.view_mode = ViewMode::Editor;
        }

        _ => {}
    }

    state.view_mode
}

/// Create a minimal test workflow
fn create_test_workflow() -> DSLWorkflow {
    DSLWorkflow {
        name: "Test Workflow".to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: None,
        create_cwd: None,
        secrets: HashMap::new(),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        limits: None,
    }
}

// ============================================================================
// Navigation Tests
// ============================================================================

#[tokio::test]
async fn test_escape_returns_to_workflow_list() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());

    let new_mode = handle_viewer_key_simulation(&mut state, key(KeyCode::Esc)).await;

    assert_eq!(new_mode, ViewMode::WorkflowList);
}

#[tokio::test]
async fn test_escape_resets_viewer_state() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 10;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Esc)).await;

    assert_eq!(state.viewer_state.scroll, 0);
}

#[tokio::test]
async fn test_e_key_switches_to_editor() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());

    let new_mode = handle_viewer_key_simulation(&mut state, key(KeyCode::Char('e'))).await;

    assert_eq!(new_mode, ViewMode::Editor);
}

#[tokio::test]
async fn test_tab_toggles_view_mode() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.view_mode = WorkflowViewMode::Condensed;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Tab)).await;

    assert_eq!(state.viewer_state.view_mode, WorkflowViewMode::Full);
}

#[tokio::test]
async fn test_tab_toggles_back_to_condensed() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.view_mode = WorkflowViewMode::Full;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Tab)).await;

    assert_eq!(state.viewer_state.view_mode, WorkflowViewMode::Condensed);
}

// ============================================================================
// Scrolling Tests - Arrow Keys
// ============================================================================

#[tokio::test]
async fn test_up_arrow_scrolls_up() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 10;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.viewer_state.scroll, 9);
}

#[tokio::test]
async fn test_up_arrow_stops_at_zero() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 0;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.viewer_state.scroll, 0);
}

#[tokio::test]
async fn test_down_arrow_scrolls_down() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 5;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;

    assert_eq!(state.viewer_state.scroll, 6);
}

#[tokio::test]
async fn test_down_arrow_respects_max_lines() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 99;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;

    // Should be capped at max_lines (100)
    assert_eq!(state.viewer_state.scroll, 100);
}

// ============================================================================
// Scrolling Tests - Vim Keys
// ============================================================================

#[tokio::test]
async fn test_k_key_scrolls_up() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 10;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Char('k'))).await;

    assert_eq!(state.viewer_state.scroll, 9);
}

#[tokio::test]
async fn test_j_key_scrolls_down() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 5;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Char('j'))).await;

    assert_eq!(state.viewer_state.scroll, 6);
}

#[tokio::test]
async fn test_multiple_k_presses() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 10;

    for _ in 0..5 {
        handle_viewer_key_simulation(&mut state, key(KeyCode::Char('k'))).await;
    }

    assert_eq!(state.viewer_state.scroll, 5);
}

#[tokio::test]
async fn test_multiple_j_presses() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 5;

    for _ in 0..5 {
        handle_viewer_key_simulation(&mut state, key(KeyCode::Char('j'))).await;
    }

    assert_eq!(state.viewer_state.scroll, 10);
}

// ============================================================================
// Page Navigation Tests
// ============================================================================

#[tokio::test]
async fn test_page_up() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 30;

    handle_viewer_key_simulation(&mut state, key(KeyCode::PageUp)).await;

    // Page size is typically 10 lines
    assert_eq!(state.viewer_state.scroll, 20);
}

#[tokio::test]
async fn test_page_up_stops_at_zero() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 5;

    handle_viewer_key_simulation(&mut state, key(KeyCode::PageUp)).await;

    assert_eq!(state.viewer_state.scroll, 0);
}

#[tokio::test]
async fn test_page_down() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 10;

    handle_viewer_key_simulation(&mut state, key(KeyCode::PageDown)).await;

    // Page size is typically 10 lines
    assert_eq!(state.viewer_state.scroll, 20);
}

#[tokio::test]
async fn test_page_down_from_near_max() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 95;

    handle_viewer_key_simulation(&mut state, key(KeyCode::PageDown)).await;

    // page_down adds 10 unconditionally (no max enforcement)
    assert_eq!(state.viewer_state.scroll, 105);
}

// ============================================================================
// Home/End Tests
// ============================================================================

#[tokio::test]
async fn test_home_key_goes_to_top() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 50;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Home)).await;

    assert_eq!(state.viewer_state.scroll, 0);
}

#[tokio::test]
async fn test_end_key_goes_to_bottom() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 0;

    handle_viewer_key_simulation(&mut state, key(KeyCode::End)).await;

    // Should jump to max_lines
    assert_eq!(state.viewer_state.scroll, 100);
}

#[tokio::test]
async fn test_home_from_zero_stays_at_zero() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 0;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Home)).await;

    assert_eq!(state.viewer_state.scroll, 0);
}

// ============================================================================
// Ignored Keys Tests
// ============================================================================

#[tokio::test]
async fn test_regular_chars_ignored() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    let original_scroll = state.viewer_state.scroll;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Char('a'))).await;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Char('x'))).await;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Char('z'))).await;

    assert_eq!(state.viewer_state.scroll, original_scroll);
    assert_eq!(state.view_mode, ViewMode::Viewer);
}

#[tokio::test]
async fn test_ctrl_keys_ignored() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    let original_scroll = state.viewer_state.scroll;

    handle_viewer_key_simulation(&mut state, ctrl_key(KeyCode::Char('s'))).await;
    handle_viewer_key_simulation(&mut state, ctrl_key(KeyCode::Char('r'))).await;

    assert_eq!(state.viewer_state.scroll, original_scroll);
    assert_eq!(state.view_mode, ViewMode::Viewer);
}

#[tokio::test]
async fn test_function_keys_ignored() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    let original_scroll = state.viewer_state.scroll;

    handle_viewer_key_simulation(&mut state, key(KeyCode::F(1))).await;
    handle_viewer_key_simulation(&mut state, key(KeyCode::F(12))).await;

    assert_eq!(state.viewer_state.scroll, original_scroll);
    assert_eq!(state.view_mode, ViewMode::Viewer);
}

// ============================================================================
// State Consistency Tests
// ============================================================================

#[tokio::test]
async fn test_scroll_state_persists_between_keys() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());

    handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.viewer_state.scroll, 1);
}

#[tokio::test]
async fn test_view_mode_persists_during_scroll() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.view_mode = WorkflowViewMode::Full;

    handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.viewer_state.view_mode, WorkflowViewMode::Full);
}

#[tokio::test]
async fn test_workflow_preserved_during_navigation() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    let workflow = create_test_workflow();
    let workflow_name = workflow.name.clone();
    state.current_workflow = Some(workflow);

    handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Tab)).await;

    assert!(state.current_workflow.is_some());
    assert_eq!(state.current_workflow.as_ref().unwrap().name, workflow_name);
}

// ============================================================================
// Complex Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_scroll_navigation_sequence() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());

    // Go down 20 lines
    for _ in 0..20 {
        handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    }
    assert_eq!(state.viewer_state.scroll, 20);

    // Page up
    handle_viewer_key_simulation(&mut state, key(KeyCode::PageUp)).await;
    assert_eq!(state.viewer_state.scroll, 10);

    // Go to top
    handle_viewer_key_simulation(&mut state, key(KeyCode::Home)).await;
    assert_eq!(state.viewer_state.scroll, 0);

    // Go to bottom
    handle_viewer_key_simulation(&mut state, key(KeyCode::End)).await;
    assert_eq!(state.viewer_state.scroll, 100);
}

#[tokio::test]
async fn test_view_mode_toggle_sequence() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.view_mode = WorkflowViewMode::Condensed;

    // Toggle to Full
    handle_viewer_key_simulation(&mut state, key(KeyCode::Tab)).await;
    assert_eq!(state.viewer_state.view_mode, WorkflowViewMode::Full);

    // Toggle back to Condensed
    handle_viewer_key_simulation(&mut state, key(KeyCode::Tab)).await;
    assert_eq!(state.viewer_state.view_mode, WorkflowViewMode::Condensed);

    // Scroll should persist
    state.viewer_state.scroll = 10;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Tab)).await;
    assert_eq!(state.viewer_state.scroll, 10);
}

#[tokio::test]
async fn test_vim_and_arrow_keys_interchangeable() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());

    handle_viewer_key_simulation(&mut state, key(KeyCode::Char('j'))).await;
    assert_eq!(state.viewer_state.scroll, 1);

    handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    assert_eq!(state.viewer_state.scroll, 2);

    handle_viewer_key_simulation(&mut state, key(KeyCode::Up)).await;
    assert_eq!(state.viewer_state.scroll, 1);

    handle_viewer_key_simulation(&mut state, key(KeyCode::Char('k'))).await;
    assert_eq!(state.viewer_state.scroll, 0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_rapid_key_presses() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());

    // Simulate rapid key presses
    for _ in 0..100 {
        handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    }

    // Should be capped at max_lines
    assert_eq!(state.viewer_state.scroll, 100);
}

#[tokio::test]
async fn test_alternating_directions() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());
    state.viewer_state.scroll = 50;

    for _ in 0..10 {
        handle_viewer_key_simulation(&mut state, key(KeyCode::Up)).await;
        handle_viewer_key_simulation(&mut state, key(KeyCode::Down)).await;
    }

    // Should return to starting position
    assert_eq!(state.viewer_state.scroll, 50);
}

#[tokio::test]
async fn test_esc_from_different_scroll_positions() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::Viewer;
    state.current_workflow = Some(create_test_workflow());

    // Test from scroll position 0
    state.viewer_state.scroll = 0;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Esc)).await;
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.viewer_state.scroll, 0);

    // Reset and test from middle
    state.view_mode = ViewMode::Viewer;
    state.viewer_state.scroll = 50;
    handle_viewer_key_simulation(&mut state, key(KeyCode::Esc)).await;
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
    assert_eq!(state.viewer_state.scroll, 0);
}
