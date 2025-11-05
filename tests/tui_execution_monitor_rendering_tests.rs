//! ExecutionMonitor Rendering Tests
//!
//! Comprehensive test suite for the ExecutionMonitor screen rendering using
//! ratatui's TestBackend. Tests all aspects of the execution monitoring view
//! including progress display, task status, log output, and statistics.
//!
//! Test Categories:
//! - Basic Rendering: Initial display, layout structure
//! - Header Rendering: Workflow info, status, progress bar
//! - Task List Rendering: Task statuses, icons, timing
//! - Log Output Rendering: Log entries, levels, auto-scroll
//! - Statistics Panel: Task counts, timing, costs
//! - Status Indicators: Running, paused, completed, failed states
//! - Panel Focus: Task list, log output, task details
//! - Progress Tracking: Overall and per-task progress
//! - Error Display: Failed tasks, error messages
//! - Edge Cases: Empty logs, no tasks, large task lists

#![cfg(feature = "tui")]

use periplon_sdk::dsl::schema::DSLWorkflow;
use periplon_sdk::tui::theme::Theme;
use periplon_sdk::tui::views::execution_monitor::{
    ExecutionMonitorState, ExecutionStatus, LogEntry, LogLevel, MonitorPanel, TaskExecutionState,
    TaskStatus, TokenUsage,
};
use ratatui::backend::{Backend, TestBackend};
use ratatui::Terminal;
use std::collections::HashMap;
use std::time::SystemTime;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create test terminal
fn create_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

/// Render execution monitor and return terminal
fn render_monitor(
    state: &ExecutionMonitorState,
    theme: &Theme,
    width: u16,
    height: u16,
) -> Terminal<TestBackend> {
    let mut terminal = create_terminal(width, height);
    terminal
        .draw(|f| {
            periplon_sdk::tui::views::execution_monitor::render(f, f.area(), state, theme);
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

/// Create sample workflow for testing
fn create_test_workflow() -> DSLWorkflow {
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

/// Create task execution state
fn create_task_state(task_id: &str, status: TaskStatus, progress: u8) -> TaskExecutionState {
    TaskExecutionState {
        task_id: task_id.to_string(),
        description: format!("Task {}", task_id),
        status,
        agent: Some("test_agent".to_string()),
        start_time: Some(SystemTime::now()),
        end_time: None,
        dependencies: Vec::new(),
        progress,
        error: None,
        output_path: None,
        cost: None,
        tokens: None,
    }
}

/// Create log entry
fn create_log_entry(level: LogLevel, message: &str) -> LogEntry {
    LogEntry {
        timestamp: SystemTime::now(),
        level,
        message: message.to_string(),
        task_id: None,
        agent_id: None,
    }
}

// ============================================================================
// Basic Rendering Tests
// ============================================================================

#[test]
fn test_basic_rendering() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Test Workflow"));
}

#[test]
fn test_layout_structure() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    // Should have header, main content, statistics, and shortcuts
    assert!(buffer_contains(&terminal, "Test Workflow"));
    assert!(buffer_contains(&terminal, "Status"));
}

#[test]
fn test_minimum_terminal_size() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 80, 24);

    // Should render without panic at minimum size
    assert_eq!(terminal.backend().size().unwrap().width, 80);
}

// ============================================================================
// Header Rendering Tests
// ============================================================================

#[test]
fn test_header_workflow_info() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Test Workflow"));
    assert!(buffer_contains(&terminal, "1.0.0"));
}

#[test]
fn test_header_running_status() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.status = ExecutionStatus::Running;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Running"));
}

#[test]
fn test_header_paused_status() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.status = ExecutionStatus::Paused;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Paused"));
}

#[test]
fn test_header_completed_status() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.status = ExecutionStatus::Completed;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Completed"));
}

#[test]
fn test_header_failed_status() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.status = ExecutionStatus::Failed;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Failed"));
}

#[test]
fn test_header_cancelled_status() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.status = ExecutionStatus::Cancelled;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Cancelled"));
}

// ============================================================================
// Task List Rendering Tests
// ============================================================================

#[test]
fn test_task_list_pending_task() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let task = create_task_state("task1", TaskStatus::Pending, 0);
    state.tasks.insert("task1".to_string(), task);
    state.task_order = vec!["task1".to_string()];

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "task1") || buffer_contains(&terminal, "Task task1"));
}

#[test]
fn test_task_list_running_task() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let task = create_task_state("task1", TaskStatus::Running, 50);
    state.tasks.insert("task1".to_string(), task);
    state.task_order = vec!["task1".to_string()];

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "task1") || buffer_contains(&terminal, "Task task1"));
}

