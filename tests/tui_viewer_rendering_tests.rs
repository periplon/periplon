//! Comprehensive ratatui backend tests for viewer screen rendering
//!
//! Tests viewer layout, condensed/full modes, syntax highlighting, and visual elements
//! using the ratatui TestBackend.

#![cfg(feature = "tui")]

use periplon_sdk::dsl::{
    AgentSpec, DSLWorkflow, InputSpec, OutputDataSource, OutputSpec, TaskSpec,
};
use periplon_sdk::tui::state::{ViewerState, WorkflowViewMode};
use periplon_sdk::tui::theme::Theme;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::collections::HashMap;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test terminal with specified dimensions
fn create_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Render viewer and return terminal for assertions
fn render_viewer(
    workflow: &DSLWorkflow,
    state: &ViewerState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend> {
    let mut terminal = create_test_terminal(width, height);

    terminal
        .draw(|frame| {
            let area = frame.area();
            periplon_sdk::tui::ui::viewer::render(frame, area, workflow, state, theme);
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

/// Create a minimal test workflow
fn create_minimal_workflow() -> DSLWorkflow {
    DSLWorkflow {
        provider: Default::default(),
        model: None,
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

/// Create a complete test workflow with agents and tasks
fn create_complete_workflow() -> DSLWorkflow {
    let mut agents = HashMap::new();
    agents.insert(
        "researcher".to_string(),
        AgentSpec {
            provider: None,
            description: "Research and gather information".to_string(),
            model: Some("claude-sonnet-4-5".to_string()),
            system_prompt: None,
            tools: vec!["Read".to_string(), "WebSearch".to_string()],
            permissions: Default::default(),
            max_turns: Some(10),
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        },
    );

    agents.insert(
        "analyzer".to_string(),
        AgentSpec {
            provider: None,
            description: "Analyze data".to_string(),
            model: None,
            system_prompt: None,
            tools: vec!["Read".to_string()],
            permissions: Default::default(),
            max_turns: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
        },
    );

    let mut tasks = HashMap::new();
    tasks.insert(
        "research_task".to_string(),
        TaskSpec {
            description: "Perform research".to_string(),
            agent: Some("researcher".to_string()),
            depends_on: vec![],
            subtasks: vec![],
            output: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            ..Default::default()
        },
    );

    tasks.insert(
        "analysis_task".to_string(),
        TaskSpec {
            description: "Analyze results".to_string(),
            agent: Some("analyzer".to_string()),
            depends_on: vec!["research_task".to_string()],
            subtasks: vec![],
            output: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            ..Default::default()
        },
    );

    DSLWorkflow {
        provider: Default::default(),
        model: None,
        name: "Complete Workflow".to_string(),
        version: "2.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: Some("/workspace".to_string()),
        create_cwd: None,
        secrets: HashMap::new(),
        agents,
        tasks,
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
// Basic Rendering Tests
// ============================================================================

#[test]
fn test_viewer_basic_rendering() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should have borders
    assert!(count_border_chars(&terminal) > 0);

    // Should show workflow name
    assert!(buffer_contains_text(&terminal, "Test Workflow"));

    // Should show version
    assert!(buffer_contains_text(&terminal, "1.0.0"));
}

#[test]
fn test_viewer_empty_workflow() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_viewer_with_complete_workflow() {
    let workflow = create_complete_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 30);

    // Should show workflow name
    assert!(buffer_contains_text(&terminal, "Complete Workflow"));

    // Should show version
    assert!(buffer_contains_text(&terminal, "2.0.0"));

    // Should show agents section
    assert!(buffer_contains_text(&terminal, "Agents"));

    // Should show tasks section
    assert!(buffer_contains_text(&terminal, "Tasks"));
}

// ============================================================================
// Header Rendering Tests
// ============================================================================

#[test]
fn test_header_shows_workflow_name() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Workflow:"));
    assert!(buffer_contains_text(&terminal, "Test Workflow"));
}

#[test]
fn test_header_shows_version() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Version:"));
    assert!(buffer_contains_text(&terminal, "1.0.0"));
}

#[test]
fn test_header_shows_view_mode_condensed() {
    let workflow = create_minimal_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "View:"));
    assert!(buffer_contains_text(&terminal, "Condensed"));
}

#[test]
fn test_header_shows_view_mode_full() {
    let workflow = create_minimal_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Full;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    assert!(buffer_contains_text(&terminal, "Full YAML"));
}

// ============================================================================
// Condensed View Tests
// ============================================================================

#[test]
fn test_condensed_view_metadata_section() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show metadata section
    assert!(buffer_contains_text(&terminal, "Workflow Metadata"));
    assert!(buffer_contains_text(&terminal, "Name:"));
    assert!(buffer_contains_text(&terminal, "DSL Version:"));
    assert!(buffer_contains_text(&terminal, "Working Dir:"));
    assert!(buffer_contains_text(&terminal, "/workspace"));
}

#[test]
fn test_condensed_view_agents_section() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show agents section with count
    assert!(buffer_contains_text(&terminal, "Agents"));
    assert!(buffer_contains_text(&terminal, "2")); // Two agents

    // Should show agent details
    assert!(buffer_contains_text(&terminal, "researcher"));
    assert!(buffer_contains_text(&terminal, "analyzer"));
    assert!(buffer_contains_text(&terminal, "Research and gather"));
}

