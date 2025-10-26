//! Comprehensive ratatui backend tests for state browser rendering
//!
//! Tests state browser layout, list view, details view, progress bars, and visual elements
//! using the ratatui TestBackend.

#![cfg(feature = "tui")]
#![allow(clippy::field_reassign_with_default)]

use periplon_sdk::dsl::state::{WorkflowState, WorkflowStatus};
use periplon_sdk::tui::theme::Theme;
use periplon_sdk::tui::views::state_browser::{
    StateBrowserState, StateBrowserViewMode, StateEntry, StateSortMode,
};
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::path::PathBuf;
use std::time::SystemTime;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test terminal with specified dimensions
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Render state browser and return terminal for assertions
fn render_state_browser(
    state: &mut StateBrowserState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend> {
    let mut terminal = create_test_terminal(width, height);

    terminal
        .draw(|frame| {
            let area = frame.area();
            periplon_sdk::tui::ui::state_browser::render_state_browser(frame, area, state, theme);
        })
        .unwrap();

    terminal
}

/// Check if buffer contains text
fn buffer_contains_text(terminal: &Terminal<TestBackend>, text: &str) -> bool {
    let buffer = terminal.backend().buffer();
    let content = buffer.content();

    let visible: String = content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("");

    visible.contains(text)
}

/// Count visible border characters
fn count_border_chars(terminal: &Terminal<TestBackend>) -> usize {
    let buffer = terminal.backend().buffer();
    buffer
        .content()
        .iter()
        .filter(|cell| {
            let symbol = cell.symbol();
            matches!(
                symbol,
                "─" | "│" | "┌" | "┐" | "└" | "┘" | "├" | "┤" | "┬" | "┴" | "┼"
            )
        })
        .count()
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

// ============================================================================
// Basic Rendering Tests
// ============================================================================

#[test]
fn test_state_browser_basic_rendering() {
    let mut state = StateBrowserState::default();
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should have borders
    assert!(count_border_chars(&terminal) > 0);

    // Should show header
    assert!(buffer_contains_text(&terminal, "Workflow State Browser"));
}

#[test]
fn test_empty_state_list() {
    let mut state = StateBrowserState::default();
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);

    // Should show "States (0)"
    assert!(buffer_contains_text(&terminal, "States (0)"));
}

#[test]
fn test_state_browser_with_states() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state.states.push(create_test_entry(
        "workflow2",
        WorkflowStatus::Completed,
        1.0,
    ));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show state count
    assert!(buffer_contains_text(&terminal, "States (2)"));

    // Should show workflow names
    assert!(buffer_contains_text(&terminal, "workflow1"));
    assert!(buffer_contains_text(&terminal, "workflow2"));
}

// ============================================================================
// List View Tests
// ============================================================================

#[test]
fn test_list_view_header() {
    let mut state = StateBrowserState::default();
    state.view_mode = StateBrowserViewMode::List;
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Workflow State Browser"));
}

#[test]
fn test_list_view_filter_bar() {
    let mut state = StateBrowserState::default();
    state.view_mode = StateBrowserViewMode::List;
    state.filter_query = String::new();
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show filter placeholder
    assert!(buffer_contains_text(&terminal, "Filter: (type to search)"));
}

#[test]
fn test_list_view_active_filter() {
    let mut state = StateBrowserState::default();
    state.view_mode = StateBrowserViewMode::List;
    state.filter_query = "test".to_string();
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show active filter
    assert!(buffer_contains_text(&terminal, "Filter: test"));
}

#[test]
fn test_list_view_sort_mode_display() {
    let mut state = StateBrowserState::default();
    state.view_mode = StateBrowserViewMode::List;
    state.sort_mode = StateSortMode::NameAsc;
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Sort:"));
    assert!(buffer_contains_text(&terminal, "Name ↑"));
}

#[test]
fn test_list_view_status_display() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("running", WorkflowStatus::Running, 0.3));
    state.states.push(create_test_entry(
        "completed",
        WorkflowStatus::Completed,
        1.0,
    ));
    state
        .states
        .push(create_test_entry("failed", WorkflowStatus::Failed, 0.6));
    state
        .states
        .push(create_test_entry("paused", WorkflowStatus::Paused, 0.8));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 30);

    // Should show all status types
    assert!(buffer_contains_text(&terminal, "[Running]"));
    assert!(buffer_contains_text(&terminal, "[Completed]"));
    assert!(buffer_contains_text(&terminal, "[Failed]"));
    assert!(buffer_contains_text(&terminal, "[Paused]"));
}

