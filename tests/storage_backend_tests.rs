//! Storage Backend Integration Tests
//!
//! Comprehensive tests for storage backend implementations including
//! filesystem-based persistence, workflow/execution CRUD operations,
//! checkpoint management, and data integrity.

#![cfg(feature = "server")]

use chrono::Utc;
use periplon_sdk::dsl::schema::DSLWorkflow;
use periplon_sdk::server::storage::filesystem::FilesystemStorage;
use periplon_sdk::server::storage::{
    Checkpoint, CheckpointStorage, Execution, ExecutionFilter, ExecutionLog, ExecutionStatus,
    ExecutionStorage, WorkflowFilter, WorkflowMetadata, WorkflowStorage,
};
use serde_json::json;
use std::collections::HashMap;
use tempfile::TempDir;
use uuid::Uuid;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_workflow(name: &str) -> (DSLWorkflow, WorkflowMetadata) {
    let workflow = DSLWorkflow {
        provider: Default::default(),
        model: None,
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
            ExecutionStatus::Completed | ExecutionStatus::Failed
        ) {
            Some(Utc::now())
        } else {
            None
        },
        created_at: Utc::now(),
        triggered_by: Some("test_user".to_string()),
        trigger_type: "manual".to_string(),
        input_params: Some(json!({"param": "value"})),
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

fn create_test_checkpoint(execution_id: Uuid, name: &str) -> Checkpoint {
    Checkpoint {
        id: Uuid::new_v4(),
        execution_id,
        checkpoint_name: name.to_string(),
        state: json!({
            "current_task": "task_1",
            "progress": 50,
            "data": {"key": "value"}
        }),
        created_at: Utc::now(),
    }
}

async fn setup_filesystem_storage() -> (FilesystemStorage, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let storage = FilesystemStorage::new(
        temp_dir.path().to_path_buf(),
        "workflows".to_string(),
        "executions".to_string(),
        "checkpoints".to_string(),
        "logs".to_string(),
    )
    .await
    .unwrap();

    (storage, temp_dir)
}

// ============================================================================
// Workflow Storage Tests
// ============================================================================

#[tokio::test]
async fn test_filesystem_store_and_retrieve_workflow() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    // Store workflow
    let stored_id = storage.store_workflow(&workflow, &metadata).await.unwrap();
    assert_eq!(stored_id, workflow_id);

    // Retrieve workflow
    let retrieved = storage.get_workflow(workflow_id).await.unwrap();
    assert!(retrieved.is_some());
    let (retrieved_wf, retrieved_meta) = retrieved.unwrap();
    assert_eq!(retrieved_wf.name, "test-workflow");
    assert_eq!(retrieved_meta.id, workflow_id);
    assert_eq!(retrieved_meta.version, "1.0.0");
}

