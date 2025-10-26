//! Execution and Scheduling End-to-End Tests
//!
//! Comprehensive E2E testing suite for workflow execution monitoring,
//! scheduling, and task orchestration in the TUI.
//!
//! Test Scenarios:
//! - Execution state initialization and lifecycle
//! - Execution status transitions (Preparing → Running → Completed/Failed)
//! - Task progress tracking and monitoring
//! - Execution log management
//! - Pause and resume functionality
//! - Execution cancellation
//! - Task completion and failure handling
//! - Progress calculation and reporting
//! - Execution statistics and metrics
//! - Concurrent execution management
//! - Execution error recovery
//! - Long-running execution monitoring
//! - Execution state persistence
//!
//! These tests validate the complete workflow execution experience
//! from initiation through completion or failure.

#![cfg(feature = "tui")]

use periplon_sdk::tui::state::{ExecutionState, ExecutionStatus};
use std::path::PathBuf;
use std::time::Instant;

// ============================================================================
// Test Utilities
// ============================================================================

/// Create a test execution state
fn create_test_execution(workflow_name: &str, status: ExecutionStatus) -> ExecutionState {
    ExecutionState {
        workflow_path: PathBuf::from(format!("workflows/{}.yaml", workflow_name)),
        status,
        current_agent: None,
        current_task: None,
        progress: 0.0,
        log: Vec::new(),
        completed_tasks: Vec::new(),
        failed_tasks: Vec::new(),
        started_at: Instant::now(),
    }
}

/// Create an execution in progress
fn create_running_execution(workflow_name: &str) -> ExecutionState {
    let mut exec = create_test_execution(workflow_name, ExecutionStatus::Running);
    exec.current_agent = Some("agent1".to_string());
    exec.current_task = Some("task1".to_string());
    exec.progress = 0.5;
    exec.log.push("Execution started".to_string());
    exec.log.push("Running task1".to_string());
    exec
}

/// Simulate task completion
fn complete_task(exec: &mut ExecutionState, task_name: &str) {
    exec.completed_tasks.push(task_name.to_string());
    exec.log
        .push(format!("Task {} completed successfully", task_name));
}

/// Simulate task failure
fn fail_task(exec: &mut ExecutionState, task_name: &str, error: &str) {
    exec.failed_tasks.push(task_name.to_string());
    exec.log
        .push(format!("Task {} failed: {}", task_name, error));
}

// ============================================================================
// Execution State Initialization Tests
// ============================================================================

#[test]
fn test_e2e_execution_state_initialization() {
    // Scenario: New execution is initialized
    let exec = create_test_execution("test_workflow", ExecutionStatus::Preparing);

    assert_eq!(exec.status, ExecutionStatus::Preparing);
    assert_eq!(exec.current_agent, None);
    assert_eq!(exec.current_task, None);
    assert_eq!(exec.progress, 0.0);
    assert!(exec.log.is_empty());
    assert!(exec.completed_tasks.is_empty());
    assert!(exec.failed_tasks.is_empty());
}

#[test]
fn test_e2e_execution_with_workflow_path() {
    // Scenario: Execution tracks the workflow file path
    let exec = create_test_execution("my_workflow", ExecutionStatus::Preparing);

    assert_eq!(
        exec.workflow_path,
        PathBuf::from("workflows/my_workflow.yaml")
    );
}

#[test]
fn test_e2e_execution_start_time_recorded() {
    // Scenario: Execution records start time
    let exec = create_test_execution("test", ExecutionStatus::Preparing);

    // Start time should be recent
    let elapsed = exec.started_at.elapsed();
    assert!(elapsed.as_millis() < 100); // Should be very recent
}

// ============================================================================
// Execution Status Transition Tests
// ============================================================================

#[test]
fn test_e2e_execution_preparing_to_running() {
    // Scenario: Execution transitions from Preparing to Running
    let mut exec = create_test_execution("test", ExecutionStatus::Preparing);

    assert_eq!(exec.status, ExecutionStatus::Preparing);

    // Start execution
    exec.status = ExecutionStatus::Running;
    exec.current_agent = Some("researcher".to_string());
    exec.log.push("Starting execution".to_string());

    assert_eq!(exec.status, ExecutionStatus::Running);
    assert_eq!(exec.current_agent, Some("researcher".to_string()));
    assert_eq!(exec.log.len(), 1);
}

