//! Execution API Integration Tests
//!
//! Comprehensive tests for execution management API covering:
//! - Execution creation and queuing
//! - Execution lifecycle management (queued → running → completed/failed)
//! - Execution status updates and transitions
//! - Execution cancellation
//! - Execution filtering and pagination
//! - Execution logs retrieval
//! - Parent-child execution relationships
//! - Priority handling

#![cfg(feature = "server")]

use chrono::Utc;
use periplon_sdk::dsl::schema::DSLWorkflow;
use periplon_sdk::server::queue::{Job, WorkQueue};
use periplon_sdk::server::storage::{
    Execution, ExecutionFilter, ExecutionLog, ExecutionStatus, ExecutionStorage, WorkflowMetadata,
    WorkflowStorage,
};
use periplon_sdk::testing::{MockQueue, MockStorage};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_workflow(name: &str) -> (DSLWorkflow, WorkflowMetadata) {
    let workflow = DSLWorkflow {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        dsl_version: "1.0.0".to_string(),
        cwd: None,
        create_cwd: None,
        secrets: HashMap::new(),
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        agents: HashMap::new(),
        tasks: HashMap::new(),
        workflows: HashMap::new(),
        tools: None,
        communication: None,
        mcp_servers: HashMap::new(),
        subflows: HashMap::new(),
        imports: HashMap::new(),
        notifications: None,
        limits: None,
    };

    let metadata = WorkflowMetadata {
        id: Uuid::new_v4(),
        name: name.to_string(),
        version: "1.0.0".to_string(),
        description: Some("Test workflow for execution".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Some("test_user".to_string()),
        tags: vec!["test".to_string()],
        is_active: true,
    };

    (workflow, metadata)
}

fn create_test_execution(workflow_id: Uuid, status: ExecutionStatus) -> Execution {
    let status_clone = status.clone();
    Execution {
        id: Uuid::new_v4(),
        workflow_id,
        workflow_version: "1.0.0".to_string(),
        status,
        started_at: if status_clone != ExecutionStatus::Queued {
            Some(Utc::now())
        } else {
            None
        },
        completed_at: if matches!(
            status_clone,
            ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled
        ) {
            Some(Utc::now())
        } else {
            None
        },
        created_at: Utc::now(),
        triggered_by: Some("test_user".to_string()),
        trigger_type: "manual".to_string(),
        input_params: Some(json!({"key": "value"})),
        result: if status_clone == ExecutionStatus::Completed {
            Some(json!({"result": "success"}))
        } else {
            None
        },
        error: if status_clone == ExecutionStatus::Failed {
            Some("Test error".to_string())
        } else {
            None
        },
        retry_count: 0,
        parent_execution_id: None,
    }
}

async fn setup_test_environment() -> (Arc<MockStorage>, Arc<MockQueue>) {
    let storage = Arc::new(MockStorage::new());
    let queue = Arc::new(MockQueue::new());
    (storage, queue)
}

// ============================================================================
// Execution Creation Tests
// ============================================================================

#[tokio::test]
async fn test_create_execution() {
    let (storage, queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    // Store workflow
    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create execution
    let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;

    storage.store_execution(&execution).await.unwrap();

    // Create job
    let job_payload = json!({
        "workflow": workflow,
        "input_params": {"key": "value"}
    });
    let job = Job::new(workflow_id, execution_id, job_payload);
    queue.enqueue(job).await.unwrap();

    // Verify execution stored
    let retrieved = storage.get_execution(execution_id).await.unwrap();
    assert!(retrieved.is_some());
    let exec = retrieved.unwrap();
    assert_eq!(exec.workflow_id, workflow_id);
    assert_eq!(exec.status, ExecutionStatus::Queued);

    // Verify job queued
    assert_eq!(queue.pending_count(), 1);
}

#[tokio::test]
async fn test_create_execution_with_priority() {
    let (storage, queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Create job with priority
    let job = Job::new(workflow_id, execution_id, json!({})).with_priority(10);
    queue.enqueue(job).await.unwrap();

    // Dequeue and verify priority
    let dequeued = queue.dequeue("worker-1").await.unwrap();
    assert!(dequeued.is_some());
    let job = dequeued.unwrap();
    assert_eq!(job.priority, 10);
}

#[tokio::test]
async fn test_create_execution_with_input_params() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    execution.input_params = Some(json!({
        "environment": "production",
        "debug": false,
        "config": {
            "timeout": 300,
            "retries": 3
        }
    }));

    storage.store_execution(&execution).await.unwrap();

    let retrieved = storage.get_execution(execution.id).await.unwrap().unwrap();
    assert_eq!(retrieved.input_params, execution.input_params);
    assert_eq!(retrieved.input_params.unwrap()["environment"], "production");
}

#[tokio::test]
async fn test_create_child_execution() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create parent execution
    let parent_execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let parent_id = parent_execution.id;
    storage.store_execution(&parent_execution).await.unwrap();

    // Create child execution
    let mut child_execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    child_execution.parent_execution_id = Some(parent_id);

    storage.store_execution(&child_execution).await.unwrap();

    let retrieved = storage
        .get_execution(child_execution.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.parent_execution_id, Some(parent_id));
}

// ============================================================================
// Execution Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_execution_lifecycle_queued_to_running() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create queued execution
    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Transition to running
    execution.status = ExecutionStatus::Running;
    execution.started_at = Some(Utc::now());
    storage
        .update_execution(execution_id, &execution)
        .await
        .unwrap();

    // Verify status change
    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Running);
    assert!(retrieved.started_at.is_some());
}

