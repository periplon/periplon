//! Schedule API Integration Tests
//!
//! Comprehensive tests for schedule management API covering:
//! - Schedule creation and validation
//! - CRUD operations on schedules
//! - Schedule filtering and pagination
//! - Due schedule detection
//! - Schedule run tracking
//! - Manual schedule triggering

#![cfg(feature = "server")]

use chrono::{Duration, Utc};
use periplon_sdk::dsl::schema::DSLWorkflow;
use periplon_sdk::server::storage::{
    Schedule, ScheduleFilter, ScheduleRun, ScheduleRunStatus, ScheduleStorage, WorkflowMetadata,
    WorkflowStorage,
};
use periplon_sdk::testing::MockStorage;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
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
        description: Some("Test workflow for scheduling".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Some("test_user".to_string()),
        tags: vec!["test".to_string()],
        is_active: true,
    };

    (workflow, metadata)
}

fn create_test_schedule(workflow_id: Uuid, cron_expression: &str) -> Schedule {
    Schedule {
        id: Uuid::new_v4(),
        workflow_id,
        cron_expression: cron_expression.to_string(),
        timezone: "UTC".to_string(),
        is_active: true,
        input_params: Some(json!({"key": "value"})),
        created_at: Utc::now(),
        updated_at: Utc::now(),
        created_by: Some("test_user".to_string()),
        last_run_at: None,
        next_run_at: None,
        description: Some("Test schedule".to_string()),
    }
}

fn create_test_schedule_run(
    schedule_id: Uuid,
    execution_id: Option<Uuid>,
    status: ScheduleRunStatus,
) -> ScheduleRun {
    let status_clone = status.clone();
    ScheduleRun {
        id: Uuid::new_v4(),
        schedule_id,
        execution_id,
        scheduled_for: Utc::now(),
        started_at: if status_clone != ScheduleRunStatus::Scheduled {
            Some(Utc::now())
        } else {
            None
        },
        status,
        error: if status_clone == ScheduleRunStatus::Failed {
            Some("Test error".to_string())
        } else {
            None
        },
        created_at: Utc::now(),
    }
}

async fn setup_storage() -> Arc<MockStorage> {
    Arc::new(MockStorage::new())
}

// ============================================================================
// Schedule CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_and_retrieve_schedule() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    // Store workflow first
    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create schedule
    let schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;

    storage.store_schedule(&schedule).await.unwrap();
    assert_eq!(storage.schedule_count(), 1);

    // Retrieve schedule
    let retrieved = storage.get_schedule(schedule_id).await.unwrap();
    assert!(retrieved.is_some());
    let retrieved_schedule = retrieved.unwrap();
    assert_eq!(retrieved_schedule.workflow_id, workflow_id);
    assert_eq!(retrieved_schedule.cron_expression, "0 0 * * *");
    assert_eq!(retrieved_schedule.timezone, "UTC");
    assert!(retrieved_schedule.is_active);
}

