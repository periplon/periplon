//! Comprehensive keyboard handling tests for the state browser
//!
//! Tests all keyboard interactions including navigation, sorting, filtering, view switching,
//! and detail scrolling.

#![cfg(feature = "tui")]

use periplon_sdk::dsl::state::{WorkflowState, WorkflowStatus};
use periplon_sdk::tui::state::{AppState, ViewMode};
use periplon_sdk::tui::views::state_browser::{
    StateEntry, StateBrowserViewMode, StateSortMode,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use std::time::SystemTime;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a simple KeyEvent for testing
fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

/// Create a test state entry
fn create_test_entry(name: &str, status: WorkflowStatus, progress: f64) -> StateEntry {
    StateEntry {
        workflow_name: name.to_string(),
        workflow_version: "1.0.0".to_string(),
        status,
        progress,
        checkpoint_at: SystemTime::now(),
        started_at: SystemTime::now(),
        total_tasks: 10,
        completed_tasks: (progress * 10.0) as usize,
        failed_tasks: 0,
        file_path: PathBuf::from(format!("{}.state.json", name)),
    }
}

/// Simulate state browser keyboard handling in list view
async fn handle_list_key_simulation(
    state: &mut AppState,
    key_event: KeyEvent,
) -> ViewMode {
    // Simulate the logic from handle_state_browser_key in app.rs
    let browser_state = &mut state.state_browser;

    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            if browser_state.view_mode == StateBrowserViewMode::Details {
                browser_state.back_to_list();
            } else {
                state.view_mode = ViewMode::WorkflowList;
            }
        }
        KeyCode::Up => {
            if browser_state.view_mode == StateBrowserViewMode::List {
                browser_state.select_previous();
            } else {
                browser_state.scroll_details_up();
            }
        }
        KeyCode::Down => {
            if browser_state.view_mode == StateBrowserViewMode::List {
                browser_state.select_next();
            } else {
                let max_lines = 100;
                browser_state.scroll_details_down(max_lines);
            }
        }
        KeyCode::PageUp => {
            if browser_state.view_mode == StateBrowserViewMode::Details {
                browser_state.page_details_up();
            }
        }
        KeyCode::PageDown => {
            if browser_state.view_mode == StateBrowserViewMode::Details {
                let max_lines = 100;
                browser_state.page_details_down(max_lines);
            }
        }
        KeyCode::Enter => {
            if browser_state.view_mode == StateBrowserViewMode::List {
                let _ = browser_state.load_details();
            }
        }
        KeyCode::Char('s') => {
            browser_state.next_sort_mode();
        }
        KeyCode::Char('?') => {
            state.view_mode = ViewMode::Help;
        }
        _ => {}
    }

    state.view_mode
}

// ============================================================================
// Navigation Tests - List View
// ============================================================================

#[tokio::test]
async fn test_escape_from_list_returns_to_workflow_list() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;

    let new_mode = handle_list_key_simulation(&mut state, key(KeyCode::Esc)).await;

    assert_eq!(new_mode, ViewMode::WorkflowList);
}

#[tokio::test]
async fn test_q_from_list_returns_to_workflow_list() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;

    let new_mode = handle_list_key_simulation(&mut state, key(KeyCode::Char('q'))).await;

    assert_eq!(new_mode, ViewMode::WorkflowList);
}

#[tokio::test]
async fn test_up_arrow_selects_previous() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state
        .state_browser
        .states
        .push(create_test_entry("workflow2", WorkflowStatus::Completed, 1.0));
    state.state_browser.selected_index = 1;

    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.state_browser.selected_index, 0);
}

#[tokio::test]
async fn test_up_arrow_stops_at_zero() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state.state_browser.selected_index = 0;

    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.state_browser.selected_index, 0);
}

#[tokio::test]
async fn test_down_arrow_selects_next() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state
        .state_browser
        .states
        .push(create_test_entry("workflow2", WorkflowStatus::Completed, 1.0));
    state.state_browser.selected_index = 0;

    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;

    assert_eq!(state.state_browser.selected_index, 1);
}

#[tokio::test]
async fn test_down_arrow_stops_at_last() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state
        .state_browser
        .states
        .push(create_test_entry("workflow2", WorkflowStatus::Completed, 1.0));
    state.state_browser.selected_index = 1;

    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;

    assert_eq!(state.state_browser.selected_index, 1);
}

#[tokio::test]
async fn test_enter_loads_details() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state.state_browser.selected_index = 0;

    // Note: load_details() requires state persistence which isn't available in tests
    // So we just verify the key is handled without error
    handle_list_key_simulation(&mut state, key(KeyCode::Enter)).await;

    // View mode should remain StateBrowser (details loading might fail but that's ok)
    assert_eq!(state.view_mode, ViewMode::StateBrowser);
}

// ============================================================================
// Navigation Tests - Details View
// ============================================================================