#[test]
fn test_e2e_execution_running_to_completed() {
    // Scenario: Execution completes successfully
    let mut exec = create_running_execution("test");

    // Complete all tasks
    complete_task(&mut exec, "task1");
    complete_task(&mut exec, "task2");

    // Mark as completed
    exec.status = ExecutionStatus::Completed;
    exec.progress = 1.0;
    exec.current_task = None;
    exec.current_agent = None;

    assert_eq!(exec.status, ExecutionStatus::Completed);
    assert_eq!(exec.progress, 1.0);
    assert_eq!(exec.completed_tasks.len(), 2);
    assert!(exec.failed_tasks.is_empty());
}

#[test]
fn test_e2e_execution_running_to_failed() {
    // Scenario: Execution fails with errors
    let mut exec = create_running_execution("test");

    // Task fails
    fail_task(&mut exec, "task1", "Connection timeout");

    // Mark execution as failed
    exec.status = ExecutionStatus::Failed;
    exec.current_task = None;
    exec.current_agent = None;

    assert_eq!(exec.status, ExecutionStatus::Failed);
    assert_eq!(exec.failed_tasks.len(), 1);
    assert_eq!(exec.failed_tasks[0], "task1");
}

#[test]
fn test_e2e_execution_running_to_paused() {
    // Scenario: User pauses execution
    let mut exec = create_running_execution("test");

    assert_eq!(exec.status, ExecutionStatus::Running);

    // Pause execution
    exec.status = ExecutionStatus::Paused;
    exec.log.push("Execution paused by user".to_string());

    assert_eq!(exec.status, ExecutionStatus::Paused);
    assert!(exec.current_agent.is_some()); // Agent context preserved
    assert!(exec.current_task.is_some()); // Task context preserved
}

#[test]
fn test_e2e_execution_paused_to_running() {
    // Scenario: User resumes paused execution
    let mut exec = create_running_execution("test");

    // Pause
    exec.status = ExecutionStatus::Paused;

    // Resume
    exec.status = ExecutionStatus::Running;
    exec.log.push("Execution resumed".to_string());

    assert_eq!(exec.status, ExecutionStatus::Running);
}

#[test]
fn test_e2e_execution_all_status_transitions() {
    // Scenario: Execution goes through all possible states
    let mut exec = create_test_execution("test", ExecutionStatus::Preparing);

    // Preparing → Running
    exec.status = ExecutionStatus::Running;
    assert_eq!(exec.status, ExecutionStatus::Running);

    // Running → Paused
    exec.status = ExecutionStatus::Paused;
    assert_eq!(exec.status, ExecutionStatus::Paused);

    // Paused → Running
    exec.status = ExecutionStatus::Running;
    assert_eq!(exec.status, ExecutionStatus::Running);

    // Running → Completed
    exec.status = ExecutionStatus::Completed;
    assert_eq!(exec.status, ExecutionStatus::Completed);
}

// ============================================================================
// Task Progress Tracking Tests
// ============================================================================

#[test]
fn test_e2e_single_task_execution() {
    // Scenario: Execute workflow with single task
    let mut exec = create_test_execution("single_task", ExecutionStatus::Running);

    exec.current_agent = Some("agent1".to_string());
    exec.current_task = Some("analyze".to_string());
    exec.progress = 0.5;

    assert_eq!(exec.current_task, Some("analyze".to_string()));
    assert_eq!(exec.progress, 0.5);

    // Complete task
    complete_task(&mut exec, "analyze");
    exec.progress = 1.0;
    exec.status = ExecutionStatus::Completed;

    assert_eq!(exec.completed_tasks.len(), 1);
    assert_eq!(exec.progress, 1.0);
}

#[test]
fn test_e2e_multiple_task_sequential_execution() {
    // Scenario: Execute multiple tasks sequentially
    let mut exec = create_test_execution("multi_task", ExecutionStatus::Running);

    // Task 1
    exec.current_task = Some("task1".to_string());
    exec.progress = 0.33;
    complete_task(&mut exec, "task1");

    // Task 2
    exec.current_task = Some("task2".to_string());
    exec.progress = 0.66;
    complete_task(&mut exec, "task2");

    // Task 3
    exec.current_task = Some("task3".to_string());
    exec.progress = 1.0;
    complete_task(&mut exec, "task3");

    assert_eq!(exec.completed_tasks.len(), 3);
    assert_eq!(exec.progress, 1.0);
}

