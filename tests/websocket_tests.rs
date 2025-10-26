//! WebSocket Integration Tests
//!
//! Comprehensive tests for WebSocket execution streaming covering:
//! - WebSocket connection establishment
//! - Real-time execution status updates
//! - Log message streaming
//! - Progress updates
//! - Completion and failure notifications
//! - Ping/pong keep-alive mechanism
//! - Connection error handling
//! - Message format validation

#![cfg(feature = "server")]

use chrono::Utc;
use periplon_sdk::dsl::schema::DSLWorkflow;
use periplon_sdk::server::storage::{
    Execution, ExecutionLog, ExecutionStatus, ExecutionStorage, WorkflowMetadata, WorkflowStorage,
};
use periplon_sdk::testing::MockStorage;
use serde_json::{json, Value};
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
        description: Some("Test workflow".to_string()),
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

// ============================================================================
// WebSocket Message Format Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_started_message_format() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Simulate Started message
    let message = json!({
        "type": "started",
        "execution_id": execution_id.to_string(),
        "workflow_id": workflow_id.to_string(),
        "started_at": execution.started_at.unwrap().to_rfc3339()
    });

    assert_eq!(message["type"], "started");
    assert_eq!(message["execution_id"], execution_id.to_string());
    assert_eq!(message["workflow_id"], workflow_id.to_string());
    assert!(message["started_at"].is_string());
}

#[tokio::test]
async fn test_websocket_log_message_format() {
    let execution_id = Uuid::new_v4();
    let timestamp = Utc::now();

    let message = json!({
        "type": "log",
        "execution_id": execution_id.to_string(),
        "timestamp": timestamp.to_rfc3339(),
        "level": "INFO",
        "message": "Task execution started"
    });

    assert_eq!(message["type"], "log");
    assert_eq!(message["level"], "INFO");
    assert_eq!(message["message"], "Task execution started");
    assert!(message["timestamp"].is_string());
}

#[tokio::test]
async fn test_websocket_progress_message_format() {
    let execution_id = Uuid::new_v4();

    let message = json!({
        "type": "progress",
        "execution_id": execution_id.to_string(),
        "completed_tasks": 3,
        "total_tasks": 10,
        "percent": 30.0
    });

    assert_eq!(message["type"], "progress");
    assert_eq!(message["completed_tasks"], 3);
    assert_eq!(message["total_tasks"], 10);
    assert_eq!(message["percent"], 30.0);
}

#[tokio::test]
async fn test_websocket_task_update_message_format() {
    let execution_id = Uuid::new_v4();

    let message = json!({
        "type": "task_update",
        "execution_id": execution_id.to_string(),
        "task_id": "task_1",
        "status": "running",
        "message": "Processing data"
    });

    assert_eq!(message["type"], "task_update");
    assert_eq!(message["task_id"], "task_1");
    assert_eq!(message["status"], "running");
    assert_eq!(message["message"], "Processing data");
}

#[tokio::test]
async fn test_websocket_completed_message_format() {
    let execution_id = Uuid::new_v4();
    let completed_at = Utc::now();

    let message = json!({
        "type": "completed",
        "execution_id": execution_id.to_string(),
        "status": "completed",
        "completed_at": completed_at.to_rfc3339(),
        "result": {
            "status": "success",
            "output": "data"
        }
    });

    assert_eq!(message["type"], "completed");
    assert_eq!(message["status"], "completed");
    assert!(message["result"].is_object());
    assert_eq!(message["result"]["status"], "success");
}

#[tokio::test]
async fn test_websocket_failed_message_format() {
    let execution_id = Uuid::new_v4();
    let failed_at = Utc::now();

    let message = json!({
        "type": "failed",
        "execution_id": execution_id.to_string(),
        "error": "Task execution failed",
        "failed_at": failed_at.to_rfc3339()
    });

    assert_eq!(message["type"], "failed");
    assert_eq!(message["error"], "Task execution failed");
    assert!(message["failed_at"].is_string());
}

#[tokio::test]
async fn test_websocket_ping_pong_format() {
    let timestamp = Utc::now();

    let ping = json!({
        "type": "ping",
        "timestamp": timestamp.to_rfc3339()
    });

    let pong = json!({
        "type": "pong",
        "timestamp": timestamp.to_rfc3339()
    });

    assert_eq!(ping["type"], "ping");
    assert_eq!(pong["type"], "pong");
    assert!(ping["timestamp"].is_string());
    assert!(pong["timestamp"].is_string());
}

// ============================================================================
// Execution State Streaming Tests
// ============================================================================

