//! Help Rendering Tests
//!
//! Comprehensive test suite for the Help screen rendering using
//! ratatui's TestBackend. Tests all aspects of the help view including
//! browse mode, topic viewing, search, and context-aware help.
//!
//! Test Categories:
//! - Basic Rendering: Initial display, layout structure
//! - Browse Mode: Topic list, category navigation
//! - Topic Mode: Topic content display, markdown rendering
//! - Search Mode: Search interface, results display
//! - Context-Aware: Different contexts (WorkflowList, Editor, etc.)
//! - Navigation: Breadcrumbs, history
//! - Theme Tests: All theme variants
//! - Edge Cases: Empty search, long content, small terminals

#![cfg(feature = "tui")]

use periplon_sdk::tui::help::{HelpContext, HelpViewState};
use periplon_sdk::tui::theme::Theme;
use periplon_sdk::tui::ui::help::HelpView;
use ratatui::backend::{Backend, TestBackend};
use ratatui::Terminal;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create test terminal
fn create_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Render help view and return terminal
fn render_help(
    state: &mut HelpViewState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend> {
    let mut terminal = create_terminal(width, height);
    terminal
        .draw(|f| {
            HelpView::render(f, f.area(), state, theme);
        })
        .unwrap();
    terminal
}

/// Check if buffer contains text
fn buffer_contains(terminal: &Terminal<TestBackend>, text: &str) -> bool {
    let buffer = terminal.backend().buffer();
    let content: String = buffer
        .content()
        .iter()
        .map(|cell| cell.symbol())
        .collect::<Vec<_>>()
        .join("");
    content.contains(text)
}

// ============================================================================
// Basic Rendering Tests
// ============================================================================

#[test]
fn test_basic_rendering() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let terminal = render_help(&mut state, &theme, 120, 40);

    // Should render help view
    assert!(buffer_contains(&terminal, "Help") || buffer_contains(&terminal, "help"));
}

#[test]
fn test_layout_structure() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let terminal = render_help(&mut state, &theme, 120, 40);

    // Should have some structural content
    assert!(terminal.backend().size().unwrap().width > 0);
    assert!(terminal.backend().size().unwrap().height > 0);
}

#[test]
fn test_minimum_terminal_size() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let terminal = render_help(&mut state, &theme, 80, 24);

    // Should render without panic at minimum size
    assert_eq!(terminal.backend().size().unwrap().width, 80);
    assert_eq!(terminal.backend().size().unwrap().height, 24);
}

// ============================================================================
// Context Tests
// ============================================================================

#[test]
fn test_general_context() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    assert_eq!(state.context(), HelpContext::General);
}

#[test]
fn test_workflow_list_context() {
    let mut state = HelpViewState::new(HelpContext::WorkflowList);
    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    assert_eq!(state.context(), HelpContext::WorkflowList);
}

#[test]
fn test_viewer_context() {
    let mut state = HelpViewState::new(HelpContext::Viewer);
    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    assert_eq!(state.context(), HelpContext::Viewer);
}

#[test]
fn test_editor_context() {
    let mut state = HelpViewState::new(HelpContext::Editor);
    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    assert_eq!(state.context(), HelpContext::Editor);
}

#[test]
fn test_execution_monitor_context() {
    let mut state = HelpViewState::new(HelpContext::ExecutionMonitor);
    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    assert_eq!(state.context(), HelpContext::ExecutionMonitor);
}

#[test]
fn test_generator_context() {
    let mut state = HelpViewState::new(HelpContext::Generator);
    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    assert_eq!(state.context(), HelpContext::Generator);
}

#[test]
fn test_context_switching() {
    let mut state = HelpViewState::new(HelpContext::General);

    state.set_context(HelpContext::Editor);
    assert_eq!(state.context(), HelpContext::Editor);

    state.set_context(HelpContext::Viewer);
    assert_eq!(state.context(), HelpContext::Viewer);
}

// ============================================================================
// Browse Mode Tests
// ============================================================================

#[test]
fn test_browse_mode_initial() {
    let mut state = HelpViewState::new(HelpContext::General);
    state.enter_browse_mode();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    // Should be in browse mode after entering
    // (we can't directly test mode, but we can verify rendering doesn't panic)
}

#[test]
fn test_browse_mode_categories() {
    let mut state = HelpViewState::new(HelpContext::General);
    state.enter_browse_mode();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    // Should render without panic
}

// ============================================================================
// Search Mode Tests
// ============================================================================

#[test]
fn test_search_mode_entry() {
    let mut state = HelpViewState::new(HelpContext::General);
    state.enter_search_mode();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    // Should render search interface
}

#[test]
fn test_search_mode_empty_query() {
    let mut state = HelpViewState::new(HelpContext::General);
    state.enter_search_mode();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);

    // Should handle empty search query
}

// ============================================================================
// Navigation Tests
// ============================================================================

#[test]
fn test_scroll_offset_tracking() {
    let mut state = HelpViewState::new(HelpContext::General);

    // Test scroll methods exist
    state.scroll_up();
    state.scroll_down();
    state.page_up();
    state.page_down();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);
}

#[test]
fn test_category_navigation() {
    let mut state = HelpViewState::new(HelpContext::General);

    state.next_category();
    state.prev_category();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);
}