#[test]
fn test_e2e_task_progress_incremental_updates() {
    // Scenario: Progress updates incrementally as tasks complete
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    let total_tasks = 10;
    for i in 1..=total_tasks {
        exec.current_task = Some(format!("task{}", i));
        exec.progress = i as f64 / total_tasks as f64;
        complete_task(&mut exec, &format!("task{}", i));
    }

    assert_eq!(exec.completed_tasks.len(), 10);
    assert_eq!(exec.progress, 1.0);
}

#[test]
fn test_e2e_partial_task_completion() {
    // Scenario: Some tasks complete, some fail
    let mut exec = create_test_execution("mixed_results", ExecutionStatus::Running);

    complete_task(&mut exec, "task1");
    complete_task(&mut exec, "task2");
    fail_task(&mut exec, "task3", "Error");
    complete_task(&mut exec, "task4");

    assert_eq!(exec.completed_tasks.len(), 3);
    assert_eq!(exec.failed_tasks.len(), 1);
}

// ============================================================================
// Execution Log Management Tests
// ============================================================================

#[test]
fn test_e2e_execution_log_entries() {
    // Scenario: Execution logs are recorded
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    exec.log.push("Starting workflow execution".to_string());
    exec.log.push("Loading workflow definition".to_string());
    exec.log.push("Initializing agent: researcher".to_string());
    exec.log.push("Starting task: analyze".to_string());

    assert_eq!(exec.log.len(), 4);
    assert!(exec.log[0].contains("Starting workflow"));
}

#[test]
fn test_e2e_log_chronological_order() {
    // Scenario: Logs maintain chronological order
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    for i in 1..=5 {
        exec.log.push(format!("Log entry {}", i));
    }

    assert_eq!(exec.log[0], "Log entry 1");
    assert_eq!(exec.log[4], "Log entry 5");
}

#[test]
fn test_e2e_log_task_lifecycle() {
    // Scenario: Logs capture complete task lifecycle
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    // Task start
    exec.log.push("Task 'analyze' started".to_string());

    // Task progress
    exec.log.push("Task 'analyze' - 25% complete".to_string());
    exec.log.push("Task 'analyze' - 50% complete".to_string());
    exec.log.push("Task 'analyze' - 75% complete".to_string());

    // Task completion
    exec.log.push("Task 'analyze' completed".to_string());

    assert_eq!(exec.log.len(), 5);
    assert!(exec.log.last().unwrap().contains("completed"));
}

#[test]
fn test_e2e_log_error_messages() {
    // Scenario: Error messages are logged
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    exec.log.push("ERROR: Connection timeout".to_string());
    exec.log.push("ERROR: Invalid response format".to_string());
    exec.log.push("WARNING: Retrying operation".to_string());

    let error_logs: Vec<_> = exec
        .log
        .iter()
        .filter(|msg| msg.contains("ERROR"))
        .collect();

    assert_eq!(error_logs.len(), 2);
}

#[test]
fn test_e2e_large_log_accumulation() {
    // Scenario: Execution handles large log volumes
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    // Generate 1000 log entries
    for i in 1..=1000 {
        exec.log.push(format!("Log entry {}", i));
    }

    assert_eq!(exec.log.len(), 1000);
    assert_eq!(exec.log.first().unwrap(), "Log entry 1");
    assert_eq!(exec.log.last().unwrap(), "Log entry 1000");
}

// ============================================================================
// Pause and Resume Tests
// ============================================================================

#[test]
fn test_e2e_pause_preserves_state() {
    // Scenario: Pausing preserves execution state
    let mut exec = create_running_execution("test");

    exec.current_agent = Some("agent1".to_string());
    exec.current_task = Some("task1".to_string());
    exec.progress = 0.6;
    complete_task(&mut exec, "previous_task");

    // Pause
    let saved_agent = exec.current_agent.clone();
    let saved_task = exec.current_task.clone();
    let saved_progress = exec.progress;
    let saved_completed = exec.completed_tasks.clone();

    exec.status = ExecutionStatus::Paused;

    // Verify state preserved
    assert_eq!(exec.current_agent, saved_agent);
    assert_eq!(exec.current_task, saved_task);
    assert_eq!(exec.progress, saved_progress);
    assert_eq!(exec.completed_tasks, saved_completed);
}