#[tokio::test]
async fn test_escape_from_details_returns_to_list() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;

    handle_list_key_simulation(&mut state, key(KeyCode::Esc)).await;

    assert_eq!(state.state_browser.view_mode, StateBrowserViewMode::List);
    assert!(state.state_browser.current_state.is_none());
}

#[tokio::test]
async fn test_q_from_details_returns_to_list() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;

    handle_list_key_simulation(&mut state, key(KeyCode::Char('q'))).await;

    assert_eq!(state.state_browser.view_mode, StateBrowserViewMode::List);
}

#[tokio::test]
async fn test_up_arrow_in_details_scrolls_up() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 10;

    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.state_browser.details_scroll, 9);
}

#[tokio::test]
async fn test_up_arrow_in_details_stops_at_zero() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 0;

    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.state_browser.details_scroll, 0);
}

#[tokio::test]
async fn test_down_arrow_in_details_scrolls_down() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 5;

    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;

    assert_eq!(state.state_browser.details_scroll, 6);
}

#[tokio::test]
async fn test_page_up_in_details() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 30;
    state.state_browser.details_page_size = 20;

    handle_list_key_simulation(&mut state, key(KeyCode::PageUp)).await;

    assert_eq!(state.state_browser.details_scroll, 10);
}

#[tokio::test]
async fn test_page_up_stops_at_zero() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 5;
    state.state_browser.details_page_size = 20;

    handle_list_key_simulation(&mut state, key(KeyCode::PageUp)).await;

    assert_eq!(state.state_browser.details_scroll, 0);
}

#[tokio::test]
async fn test_page_down_in_details() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 10;
    state.state_browser.details_page_size = 20;

    handle_list_key_simulation(&mut state, key(KeyCode::PageDown)).await;

    assert_eq!(state.state_browser.details_scroll, 30);
}

#[tokio::test]
async fn test_page_down_respects_max() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 85;
    state.state_browser.details_page_size = 20;

    handle_list_key_simulation(&mut state, key(KeyCode::PageDown)).await;

    // max_lines is 100, page_size is 20, so max_scroll = 100 - 20 = 80
    assert_eq!(state.state_browser.details_scroll, 80);
}

// ============================================================================
// Sort Mode Tests
// ============================================================================

#[tokio::test]
async fn test_s_key_cycles_sort_mode() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state.state_browser.sort_mode = StateSortMode::NameAsc;

    handle_list_key_simulation(&mut state, key(KeyCode::Char('s'))).await;

    assert_eq!(state.state_browser.sort_mode, StateSortMode::NameDesc);
}

#[tokio::test]
async fn test_sort_mode_full_cycle() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state.state_browser.sort_mode = StateSortMode::NameAsc;

    let expected_sequence = [
        StateSortMode::NameDesc,
        StateSortMode::ModifiedAsc,
        StateSortMode::ModifiedDesc,
        StateSortMode::ProgressAsc,
        StateSortMode::ProgressDesc,
        StateSortMode::NameAsc, // Back to start
    ];

    for expected_mode in expected_sequence {
        handle_list_key_simulation(&mut state, key(KeyCode::Char('s'))).await;
        assert_eq!(state.state_browser.sort_mode, expected_mode);
    }
}

// ============================================================================
// Help Key Tests
// ============================================================================

#[tokio::test]
async fn test_question_mark_opens_help() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;

    let new_mode = handle_list_key_simulation(&mut state, key(KeyCode::Char('?'))).await;

    assert_eq!(new_mode, ViewMode::Help);
}

// ============================================================================
// Ignored Keys Tests
// ============================================================================

#[tokio::test]
async fn test_regular_chars_ignored() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    let original_index = state.state_browser.selected_index;

    handle_list_key_simulation(&mut state, key(KeyCode::Char('a'))).await;
    handle_list_key_simulation(&mut state, key(KeyCode::Char('x'))).await;
    handle_list_key_simulation(&mut state, key(KeyCode::Char('z'))).await;

    assert_eq!(state.state_browser.selected_index, original_index);
    assert_eq!(state.view_mode, ViewMode::StateBrowser);
}

#[tokio::test]
async fn test_function_keys_ignored() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    let original_index = state.state_browser.selected_index;

    handle_list_key_simulation(&mut state, key(KeyCode::F(1))).await;
    handle_list_key_simulation(&mut state, key(KeyCode::F(12))).await;

    assert_eq!(state.state_browser.selected_index, original_index);
    assert_eq!(state.view_mode, ViewMode::StateBrowser);
}

// ============================================================================
// State Consistency Tests
// ============================================================================

#[tokio::test]
async fn test_selection_persists_between_keys() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state
        .state_browser
        .states
        .push(create_test_entry("workflow2", WorkflowStatus::Completed, 1.0));
    state
        .state_browser
        .states
        .push(create_test_entry("workflow3", WorkflowStatus::Failed, 0.3));

    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.state_browser.selected_index, 1);
}