#[tokio::test]
async fn test_filesystem_workflow_not_found() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let non_existent_id = Uuid::new_v4();

    let result = storage.get_workflow(non_existent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_filesystem_update_workflow() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let (mut workflow, mut metadata) = create_test_workflow("original-name");
    let workflow_id = metadata.id;

    // Store original
    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Update workflow
    workflow.name = "updated-name".to_string();
    metadata.description = Some("Updated description".to_string());
    storage
        .update_workflow(workflow_id, &workflow, &metadata)
        .await
        .unwrap();

    // Verify update
    let retrieved = storage.get_workflow(workflow_id).await.unwrap().unwrap();
    assert_eq!(retrieved.0.name, "updated-name");
    assert_eq!(
        retrieved.1.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
async fn test_filesystem_delete_workflow() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let (workflow, metadata) = create_test_workflow("to-delete");
    let workflow_id = metadata.id;

    // Store and verify
    storage.store_workflow(&workflow, &metadata).await.unwrap();
    assert!(storage.get_workflow(workflow_id).await.unwrap().is_some());

    // Delete
    storage.delete_workflow(workflow_id).await.unwrap();

    // Verify deletion
    assert!(storage.get_workflow(workflow_id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_filesystem_list_workflows() {
    let (storage, _temp) = setup_filesystem_storage().await;

    // Store multiple workflows
    for i in 0..5 {
        let (workflow, metadata) = create_test_workflow(&format!("workflow-{}", i));
        storage.store_workflow(&workflow, &metadata).await.unwrap();
    }

    // List all
    let filter = WorkflowFilter::default();
    let workflows = storage.list_workflows(&filter).await.unwrap();
    assert_eq!(workflows.len(), 5);
}

#[tokio::test]
async fn test_filesystem_list_workflows_with_filter() {
    let (storage, _temp) = setup_filesystem_storage().await;

    // Store workflows with different names
    let (workflow1, metadata1) = create_test_workflow("alpha");
    let (workflow2, metadata2) = create_test_workflow("beta");
    let (workflow3, metadata3) = create_test_workflow("alpha");

    storage
        .store_workflow(&workflow1, &metadata1)
        .await
        .unwrap();
    storage
        .store_workflow(&workflow2, &metadata2)
        .await
        .unwrap();
    storage
        .store_workflow(&workflow3, &metadata3)
        .await
        .unwrap();

    // Filter by name
    let filter = WorkflowFilter {
        name: Some("alpha".to_string()),
        ..Default::default()
    };
    let workflows = storage.list_workflows(&filter).await.unwrap();
    assert_eq!(workflows.len(), 2);
}

#[tokio::test]
async fn test_filesystem_workflow_versioning() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let (workflow, mut metadata) = create_test_workflow("versioned-workflow");
    let workflow_id = metadata.id;

    // First store the workflow to create the directory structure
    metadata.version = "1.0.0".to_string();
    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Now store a version
    storage
        .store_workflow_version(&workflow, &metadata)
        .await
        .unwrap();

    // Retrieve specific version
    let retrieved = storage
        .get_workflow_version(workflow_id, "1.0.0")
        .await
        .unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().1.version, "1.0.0");
}

// ============================================================================
// Execution Storage Tests
// ============================================================================

#[tokio::test]
async fn test_filesystem_store_and_retrieve_execution() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let workflow_id = Uuid::new_v4();
    let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;

    // Store execution
    let stored_id = storage.store_execution(&execution).await.unwrap();
    assert_eq!(stored_id, execution_id);

    // Retrieve execution
    let retrieved = storage.get_execution(execution_id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_exec = retrieved.unwrap();
    assert_eq!(retrieved_exec.workflow_id, workflow_id);
    assert_eq!(retrieved_exec.status, ExecutionStatus::Queued);
}

#[tokio::test]
async fn test_filesystem_update_execution_status() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let workflow_id = Uuid::new_v4();
    let mut execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;

    // Store with Queued status
    storage.store_execution(&execution).await.unwrap();

    // Update to Running
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
async fn test_filesystem_list_executions_by_workflow() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let workflow_id = Uuid::new_v4();
    let other_workflow_id = Uuid::new_v4();

    // Store executions for target workflow
    for _ in 0..3 {
        let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
        storage.store_execution(&execution).await.unwrap();
    }

    // Store executions for other workflow
    for _ in 0..2 {
        let execution = create_test_execution(other_workflow_id, ExecutionStatus::Queued);
        storage.store_execution(&execution).await.unwrap();
    }

    // Filter by workflow
    let filter = ExecutionFilter {
        workflow_id: Some(workflow_id),
        ..Default::default()
    };
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 3);
    assert!(executions.iter().all(|e| e.workflow_id == workflow_id));
}

#[tokio::test]
async fn test_filesystem_list_executions_by_status() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let workflow_id = Uuid::new_v4();

    // Store executions with different statuses
    for _ in 0..2 {
        let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
        storage.store_execution(&execution).await.unwrap();
    }
    for _ in 0..3 {
        let execution = create_test_execution(workflow_id, ExecutionStatus::Completed);
        storage.store_execution(&execution).await.unwrap();
    }

    // Filter by status
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
async fn test_filesystem_delete_execution() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let workflow_id = Uuid::new_v4();
    let execution = create_test_execution(workflow_id, ExecutionStatus::Queued);
    let execution_id = execution.id;

    // Store and verify
    storage.store_execution(&execution).await.unwrap();
    assert!(storage.get_execution(execution_id).await.unwrap().is_some());

    // Delete
    storage.delete_execution(execution_id).await.unwrap();

    // Verify deletion
    assert!(storage.get_execution(execution_id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_filesystem_execution_logs() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let workflow_id = Uuid::new_v4();
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

// ============================================================================
// Checkpoint Storage Tests
// ============================================================================

#[tokio::test]
async fn test_filesystem_store_and_retrieve_checkpoint() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let execution_id = Uuid::new_v4();
    let checkpoint = create_test_checkpoint(execution_id, "checkpoint-1");
    let checkpoint_id = checkpoint.id;

    // Store checkpoint
    let stored_id = storage.store_checkpoint(&checkpoint).await.unwrap();
    assert_eq!(stored_id, checkpoint_id);

    // Retrieve by execution and name
    let retrieved = storage
        .get_checkpoint(execution_id, "checkpoint-1")
        .await
        .unwrap();
    assert!(retrieved.is_some());
    let retrieved_cp = retrieved.unwrap();
    assert_eq!(retrieved_cp.checkpoint_name, "checkpoint-1");
    assert_eq!(retrieved_cp.execution_id, execution_id);
}

#[tokio::test]
async fn test_filesystem_list_checkpoints_for_execution() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let execution_id = Uuid::new_v4();
    let other_execution_id = Uuid::new_v4();

    // Store checkpoints for target execution
    for i in 0..3 {
        let checkpoint = create_test_checkpoint(execution_id, &format!("checkpoint-{}", i));
        storage.store_checkpoint(&checkpoint).await.unwrap();
    }

    // Store checkpoint for other execution
    let other_checkpoint = create_test_checkpoint(other_execution_id, "other-checkpoint");
    storage.store_checkpoint(&other_checkpoint).await.unwrap();

    // List checkpoints for target execution
    let checkpoints = storage.list_checkpoints(execution_id).await.unwrap();
    assert_eq!(checkpoints.len(), 3);
    assert!(checkpoints.iter().all(|c| c.execution_id == execution_id));
}