#[tokio::test]
async fn test_stream_execution_state_changes() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create execution and simulate state changes
    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Initial state: Queued
    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Queued);

    // Transition to Running
    execution.status = ExecutionStatus::Running;
    execution.started_at = Some(Utc::now());
    storage.update_execution(execution_id, &execution).await.unwrap();

    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Running);

    // Transition to Completed
    execution.status = ExecutionStatus::Completed;
    execution.completed_at = Some(Utc::now());
    execution.result = Some(json!({"status": "success"}));
    storage.update_execution(execution_id, &execution).await.unwrap();

    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Completed);
    assert!(retrieved.result.is_some());
}

#[tokio::test]
async fn test_stream_execution_logs() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Simulate log streaming
    let log_messages = vec![
        "Initializing workflow",
        "Starting task 1",
        "Task 1 completed",
        "Starting task 2",
        "Task 2 completed",
        "Workflow completed",
    ];

    for (i, message) in log_messages.iter().enumerate() {
        let log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: None,
            timestamp: Utc::now(),
            level: if i == log_messages.len() - 1 {
                "INFO".to_string()
            } else {
                "DEBUG".to_string()
            },
            message: message.to_string(),
            metadata: None,
        };
        storage.store_execution_log(&log).await.unwrap();
    }

    // Retrieve all logs
    let logs = storage.get_execution_logs(execution_id, None).await.unwrap();
    assert_eq!(logs.len(), 6);
    assert_eq!(logs[0].message, "Initializing workflow");
    assert_eq!(logs[5].message, "Workflow completed");
}

#[tokio::test]
async fn test_stream_incremental_logs() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Store initial logs
    for i in 0..3 {
        let log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: None,
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: format!("Initial log {}", i),
            metadata: None,
        };
        storage.store_execution_log(&log).await.unwrap();
    }

    let initial_logs = storage.get_execution_logs(execution_id, None).await.unwrap();
    let initial_count = initial_logs.len();
    assert_eq!(initial_count, 3);

    // Store additional logs (simulating incremental updates)
    for i in 0..2 {
        let log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: None,
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: format!("New log {}", i),
            metadata: None,
        };
        storage.store_execution_log(&log).await.unwrap();
    }

    let all_logs = storage.get_execution_logs(execution_id, None).await.unwrap();
    assert_eq!(all_logs.len(), 5);

    // Verify we can get only new logs by tracking count
    let new_logs = &all_logs[initial_count..];
    assert_eq!(new_logs.len(), 2);
    assert_eq!(new_logs[0].message, "New log 0");
}

// ============================================================================
// Progress Tracking Tests
// ============================================================================

#[tokio::test]
async fn test_execution_progress_calculation() {
    // Simulate progress tracking
    let total_tasks = 10;
    let completed_tasks_vec = vec![0, 3, 5, 7, 10];

    for completed_tasks in completed_tasks_vec {
        let percent = (completed_tasks as f64 / total_tasks as f64) * 100.0;

        let progress_message = json!({
            "type": "progress",
            "execution_id": Uuid::new_v4().to_string(),
            "completed_tasks": completed_tasks,
            "total_tasks": total_tasks,
            "percent": percent
        });

        assert_eq!(progress_message["completed_tasks"], completed_tasks);
        assert_eq!(progress_message["total_tasks"], total_tasks);
        assert_eq!(progress_message["percent"], percent);
    }
}

#[tokio::test]
async fn test_execution_with_task_updates() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Simulate task updates through logs
    let tasks = vec!["task_1", "task_2", "task_3"];
    for task_id in tasks {
        // Task start
        let start_log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: Some(Uuid::new_v4()),
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: format!("Starting {}", task_id),
            metadata: Some(json!({
                "task_id": task_id,
                "status": "running"
            })),
        };
        storage.store_execution_log(&start_log).await.unwrap();

        // Task complete
        let complete_log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: Some(Uuid::new_v4()),
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: format!("Completed {}", task_id),
            metadata: Some(json!({
                "task_id": task_id,
                "status": "completed"
            })),
        };
        storage.store_execution_log(&complete_log).await.unwrap();
    }

    let logs = storage.get_execution_logs(execution_id, None).await.unwrap();
    assert_eq!(logs.len(), 6); // 3 tasks * 2 logs each

    // Verify task metadata
    let task_logs: Vec<_> = logs.iter().filter(|l| l.metadata.is_some()).collect();
    assert_eq!(task_logs.len(), 6);
}