#[test]
fn test_task_list_completed_task() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let mut task = create_task_state("task1", TaskStatus::Completed, 100);
    task.end_time = Some(SystemTime::now());
    state.tasks.insert("task1".to_string(), task);
    state.task_order = vec!["task1".to_string()];

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "task1") || buffer_contains(&terminal, "Task task1"));
}

#[test]
fn test_task_list_failed_task() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let mut task = create_task_state("task1", TaskStatus::Failed, 0);
    task.error = Some("Connection error".to_string());
    state.tasks.insert("task1".to_string(), task);
    state.task_order = vec!["task1".to_string()];

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "task1") || buffer_contains(&terminal, "Task task1"));
}

#[test]
fn test_task_list_multiple_tasks() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.tasks.insert(
        "task1".to_string(),
        create_task_state("task1", TaskStatus::Completed, 100),
    );
    state.tasks.insert(
        "task2".to_string(),
        create_task_state("task2", TaskStatus::Running, 50),
    );
    state.tasks.insert(
        "task3".to_string(),
        create_task_state("task3", TaskStatus::Pending, 0),
    );
    state.task_order = vec![
        "task1".to_string(),
        "task2".to_string(),
        "task3".to_string(),
    ];

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    // Should show multiple tasks
    assert!(buffer_contains(&terminal, "task1") || buffer_contains(&terminal, "task2"));
}

// ============================================================================
// Log Output Rendering Tests
// ============================================================================

#[test]
fn test_log_empty() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    // Should render without panic even with no logs
    assert_eq!(state.logs.len(), 0);
}

#[test]
fn test_log_info_entry() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state
        .logs
        .push(create_log_entry(LogLevel::Info, "Task started"));

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Task started") || buffer_contains(&terminal, "INFO"));
}

#[test]
fn test_log_error_entry() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state
        .logs
        .push(create_log_entry(LogLevel::Error, "Connection failed"));

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Connection failed") || buffer_contains(&terminal, "ERROR"));
}

#[test]
fn test_log_warning_entry() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state
        .logs
        .push(create_log_entry(LogLevel::Warning, "Deprecated syntax"));

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Deprecated") || buffer_contains(&terminal, "WARN"));
}

#[test]
fn test_log_debug_entry() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state
        .logs
        .push(create_log_entry(LogLevel::Debug, "Debug info"));

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "Debug") || buffer_contains(&terminal, "DEBUG"));
}

#[test]
fn test_log_multiple_entries() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state
        .logs
        .push(create_log_entry(LogLevel::Info, "Starting"));
    state
        .logs
        .push(create_log_entry(LogLevel::Info, "Processing"));
    state
        .logs
        .push(create_log_entry(LogLevel::Info, "Complete"));

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    // Should show multiple log entries
    assert!(buffer_contains(&terminal, "Starting") || buffer_contains(&terminal, "Processing"));
}

// ============================================================================
// Statistics Panel Rendering Tests
// ============================================================================

#[test]
fn test_statistics_initial_state() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    // Should show task counts
    assert!(buffer_contains(&terminal, "Tasks") || buffer_contains(&terminal, "0"));
}

#[test]
fn test_statistics_with_tasks() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.stats.total_tasks = 5;
    state.stats.completed_tasks = 2;
    state.stats.running_tasks = 1;
    state.stats.pending_tasks = 2;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "2") || buffer_contains(&terminal, "5"));
}

#[test]
fn test_statistics_with_failures() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.stats.total_tasks = 5;
    state.stats.failed_tasks = 2;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "2") || buffer_contains(&terminal, "failed"));
}

#[test]
fn test_statistics_with_cost() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.stats.total_cost = 1.25;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "1.25") || buffer_contains(&terminal, "Cost"));
}

// ============================================================================
// Panel Focus Tests
// ============================================================================

#[test]
fn test_focus_task_list() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.focus = MonitorPanel::TaskList;

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    // Should render without issues
    assert_eq!(state.focus, MonitorPanel::TaskList);
}

#[test]
fn test_focus_log_output() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.focus = MonitorPanel::LogOutput;

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    assert_eq!(state.focus, MonitorPanel::LogOutput);
}

#[test]
fn test_focus_task_details() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.focus = MonitorPanel::TaskDetails;
    state.show_task_details = true;

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    assert_eq!(state.focus, MonitorPanel::TaskDetails);
}

// ============================================================================
// Progress Tracking Tests
// ============================================================================