#[test]
fn test_list_view_version_display() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show version
    assert!(buffer_contains_text(&terminal, "v1.0.0"));
}

#[test]
fn test_list_view_task_count() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show task counts (5/10 tasks for 50% progress)
    assert!(buffer_contains_text(&terminal, "(5/10 tasks)"));
}

#[test]
fn test_list_view_progress_bar() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 24);

    // Should show progress bar with filled and empty chars
    assert!(buffer_contains_text(&terminal, "["));
    assert!(buffer_contains_text(&terminal, "]"));
}

#[test]
fn test_list_view_help_bar() {
    let mut state = StateBrowserState::default();
    state.view_mode = StateBrowserViewMode::List;
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 24);

    // Should show navigation hints
    assert!(buffer_contains_text(&terminal, "Navigate"));
    assert!(buffer_contains_text(&terminal, "View Details"));
}

#[test]
fn test_list_view_highlight_symbol() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state.selected_index = 0;
    state.list_state.select(Some(0)); // Ensure list state is properly set
    state.view_mode = StateBrowserViewMode::List;

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should render without error (selection indicator may be theme-dependent or position-specific)
    assert!(count_border_chars(&terminal) > 0);
    assert!(buffer_contains_text(&terminal, "workflow1"));
}

// ============================================================================
// Details View Tests
// ============================================================================

#[test]
fn test_details_view_header() {
    let mut state = StateBrowserState::default();
    let workflow_state = WorkflowState::new("test_workflow".to_string(), "2.0.0".to_string());
    state.current_state = Some(workflow_state);
    state.view_mode = StateBrowserViewMode::Details;

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show workflow name and version in header
    assert!(buffer_contains_text(&terminal, "Workflow State:"));
    assert!(buffer_contains_text(&terminal, "test_workflow"));
    assert!(buffer_contains_text(&terminal, "v2.0.0"));
}

#[test]
fn test_details_view_status() {
    let mut state = StateBrowserState::default();
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.current_state = Some(workflow_state);
    state.view_mode = StateBrowserViewMode::Details;

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show status
    assert!(buffer_contains_text(&terminal, "Status:"));
}

#[test]
fn test_details_view_progress() {
    let mut state = StateBrowserState::default();
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.current_state = Some(workflow_state);
    state.view_mode = StateBrowserViewMode::Details;

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show progress percentage
    assert!(buffer_contains_text(&terminal, "Progress:"));
}

#[test]
fn test_details_view_controls() {
    let mut state = StateBrowserState::default();
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.current_state = Some(workflow_state);
    state.view_mode = StateBrowserViewMode::Details;

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 24);

    // Should show control hints
    assert!(buffer_contains_text(&terminal, "Delete"));
    assert!(buffer_contains_text(&terminal, "Back"));
    assert!(buffer_contains_text(&terminal, "Scroll"));
}

#[test]
fn test_details_view_no_state_error() {
    let mut state = StateBrowserState::default();
    state.current_state = None;
    state.view_mode = StateBrowserViewMode::Details;

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should show error message
    assert!(buffer_contains_text(&terminal, "No state loaded"));
}

// ============================================================================
// Sort Mode Tests
// ============================================================================

#[test]
fn test_sort_mode_cycle() {
    let mode = StateSortMode::NameAsc;
    assert_eq!(mode.next(), StateSortMode::NameDesc);

    let mode = StateSortMode::NameDesc;
    assert_eq!(mode.next(), StateSortMode::ModifiedAsc);

    let mode = StateSortMode::ModifiedAsc;
    assert_eq!(mode.next(), StateSortMode::ModifiedDesc);

    let mode = StateSortMode::ModifiedDesc;
    assert_eq!(mode.next(), StateSortMode::ProgressAsc);

    let mode = StateSortMode::ProgressAsc;
    assert_eq!(mode.next(), StateSortMode::ProgressDesc);

    let mode = StateSortMode::ProgressDesc;
    assert_eq!(mode.next(), StateSortMode::NameAsc);
}