#[tokio::test]
async fn test_execution_lifecycle_running_to_completed() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Transition to completed
    execution.status = ExecutionStatus::Completed;
    execution.completed_at = Some(Utc::now());
    execution.result = Some(json!({"status": "success", "output": "data"}));

    storage
        .update_execution(execution_id, &execution)
        .await
        .unwrap();

    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Completed);
    assert!(retrieved.completed_at.is_some());
    assert!(retrieved.result.is_some());
}

#[tokio::test]
async fn test_execution_lifecycle_running_to_failed() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Transition to failed
    execution.status = ExecutionStatus::Failed;
    execution.completed_at = Some(Utc::now());
    execution.error = Some("Task execution failed".to_string());

    storage
        .update_execution(execution_id, &execution)
        .await
        .unwrap();

    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Failed);
    assert!(retrieved.completed_at.is_some());
    assert_eq!(retrieved.error, Some("Task execution failed".to_string()));
}

#[tokio::test]
async fn test_execution_cancellation() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create running execution
    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Cancel execution
    execution.status = ExecutionStatus::Cancelled;
    execution.completed_at = Some(Utc::now());

    storage
        .update_execution(execution_id, &execution)
        .await
        .unwrap();

    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Cancelled);
    assert!(retrieved.completed_at.is_some());
}

#[tokio::test]
async fn test_execution_retry_increment() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Failed);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Increment retry count
    execution.retry_count += 1;
    execution.status = ExecutionStatus::Queued; // Re-queue for retry

    storage
        .update_execution(execution_id, &execution)
        .await
        .unwrap();

    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.retry_count, 1);
    assert_eq!(retrieved.status, ExecutionStatus::Queued);
}

// ============================================================================
// Execution Listing and Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_list_all_executions() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create multiple executions
    for i in 0..5 {
        let status = match i % 3 {
            0 => ExecutionStatus::Queued,
            1 => ExecutionStatus::Running,
            _ => ExecutionStatus::Completed,
        };
        let execution = create_test_execution(workflow_id, status);
        storage.store_execution(&execution).await.unwrap();
    }

    let filter = ExecutionFilter::default();
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 5);
}

#[tokio::test]
async fn test_filter_executions_by_workflow() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow1, metadata1) = create_test_workflow("workflow-1");
    let (workflow2, metadata2) = create_test_workflow("workflow-2");
    let workflow_id1 = metadata1.id;
    let workflow_id2 = metadata2.id;

    storage
        .store_workflow(&workflow1, &metadata1)
        .await
        .unwrap();
    storage
        .store_workflow(&workflow2, &metadata2)
        .await
        .unwrap();

    // Create executions for both workflows
    for _ in 0..3 {
        let execution = create_test_execution(workflow_id1, ExecutionStatus::Queued);
        storage.store_execution(&execution).await.unwrap();
    }
    for _ in 0..2 {
        let execution = create_test_execution(workflow_id2, ExecutionStatus::Running);
        storage.store_execution(&execution).await.unwrap();
    }

    let filter = ExecutionFilter {
        workflow_id: Some(workflow_id1),
        ..Default::default()
    };
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 3);
    assert!(executions.iter().all(|e| e.workflow_id == workflow_id1));
}

#[tokio::test]
async fn test_filter_executions_by_status() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create executions with different statuses
    for _ in 0..2 {
        let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
        storage.store_execution(&execution).await.unwrap();
    }
    for _ in 0..3 {
        let execution = create_test_execution(workflow_id, ExecutionStatus::Completed);
        storage.store_execution(&execution).await.unwrap();
    }

    let filter = ExecutionFilter {
        status: Some(ExecutionStatus::Completed),
        ..Default::default()
    };
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 3);
    assert!(executions
        .iter()
        .all(|e| e.status == ExecutionStatus::Completed));
}

#[tokio::test]
async fn test_filter_executions_by_triggered_by() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create executions with different triggers
    for _ in 0..3 {
        let mut execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
        execution.triggered_by = Some("user1".to_string());
        storage.store_execution(&execution).await.unwrap();
    }
    for _ in 0..2 {
        let mut execution = create_test_execution(workflow_id, ExecutionStatus::Running);
        execution.triggered_by = Some("user2".to_string());
        storage.store_execution(&execution).await.unwrap();
    }

    let filter = ExecutionFilter {
        triggered_by: Some("user1".to_string()),
        ..Default::default()
    };
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 3);
    assert!(executions
        .iter()
        .all(|e| e.triggered_by.as_ref() == Some(&"user1".to_string())));
}