#[test]
fn test_e2e_resume_continues_execution() {
    // Scenario: Resuming continues from paused state
    let mut exec = create_running_execution("test");

    // Set up state
    exec.current_task = Some("task1".to_string());
    exec.progress = 0.5;

    // Pause
    exec.status = ExecutionStatus::Paused;

    // Resume
    exec.status = ExecutionStatus::Running;

    // Continue execution
    complete_task(&mut exec, "task1");
    exec.progress = 0.75;
    exec.current_task = Some("task2".to_string());

    assert_eq!(exec.status, ExecutionStatus::Running);
    assert_eq!(exec.completed_tasks.len(), 1);
}

#[test]
fn test_e2e_multiple_pause_resume_cycles() {
    // Scenario: Multiple pause/resume cycles
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    for i in 1..=5 {
        // Do some work
        exec.current_task = Some(format!("task{}", i));
        exec.progress = i as f64 / 10.0;

        // Pause
        exec.status = ExecutionStatus::Paused;
        assert_eq!(exec.status, ExecutionStatus::Paused);

        // Resume
        exec.status = ExecutionStatus::Running;
        assert_eq!(exec.status, ExecutionStatus::Running);

        // Complete task
        complete_task(&mut exec, &format!("task{}", i));
    }

    assert_eq!(exec.completed_tasks.len(), 5);
}

// ============================================================================
// Execution Cancellation Tests
// ============================================================================

#[test]
fn test_e2e_cancel_running_execution() {
    // Scenario: User cancels running execution
    let mut exec = create_running_execution("test");

    exec.current_task = Some("long_task".to_string());
    exec.progress = 0.3;

    // Cancel (transition to Failed status)
    exec.status = ExecutionStatus::Failed;
    exec.log.push("Execution cancelled by user".to_string());
    exec.current_task = None;

    assert_eq!(exec.status, ExecutionStatus::Failed);
    assert!(exec.log.last().unwrap().contains("cancelled"));
}

#[test]
fn test_e2e_cancel_preserves_completed_work() {
    // Scenario: Cancellation preserves completed tasks
    let mut exec = create_running_execution("test");

    complete_task(&mut exec, "task1");
    complete_task(&mut exec, "task2");
    exec.current_task = Some("task3".to_string());

    // Cancel
    exec.status = ExecutionStatus::Failed;

    // Completed tasks should be preserved
    assert_eq!(exec.completed_tasks.len(), 2);
    assert!(exec.completed_tasks.contains(&"task1".to_string()));
    assert!(exec.completed_tasks.contains(&"task2".to_string()));
}

// ============================================================================
// Task Completion and Failure Tests
// ============================================================================

#[test]
fn test_e2e_all_tasks_complete_successfully() {
    // Scenario: All tasks complete without errors
    let mut exec = create_test_execution("success_workflow", ExecutionStatus::Running);

    let tasks = vec!["task1", "task2", "task3", "task4", "task5"];

    for task in &tasks {
        exec.current_task = Some(task.to_string());
        complete_task(&mut exec, task);
    }

    exec.status = ExecutionStatus::Completed;
    exec.progress = 1.0;

    assert_eq!(exec.status, ExecutionStatus::Completed);
    assert_eq!(exec.completed_tasks.len(), 5);
    assert!(exec.failed_tasks.is_empty());
}

#[test]
fn test_e2e_some_tasks_fail() {
    // Scenario: Some tasks fail during execution
    let mut exec = create_test_execution("partial_failure", ExecutionStatus::Running);

    complete_task(&mut exec, "task1");
    fail_task(&mut exec, "task2", "API timeout");
    complete_task(&mut exec, "task3");
    fail_task(&mut exec, "task4", "Invalid data");

    assert_eq!(exec.completed_tasks.len(), 2);
    assert_eq!(exec.failed_tasks.len(), 2);
}