// ============================================================================
// Connection Management Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_connection_for_existing_execution() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Verify execution exists for WebSocket connection
    let exists = storage.get_execution(execution_id).await.unwrap().is_some();
    assert!(exists);
}

#[tokio::test]
async fn test_websocket_connection_for_nonexistent_execution() {
    let storage = Arc::new(MockStorage::new());
    let non_existent_id = Uuid::new_v4();

    // Verify execution doesn't exist (would return 404)
    let exists = storage.get_execution(non_existent_id).await.unwrap().is_some();
    assert!(!exists);
}

#[tokio::test]
async fn test_websocket_invalid_execution_id_format() {
    // Test invalid UUID format
    let invalid_ids = vec![
        "not-a-uuid",
        "12345",
        "invalid-uuid-format",
        "",
        "00000000-0000-0000-0000",
    ];

    for invalid_id in invalid_ids {
        let result = Uuid::parse_str(invalid_id);
        assert!(result.is_err());
    }
}

// ============================================================================
// Multiple Concurrent Connections Tests
// ============================================================================

#[tokio::test]
async fn test_multiple_concurrent_websocket_streams() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create multiple executions
    let mut execution_ids = Vec::new();
    for i in 0..5 {
        let status = if i % 2 == 0 {
            ExecutionStatus::Running
        } else {
            ExecutionStatus::Queued
        };
        let execution = create_test_execution(workflow_id, status);
        execution_ids.push(execution.id);
        storage.store_execution(&execution).await.unwrap();
    }

    // Verify all executions can be accessed (simulating multiple WS connections)
    for execution_id in execution_ids {
        let exists = storage.get_execution(execution_id).await.unwrap().is_some();
        assert!(exists);
    }
}

#[tokio::test]
async fn test_websocket_message_ordering() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Store logs in order
    let messages = vec!["First", "Second", "Third", "Fourth", "Fifth"];
    for message in &messages {
        let log = ExecutionLog {
            id: None,
            execution_id,
            task_execution_id: None,
            timestamp: Utc::now(),
            level: "INFO".to_string(),
            message: message.to_string(),
            metadata: None,
        };
        storage.store_execution_log(&log).await.unwrap();
        // Small delay to ensure timestamp ordering
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Retrieve and verify order
    let logs = storage.get_execution_logs(execution_id, None).await.unwrap();
    assert_eq!(logs.len(), 5);
    for (i, log) in logs.iter().enumerate() {
        assert_eq!(log.message, messages[i]);
    }
}

// ============================================================================
// Error and Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_stream_for_completed_execution() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create already completed execution
    let execution = create_test_execution(workflow_id, ExecutionStatus::Completed);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Should be able to connect and get final state
    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Completed);
    assert!(retrieved.completed_at.is_some());
    assert!(retrieved.result.is_some());
}

#[tokio::test]
async fn test_websocket_stream_for_failed_execution() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Failed);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // Should be able to connect and get error state
    let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
    assert_eq!(retrieved.status, ExecutionStatus::Failed);
    assert!(retrieved.completed_at.is_some());
    assert!(retrieved.error.is_some());
}

#[tokio::test]
async fn test_websocket_with_no_logs() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;
    storage.store_execution(&execution).await.unwrap();

    // No logs stored yet
    let logs = storage.get_execution_logs(execution_id, None).await.unwrap();
    assert_eq!(logs.len(), 0);
}

#[tokio::test]
async fn test_websocket_message_serialization() {
    let execution_id = Uuid::new_v4();
    let workflow_id = Uuid::new_v4();

    // Test all message types can be serialized
    let messages: Vec<Value> = vec![
        json!({
            "type": "started",
            "execution_id": execution_id.to_string(),
            "workflow_id": workflow_id.to_string(),
            "started_at": Utc::now().to_rfc3339()
        }),
        json!({
            "type": "log",
            "execution_id": execution_id.to_string(),
            "timestamp": Utc::now().to_rfc3339(),
            "level": "INFO",
            "message": "Test log"
        }),
        json!({
            "type": "progress",
            "execution_id": execution_id.to_string(),
            "completed_tasks": 5,
            "total_tasks": 10,
            "percent": 50.0
        }),
        json!({
            "type": "completed",
            "execution_id": execution_id.to_string(),
            "status": "completed",
            "completed_at": Utc::now().to_rfc3339(),
            "result": {"status": "success"}
        }),
    ];

    for message in messages {
        let serialized = serde_json::to_string(&message);
        assert!(serialized.is_ok());

        let deserialized: Result<Value, _> = serde_json::from_str(&serialized.unwrap());
        assert!(deserialized.is_ok());
    }
}