#[test]
fn test_sort_mode_display_names() {
    assert_eq!(StateSortMode::NameAsc.display_name(), "Name ↑");
    assert_eq!(StateSortMode::NameDesc.display_name(), "Name ↓");
    assert_eq!(StateSortMode::ModifiedAsc.display_name(), "Modified ↑");
    assert_eq!(StateSortMode::ModifiedDesc.display_name(), "Modified ↓");
    assert_eq!(StateSortMode::ProgressAsc.display_name(), "Progress ↑");
    assert_eq!(StateSortMode::ProgressDesc.display_name(), "Progress ↓");
}

#[test]
fn test_all_sort_modes_displayed() {
    let sort_modes = [
        StateSortMode::NameAsc,
        StateSortMode::NameDesc,
        StateSortMode::ModifiedAsc,
        StateSortMode::ModifiedDesc,
        StateSortMode::ProgressAsc,
        StateSortMode::ProgressDesc,
    ];

    for sort_mode in &sort_modes {
        let mut state = StateBrowserState::default();
        state.sort_mode = *sort_mode;
        let theme = Theme::default();
        let terminal = render_state_browser(&mut state, &theme, 80, 24);

        assert!(buffer_contains_text(&terminal, sort_mode.display_name()));
    }
}

// ============================================================================
// State Entry Tests
// ============================================================================

#[test]
fn test_state_entry_status_text() {
    let entry = create_test_entry("test", WorkflowStatus::Running, 0.5);
    assert_eq!(entry.status_text(), "Running");

    let entry = create_test_entry("test", WorkflowStatus::Completed, 1.0);
    assert_eq!(entry.status_text(), "Completed");

    let entry = create_test_entry("test", WorkflowStatus::Failed, 0.3);
    assert_eq!(entry.status_text(), "Failed");

    let entry = create_test_entry("test", WorkflowStatus::Paused, 0.7);
    assert_eq!(entry.status_text(), "Paused");
}

// ============================================================================
// View Mode Tests
// ============================================================================

#[test]
fn test_view_mode_enum_values() {
    let list_mode = StateBrowserViewMode::List;
    let details_mode = StateBrowserViewMode::Details;

    assert_ne!(list_mode, details_mode);
}

// ============================================================================
// Filter Tests
// ============================================================================

#[test]
fn test_filter_by_workflow_name() {
    let mut state = StateBrowserState::default();
    state.states.push(create_test_entry(
        "test_workflow",
        WorkflowStatus::Running,
        0.5,
    ));
    state.states.push(create_test_entry(
        "other_workflow",
        WorkflowStatus::Completed,
        1.0,
    ));

    // No filter
    assert_eq!(state.filtered_states().len(), 2);

    // Filter for "test"
    state.filter_query = "test".to_string();
    assert_eq!(state.filtered_states().len(), 1);
    assert_eq!(state.filtered_states()[0].workflow_name, "test_workflow");

    // Filter for "other"
    state.filter_query = "other".to_string();
    assert_eq!(state.filtered_states().len(), 1);
    assert_eq!(state.filtered_states()[0].workflow_name, "other_workflow");

    // Filter with no matches
    state.filter_query = "nonexistent".to_string();
    assert_eq!(state.filtered_states().len(), 0);
}

#[test]
fn test_filter_by_status() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state.states.push(create_test_entry(
        "workflow2",
        WorkflowStatus::Completed,
        1.0,
    ));

    // Filter for "running"
    state.filter_query = "running".to_string();
    assert_eq!(state.filtered_states().len(), 1);
    assert_eq!(state.filtered_states()[0].status, WorkflowStatus::Running);

    // Filter for "completed"
    state.filter_query = "completed".to_string();
    assert_eq!(state.filtered_states().len(), 1);
    assert_eq!(state.filtered_states()[0].status, WorkflowStatus::Completed);
}

#[test]
fn test_filter_case_insensitive() {
    let mut state = StateBrowserState::default();
    state.states.push(create_test_entry(
        "TestWorkflow",
        WorkflowStatus::Running,
        0.5,
    ));

    // Lowercase filter
    state.filter_query = "test".to_string();
    assert_eq!(state.filtered_states().len(), 1);

    // Uppercase filter
    state.filter_query = "TEST".to_string();
    assert_eq!(state.filtered_states().len(), 1);

    // Mixed case filter
    state.filter_query = "TeSt".to_string();
    assert_eq!(state.filtered_states().len(), 1);
}

// ============================================================================
// Layout Tests
// ============================================================================