#[test]
fn test_condensed_view_agent_details() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show agent properties
    assert!(buffer_contains_text(&terminal, "Description:"));
    assert!(buffer_contains_text(&terminal, "Model:"));
    assert!(buffer_contains_text(&terminal, "Tools:"));
    assert!(buffer_contains_text(&terminal, "Max Turns:"));

    // Should show specific values
    assert!(buffer_contains_text(&terminal, "claude-sonnet-4-5"));
    assert!(buffer_contains_text(&terminal, "Read"));
    assert!(buffer_contains_text(&terminal, "WebSearch"));
}

#[test]
fn test_condensed_view_tasks_section() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show tasks section with count
    assert!(buffer_contains_text(&terminal, "Tasks"));
    assert!(buffer_contains_text(&terminal, "2")); // Two tasks

    // Should show task IDs
    assert!(buffer_contains_text(&terminal, "research_task"));
    assert!(buffer_contains_text(&terminal, "analysis_task"));
}

#[test]
fn test_condensed_view_task_details() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show task properties
    assert!(buffer_contains_text(&terminal, "Description:"));
    assert!(buffer_contains_text(&terminal, "Agent:"));
    assert!(buffer_contains_text(&terminal, "Depends On:"));

    // Should show specific values
    assert!(buffer_contains_text(&terminal, "Perform research"));
    assert!(buffer_contains_text(&terminal, "Analyze results"));
}

#[test]
fn test_condensed_view_with_inputs() {
    let mut workflow = create_minimal_workflow();
    let mut inputs = HashMap::new();
    inputs.insert(
        "project_name".to_string(),
        InputSpec {
            param_type: "string".to_string(),
            default: None,
            required: true,
            description: None,
        },
    );
    inputs.insert(
        "api_key".to_string(),
        InputSpec {
            param_type: "string".to_string(),
            default: None,
            required: false,
            description: None,
        },
    );
    workflow.inputs = inputs;

    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 30);

    // Should show inputs section
    assert!(buffer_contains_text(&terminal, "Inputs"));
    assert!(buffer_contains_text(&terminal, "2")); // Two inputs
    assert!(buffer_contains_text(&terminal, "project_name"));
    assert!(buffer_contains_text(&terminal, "api_key"));
    assert!(buffer_contains_text(&terminal, "required"));
    assert!(buffer_contains_text(&terminal, "optional"));
}

#[test]
fn test_condensed_view_with_outputs() {
    let mut workflow = create_minimal_workflow();
    let mut outputs = HashMap::new();
    outputs.insert(
        "result".to_string(),
        OutputSpec {
            source: OutputDataSource::File {
                path: "result.json".to_string(),
            },
            description: None,
        },
    );
    outputs.insert(
        "summary".to_string(),
        OutputSpec {
            source: OutputDataSource::File {
                path: "summary.txt".to_string(),
            },
            description: None,
        },
    );
    workflow.outputs = outputs;

    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 30);

    // Should show outputs section
    assert!(buffer_contains_text(&terminal, "Outputs"));
    assert!(buffer_contains_text(&terminal, "result"));
    assert!(buffer_contains_text(&terminal, "summary"));
}

#[test]
fn test_condensed_view_section_headers() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Section headers use ━━━ decorators
    assert!(buffer_contains_text(&terminal, "━━━"));
}

// ============================================================================
// Full YAML View Tests
// ============================================================================

#[test]
fn test_full_view_shows_yaml() {
    let workflow = create_minimal_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Full;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should show YAML keys
    assert!(buffer_contains_text(&terminal, "name:"));
    assert!(buffer_contains_text(&terminal, "version:"));
    // Note: dsl_version is skipped if it's the default "1.0.0"
}

#[test]
fn test_full_view_yaml_structure() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Full;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show YAML structure
    assert!(buffer_contains_text(&terminal, "agents:"));
    assert!(buffer_contains_text(&terminal, "tasks:"));
    assert!(buffer_contains_text(&terminal, "cwd:"));
}

#[test]
fn test_full_view_nested_structure() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Full;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 50);

    // Should show nested YAML (indented keys)
    assert!(buffer_contains_text(&terminal, "researcher:"));
    assert!(buffer_contains_text(&terminal, "description:"));
}

// ============================================================================
// Status Bar Tests
// ============================================================================

#[test]
fn test_status_bar_condensed_mode() {
    let workflow = create_minimal_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should show condensed mode shortcuts
    assert!(buffer_contains_text(&terminal, "Tab"));
    assert!(buffer_contains_text(&terminal, "Full YAML"));
    assert!(buffer_contains_text(&terminal, "Scroll"));
    assert!(buffer_contains_text(&terminal, "Esc"));
}

#[test]
fn test_status_bar_full_mode() {
    let workflow = create_minimal_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Full;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should show full mode shortcuts
    assert!(buffer_contains_text(&terminal, "Tab"));
    assert!(buffer_contains_text(&terminal, "Condensed"));
}

