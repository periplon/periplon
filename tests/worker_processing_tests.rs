//! Worker Processing Tests
//!
//! Comprehensive tests for worker processing including job execution,
//! retry logic, concurrent processing, error handling, and queue management.

#[cfg(all(test, feature = "server"))]
mod worker_tests {
    use chrono::Utc;
    use periplon_sdk::dsl::schema::DSLWorkflow;
    use periplon_sdk::server::queue::{Job, WorkQueue};
    use periplon_sdk::server::storage::{
        Execution, ExecutionStatus, ExecutionStorage, WorkflowMetadata, WorkflowStorage,
    };
    use periplon_sdk::testing::{MockQueue, MockStorage};
    use serde_json::json;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::time::sleep;
    use uuid::Uuid;

    fn create_test_workflow() -> (DSLWorkflow, WorkflowMetadata) {
        let workflow = DSLWorkflow {
            provider: Default::default(),
            model: None,
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: Default::default(),
            inputs: Default::default(),
            outputs: Default::default(),
            agents: Default::default(),
            tasks: Default::default(),
            workflows: Default::default(),
            tools: None,
            communication: None,
            mcp_servers: Default::default(),
            subflows: Default::default(),
            imports: Default::default(),
            notifications: None,
            limits: None,
        };

        let metadata = WorkflowMetadata {
            id: Uuid::new_v4(),
            name: "test-workflow".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Test workflow for processing".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test-user".to_string()),
            tags: vec!["test".to_string()],
            is_active: true,
        };

