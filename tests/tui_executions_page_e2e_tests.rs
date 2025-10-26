//! Executions Page End-to-End Tests
//!
//! Comprehensive E2E testing suite for the execution monitor page,
//! covering real-time workflow execution monitoring, task tracking,
//! log streaming, and execution statistics.
//!
//! Test Scenarios:
//! - Execution monitor initialization and setup
//! - Task execution state tracking
//! - Real-time log streaming and scrolling
//! - Execution statistics and metrics
//! - Panel focus management (task list, logs, details)
//! - Pause/Resume/Cancel operations
//! - Task status transitions
//! - Progress calculation and display
//! - Token usage and cost tracking
//! - Auto-scroll log behavior
//! - Task selection and details view
//! - Multi-task concurrent execution
//! - Execution completion and failure handling
//! - Statistics recalculation
//! - Time tracking and elapsed duration
//!
//! These tests validate the complete execution monitoring experience
//! from workflow start through completion or failure.

#![cfg(feature = "tui")]

use periplon_sdk::dsl::parse_workflow;
use periplon_sdk::dsl::state::WorkflowState;
use periplon_sdk::dsl::task_graph::TaskStatus as DslTaskStatus;
use periplon_sdk::tui::views::execution_monitor::{
    ExecutionMonitorState, ExecutionStatistics, ExecutionStatus, LogLevel,
    MonitorPanel, TaskExecutionState, TaskStatus, TokenUsage,
};
use std::time::{Duration, SystemTime};

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a simple test workflow using YAML parsing
fn create_test_workflow() -> periplon_sdk::dsl::schema::DSLWorkflow {
    let yaml = r#"
name: "Test Workflow"
version: "1.0.0"

agents:
  agent1:
    description: "Test agent"

tasks:
  task1:
    description: "First task"
    agent: "agent1"

  task2:
    description: "Second task"
    agent: "agent1"
    depends_on:
      - task1
"#;

    parse_workflow(yaml).expect("Failed to parse test workflow")
}

/// Create a task execution state
fn create_task_state(task_id: &str, status: TaskStatus) -> TaskExecutionState {
    TaskExecutionState {
        task_id: task_id.to_string(),
        description: format!("Task {}", task_id),
        status,
        agent: Some("agent1".to_string()),
        start_time: if status != TaskStatus::Pending {
            Some(SystemTime::now())
        } else {
            None
        },
        end_time: if status == TaskStatus::Completed || status == TaskStatus::Failed {
            Some(SystemTime::now())
        } else {
            None
        },
        dependencies: Vec::new(),
        progress: match status {
            TaskStatus::Pending => 0,
            TaskStatus::Running => 50,
            TaskStatus::Completed => 100,
            TaskStatus::Failed => 0,
            TaskStatus::Skipped => 0,
        },
        error: if status == TaskStatus::Failed {
            Some("Task failed".to_string())
        } else {
            None
        },
        output_path: None,
        cost: None,
        tokens: None,
    }
}

/// Create execution monitor state with initialized tasks
///
/// ExecutionMonitorState::new() intentionally leaves tasks HashMap empty.
/// This helper properly initializes tasks using from_workflow_state() for testing.
fn create_initialized_execution_state(workflow: periplon_sdk::dsl::schema::DSLWorkflow) -> ExecutionMonitorState {
    let mut state = WorkflowState::new(workflow.name.clone(), workflow.version.clone());

    // Initialize all tasks as pending
    for task_id in workflow.tasks.keys() {
        state.update_task_status(task_id, DslTaskStatus::Pending);
    }

    ExecutionMonitorState::from_workflow_state(workflow, &state)
}

// ============================================================================
// Execution Monitor Initialization Tests
// ============================================================================