#[test]
fn test_status_bar_navigation_hints() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should show navigation hints
    assert!(buffer_contains_text(&terminal, "PgUp/PgDn"));
    assert!(buffer_contains_text(&terminal, "Home/End"));
    assert!(buffer_contains_text(&terminal, "Back"));
}

// ============================================================================
// Layout Tests
// ============================================================================

#[test]
fn test_viewer_layout_structure() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should have three sections: Header, Content, Status
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_viewer_minimum_size() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();

    // Test with small terminal
    let terminal = render_viewer(&workflow, &state, &theme, 60, 15);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_viewer_large_terminal() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();

    // Test with large terminal
    let terminal = render_viewer(&workflow, &state, &theme, 200, 60);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

// ============================================================================
// Scroll Tests
// ============================================================================

#[test]
fn test_scroll_offset_zero() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.scroll = 0;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show content from top
    assert!(buffer_contains_text(&terminal, "Workflow Metadata"));
}

#[test]
fn test_scroll_offset_nonzero() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.scroll = 10; // Scrolled down
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should render with scroll offset
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_scrollbar_with_long_content() {
    let mut workflow = create_complete_workflow();

    // Add many tasks to make content longer
    for i in 0..20 {
        workflow.tasks.insert(
            format!("task_{}", i),
            TaskSpec {
                description: format!("Task {}", i),
                agent: None,
                depends_on: vec![],
                subtasks: vec![],
                output: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                ..Default::default()
            },
        );
    }

    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should show scrollbar indicators
    assert!(buffer_contains_text(&terminal, "↑") || buffer_contains_text(&terminal, "↓"));
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_viewer_with_different_themes() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();

    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

        // Should render with all themes
        assert!(count_border_chars(&terminal) > 0);
    }
}

// ============================================================================
// View Mode Toggle Tests
// ============================================================================

#[test]
fn test_view_mode_enum_values() {
    assert_ne!(WorkflowViewMode::Condensed, WorkflowViewMode::Full);
}

#[test]
fn test_viewer_state_toggle() {
    let mut state = ViewerState::new();
    assert_eq!(state.view_mode, WorkflowViewMode::Condensed);

    state.toggle_view_mode();
    assert_eq!(state.view_mode, WorkflowViewMode::Full);

    state.toggle_view_mode();
    assert_eq!(state.view_mode, WorkflowViewMode::Condensed);
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_workflow_with_long_names() {
    let mut workflow = create_minimal_workflow();
    workflow.name =
        "This is a very long workflow name that might exceed typical display widths".to_string();

    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should render without panicking
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_workflow_with_special_characters() {
    let mut workflow = create_minimal_workflow();
    workflow.name = "Test 日本語 🎉".to_string();

    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should handle special characters
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_empty_agents_and_tasks() {
    let workflow = create_minimal_workflow();
    let state = ViewerState::new();
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 24);

    // Should render even with no agents or tasks
    assert!(count_border_chars(&terminal) > 0);
}

#[test]
fn test_viewer_state_defaults() {
    let state = ViewerState::new();

    assert_eq!(state.scroll, 0);
    assert_eq!(state.view_mode, WorkflowViewMode::Condensed);
    assert!(state.expanded.is_empty());
}

#[test]
fn test_viewer_state_reset() {
    let mut state = ViewerState::new();
    state.scroll = 10;
    state.view_mode = WorkflowViewMode::Full;
    state.expanded.push("test".to_string());

    state.reset();

    assert_eq!(state.scroll, 0);
    assert_eq!(state.view_mode, WorkflowViewMode::Condensed);
    assert!(state.expanded.is_empty());
}

// ============================================================================
// Symbol Tests
// ============================================================================

#[test]
fn test_agent_symbol_in_condensed_view() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show agent symbol (◆)
    assert!(buffer_contains_text(&terminal, "◆"));
}

#[test]
fn test_task_symbol_in_condensed_view() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show task symbol (▶)
    assert!(buffer_contains_text(&terminal, "▶"));
}

// ============================================================================
// Content Completeness Tests
// ============================================================================

#[test]
fn test_all_workflow_metadata_displayed() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // All metadata fields should be present
    assert!(buffer_contains_text(&terminal, "Name:"));
    assert!(buffer_contains_text(&terminal, "Version:"));
    assert!(buffer_contains_text(&terminal, "DSL Version:"));
    assert!(buffer_contains_text(&terminal, "Working Dir:"));
}

#[test]
fn test_agent_count_displayed() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // Should show count in section header
    assert!(buffer_contains_text(&terminal, "(2)") || buffer_contains_text(&terminal, "2"));
}

#[test]
fn test_task_dependencies_displayed() {
    let workflow = create_complete_workflow();
    let mut state = ViewerState::new();
    state.view_mode = WorkflowViewMode::Condensed;
    let theme = Theme::default();
    let terminal = render_viewer(&workflow, &state, &theme, 80, 40);

    // analysis_task depends on research_task
    assert!(buffer_contains_text(&terminal, "research_task"));
}