#[tokio::test]
async fn test_filesystem_checkpoint_state_persistence() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let execution_id = Uuid::new_v4();
    let checkpoint = create_test_checkpoint(execution_id, "state-checkpoint");

    // Store with complex state
    storage.store_checkpoint(&checkpoint).await.unwrap();

    // Retrieve and verify state
    let retrieved = storage
        .get_checkpoint(execution_id, "state-checkpoint")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved.state["current_task"], "task_1");
    assert_eq!(retrieved.state["progress"], 50);
    assert_eq!(retrieved.state["data"]["key"], "value");
}

#[tokio::test]
async fn test_filesystem_delete_checkpoint() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let execution_id = Uuid::new_v4();
    let checkpoint = create_test_checkpoint(execution_id, "to-delete");
    let checkpoint_id = checkpoint.id;

    // Store and verify
    storage.store_checkpoint(&checkpoint).await.unwrap();
    let checkpoints = storage.list_checkpoints(execution_id).await.unwrap();
    assert_eq!(checkpoints.len(), 1);

    // Note: Filesystem backend doesn't implement delete_checkpoint by ID
    // It returns NotFound error as documented in the implementation (line 563-570)
    let result = storage.delete_checkpoint(checkpoint_id).await;
    assert!(result.is_err());
    match result {
        Err(periplon_sdk::server::storage::StorageError::NotFound(_)) => {
            // Expected - filesystem backend doesn't support delete by ID
        }
        _ => panic!("Expected NotFound error for delete_checkpoint"),
    }
}

// ============================================================================
// Data Integrity Tests
// ============================================================================

#[tokio::test]
async fn test_filesystem_workflow_execution_relationship() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let (workflow, metadata) = create_test_workflow("linked-workflow");
    let workflow_id = metadata.id;

    // Store workflow
    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create multiple executions for this workflow
    for i in 0..3 {
        let status = match i {
            0 => ExecutionStatus::Queued,
            1 => ExecutionStatus::Running,
            _ => ExecutionStatus::Completed,
        };
        let execution = create_test_execution(workflow_id, status);
        storage.store_execution(&execution).await.unwrap();
    }

    // Verify relationship via filter
    let filter = ExecutionFilter {
        workflow_id: Some(workflow_id),
        ..Default::default()
    };
    let executions = storage.list_executions(&filter).await.unwrap();
    assert_eq!(executions.len(), 3);
}

#[tokio::test]
async fn test_filesystem_execution_checkpoint_relationship() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let workflow_id = Uuid::new_v4();
    let execution = create_test_execution(workflow_id, ExecutionStatus::Running);
    let execution_id = execution.id;

    // Store execution
    storage.store_execution(&execution).await.unwrap();

    // Store multiple checkpoints
    for i in 0..3 {
        let checkpoint = create_test_checkpoint(execution_id, &format!("step-{}", i));
        storage.store_checkpoint(&checkpoint).await.unwrap();
    }

    // Verify relationship
    let checkpoints = storage.list_checkpoints(execution_id).await.unwrap();
    assert_eq!(checkpoints.len(), 3);
}

#[tokio::test]
async fn test_filesystem_concurrent_workflow_operations() {
    let (storage, _temp) = setup_filesystem_storage().await;
    let storage = std::sync::Arc::new(storage);

    // Spawn multiple tasks storing workflows concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let storage_clone = storage.clone();
        let handle = tokio::spawn(async move {
            let (workflow, metadata) = create_test_workflow(&format!("concurrent-{}", i));
            storage_clone
                .store_workflow(&workflow, &metadata)
                .await
                .unwrap();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all stored
    let filter = WorkflowFilter::default();
    let workflows = storage.list_workflows(&filter).await.unwrap();
    assert_eq!(workflows.len(), 10);
}

#[tokio::test]
async fn test_filesystem_storage_persistence_across_instances() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().to_path_buf();

    // First instance - store data
    {
        let storage = FilesystemStorage::new(
            base_path.clone(),
            "workflows".to_string(),
            "executions".to_string(),
            "checkpoints".to_string(),
            "logs".to_string(),
        )
        .await
        .unwrap();

        let (workflow, metadata) = create_test_workflow("persistent-workflow");
        let workflow_id = metadata.id;
        storage.store_workflow(&workflow, &metadata).await.unwrap();

        let execution = create_test_execution(workflow_id, ExecutionStatus::Completed);
        storage.store_execution(&execution).await.unwrap();
    }

    // Second instance - verify data still exists
    {
        let storage = FilesystemStorage::new(
            base_path.clone(),
            "workflows".to_string(),
            "executions".to_string(),
            "checkpoints".to_string(),
            "logs".to_string(),
        )
        .await
        .unwrap();

        let filter = WorkflowFilter::default();
        let workflows = storage.list_workflows(&filter).await.unwrap();
        assert_eq!(workflows.len(), 1);

        let exec_filter = ExecutionFilter::default();
        let executions = storage.list_executions(&exec_filter).await.unwrap();
        assert_eq!(executions.len(), 1);
    }
}