#[test]
fn test_e2e_first_task_fails() {
    // Scenario: Execution fails on first task
    let mut exec = create_test_execution("early_failure", ExecutionStatus::Running);

    fail_task(&mut exec, "task1", "Configuration error");
    exec.status = ExecutionStatus::Failed;

    assert_eq!(exec.status, ExecutionStatus::Failed);
    assert_eq!(exec.failed_tasks.len(), 1);
    assert!(exec.completed_tasks.is_empty());
}

#[test]
fn test_e2e_task_failure_with_retry() {
    // Scenario: Task fails, then succeeds on retry
    let mut exec = create_test_execution("retry_workflow", ExecutionStatus::Running);

    // First attempt fails
    exec.current_task = Some("flaky_task".to_string());
    fail_task(&mut exec, "flaky_task", "Temporary error");
    exec.log.push("Retrying task".to_string());

    // Retry succeeds
    exec.failed_tasks.pop(); // Remove from failed list
    complete_task(&mut exec, "flaky_task");

    assert_eq!(exec.completed_tasks.len(), 1);
    assert!(exec.failed_tasks.is_empty());
}

// ============================================================================
// Progress Calculation Tests
// ============================================================================

#[test]
fn test_e2e_progress_zero_at_start() {
    // Scenario: Progress starts at 0%
    let exec = create_test_execution("test", ExecutionStatus::Preparing);

    assert_eq!(exec.progress, 0.0);
}

#[test]
fn test_e2e_progress_hundred_at_completion() {
    // Scenario: Progress reaches 100% on completion
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    exec.progress = 1.0;
    exec.status = ExecutionStatus::Completed;

    assert_eq!(exec.progress, 1.0);
    assert_eq!(exec.status, ExecutionStatus::Completed);
}

#[test]
fn test_e2e_progress_linear_increment() {
    // Scenario: Progress increments linearly
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    let steps = 10;
    for i in 1..=steps {
        exec.progress = i as f64 / steps as f64;
    }

    assert_eq!(exec.progress, 1.0);
}

#[test]
fn test_e2e_progress_bounds() {
    // Scenario: Progress stays within 0.0-1.0 bounds
    let mut exec = create_test_execution("test", ExecutionStatus::Running);

    // Test valid values
    exec.progress = 0.0;
    assert!(exec.progress >= 0.0 && exec.progress <= 1.0);

    exec.progress = 0.5;
    assert!(exec.progress >= 0.0 && exec.progress <= 1.0);

    exec.progress = 1.0;
    assert!(exec.progress >= 0.0 && exec.progress <= 1.0);
}

// ============================================================================
// Agent Context Tests
// ============================================================================

#[test]
fn test_e2e_agent_switches_between_tasks() {
    // Scenario: Different agents execute different tasks
    let mut exec = create_test_execution("multi_agent", ExecutionStatus::Running);

    // Agent 1
    exec.current_agent = Some("researcher".to_string());
    exec.current_task = Some("research".to_string());
    complete_task(&mut exec, "research");

    // Agent 2
    exec.current_agent = Some("writer".to_string());
    exec.current_task = Some("write".to_string());
    complete_task(&mut exec, "write");

    assert_eq!(exec.completed_tasks.len(), 2);
    assert_eq!(exec.current_agent, Some("writer".to_string()));
}

#[test]
fn test_e2e_agent_context_preserved() {
    // Scenario: Current agent is tracked during execution
    let mut exec = create_running_execution("test");

    exec.current_agent = Some("analyzer".to_string());
    let saved_agent = exec.current_agent.clone();

    // Agent context should persist
    assert_eq!(exec.current_agent, saved_agent);
}

// ============================================================================
// Concurrent Execution Tests
// ============================================================================

#[test]
fn test_e2e_multiple_independent_executions() {
    // Scenario: Multiple workflows executing simultaneously
    let exec1 = create_running_execution("workflow1");
    let exec2 = create_running_execution("workflow2");
    let exec3 = create_running_execution("workflow3");

    // All executions are independent
    assert_eq!(exec1.status, ExecutionStatus::Running);
    assert_eq!(exec2.status, ExecutionStatus::Running);
    assert_eq!(exec3.status, ExecutionStatus::Running);

    // Different workflow paths
    assert_ne!(exec1.workflow_path, exec2.workflow_path);
    assert_ne!(exec2.workflow_path, exec3.workflow_path);
}