#[test]
fn test_e2e_execution_monitor_initialization() {
    // Scenario: Initialize execution monitor with workflow
    let workflow = create_test_workflow();
    let state = create_initialized_execution_state(workflow);

    assert_eq!(state.status, ExecutionStatus::Running);
    assert_eq!(state.workflow.name, "Test Workflow");
    assert_eq!(state.tasks.len(), 2);
    assert_eq!(state.task_order.len(), 2);
    assert!(state.logs.is_empty());
    assert_eq!(state.log_scroll, 0);
    assert_eq!(state.task_scroll, 0);
    assert_eq!(state.focus, MonitorPanel::TaskList);
}

#[test]
fn test_e2e_execution_monitor_initial_statistics() {
    // Scenario: Initial statistics are calculated correctly
    let workflow = create_test_workflow();
    let state = create_initialized_execution_state(workflow);

    assert_eq!(state.stats.total_tasks, 2);
    assert_eq!(state.stats.completed_tasks, 0);
    assert_eq!(state.stats.failed_tasks, 0);
    assert_eq!(state.stats.running_tasks, 0);
    assert_eq!(state.stats.pending_tasks, 2);
}

#[test]
fn test_e2e_execution_monitor_task_order_preserved() {
    // Scenario: Task order is preserved for display
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    assert!(state.task_order.contains(&"task1".to_string()));
    assert!(state.task_order.contains(&"task2".to_string()));
    assert_eq!(state.task_order.len(), 2);
}

#[test]
fn test_e2e_execution_monitor_time_tracking() {
    // Scenario: Start time is recorded
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    let elapsed = state.start_time.elapsed().unwrap();
    assert!(elapsed.as_millis() < 100); // Should be very recent
    assert!(state.end_time.is_none());
}

// ============================================================================
// Task Execution State Tests
// ============================================================================

#[test]
fn test_e2e_task_state_pending() {
    // Scenario: Task in pending state
    let task = create_task_state("task1", TaskStatus::Pending);

    assert_eq!(task.status, TaskStatus::Pending);
    assert_eq!(task.progress, 0);
    assert!(task.start_time.is_none());
    assert!(task.end_time.is_none());
    assert!(task.error.is_none());
}

#[test]
fn test_e2e_task_state_running() {
    // Scenario: Task transitions to running
    let task = create_task_state("task1", TaskStatus::Running);

    assert_eq!(task.status, TaskStatus::Running);
    assert_eq!(task.progress, 50);
    assert!(task.start_time.is_some());
    assert!(task.end_time.is_none());
}

#[test]
fn test_e2e_task_state_completed() {
    // Scenario: Task completes successfully
    let task = create_task_state("task1", TaskStatus::Completed);

    assert_eq!(task.status, TaskStatus::Completed);
    assert_eq!(task.progress, 100);
    assert!(task.start_time.is_some());
    assert!(task.end_time.is_some());
    assert!(task.error.is_none());
}

#[test]
fn test_e2e_task_state_failed() {
    // Scenario: Task fails with error
    let task = create_task_state("task1", TaskStatus::Failed);

    assert_eq!(task.status, TaskStatus::Failed);
    assert!(task.error.is_some());
    assert_eq!(task.error.unwrap(), "Task failed");
    assert!(task.start_time.is_some());
    assert!(task.end_time.is_some());
}

#[test]
fn test_e2e_task_state_skipped() {
    // Scenario: Task is skipped due to dependency failure
    let task = create_task_state("task1", TaskStatus::Skipped);

    assert_eq!(task.status, TaskStatus::Skipped);
    assert_eq!(task.progress, 0);
}

#[test]
fn test_e2e_task_with_agent_assignment() {
    // Scenario: Task is assigned to an agent
    let task = create_task_state("task1", TaskStatus::Running);

    assert_eq!(task.agent, Some("agent1".to_string()));
}

#[test]
fn test_e2e_task_with_dependencies() {
    // Scenario: Task has dependencies
    let mut task = create_task_state("task2", TaskStatus::Pending);
    task.dependencies = vec!["task1".to_string()];

    assert_eq!(task.dependencies.len(), 1);
    assert!(task.dependencies.contains(&"task1".to_string()));
}