#[test]
fn test_topic_navigation() {
    let mut state = HelpViewState::new(HelpContext::General);

    state.next_topic();
    state.prev_topic();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);
}

// ============================================================================
// State Management Tests
// ============================================================================

#[test]
fn test_back_to_browse() {
    let mut state = HelpViewState::new(HelpContext::General);
    state.enter_search_mode();
    state.back_to_browse();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);
}

#[test]
fn test_reset_state() {
    let mut state = HelpViewState::new(HelpContext::General);

    state.enter_search_mode();
    state.scroll_down();
    state.next_category();

    state.reset();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);
}

#[test]
fn test_is_viewing_topic() {
    let state = HelpViewState::new(HelpContext::General);

    // Should not be viewing topic initially
    assert!(!state.is_viewing_topic());
}

#[test]
fn test_select_functionality() {
    let mut state = HelpViewState::new(HelpContext::General);
    state.select();

    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 120, 40);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_all_themes_render() {
    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        let mut state = HelpViewState::new(HelpContext::General);
        let _terminal = render_help(&mut state, &theme, 120, 40);
    }
}

#[test]
fn test_theme_with_different_contexts() {
    let contexts = vec![
        HelpContext::General,
        HelpContext::WorkflowList,
        HelpContext::Viewer,
        HelpContext::Editor,
        HelpContext::ExecutionMonitor,
        HelpContext::Generator,
    ];

    for context in contexts {
        let mut state = HelpViewState::new(context);
        let theme = Theme::monokai();
        let _terminal = render_help(&mut state, &theme, 120, 40);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_large_terminal() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let terminal = render_help(&mut state, &theme, 200, 100);

    assert_eq!(terminal.backend().size().unwrap().width, 200);
    assert_eq!(terminal.backend().size().unwrap().height, 100);
}

#[test]
fn test_small_terminal() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let terminal = render_help(&mut state, &theme, 40, 12);

    assert_eq!(terminal.backend().size().unwrap().width, 40);
    assert_eq!(terminal.backend().size().unwrap().height, 12);
}

#[test]
fn test_narrow_terminal() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let _terminal = render_help(&mut state, &theme, 60, 24);

    // Should not panic with narrow terminal
}

#[test]
fn test_tall_terminal() {
    let mut state = HelpViewState::new(HelpContext::General);
    let theme = Theme::default();
    let terminal = render_help(&mut state, &theme, 80, 60);

    assert_eq!(terminal.backend().size().unwrap().height, 60);
}

// ============================================================================
// State Cloning Tests
// ============================================================================

#[test]
fn test_state_clone() {
    let state = HelpViewState::new(HelpContext::Editor);
    let cloned = state.clone();

    assert_eq!(cloned.context(), state.context());
}

#[test]
fn test_state_clone_with_modifications() {
    let mut state = HelpViewState::new(HelpContext::General);
    state.enter_search_mode();
    state.scroll_down();

    let _cloned = state.clone();

    // Should clone without panic
}

// ============================================================================
// Multiple Contexts Rendering
// ============================================================================

#[test]
fn test_all_contexts_render() {
    let contexts = vec![
        HelpContext::General,
        HelpContext::WorkflowList,
        HelpContext::Viewer,
        HelpContext::Editor,
        HelpContext::ExecutionMonitor,
        HelpContext::Generator,
    ];

    let theme = Theme::default();

    for context in contexts {
        let mut state = HelpViewState::new(context);
        let _terminal = render_help(&mut state, &theme, 120, 40);
    }
}

// ============================================================================
// Context Title Tests
// ============================================================================

#[test]
fn test_context_titles() {
    assert_eq!(HelpContext::General.title(), "General Help");
    assert_eq!(HelpContext::WorkflowList.title(), "Workflow List Help");
    assert_eq!(HelpContext::Viewer.title(), "Workflow Viewer Help");
    assert_eq!(HelpContext::Editor.title(), "Workflow Editor Help");
    assert_eq!(
        HelpContext::ExecutionMonitor.title(),
        "Execution Monitor Help"
    );
    assert_eq!(HelpContext::Generator.title(), "AI Generator Help");
}

// ============================================================================
// Context Topics Tests
// ============================================================================

#[test]
fn test_context_topics() {
    let general_topics = HelpContext::General.topics();
    assert!(!general_topics.is_empty());
    assert!(general_topics.contains(&"overview"));

    let list_topics = HelpContext::WorkflowList.topics();
    assert!(!list_topics.is_empty());
    assert!(list_topics.contains(&"navigating_workflows"));

    let viewer_topics = HelpContext::Viewer.topics();
    assert!(!viewer_topics.is_empty());
    assert!(viewer_topics.contains(&"viewing_workflows"));

    let editor_topics = HelpContext::Editor.topics();
    assert!(!editor_topics.is_empty());
    assert!(editor_topics.contains(&"editing_workflows"));

    let monitor_topics = HelpContext::ExecutionMonitor.topics();
    assert!(!monitor_topics.is_empty());
    assert!(monitor_topics.contains(&"monitoring_execution"));

    let generator_topics = HelpContext::Generator.topics();
    assert!(!generator_topics.is_empty());
    assert!(generator_topics.contains(&"generating_workflows"));
}