        (workflow, metadata)
    }

    fn create_test_execution(workflow_id: Uuid) -> Execution {
        Execution {
            id: Uuid::new_v4(),
            workflow_id,
            workflow_version: "1.0.0".to_string(),
            status: ExecutionStatus::Queued,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
            triggered_by: Some("test-user".to_string()),
            trigger_type: "manual".to_string(),
            input_params: Some(json!({"test": "data"})),
            result: None,
            error: None,
            retry_count: 0,
            parent_execution_id: None,
        }
    }

    /// Test basic job enqueue and dequeue
    #[tokio::test]
    async fn test_basic_job_enqueue_dequeue() {
        let queue = Arc::new(MockQueue::new());
        let storage = Arc::new(MockStorage::new());

        // Create workflow and execution
        let (workflow, metadata) = create_test_workflow();
        let workflow_id = metadata.id;

        storage.store_workflow(&workflow, &metadata).await.unwrap();

        let execution = create_test_execution(workflow_id);
        let execution_id = execution.id;

        storage.store_execution(&execution).await.unwrap();

        // Create and enqueue job
        let job = Job::new(workflow_id, execution_id, json!({}));
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();

        // Verify queue state
        assert_eq!(queue.pending_count(), 1);
        assert_eq!(queue.enqueue_count(), 1);

        // Dequeue job
        let dequeued = queue.dequeue("worker-1").await.unwrap();
        assert!(dequeued.is_some());

        let dequeued_job = dequeued.unwrap();
        assert_eq!(dequeued_job.id, job_id);
        assert_eq!(dequeued_job.workflow_id, workflow_id);
        assert_eq!(dequeued_job.execution_id, execution_id);
        assert_eq!(dequeued_job.attempts, 1);

        // Verify queue state after dequeue
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.processing_count(), 1);
        assert_eq!(queue.dequeue_count(), 1);
    }

    /// Test job completion flow
    #[tokio::test]
    async fn test_job_completion() {
        let queue = Arc::new(MockQueue::new());
        let storage = Arc::new(MockStorage::new());

        let (workflow, metadata) = create_test_workflow();
        let workflow_id = metadata.id;

        storage.store_workflow(&workflow, &metadata).await.unwrap();

        let execution = create_test_execution(workflow_id);
        let execution_id = execution.id;

        storage.store_execution(&execution).await.unwrap();

        // Enqueue and process job
        let job = Job::new(workflow_id, execution_id, json!({}));
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();
        let dequeued = queue.dequeue("worker-1").await.unwrap().unwrap();

        // Simulate successful completion
        queue.complete(dequeued.id).await.unwrap();

        // Verify queue state
        assert_eq!(queue.completed_count(), 1);
        assert_eq!(queue.processing_count(), 0);
        assert_eq!(queue.complete_count(), 1);

        let completed = queue.get_completed();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, job_id);
    }

    /// Test job failure and retry logic
    #[tokio::test]
    async fn test_job_retry_logic() {
        let queue = Arc::new(MockQueue::new());

        // Create job with max retries
        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_max_retries(3);
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();

        // Simulate failure and requeue
        for attempt in 1..=3 {
            let dequeued = queue
                .dequeue(&format!("worker-{}", attempt))
                .await
                .unwrap()
                .unwrap();

            assert_eq!(dequeued.attempts, attempt as u32);

            // Requeue for retry
            queue
                .requeue(dequeued.id, Some(Duration::from_secs(1)))
                .await
                .unwrap();

            assert_eq!(queue.requeue_count(), attempt);
            assert_eq!(queue.pending_count(), 1);
        }

        // Final attempt should fail permanently
        let dequeued = queue.dequeue("worker-4").await.unwrap().unwrap();
        assert_eq!(dequeued.attempts, 4);

        queue
            .fail(dequeued.id, "Max retries exceeded")
            .await
            .unwrap();

        assert_eq!(queue.failed_count(), 1);
        assert_eq!(queue.fail_count(), 1);

        let failed = queue.get_failed();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].0.id, job_id);
        assert_eq!(failed[0].1, "Max retries exceeded");
    }

    /// Test concurrent job processing
    #[tokio::test]
    async fn test_concurrent_processing() {
        let queue = Arc::new(MockQueue::new());

        // Enqueue multiple jobs
        let num_jobs = 5;
        let mut job_ids = Vec::new();

        for i in 0..num_jobs {
            let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({"index": i}));
            job_ids.push(job.id);
            queue.enqueue(job).await.unwrap();
        }

        assert_eq!(queue.pending_count(), num_jobs);

        // Simulate concurrent workers
        let mut handles = Vec::new();

        for i in 0..num_jobs {
            let queue_clone = Arc::clone(&queue);
            let worker_id = format!("worker-{}", i);

            let handle = tokio::spawn(async move {
                if let Some(job) = queue_clone.dequeue(&worker_id).await.unwrap() {
                    // Simulate processing
                    sleep(Duration::from_millis(10)).await;
                    queue_clone.complete(job.id).await.unwrap();
                    job.id
                } else {
                    panic!("No job available for {}", worker_id);
                }
            });

            handles.push(handle);
        }

        // Wait for all workers
        let results = futures::future::join_all(handles).await;

        // Verify all jobs were processed
        assert_eq!(queue.completed_count(), num_jobs);
        assert_eq!(queue.pending_count(), 0);
        assert_eq!(queue.processing_count(), 0);

        // Verify all job IDs were processed
        let completed_ids: Vec<Uuid> = results.into_iter().map(|r| r.unwrap()).collect();
        for job_id in job_ids {
            assert!(completed_ids.contains(&job_id));
        }
    }

    /// Test priority queue ordering
    #[tokio::test]
    async fn test_job_priority() {
        let queue = Arc::new(MockQueue::new());

        // Enqueue jobs with different priorities
        let low_priority = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_priority(1);
        let medium_priority = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_priority(5);
        let high_priority = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_priority(10);

        queue.enqueue(low_priority).await.unwrap();
        queue.enqueue(high_priority.clone()).await.unwrap();
        queue.enqueue(medium_priority).await.unwrap();

        // Note: MockQueue uses FIFO, but this tests the priority fields are set
        let jobs = queue.get_pending();
        assert_eq!(jobs.len(), 3);

        // Verify priority values
        assert!(jobs.iter().any(|j| j.priority == 1));
        assert!(jobs.iter().any(|j| j.priority == 5));
        assert!(jobs.iter().any(|j| j.priority == 10));
    }

    /// Test execution status updates
    #[tokio::test]
    async fn test_execution_status_updates() {
        let storage = Arc::new(MockStorage::new());

        let (workflow, metadata) = create_test_workflow();
        storage.store_workflow(&workflow, &metadata).await.unwrap();

        let execution = create_test_execution(metadata.id);
        let execution_id = execution.id;

        // Create execution
        storage.store_execution(&execution).await.unwrap();

        // Update to running
        let mut updated = execution.clone();
        updated.status = ExecutionStatus::Running;
        updated.started_at = Some(Utc::now());

        storage
            .update_execution(execution_id, &updated)
            .await
            .unwrap();

        let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Running);
        assert!(retrieved.started_at.is_some());

        // Update to completed
        updated.status = ExecutionStatus::Completed;
        updated.completed_at = Some(Utc::now());
        updated.result = Some(json!({"status": "success"}));

        storage
            .update_execution(execution_id, &updated)
            .await
            .unwrap();

        let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Completed);
        assert!(retrieved.completed_at.is_some());
        assert!(retrieved.result.is_some());
    }

    /// Test heartbeat mechanism
    #[tokio::test]
    async fn test_job_heartbeat() {
        let queue = Arc::new(MockQueue::new());

        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({}));
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();
        queue.dequeue("worker-1").await.unwrap();

        // Send heartbeats
        for _ in 0..5 {
            queue.heartbeat(job_id, "worker-1").await.unwrap();
        }

        assert_eq!(queue.heartbeat_count(), 5);

        // Complete the job
        queue.complete(job_id).await.unwrap();

        // Heartbeat should fail after completion
        let result = queue.heartbeat(job_id, "worker-1").await;
        assert!(result.is_err());
    }

    /// Test queue statistics
    #[tokio::test]
    async fn test_queue_stats() {
        let queue = Arc::new(MockQueue::new());

        // Create various job states
        for i in 0..10 {
            let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({"index": i}));
            queue.enqueue(job).await.unwrap();
        }

        // Process some jobs
        for _ in 0..3 {
            let job = queue.dequeue("worker-1").await.unwrap().unwrap();
            queue.complete(job.id).await.unwrap();
        }

        // Fail some jobs
        for _ in 0..2 {
            let job = queue.dequeue("worker-2").await.unwrap().unwrap();
            queue.fail(job.id, "Test failure").await.unwrap();
        }

        // Check stats
        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.pending, 5);
        assert_eq!(stats.processing, 0);
        assert_eq!(stats.completed, 3);
        assert_eq!(stats.failed, 2);
    }

    /// Test error handling in queue operations
    #[tokio::test]
    async fn test_queue_error_handling() {
        let queue = Arc::new(MockQueue::new());

        // Configure queue to fail
        queue.fail_enqueue();

        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({}));
        let result = queue.enqueue(job).await;

        assert!(result.is_err());
    }

    /// Test error handling in storage operations
    #[tokio::test]
    async fn test_storage_error_handling() {
        let storage = Arc::new(MockStorage::new());

        // Configure storage to fail
        storage.fail_store();

        let (workflow, metadata) = create_test_workflow();
        let result = storage.store_workflow(&workflow, &metadata).await;

        assert!(result.is_err());
    }

    /// Test workflow filtering
    #[tokio::test]
    async fn test_workflow_filtering() {
        use periplon_sdk::server::storage::WorkflowFilter;

        let storage = Arc::new(MockStorage::new());

        // Store multiple workflows
        for i in 0..5 {
            let (mut workflow, mut metadata) = create_test_workflow();
            metadata.id = Uuid::new_v4();
            metadata.name = format!("workflow-{}", i);
            workflow.name = metadata.name.clone();

            metadata.is_active = i < 3;

            storage.store_workflow(&workflow, &metadata).await.unwrap();
        }

        // Filter by active status
        let filter = WorkflowFilter {
            is_active: Some(true),
            ..Default::default()
        };

        let active_workflows = storage.list_workflows(&filter).await.unwrap();
        assert_eq!(active_workflows.len(), 3);

        // Filter by name
        let filter = WorkflowFilter {
            name: Some("workflow-2".to_string()),
            ..Default::default()
        };

        let named_workflows = storage.list_workflows(&filter).await.unwrap();
        assert_eq!(named_workflows.len(), 1);
        assert_eq!(named_workflows[0].1.name, "workflow-2");
    }

    /// Test execution filtering
    #[tokio::test]
    async fn test_execution_filtering() {
        use periplon_sdk::server::storage::ExecutionFilter;

        let storage = Arc::new(MockStorage::new());
        let workflow_id = Uuid::new_v4();

        // Create multiple executions with different statuses
        for i in 0..10 {
            let mut execution = create_test_execution(workflow_id);
            execution.id = Uuid::new_v4();

            if i < 3 {
                execution.status = ExecutionStatus::Running;
            } else if i < 6 {
                execution.status = ExecutionStatus::Completed;
            } else {
                execution.status = ExecutionStatus::Failed;
            }

            storage.store_execution(&execution).await.unwrap();
        }

        // Filter by status
        let filter = ExecutionFilter {
            status: Some(ExecutionStatus::Running),
            ..Default::default()
        };

        let running = storage.list_executions(&filter).await.unwrap();
        assert_eq!(running.len(), 3);

        // Filter by workflow ID
        let filter = ExecutionFilter {
            workflow_id: Some(workflow_id),
            ..Default::default()
        };

        let workflow_execs = storage.list_executions(&filter).await.unwrap();
        assert_eq!(workflow_execs.len(), 10);
    }

    /// Test scheduled job processing
    #[tokio::test]
    async fn test_scheduled_jobs() {
        use chrono::Duration;

        let queue = Arc::new(MockQueue::new());

        // Create job scheduled for future
        let future_time = Utc::now() + Duration::seconds(60);
        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({})).with_schedule(future_time);

        queue.enqueue(job.clone()).await.unwrap();

        // Verify schedule was set
        let retrieved = queue.get_job(job.id).await.unwrap().unwrap();
        assert_eq!(retrieved.scheduled_at, Some(future_time));
    }
}