#[tokio::test]
async fn test_schedule_not_found() {
    let storage = setup_storage().await;
    let non_existent_id = Uuid::new_v4();

    let result = storage.get_schedule(non_existent_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_update_schedule() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create original schedule
    let mut schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    storage.store_schedule(&schedule).await.unwrap();

    // Update schedule
    schedule.cron_expression = "0 12 * * *".to_string();
    schedule.timezone = "America/New_York".to_string();
    schedule.is_active = false;
    schedule.description = Some("Updated description".to_string());

    storage
        .update_schedule(schedule_id, &schedule)
        .await
        .unwrap();

    // Verify update
    let retrieved = storage.get_schedule(schedule_id).await.unwrap().unwrap();
    assert_eq!(retrieved.cron_expression, "0 12 * * *");
    assert_eq!(retrieved.timezone, "America/New_York");
    assert!(!retrieved.is_active);
    assert_eq!(
        retrieved.description,
        Some("Updated description".to_string())
    );
}

#[tokio::test]
async fn test_delete_schedule() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create and store schedule
    let schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    storage.store_schedule(&schedule).await.unwrap();
    assert_eq!(storage.schedule_count(), 1);

    // Delete schedule
    storage.delete_schedule(schedule_id).await.unwrap();

    // Verify deletion
    assert_eq!(storage.schedule_count(), 0);
    assert!(storage.get_schedule(schedule_id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_schedule() {
    let storage = setup_storage().await;
    let non_existent_id = Uuid::new_v4();

    let result = storage.delete_schedule(non_existent_id).await;
    assert!(result.is_err());
}

// ============================================================================
// Schedule Listing and Filtering Tests
// ============================================================================

#[tokio::test]
async fn test_list_all_schedules() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create multiple schedules
    for i in 0..5 {
        let schedule = create_test_schedule(workflow_id, &format!("0 {} * * *", i));
        storage.store_schedule(&schedule).await.unwrap();
    }

    // List all schedules
    let filter = ScheduleFilter::default();
    let schedules = storage.list_schedules(&filter).await.unwrap();
    assert_eq!(schedules.len(), 5);
}

#[tokio::test]
async fn test_filter_schedules_by_workflow() {
    let storage = setup_storage().await;
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

    // Create schedules for both workflows
    for _ in 0..3 {
        let schedule = create_test_schedule(workflow_id1, "0 0 * * *");
        storage.store_schedule(&schedule).await.unwrap();
    }
    for _ in 0..2 {
        let schedule = create_test_schedule(workflow_id2, "0 12 * * *");
        storage.store_schedule(&schedule).await.unwrap();
    }

    // Filter by workflow_id1
    let filter = ScheduleFilter {
        workflow_id: Some(workflow_id1),
        ..Default::default()
    };
    let schedules = storage.list_schedules(&filter).await.unwrap();
    assert_eq!(schedules.len(), 3);
    assert!(schedules.iter().all(|s| s.workflow_id == workflow_id1));
}

#[tokio::test]
async fn test_filter_schedules_by_active_status() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create active schedules
    for _ in 0..3 {
        let mut schedule = create_test_schedule(workflow_id, "0 0 * * *");
        schedule.is_active = true;
        storage.store_schedule(&schedule).await.unwrap();
    }

    // Create inactive schedules
    for _ in 0..2 {
        let mut schedule = create_test_schedule(workflow_id, "0 12 * * *");
        schedule.is_active = false;
        storage.store_schedule(&schedule).await.unwrap();
    }

    // Filter by active status
    let filter = ScheduleFilter {
        is_active: Some(true),
        ..Default::default()
    };
    let schedules = storage.list_schedules(&filter).await.unwrap();
    assert_eq!(schedules.len(), 3);
    assert!(schedules.iter().all(|s| s.is_active));
}

#[tokio::test]
async fn test_filter_schedules_by_created_by() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create schedules with different creators
    for _ in 0..3 {
        let mut schedule = create_test_schedule(workflow_id, "0 0 * * *");
        schedule.created_by = Some("user1".to_string());
        storage.store_schedule(&schedule).await.unwrap();
    }
    for _ in 0..2 {
        let mut schedule = create_test_schedule(workflow_id, "0 12 * * *");
        schedule.created_by = Some("user2".to_string());
        storage.store_schedule(&schedule).await.unwrap();
    }

    // Filter by created_by
    let filter = ScheduleFilter {
        created_by: Some("user1".to_string()),
        ..Default::default()
    };
    let schedules = storage.list_schedules(&filter).await.unwrap();
    assert_eq!(schedules.len(), 3);
    assert!(schedules
        .iter()
        .all(|s| s.created_by.as_ref() == Some(&"user1".to_string())));
}

#[tokio::test]
async fn test_schedule_pagination() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create 10 schedules
    for i in 0..10 {
        let schedule = create_test_schedule(workflow_id, &format!("0 {} * * *", i));
        storage.store_schedule(&schedule).await.unwrap();
    }

    // Test limit
    let filter = ScheduleFilter {
        limit: Some(5),
        ..Default::default()
    };
    let schedules = storage.list_schedules(&filter).await.unwrap();
    assert_eq!(schedules.len(), 5);
}

// ============================================================================
// Due Schedule Tests
// ============================================================================