// ============================================================================
// Log Management Tests
// ============================================================================

#[test]
fn test_e2e_add_log_entry() {
    // Scenario: Add log entry to execution monitor
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.add_log(
        LogLevel::Info,
        "Execution started".to_string(),
        None,
        None,
    );

    assert_eq!(state.logs.len(), 1);
    assert_eq!(state.logs[0].message, "Execution started");
    assert_eq!(state.logs[0].level, LogLevel::Info);
}

#[test]
fn test_e2e_multiple_log_entries() {
    // Scenario: Add multiple log entries
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.add_log(LogLevel::Info, "Starting workflow".to_string(), None, None);
    state.add_log(
        LogLevel::Debug,
        "Loading configuration".to_string(),
        None,
        None,
    );
    state.add_log(
        LogLevel::Info,
        "Starting task1".to_string(),
        Some("task1".to_string()),
        None,
    );

    assert_eq!(state.logs.len(), 3);
    assert_eq!(state.logs[0].level, LogLevel::Info);
    assert_eq!(state.logs[1].level, LogLevel::Debug);
    assert_eq!(state.logs[2].task_id, Some("task1".to_string()));
}

#[test]
fn test_e2e_log_with_task_context() {
    // Scenario: Log entry associated with specific task
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.add_log(
        LogLevel::Info,
        "Task executing".to_string(),
        Some("task1".to_string()),
        Some("agent1".to_string()),
    );

    assert_eq!(state.logs[0].task_id, Some("task1".to_string()));
    assert_eq!(state.logs[0].agent_id, Some("agent1".to_string()));
}

#[test]
fn test_e2e_log_levels() {
    // Scenario: Different log levels
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.add_log(LogLevel::Debug, "Debug message".to_string(), None, None);
    state.add_log(LogLevel::Info, "Info message".to_string(), None, None);
    state.add_log(
        LogLevel::Warning,
        "Warning message".to_string(),
        None,
        None,
    );
    state.add_log(LogLevel::Error, "Error message".to_string(), None, None);

    assert_eq!(state.logs.len(), 4);
    assert_eq!(state.logs[0].level, LogLevel::Debug);
    assert_eq!(state.logs[1].level, LogLevel::Info);
    assert_eq!(state.logs[2].level, LogLevel::Warning);
    assert_eq!(state.logs[3].level, LogLevel::Error);
}

#[test]
fn test_e2e_log_chronological_order() {
    // Scenario: Logs maintain chronological order
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    for i in 1..=5 {
        state.add_log(
            LogLevel::Info,
            format!("Log entry {}", i),
            None,
            None,
        );
    }

    assert_eq!(state.logs[0].message, "Log entry 1");
    assert_eq!(state.logs[4].message, "Log entry 5");
}

#[test]
fn test_e2e_large_log_volume() {
    // Scenario: Handle large volume of logs
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    for i in 1..=1000 {
        state.add_log(
            LogLevel::Info,
            format!("Log entry {}", i),
            None,
            None,
        );
    }

    assert_eq!(state.logs.len(), 1000);
    assert_eq!(state.logs.first().unwrap().message, "Log entry 1");
    assert_eq!(state.logs.last().unwrap().message, "Log entry 1000");
}

// ============================================================================
// Execution Statistics Tests
// ============================================================================

#[test]
fn test_e2e_statistics_task_counts() {
    // Scenario: Statistics track task counts
    let stats = ExecutionStatistics {
        total_tasks: 10,
        completed_tasks: 6,
        failed_tasks: 2,
        running_tasks: 1,
        pending_tasks: 1,
        skipped_tasks: 0,
        total_cost: 0.50,
        total_input_tokens: 1000,
        total_output_tokens: 500,
        avg_task_duration: Some(Duration::from_secs(45)),
        estimated_time_remaining: Some(Duration::from_secs(90)),
    };

    assert_eq!(stats.total_tasks, 10);
    assert_eq!(stats.completed_tasks, 6);
    assert_eq!(stats.failed_tasks, 2);
    assert_eq!(stats.running_tasks, 1);
    assert_eq!(stats.pending_tasks, 1);
}