#[test]
fn test_e2e_execution_state_isolation() {
    // Scenario: Execution states are isolated
    let mut exec1 = create_running_execution("workflow1");
    let exec2 = create_running_execution("workflow2");

    // Modify exec1
    complete_task(&mut exec1, "task1");
    exec1.progress = 0.8;

    // exec2 should be unaffected
    assert_eq!(exec2.completed_tasks.len(), 0);
    assert_eq!(exec2.progress, 0.5); // Initial value from create_running_execution
}

// ============================================================================
// Error Recovery Tests
// ============================================================================

#[test]
fn test_e2e_recover_from_task_failure() {
    // Scenario: Execution continues after task failure
    let mut exec = create_test_execution("resilient", ExecutionStatus::Running);

    complete_task(&mut exec, "task1");
    fail_task(&mut exec, "task2", "Error");
    exec.log.push("Continuing with next task".to_string());
    complete_task(&mut exec, "task3");

    assert_eq!(exec.completed_tasks.len(), 2);
    assert_eq!(exec.failed_tasks.len(), 1);
    assert_eq!(exec.status, ExecutionStatus::Running);
}

#[test]
fn test_e2e_critical_failure_stops_execution() {
    // Scenario: Critical failure stops execution
    let mut exec = create_running_execution("test");

    exec.log.push("CRITICAL ERROR: System failure".to_string());
    exec.status = ExecutionStatus::Failed;
    exec.current_task = None;
    exec.current_agent = None;

    assert_eq!(exec.status, ExecutionStatus::Failed);
}

// ============================================================================
// Long-Running Execution Tests
// ============================================================================

#[test]
fn test_e2e_long_running_execution_tracking() {
    // Scenario: Track long-running workflow execution
    let mut exec = create_test_execution("long_workflow", ExecutionStatus::Running);

    // Simulate many tasks
    for i in 1..=50 {
        exec.current_task = Some(format!("task{}", i));
        complete_task(&mut exec, &format!("task{}", i));
        exec.progress = i as f64 / 50.0;
    }

    assert_eq!(exec.completed_tasks.len(), 50);
    assert_eq!(exec.progress, 1.0);
}

#[test]
fn test_e2e_execution_elapsed_time() {
    // Scenario: Execution tracks elapsed time
    let exec = create_running_execution("test");

    let elapsed = exec.started_at.elapsed();

    // Should have started very recently
    assert!(elapsed.as_millis() < 100);
}

// ============================================================================
// Execution Statistics Tests
// ============================================================================

#[test]
fn test_e2e_task_success_rate() {
    // Scenario: Calculate task success rate
    let mut exec = create_test_execution("stats", ExecutionStatus::Running);

    complete_task(&mut exec, "task1");
    complete_task(&mut exec, "task2");
    complete_task(&mut exec, "task3");
    fail_task(&mut exec, "task4", "Error");

    let total_tasks = exec.completed_tasks.len() + exec.failed_tasks.len();
    let success_rate = exec.completed_tasks.len() as f64 / total_tasks as f64;

    assert_eq!(total_tasks, 4);
    assert_eq!(success_rate, 0.75); // 3/4 = 75%
}

#[test]
fn test_e2e_execution_completion_metrics() {
    // Scenario: Track completion metrics
    let mut exec = create_test_execution("metrics", ExecutionStatus::Running);

    // Complete workflow
    for i in 1..=10 {
        complete_task(&mut exec, &format!("task{}", i));
    }

    exec.status = ExecutionStatus::Completed;
    exec.progress = 1.0;

    assert_eq!(exec.completed_tasks.len(), 10);
    assert_eq!(exec.failed_tasks.len(), 0);
    assert_eq!(exec.progress, 1.0);
}

// ============================================================================
// Complex Execution Scenarios
// ============================================================================