#[tokio::test]
async fn test_get_due_schedules() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let now = Utc::now();

    // Create schedules with different next_run_at times
    // Schedule 1: Due now (should be returned)
    let mut schedule1 = create_test_schedule(workflow_id, "0 0 * * *");
    schedule1.next_run_at = Some(now - Duration::minutes(5));
    storage.store_schedule(&schedule1).await.unwrap();

    // Schedule 2: Due in future (should not be returned)
    let mut schedule2 = create_test_schedule(workflow_id, "0 12 * * *");
    schedule2.next_run_at = Some(now + Duration::hours(1));
    storage.store_schedule(&schedule2).await.unwrap();

    // Schedule 3: Due now but inactive (should not be returned)
    let mut schedule3 = create_test_schedule(workflow_id, "0 6 * * *");
    schedule3.next_run_at = Some(now - Duration::minutes(10));
    schedule3.is_active = false;
    storage.store_schedule(&schedule3).await.unwrap();

    // Schedule 4: Due now (should be returned)
    let mut schedule4 = create_test_schedule(workflow_id, "0 18 * * *");
    schedule4.next_run_at = Some(now - Duration::minutes(1));
    storage.store_schedule(&schedule4).await.unwrap();

    // Get due schedules
    let due_schedules = storage.get_due_schedules(now).await.unwrap();
    assert_eq!(due_schedules.len(), 2);
    assert!(due_schedules.iter().all(|s| s.is_active));
    assert!(due_schedules.iter().all(|s| s.next_run_at.unwrap() <= now));
}

#[tokio::test]
async fn test_no_due_schedules() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let now = Utc::now();

    // Create schedules all in the future
    for i in 0..3 {
        let mut schedule = create_test_schedule(workflow_id, &format!("0 {} * * *", i));
        schedule.next_run_at = Some(now + Duration::hours(i + 1));
        storage.store_schedule(&schedule).await.unwrap();
    }

    // No schedules should be due
    let due_schedules = storage.get_due_schedules(now).await.unwrap();
    assert_eq!(due_schedules.len(), 0);
}

// ============================================================================
// Schedule Run Tests
// ============================================================================

#[tokio::test]
async fn test_store_and_retrieve_schedule_runs() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    storage.store_schedule(&schedule).await.unwrap();

    // Store multiple runs
    for i in 0..5 {
        let execution_id = if i % 2 == 0 {
            Some(Uuid::new_v4())
        } else {
            None
        };
        let status = match i % 3 {
            0 => ScheduleRunStatus::Completed,
            1 => ScheduleRunStatus::Running,
            _ => ScheduleRunStatus::Failed,
        };
        let run = create_test_schedule_run(schedule_id, execution_id, status);
        storage.store_schedule_run(&run).await.unwrap();
    }

    // Retrieve runs
    let runs = storage.get_schedule_runs(schedule_id, None).await.unwrap();
    assert_eq!(runs.len(), 5);
}

#[tokio::test]
async fn test_schedule_run_pagination() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    storage.store_schedule(&schedule).await.unwrap();

    // Store 10 runs
    for _ in 0..10 {
        let run = create_test_schedule_run(
            schedule_id,
            Some(Uuid::new_v4()),
            ScheduleRunStatus::Completed,
        );
        storage.store_schedule_run(&run).await.unwrap();
    }

    // Get with limit
    let runs = storage
        .get_schedule_runs(schedule_id, Some(5))
        .await
        .unwrap();
    assert_eq!(runs.len(), 5);
}

#[tokio::test]
async fn test_schedule_run_status_tracking() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    storage.store_schedule(&schedule).await.unwrap();

    // Create runs with different statuses
    let statuses = vec![
        ScheduleRunStatus::Scheduled,
        ScheduleRunStatus::Running,
        ScheduleRunStatus::Completed,
        ScheduleRunStatus::Failed,
        ScheduleRunStatus::Skipped,
    ];

    for status in statuses {
        let run = create_test_schedule_run(schedule_id, Some(Uuid::new_v4()), status.clone());
        storage.store_schedule_run(&run).await.unwrap();
    }

    let runs = storage.get_schedule_runs(schedule_id, None).await.unwrap();
    assert_eq!(runs.len(), 5);

    // Verify all statuses are represented
    assert!(runs
        .iter()
        .any(|r| r.status == ScheduleRunStatus::Scheduled));
    assert!(runs.iter().any(|r| r.status == ScheduleRunStatus::Running));
    assert!(runs
        .iter()
        .any(|r| r.status == ScheduleRunStatus::Completed));
    assert!(runs.iter().any(|r| r.status == ScheduleRunStatus::Failed));
    assert!(runs.iter().any(|r| r.status == ScheduleRunStatus::Skipped));
}