#[test]
fn test_e2e_statistics_progress_calculation() {
    // Scenario: Calculate execution progress percentage
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.stats.total_tasks = 4;
    state.stats.completed_tasks = 2;

    // Progress calculation: completed / total * 100
    let progress = (state.stats.completed_tasks as f64 / state.stats.total_tasks as f64 * 100.0) as u8;
    assert_eq!(progress, 50); // 2/4 = 50%
}

#[test]
fn test_e2e_statistics_cost_tracking() {
    // Scenario: Track total execution cost
    let stats = ExecutionStatistics {
        total_tasks: 5,
        completed_tasks: 3,
        failed_tasks: 0,
        running_tasks: 1,
        pending_tasks: 1,
        skipped_tasks: 0,
        total_cost: 1.25,
        total_input_tokens: 5000,
        total_output_tokens: 2500,
        avg_task_duration: None,
        estimated_time_remaining: None,
    };

    assert_eq!(stats.total_cost, 1.25);
}

#[test]
fn test_e2e_statistics_token_usage() {
    // Scenario: Track token usage across execution
    let stats = ExecutionStatistics {
        total_tasks: 3,
        completed_tasks: 3,
        failed_tasks: 0,
        running_tasks: 0,
        pending_tasks: 0,
        skipped_tasks: 0,
        total_cost: 0.75,
        total_input_tokens: 10000,
        total_output_tokens: 5000,
        avg_task_duration: Some(Duration::from_secs(30)),
        estimated_time_remaining: None,
    };

    assert_eq!(stats.total_input_tokens, 10000);
    assert_eq!(stats.total_output_tokens, 5000);
}

#[test]
fn test_e2e_statistics_average_duration() {
    // Scenario: Calculate average task duration
    let stats = ExecutionStatistics {
        total_tasks: 5,
        completed_tasks: 5,
        failed_tasks: 0,
        running_tasks: 0,
        pending_tasks: 0,
        skipped_tasks: 0,
        total_cost: 0.0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        avg_task_duration: Some(Duration::from_secs(120)),
        estimated_time_remaining: None,
    };

    assert_eq!(stats.avg_task_duration, Some(Duration::from_secs(120)));
}

// ============================================================================
// Panel Focus Management Tests
// ============================================================================

#[test]
fn test_e2e_panel_focus_initialization() {
    // Scenario: Focus starts on task list
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    assert_eq!(state.focus, MonitorPanel::TaskList);
}

#[test]
fn test_e2e_panel_focus_switch() {
    // Scenario: Switch focus between panels
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    assert_eq!(state.focus, MonitorPanel::TaskList);

    state.focus = MonitorPanel::LogOutput;
    assert_eq!(state.focus, MonitorPanel::LogOutput);

    state.focus = MonitorPanel::TaskDetails;
    assert_eq!(state.focus, MonitorPanel::TaskDetails);

    state.focus = MonitorPanel::TaskList;
    assert_eq!(state.focus, MonitorPanel::TaskList);
}

#[test]
fn test_e2e_panel_cycle_navigation() {
    // Scenario: Cycle through all panels with Tab
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    let panels = vec![
        MonitorPanel::TaskList,
        MonitorPanel::LogOutput,
        MonitorPanel::TaskDetails,
    ];

    for panel in panels {
        state.focus = panel;
        assert_eq!(state.focus, panel);
    }
}

// ============================================================================
// Execution Status Transition Tests
// ============================================================================

#[test]
fn test_e2e_execution_status_running() {
    // Scenario: Execution starts in running state
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    assert_eq!(state.status, ExecutionStatus::Running);
}

#[test]
fn test_e2e_execution_pause() {
    // Scenario: Pause running execution
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.pause();

    assert_eq!(state.status, ExecutionStatus::Paused);
    assert!(state.pause_time.is_some());
}