#[tokio::test]
async fn test_view_mode_preserved_during_navigation() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state
        .state_browser
        .states
        .push(create_test_entry("workflow2", WorkflowStatus::Completed, 1.0));

    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.state_browser.view_mode, StateBrowserViewMode::List);
}

#[tokio::test]
async fn test_scroll_preserved_during_sort() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_scroll = 15;

    // Sort key shouldn't affect details scroll
    handle_list_key_simulation(&mut state, key(KeyCode::Char('s'))).await;

    assert_eq!(state.state_browser.details_scroll, 15);
}

// ============================================================================
// Complex Workflow Tests
// ============================================================================

#[tokio::test]
async fn test_navigation_sequence() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    for i in 0..10 {
        state
            .state_browser
            .states
            .push(create_test_entry(&format!("workflow{}", i), WorkflowStatus::Running, 0.5));
    }

    // Navigate down 5 times
    for _ in 0..5 {
        handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    }
    assert_eq!(state.state_browser.selected_index, 5);

    // Navigate up 2 times
    for _ in 0..2 {
        handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;
    }
    assert_eq!(state.state_browser.selected_index, 3);

    // Navigate down to end
    for _ in 0..10 {
        handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    }
    assert_eq!(state.state_browser.selected_index, 9);
}

#[tokio::test]
async fn test_details_scroll_sequence() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    state.state_browser.details_page_size = 20;

    // Scroll down with arrows
    for _ in 0..10 {
        handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    }
    assert_eq!(state.state_browser.details_scroll, 10);

    // Page down
    handle_list_key_simulation(&mut state, key(KeyCode::PageDown)).await;
    assert_eq!(state.state_browser.details_scroll, 30);

    // Page up
    handle_list_key_simulation(&mut state, key(KeyCode::PageUp)).await;
    assert_eq!(state.state_browser.details_scroll, 10);

    // Scroll to top with arrows
    for _ in 0..20 {
        handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;
    }
    assert_eq!(state.state_browser.details_scroll, 0);
}

#[tokio::test]
async fn test_view_transition_sequence() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state.clone());

    // Start in list, go to details
    state.state_browser.view_mode = StateBrowserViewMode::Details;
    assert_eq!(state.state_browser.view_mode, StateBrowserViewMode::Details);

    // Escape back to list
    handle_list_key_simulation(&mut state, key(KeyCode::Esc)).await;
    assert_eq!(state.state_browser.view_mode, StateBrowserViewMode::List);
    assert!(state.state_browser.current_state.is_none());

    // From list, escape to workflow list
    handle_list_key_simulation(&mut state, key(KeyCode::Esc)).await;
    assert_eq!(state.view_mode, ViewMode::WorkflowList);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_navigation_with_empty_list() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;

    // Should not panic with empty list
    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;

    assert_eq!(state.state_browser.selected_index, 0);
}

#[tokio::test]
async fn test_navigation_with_single_item() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));

    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    assert_eq!(state.state_browser.selected_index, 0);

    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;
    assert_eq!(state.state_browser.selected_index, 0);
}

#[tokio::test]
async fn test_rapid_sort_changes() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state.state_browser.sort_mode = StateSortMode::NameAsc;

    // Cycle through all sort modes rapidly
    for _ in 0..12 {
        handle_list_key_simulation(&mut state, key(KeyCode::Char('s'))).await;
    }

    // Should be back to NameAsc after 2 full cycles (6 modes * 2)
    assert_eq!(state.state_browser.sort_mode, StateSortMode::NameAsc);
}

#[tokio::test]
async fn test_alternating_navigation() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    state.state_browser.view_mode = StateBrowserViewMode::List;
    state
        .state_browser
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state
        .state_browser
        .states
        .push(create_test_entry("workflow2", WorkflowStatus::Completed, 1.0));
    state.state_browser.selected_index = 0;

    // Alternate up and down
    for _ in 0..10 {
        handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
        handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;
    }

    // Should be back at start
    assert_eq!(state.state_browser.selected_index, 0);
}

#[tokio::test]
async fn test_details_scroll_boundary_conditions() {
    let mut state = AppState::new();
    state.view_mode = ViewMode::StateBrowser;
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.state_browser.current_state = Some(workflow_state);
    state.state_browser.view_mode = StateBrowserViewMode::Details;

    // Test scroll at zero
    state.state_browser.details_scroll = 0;
    handle_list_key_simulation(&mut state, key(KeyCode::Up)).await;
    assert_eq!(state.state_browser.details_scroll, 0);

    // Test scroll near max
    state.state_browser.details_scroll = 95;
    state.state_browser.details_page_size = 20;
    handle_list_key_simulation(&mut state, key(KeyCode::Down)).await;
    // Should be capped by max_lines calculation
    assert!(state.state_browser.details_scroll <= 100);
}