// ============================================================================
// Schedule Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_schedule_activation_deactivation() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create active schedule
    let mut schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    schedule.is_active = true;
    storage.store_schedule(&schedule).await.unwrap();

    // Verify active
    let retrieved = storage.get_schedule(schedule_id).await.unwrap().unwrap();
    assert!(retrieved.is_active);

    // Deactivate
    schedule.is_active = false;
    storage
        .update_schedule(schedule_id, &schedule)
        .await
        .unwrap();

    // Verify inactive
    let retrieved = storage.get_schedule(schedule_id).await.unwrap().unwrap();
    assert!(!retrieved.is_active);

    // Reactivate
    schedule.is_active = true;
    storage
        .update_schedule(schedule_id, &schedule)
        .await
        .unwrap();

    // Verify active again
    let retrieved = storage.get_schedule(schedule_id).await.unwrap().unwrap();
    assert!(retrieved.is_active);
}

#[tokio::test]
async fn test_schedule_with_input_params() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Create schedule with custom input params
    let mut schedule = create_test_schedule(workflow_id, "0 0 * * *");
    schedule.input_params = Some(json!({
        "environment": "production",
        "retries": 3,
        "timeout": 300,
        "config": {
            "debug": false,
            "log_level": "info"
        }
    }));

    storage.store_schedule(&schedule).await.unwrap();

    // Retrieve and verify params
    let retrieved = storage.get_schedule(schedule.id).await.unwrap().unwrap();
    assert_eq!(retrieved.input_params, schedule.input_params);
    assert_eq!(retrieved.input_params.unwrap()["environment"], "production");
}

#[tokio::test]
async fn test_schedule_timezone_handling() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Test different timezones
    let timezones = vec![
        "UTC",
        "America/New_York",
        "Europe/London",
        "Asia/Tokyo",
        "Australia/Sydney",
    ];

    for tz in timezones {
        let mut schedule = create_test_schedule(workflow_id, "0 0 * * *");
        schedule.timezone = tz.to_string();
        storage.store_schedule(&schedule).await.unwrap();

        let retrieved = storage.get_schedule(schedule.id).await.unwrap().unwrap();
        assert_eq!(retrieved.timezone, tz);
    }
}

#[tokio::test]
async fn test_schedule_last_run_tracking() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let mut schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    schedule.last_run_at = None;
    storage.store_schedule(&schedule).await.unwrap();

    // Update with last run time
    let last_run = Utc::now();
    schedule.last_run_at = Some(last_run);
    storage
        .update_schedule(schedule_id, &schedule)
        .await
        .unwrap();

    // Verify last run was recorded
    let retrieved = storage.get_schedule(schedule_id).await.unwrap().unwrap();
    assert!(retrieved.last_run_at.is_some());
    assert_eq!(
        retrieved.last_run_at.unwrap().timestamp(),
        last_run.timestamp()
    );
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_storage_failure_handling() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Configure storage to fail
    storage.fail_store();

    let schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let result = storage.store_schedule(&schedule).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_concurrent_schedule_operations() {
    let storage = Arc::new(MockStorage::new());
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    // Spawn multiple tasks creating schedules concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let storage_clone = storage.clone();
        let wf_id = workflow_id;
        let handle = tokio::spawn(async move {
            let schedule = create_test_schedule(wf_id, &format!("0 {} * * *", i));
            storage_clone.store_schedule(&schedule).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all stored
    assert_eq!(storage.schedule_count(), 10);
}

#[tokio::test]
async fn test_delete_schedule_cascade() {
    let storage = setup_storage().await;
    let (workflow, metadata) = create_test_workflow("test-workflow");
    let workflow_id = metadata.id;

    storage.store_workflow(&workflow, &metadata).await.unwrap();

    let schedule = create_test_schedule(workflow_id, "0 0 * * *");
    let schedule_id = schedule.id;
    storage.store_schedule(&schedule).await.unwrap();

    // Create schedule runs
    for _ in 0..3 {
        let run = create_test_schedule_run(
            schedule_id,
            Some(Uuid::new_v4()),
            ScheduleRunStatus::Completed,
        );
        storage.store_schedule_run(&run).await.unwrap();
    }

    // Verify runs exist
    let runs = storage.get_schedule_runs(schedule_id, None).await.unwrap();
    assert_eq!(runs.len(), 3);

    // Delete schedule (should cascade to runs)
    storage.delete_schedule(schedule_id).await.unwrap();

    // Verify schedule deleted
    assert!(storage.get_schedule(schedule_id).await.unwrap().is_none());

    // Verify runs deleted
    let runs = storage.get_schedule_runs(schedule_id, None).await.unwrap();
    assert_eq!(runs.len(), 0);
}