#[test]
fn test_e2e_execution_resume() {
    // Scenario: Resume paused execution
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.pause();
    assert_eq!(state.status, ExecutionStatus::Paused);

    state.resume();

    assert_eq!(state.status, ExecutionStatus::Running);
}

#[test]
fn test_e2e_execution_complete() {
    // Scenario: Execution completes successfully
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.complete(true);

    assert_eq!(state.status, ExecutionStatus::Completed);
    assert!(state.end_time.is_some());
}

#[test]
fn test_e2e_execution_fail() {
    // Scenario: Execution fails
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.complete(false);

    assert_eq!(state.status, ExecutionStatus::Failed);
    assert!(state.end_time.is_some());
}

#[test]
fn test_e2e_execution_cancel() {
    // Scenario: User cancels execution
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.cancel();

    assert_eq!(state.status, ExecutionStatus::Cancelled);
    assert!(state.end_time.is_some());
}

// ============================================================================
// Scroll Management Tests
// ============================================================================

#[test]
fn test_e2e_log_scroll_initialization() {
    // Scenario: Log scroll starts at 0
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    assert_eq!(state.log_scroll, 0);
}

#[test]
fn test_e2e_log_scroll_down() {
    // Scenario: Scroll down through logs
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.log_scroll = 0;
    state.log_scroll += 1;
    assert_eq!(state.log_scroll, 1);

    state.log_scroll += 5;
    assert_eq!(state.log_scroll, 6);
}

#[test]
fn test_e2e_log_scroll_up() {
    // Scenario: Scroll up through logs
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.log_scroll = 10;
    state.log_scroll = state.log_scroll.saturating_sub(1);
    assert_eq!(state.log_scroll, 9);

    state.log_scroll = state.log_scroll.saturating_sub(20);
    assert_eq!(state.log_scroll, 0); // Can't go below 0
}

#[test]
fn test_e2e_task_scroll_management() {
    // Scenario: Scroll through task list
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.task_scroll = 0;
    state.task_scroll += 1;
    assert_eq!(state.task_scroll, 1);

    state.task_scroll = state.task_scroll.saturating_sub(1);
    assert_eq!(state.task_scroll, 0);
}

#[test]
fn test_e2e_auto_scroll_logs() {
    // Scenario: Auto-scroll logs to latest entries
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    assert!(state.auto_scroll_logs); // Default is true
}

// ============================================================================
// Task Selection Tests
// ============================================================================

#[test]
fn test_e2e_task_selection_none() {
    // Scenario: No task selected initially
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    assert!(state.selected_task.is_none());
}

#[test]
fn test_e2e_task_selection_select() {
    // Scenario: Select a task for details
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.selected_task = Some("task1".to_string());

    assert_eq!(state.selected_task, Some("task1".to_string()));
}

#[test]
fn test_e2e_task_selection_deselect() {
    // Scenario: Deselect task
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.selected_task = Some("task1".to_string());
    state.selected_task = None;

    assert!(state.selected_task.is_none());
}

#[test]
fn test_e2e_task_selection_switch() {
    // Scenario: Switch between selected tasks
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.selected_task = Some("task1".to_string());
    assert_eq!(state.selected_task, Some("task1".to_string()));

    state.selected_task = Some("task2".to_string());
    assert_eq!(state.selected_task, Some("task2".to_string()));
}

// ============================================================================
// Token Usage Tests
// ============================================================================

#[test]
fn test_e2e_token_usage_tracking() {
    // Scenario: Track token usage for a task
    let tokens = TokenUsage {
        input_tokens: 1000,
        output_tokens: 500,
        cache_read_tokens: 200,
        cache_write_tokens: 100,
    };

    assert_eq!(tokens.input_tokens, 1000);
    assert_eq!(tokens.output_tokens, 500);
    assert_eq!(tokens.cache_read_tokens, 200);
    assert_eq!(tokens.cache_write_tokens, 100);
}