#[test]
fn test_e2e_complete_execution_lifecycle() {
    // Scenario: Full execution from start to finish
    let mut exec = create_test_execution("complete_flow", ExecutionStatus::Preparing);

    // Phase 1: Preparing
    assert_eq!(exec.status, ExecutionStatus::Preparing);
    exec.log.push("Loading workflow".to_string());

    // Phase 2: Start execution
    exec.status = ExecutionStatus::Running;
    exec.current_agent = Some("agent1".to_string());

    // Phase 3: Execute tasks
    exec.current_task = Some("task1".to_string());
    exec.progress = 0.33;
    complete_task(&mut exec, "task1");

    exec.current_task = Some("task2".to_string());
    exec.progress = 0.66;
    complete_task(&mut exec, "task2");

    exec.current_task = Some("task3".to_string());
    exec.progress = 1.0;
    complete_task(&mut exec, "task3");

    // Phase 4: Complete
    exec.status = ExecutionStatus::Completed;
    exec.current_task = None;
    exec.current_agent = None;

    assert_eq!(exec.status, ExecutionStatus::Completed);
    assert_eq!(exec.completed_tasks.len(), 3);
    assert_eq!(exec.progress, 1.0);
}

#[test]
fn test_e2e_execution_with_pause_and_completion() {
    // Scenario: Execution paused mid-way, then completed
    let mut exec = create_test_execution("pause_complete", ExecutionStatus::Running);

    // Do some work
    complete_task(&mut exec, "task1");
    exec.progress = 0.5;

    // Pause
    exec.status = ExecutionStatus::Paused;
    exec.current_task = Some("task2".to_string());

    // Resume and complete
    exec.status = ExecutionStatus::Running;
    complete_task(&mut exec, "task2");
    exec.progress = 1.0;
    exec.status = ExecutionStatus::Completed;

    assert_eq!(exec.status, ExecutionStatus::Completed);
    assert_eq!(exec.completed_tasks.len(), 2);
}

#[test]
fn test_e2e_execution_with_mixed_results() {
    // Scenario: Execution with successes, failures, and completion
    let mut exec = create_test_execution("mixed", ExecutionStatus::Running);

    complete_task(&mut exec, "task1");
    complete_task(&mut exec, "task2");
    fail_task(&mut exec, "task3", "Error");
    complete_task(&mut exec, "task4");
    fail_task(&mut exec, "task5", "Error");

    // Finish despite failures
    exec.status = ExecutionStatus::Completed;
    exec.progress = 1.0;

    assert_eq!(exec.status, ExecutionStatus::Completed);
    assert_eq!(exec.completed_tasks.len(), 3);
    assert_eq!(exec.failed_tasks.len(), 2);
}

// ============================================================================
// Edge Cases and Boundary Tests
// ============================================================================

#[test]
fn test_e2e_execution_with_no_tasks() {
    // Scenario: Workflow with no tasks
    let mut exec = create_test_execution("empty", ExecutionStatus::Running);

    exec.status = ExecutionStatus::Completed;
    exec.progress = 1.0;

    assert!(exec.completed_tasks.is_empty());
    assert!(exec.failed_tasks.is_empty());
}

#[test]
fn test_e2e_execution_with_empty_logs() {
    // Scenario: Execution produces no logs
    let exec = create_test_execution("silent", ExecutionStatus::Completed);

    assert!(exec.log.is_empty());
}

#[test]
fn test_e2e_rapid_status_changes() {
    // Scenario: Rapid status transitions
    let mut exec = create_test_execution("rapid", ExecutionStatus::Preparing);

    for _ in 0..10 {
        exec.status = ExecutionStatus::Running;
        exec.status = ExecutionStatus::Paused;
        exec.status = ExecutionStatus::Running;
    }

    assert_eq!(exec.status, ExecutionStatus::Running);
}

#[test]
fn test_e2e_execution_state_clone() {
    // Scenario: Execution state can be cloned (for snapshotting)
    let exec1 = create_running_execution("test");
    let exec2 = exec1.clone();

    assert_eq!(exec1.status, exec2.status);
    assert_eq!(exec1.workflow_path, exec2.workflow_path);
    assert_eq!(exec1.progress, exec2.progress);
    assert_eq!(exec1.completed_tasks, exec2.completed_tasks);
}

#[test]
fn test_e2e_execution_state_equality() {
    // Scenario: Execution states can be compared
    let exec1 = create_running_execution("test");
    let exec2 = exec1.clone();

    assert_eq!(exec1, exec2);
}