#[tokio::test]
async fn test_execution_pagination() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create 10 executions
    for _ in 0..10 {
        let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
        storage.store_execution(&execution).await.unwrap();
    }

    // Test limit
    let filter = ExecutionFilter {
        limit: Some(5),
        ..Default::default()
    };
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 5);
}

// ============================================================================
// Execution Log Tests
// ============================================================================

#[tokio::test]
async fn test_store_and_retrieve_execution_logs() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Store logs
    for i in 0..5 {
        let log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: None,
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: format!("Log message {}", i),
            metadata: None,
        };
        storage.store_execution_log(&log).await.unwrap();
    }

    // Retrieve logs
    let logs = storage
        .get_execution_logs(execution_id, None)
        .await
        .unwrap();
    assert_eq!(logs.len(), 5);
}

#[tokio::test]
async fn test_execution_logs_with_different_levels() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    let levels = vec!["DEBUG", "INFO", "WARN", "ERROR"];
    for level in levels {
        let log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: None,
            timestamp: Utc::now(),
            level: level.to_string(),
            message: format!("Test {} message", level),
            metadata: None,
        };
        storage.store_execution_log(&log).await.unwrap();
    }

    let logs = storage
        .get_execution_logs(execution_id, None)
        .await
        .unwrap();
    assert_eq!(logs.len(), 4);
    assert!(logs.iter().any(|l| l.level == "DEBUG"));
    assert!(logs.iter().any(|l| l.level == "ERROR"));
}

#[tokio::test]
async fn test_execution_logs_with_metadata() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    let log = ExecutionLog {
        id: None,
        execution_id,
        task_execution_id: None,
        timestamp: Utc::now(),
        level: "INFO".to_string(),
        message: "Processing task".to_string(),
        metadata: Some(json!({
            "task_id": "task_1",
            "duration_ms": 1500,
            "memory_mb": 128
        })),
    };
    storage.store_execution_log(&log).await.unwrap();

    let logs = storage
        .get_execution_logs(execution_id, None)
        .await
        .unwrap();
    assert_eq!(logs.len(), 1);
    assert!(logs[0].metadata.is_some());
    assert_eq!(logs[0].metadata.as_ref().unwrap()["task_id"], "task_1");
}

// ============================================================================
// Concurrent Execution Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_execution_creation() {
    let storage = Arc::new(MockStorage::new());
    let queue = Arc::new(MockQueue::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Spawn multiple tasks creating executions concurrently
    let mut handles = vec![];
    for _ in 0..10 {
        let storage_clone = storage.clone();
        let queue_clone = queue.clone();
        let wf_id = workflow_id;
        let handle = tokio::spawn(async move {
            let execution = create_test_execution(wf_id, ExecutionStatus::Queued);
            let execution_id = execution.id;
            storage_clone.store_execution(&execution).await.unwrap();

            let job = Job::new(wf_id, execution_id, json!({}));
            queue_clone.enqueue(job).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all stored
    assert_eq!(storage.execution_count(), 10);
    assert_eq!(queue.pending_count(), 10);
}

#[tokio::test]
async fn test_execution_state_consistency() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Multiple concurrent updates
    let storage_clone1 = Arc::clone(&Arc::new(storage.clone()));
    let storage_clone2 = Arc::clone(&Arc::new(storage.clone()));

    let handle1 = tokio::spawn(async move {
        let mut exec = storage_clone1
            .get_execution(execution_id)
            .await
            .unwrap()
            .unwrap();
        exec.status = ExecutionStatus::Running;
        storage_clone1
            .update_execution(execution_id, &exec)
            .await
            .unwrap();
    });

    let handle2 = tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let mut exec = storage_clone2
            .get_execution(execution_id)
            .await
            .unwrap()
            .unwrap();
        exec.status = ExecutionStatus::Completed;
        storage_clone2
            .update_execution(execution_id, &exec)
            .await
            .unwrap();
    });

    handle1.await.unwrap();
    handle2.await.unwrap();

    // Final state should be from last update
    let final_exec = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(final_exec.status, ExecutionStatus::Completed);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_execution_not_found() {
    let (storage, _queue) = setup_test_environment().await;
    let non_existent_id = Uuid::new_v4();

    let result = storage.get_execution(non_existent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_storage_failure_handling() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Configure storage to fail
    storage.fail_store();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let result = storage.store_execution(&execution).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_execution_with_logs() {
    let (storage, _queue) = setup_test_environment().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Store logs
    for i in 0..3 {
        let log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: None,
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: format!("Log {}", i),
            metadata: None,
        };
        storage.store_execution_log(&log).await.unwrap();
    }

    // Delete execution (should cascade to logs)
    storage.delete_execution(execution_id).await.unwrap();

    // Verify execution deleted
    assert!(storage.get_execution(execution_id).await.unwrap().is_none());

    // Verify logs deleted
    let logs = storage
        .get_execution_logs(execution_id, None)
        .await
        .unwrap();
    assert_eq!(logs.len(), 0);
}