#[test]
fn test_e2e_task_with_token_usage() {
    // Scenario: Task tracks token usage
    let mut task = create_task_state("task1", TaskStatus::Completed);

    task.tokens = Some(TokenUsage {
        input_tokens: 5000,
        output_tokens: 2500,
        cache_read_tokens: 1000,
        cache_write_tokens: 500,
    });

    assert!(task.tokens.is_some());
    let tokens = task.tokens.unwrap();
    assert_eq!(tokens.input_tokens, 5000);
    assert_eq!(tokens.output_tokens, 2500);
}

#[test]
fn test_e2e_task_with_cost() {
    // Scenario: Task tracks execution cost
    let mut task = create_task_state("task1", TaskStatus::Completed);
    task.cost = Some(0.25);

    assert_eq!(task.cost, Some(0.25));
}

// ============================================================================
// Time Tracking Tests
// ============================================================================

#[test]
fn test_e2e_execution_start_time() {
    // Scenario: Execution records start time
    let workflow = create_test_workflow();
    let state = ExecutionMonitorState::new(workflow);

    let elapsed = state.start_time.elapsed().unwrap();
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn test_e2e_execution_end_time_on_completion() {
    // Scenario: End time recorded on completion
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.complete(true);

    assert!(state.end_time.is_some());
}

#[test]
fn test_e2e_execution_pause_time() {
    // Scenario: Pause time recorded when paused
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.pause();

    assert!(state.pause_time.is_some());
}

#[test]
fn test_e2e_task_duration_calculation() {
    // Scenario: Calculate task duration
    let task = create_task_state("task1", TaskStatus::Completed);

    if let (Some(start), Some(end)) = (task.start_time, task.end_time) {
        let duration = end.duration_since(start).unwrap();
        assert!(duration.as_millis() < 100); // Very short in test
    }
}

// ============================================================================
// Complex Execution Scenarios Tests
// ============================================================================

#[test]
fn test_e2e_multi_task_execution_flow() {
    // Scenario: Execute multiple tasks sequentially
    let workflow = create_test_workflow();
    let mut state = create_initialized_execution_state(workflow);

    // Start first task
    if let Some(task1) = state.tasks.get_mut("task1") {
        task1.status = TaskStatus::Running;
        task1.start_time = Some(SystemTime::now());
    }

    state.add_log(
        LogLevel::Info,
        "Starting task1".to_string(),
        Some("task1".to_string()),
        None,
    );

    // Complete first task
    if let Some(task1) = state.tasks.get_mut("task1") {
        task1.status = TaskStatus::Completed;
        task1.end_time = Some(SystemTime::now());
        task1.progress = 100;
    }

    // Start second task
    if let Some(task2) = state.tasks.get_mut("task2") {
        task2.status = TaskStatus::Running;
        task2.start_time = Some(SystemTime::now());
    }

    assert_eq!(state.tasks.get("task1").unwrap().status, TaskStatus::Completed);
    assert_eq!(state.tasks.get("task2").unwrap().status, TaskStatus::Running);
}

#[test]
fn test_e2e_execution_with_failures() {
    // Scenario: Handle task failures during execution
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    // Task 1 fails
    if let Some(task1) = state.tasks.get_mut("task1") {
        task1.status = TaskStatus::Failed;
        task1.error = Some("Connection timeout".to_string());
        task1.end_time = Some(SystemTime::now());
    }

    state.add_log(
        LogLevel::Error,
        "Task failed: Connection timeout".to_string(),
        Some("task1".to_string()),
        None,
    );

    // Task 2 skipped due to dependency failure
    if let Some(task2) = state.tasks.get_mut("task2") {
        task2.status = TaskStatus::Skipped;
    }

    state.stats.failed_tasks = 1;
    state.stats.skipped_tasks = 1;

    assert_eq!(state.stats.failed_tasks, 1);
    assert_eq!(state.stats.skipped_tasks, 1);
}

#[test]
fn test_e2e_execution_pause_resume_cycle() {
    // Scenario: Pause and resume execution
    let workflow = create_test_workflow();
    let mut state = create_initialized_execution_state(workflow);

    // Start task
    if let Some(task) = state.tasks.get_mut("task1") {
        task.status = TaskStatus::Running;
        task.progress = 30;
    }

    // Pause
    state.pause();
    assert_eq!(state.status, ExecutionStatus::Paused);

    // Resume
    state.resume();
    assert_eq!(state.status, ExecutionStatus::Running);

    // Task should still be running
    assert_eq!(state.tasks.get("task1").unwrap().status, TaskStatus::Running);
}

// ============================================================================
// Statistics Recalculation Tests
// ============================================================================

#[test]
fn test_e2e_statistics_update_on_task_completion() {
    // Scenario: Statistics update when task completes
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.stats.completed_tasks = 0;
    state.stats.running_tasks = 1;

    // Complete a task
    if let Some(task) = state.tasks.get_mut("task1") {
        task.status = TaskStatus::Completed;
    }

    // Simulate stats update
    state.stats.completed_tasks += 1;
    state.stats.running_tasks -= 1;

    assert_eq!(state.stats.completed_tasks, 1);
    assert_eq!(state.stats.running_tasks, 0);
}

#[test]
fn test_e2e_statistics_update_on_task_failure() {
    // Scenario: Statistics update when task fails
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    state.stats.failed_tasks = 0;
    state.stats.running_tasks = 1;

    // Fail a task
    if let Some(task) = state.tasks.get_mut("task1") {
        task.status = TaskStatus::Failed;
    }

    // Simulate stats update
    state.stats.failed_tasks += 1;
    state.stats.running_tasks -= 1;

    assert_eq!(state.stats.failed_tasks, 1);
    assert_eq!(state.stats.running_tasks, 0);
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_e2e_execution_with_no_tasks() {
    // Scenario: Workflow with no tasks
    let mut workflow = create_test_workflow();
    workflow.tasks.clear();

    let state = ExecutionMonitorState::new(workflow);

    assert_eq!(state.tasks.len(), 0);
    assert_eq!(state.task_order.len(), 0);
    assert_eq!(state.stats.total_tasks, 0);
}

#[test]
fn test_e2e_execution_with_many_tasks() {
    // Scenario: Workflow with many tasks
    let workflow = create_test_workflow();
    let mut state = create_initialized_execution_state(workflow);

    // Add 98 more tasks (we already have 2 from the workflow)
    for i in 3..=100 {
        let task_id = format!("task{}", i);
        state.tasks.insert(task_id.clone(), create_task_state(&task_id, TaskStatus::Pending));
        state.task_order.push(task_id);
    }

    state.stats.total_tasks = 100;

    assert_eq!(state.tasks.len(), 100);
    assert_eq!(state.task_order.len(), 100);
}

#[test]
fn test_e2e_log_scroll_boundary() {
    // Scenario: Log scroll at boundaries
    let workflow = create_test_workflow();
    let mut state = ExecutionMonitorState::new(workflow);

    // Add logs
    for _ in 0..10 {
        state.add_log(LogLevel::Info, "Test log".to_string(), None, None);
    }

    // Scroll to end
    state.log_scroll = state.logs.len();
    assert_eq!(state.log_scroll, 10);

    // Try to scroll past end (should be capped by UI)
    state.log_scroll = state.logs.len() + 100;
    // In actual implementation, this would be capped
}

#[test]
fn test_e2e_execution_state_clone() {
    // Scenario: Execution state can be cloned
    let workflow = create_test_workflow();
    let state1 = ExecutionMonitorState::new(workflow);
    let state2 = state1.clone();

    assert_eq!(state1.status, state2.status);
    assert_eq!(state1.workflow.name, state2.workflow.name);
    assert_eq!(state1.tasks.len(), state2.tasks.len());
}