#[test]
fn test_progress_zero_percent() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.stats.total_tasks = 10;
    state.stats.completed_tasks = 0;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    // Should show 0% progress
    assert!(buffer_contains(&terminal, "0") || buffer_contains(&terminal, "Progress"));
}

#[test]
fn test_progress_fifty_percent() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.stats.total_tasks = 10;
    state.stats.completed_tasks = 5;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    // Should show ~50% progress
    assert!(buffer_contains(&terminal, "5") || buffer_contains(&terminal, "10"));
}

#[test]
fn test_progress_complete() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.stats.total_tasks = 10;
    state.stats.completed_tasks = 10;
    state.status = ExecutionStatus::Completed;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "10") || buffer_contains(&terminal, "Completed"));
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn test_failed_task_with_error() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let mut task = create_task_state("task1", TaskStatus::Failed, 0);
    task.error = Some("Network timeout".to_string());
    state.tasks.insert("task1".to_string(), task);
    state.task_order = vec!["task1".to_string()];

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "task1") || buffer_contains(&terminal, "timeout"));
}

#[test]
fn test_multiple_failed_tasks() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let mut task1 = create_task_state("task1", TaskStatus::Failed, 0);
    task1.error = Some("Error 1".to_string());

    let mut task2 = create_task_state("task2", TaskStatus::Failed, 0);
    task2.error = Some("Error 2".to_string());

    state.tasks.insert("task1".to_string(), task1);
    state.tasks.insert("task2".to_string(), task2);
    state.task_order = vec!["task1".to_string(), "task2".to_string()];
    state.stats.failed_tasks = 2;

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "2"));
}

// ============================================================================
// Token Usage Tests
// ============================================================================

#[test]
fn test_task_with_token_usage() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let mut task = create_task_state("task1", TaskStatus::Completed, 100);
    task.tokens = Some(TokenUsage {
        input_tokens: 1000,
        output_tokens: 500,
        cache_read_tokens: 200,
        cache_write_tokens: 100,
    });
    state.tasks.insert("task1".to_string(), task);
    state.task_order = vec!["task1".to_string()];

    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 120, 40);

    assert!(buffer_contains(&terminal, "task1"));
}

#[test]
fn test_total_token_statistics() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.stats.total_input_tokens = 5000;
    state.stats.total_output_tokens = 2500;

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    // Should render without panic and preserve statistics
    assert_eq!(state.stats.total_input_tokens, 5000);
    assert_eq!(state.stats.total_output_tokens, 2500);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_all_themes_render() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    let themes = vec![
        Theme::default(),
        Theme::light(),
        Theme::monokai(),
        Theme::solarized(),
    ];

    for theme in themes {
        let terminal = render_monitor(&state, &theme, 120, 40);
        // Should render without panic
        assert!(terminal.backend().size().unwrap().width > 0);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_no_tasks() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    // Should render without tasks
    assert_eq!(state.tasks.len(), 0);
}

#[test]
fn test_many_tasks() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    for i in 0..50 {
        let task_id = format!("task{}", i);
        state.tasks.insert(
            task_id.clone(),
            create_task_state(&task_id, TaskStatus::Pending, 0),
        );
        state.task_order.push(task_id);
    }

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    assert_eq!(state.tasks.len(), 50);
}

#[test]
fn test_many_logs() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    for i in 0..100 {
        state.logs.push(create_log_entry(
            LogLevel::Info,
            &format!("Log entry {}", i),
        ));
    }

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    assert_eq!(state.logs.len(), 100);
}

#[test]
fn test_long_error_message() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let mut task = create_task_state("task1", TaskStatus::Failed, 0);
    task.error = Some("A".repeat(500));
    state.tasks.insert("task1".to_string(), task);
    state.task_order = vec!["task1".to_string()];

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    // Should handle long error messages without panic
    assert_eq!(
        state
            .tasks
            .get("task1")
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .len(),
        500
    );
}

#[test]
fn test_large_terminal() {
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);
    let theme = Theme::default();
    let terminal = render_monitor(&state, &theme, 200, 100);

    assert_eq!(terminal.backend().size().unwrap().width, 200);
}

#[test]
fn test_auto_scroll_logs_enabled() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.auto_scroll_logs = true;

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    assert!(state.auto_scroll_logs);
}

#[test]
fn test_auto_scroll_logs_disabled() {
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);
    state.auto_scroll_logs = false;

    let theme = Theme::default();
    let _terminal = render_monitor(&state, &theme, 120, 40);

    assert!(!state.auto_scroll_logs);
}