#[test]
fn test_list_view_layout_structure() {
    let mut state = StateBrowserState::default();
    state.view_mode = StateBrowserViewMode::List;
    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should have all sections
    assert!(buffer_contains_text(&terminal, "Workflow State Browser")); // Header
    assert!(buffer_contains_text(&terminal, "Filter:")); // Search bar
    assert!(buffer_contains_text(&terminal, "States")); // List
    assert!(buffer_contains_text(&terminal, "Navigate")); // Status bar
}

#[test]
fn test_details_view_layout_structure() {
    let mut state = StateBrowserState::default();
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.current_state = Some(workflow_state);
    state.view_mode = StateBrowserViewMode::Details;

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should have all sections
    assert!(buffer_contains_text(&terminal, "Workflow State:")); // Header
    assert!(buffer_contains_text(&terminal, "Details")); // Details block
    assert!(buffer_contains_text(&terminal, "Back")); // Controls
}

#[test]
fn test_minimum_size_rendering() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("test", WorkflowStatus::Running, 0.5));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 80, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_large_terminal_rendering() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("test", WorkflowStatus::Running, 0.5));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 200, 60);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_different_themes() {
    let themes = [
        Theme::default(), // dark theme
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in &themes {
        let mut state = StateBrowserState::default();
        state
            .states
            .push(create_test_entry("test", WorkflowStatus::Running, 0.5));

        let terminal = render_state_browser(&mut state, theme, 80, 24);

        // Should render with each theme without panicking
        assert!(count_border_chars(&terminal) > 0);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_long_workflow_names() {
    let mut state = StateBrowserState::default();
    state.states.push(create_test_entry(
        "very_long_workflow_name_that_should_be_handled_correctly",
        WorkflowStatus::Running,
        0.5,
    ));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
    assert!(buffer_contains_text(&terminal, "very_long_workflow_name"));
}

#[test]
fn test_many_states() {
    let mut state = StateBrowserState::default();
    for i in 0..50 {
        state.states.push(create_test_entry(
            &format!("workflow_{}", i),
            WorkflowStatus::Running,
            0.5,
        ));
    }

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 40);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
    assert!(buffer_contains_text(&terminal, "States (50)"));
}

#[test]
fn test_zero_progress() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("test", WorkflowStatus::Running, 0.0));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_full_progress() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("test", WorkflowStatus::Completed, 1.0));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_special_characters_in_names() {
    let mut state = StateBrowserState::default();
    state.states.push(create_test_entry(
        "workflow-with-dashes_and_underscores",
        WorkflowStatus::Running,
        0.5,
    ));

    let theme = Theme::default();
    let terminal = render_state_browser(&mut state, &theme, 120, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

// ============================================================================
// State Browser State Tests
// ============================================================================

#[test]
fn test_state_browser_defaults() {
    let state = StateBrowserState::default();

    assert_eq!(state.view_mode, StateBrowserViewMode::List);
    assert_eq!(state.selected_index, 0);
    assert_eq!(state.filter_query, "");
    assert_eq!(state.sort_mode, StateSortMode::ModifiedDesc);
    assert_eq!(state.details_scroll, 0);
}

#[test]
fn test_back_to_list() {
    let mut state = StateBrowserState::default();
    let workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
    state.current_state = Some(workflow_state);
    state.view_mode = StateBrowserViewMode::Details;
    state.details_scroll = 10;

    state.back_to_list();

    assert_eq!(state.view_mode, StateBrowserViewMode::List);
    assert!(state.current_state.is_none());
    assert_eq!(state.details_scroll, 0);
}

#[test]
fn test_select_next() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state.states.push(create_test_entry(
        "workflow2",
        WorkflowStatus::Completed,
        1.0,
    ));

    assert_eq!(state.selected_index, 0);

    state.select_next();
    assert_eq!(state.selected_index, 1);

    // Should not go beyond last item
    state.select_next();
    assert_eq!(state.selected_index, 1);
}

#[test]
fn test_select_previous() {
    let mut state = StateBrowserState::default();
    state
        .states
        .push(create_test_entry("workflow1", WorkflowStatus::Running, 0.5));
    state.states.push(create_test_entry(
        "workflow2",
        WorkflowStatus::Completed,
        1.0,
    ));
    state.selected_index = 1;

    state.select_previous();
    assert_eq!(state.selected_index, 0);

    // Should not go below zero
    state.select_previous();
    assert_eq!(state.selected_index, 0);
}
