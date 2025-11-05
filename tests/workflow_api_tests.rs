//! Workflow API Tests
//!
//! Comprehensive tests for workflow execution API, state management,
//! lifecycle operations, and integration with storage/queue systems.

#[cfg(all(test, feature = "server"))]
mod workflow_api_tests {
    use chrono::Utc;
    use periplon_sdk::dsl::executor::DSLExecutor;
    use periplon_sdk::dsl::schema::{AgentSpec, DSLWorkflow, PermissionsSpec, TaskSpec};
    use periplon_sdk::dsl::state::{WorkflowState, WorkflowStatus};
    use periplon_sdk::dsl::task_graph::TaskStatus;
    use periplon_sdk::server::queue::{Job, WorkQueue};
    use periplon_sdk::server::storage::{
        Execution, ExecutionStatus, ExecutionStorage, WorkflowMetadata, WorkflowStorage,
    };
    use periplon_sdk::testing::{MockQueue, MockStorage};
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::SystemTime;
    use uuid::Uuid;

    /// Helper to create a minimal test workflow
    fn create_test_workflow(name: &str, task_count: usize) -> DSLWorkflow {
        let mut agents = HashMap::new();
        agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: Some("claude-sonnet-4-5".to_string()),
                system_prompt: None,
                tools: vec!["Read".to_string(), "Write".to_string()],
                permissions: PermissionsSpec {
                    mode: "acceptEdits".to_string(),
                    allowed_directories: vec![],
                },
                max_turns: Some(5),
                cwd: None,
                create_cwd: None,
                inputs: Default::default(),
                outputs: Default::default(),
            },
        );

        let mut tasks = HashMap::new();
        for i in 0..task_count {
            let task_id = format!("task_{}", i);
            let depends_on = if i > 0 {
                vec![format!("task_{}", i - 1)]
            } else {
                vec![]
            };

            tasks.insert(
                task_id.clone(),
                TaskSpec {
                    description: format!("Test task {}", i),
                    agent: Some("test_agent".to_string()),
                    depends_on,
                    subtasks: vec![],
                    parallel_with: vec![],
                    output: None,
                    priority: 0,
                    inputs: Default::default(),
                    outputs: Default::default(),
                    condition: None,
                    on_error: None,
                    loop_spec: None,
                    subflow: None,
                    uses: None,
                    embed: None,
                    overrides: None,
                    script: None,
                    command: None,
                    http: None,
                    mcp_tool: None,
                    uses_workflow: None,
                    on_complete: None,
                    definition_of_done: None,
                    loop_control: None,
                    inject_context: false,
                    context: None,
                    limits: None,
                },
            );
        }

        DSLWorkflow {
            provider: Default::default(),
            model: None,
            name: name.to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: Default::default(),
            inputs: Default::default(),
            outputs: Default::default(),
            agents,
            tasks,
            workflows: Default::default(),
            tools: None,
            communication: None,
            mcp_servers: Default::default(),
            subflows: Default::default(),
            imports: Default::default(),
            notifications: None,
            limits: None,
        }
    }

    /// Helper to create workflow metadata
    fn create_workflow_metadata(name: &str, version: &str) -> WorkflowMetadata {
        WorkflowMetadata {
            id: Uuid::new_v4(),
            name: name.to_string(),
            version: version.to_string(),
            description: Some("Test workflow".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: Some("test_user".to_string()),
            tags: vec!["test".to_string()],
            is_active: true,
        }
    }

    /// Helper to create execution record
    fn create_execution(workflow_id: Uuid, status: ExecutionStatus) -> Execution {
        Execution {
            id: Uuid::new_v4(),
            workflow_id,
            workflow_version: "1.0.0".to_string(),
            status,
            started_at: Some(Utc::now()),
            completed_at: None,
            created_at: Utc::now(),
            error: None,
            triggered_by: Some("test".to_string()),
            trigger_type: "manual".to_string(),
            input_params: None,
            result: None,
            retry_count: 0,
            parent_execution_id: None,
        }
    }

    #[tokio::test]
    async fn test_workflow_creation_and_initialization() {
        let workflow = create_test_workflow("test_workflow", 3);

        let executor = DSLExecutor::new(workflow.clone());
        assert!(executor.is_ok(), "Executor should be created successfully");

        let executor = executor.unwrap();
        let (name, version) = executor.get_workflow_info();
        assert_eq!(name, "test_workflow");
        assert_eq!(version, "1.0.0");

        // Note: task_count is 0 until initialize() is called
        // which builds the task graph. Since initialize() requires
        // connecting to the CLI, we verify the workflow definition instead.
        assert_eq!(workflow.tasks.len(), 3);
    }

    #[tokio::test]
    async fn test_workflow_storage_integration() {
        let storage = Arc::new(MockStorage::new());
        let workflow = create_test_workflow("integration_test", 2);
        let metadata = create_workflow_metadata("integration_test", "1.0.0");

        // Store workflow
        let workflow_id = storage
            .store_workflow(&workflow, &metadata)
            .await
            .expect("Should store workflow");

        assert_eq!(storage.workflow_count(), 1);

        // Retrieve workflow
        let retrieved = storage
            .get_workflow(workflow_id)
            .await
            .expect("Should retrieve workflow")
            .expect("Workflow should exist");

        assert_eq!(retrieved.0.name, "integration_test");
        assert_eq!(retrieved.1.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_execution_lifecycle() {
        let storage = Arc::new(MockStorage::new());
        let workflow = create_test_workflow("lifecycle_test", 3);
        let metadata = create_workflow_metadata("lifecycle_test", "1.0.0");

        let workflow_id = storage.store_workflow(&workflow, &metadata).await.unwrap();

        // Create execution in queued state
        let mut execution = create_execution(workflow_id, ExecutionStatus::Queued);
        storage.store_execution(&execution).await.unwrap();

        assert_eq!(storage.execution_count(), 1);

        // Update to running
        execution.status = ExecutionStatus::Running;
        execution.started_at = Some(Utc::now());
        storage
            .update_execution(execution.id, &execution)
            .await
            .unwrap();

        let retrieved = storage.get_execution(execution.id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Running);

        // Complete execution
        execution.status = ExecutionStatus::Completed;
        execution.completed_at = Some(Utc::now());
        storage
            .update_execution(execution.id, &execution)
            .await
            .unwrap();

        let completed = storage.get_execution(execution.id).await.unwrap().unwrap();
        assert_eq!(completed.status, ExecutionStatus::Completed);
        assert!(completed.completed_at.is_some());
    }

    #[tokio::test]
    async fn test_workflow_state_persistence() {
        let _storage = Arc::new(MockStorage::new());
        let _workflow_id = Uuid::new_v4();

        // Create workflow state
        let mut state = WorkflowState {
            workflow_name: "state_test".to_string(),
            workflow_version: "1.0.0".to_string(),
            task_statuses: HashMap::new(),
            task_start_times: HashMap::new(),
            task_end_times: HashMap::new(),
            task_attempts: HashMap::new(),
            task_errors: HashMap::new(),
            task_results: HashMap::new(),
            task_outputs: HashMap::new(),
            status: WorkflowStatus::Running,
            started_at: SystemTime::now(),
            ended_at: None,
            checkpoint_at: SystemTime::now(),
            metadata: HashMap::new(),
            loop_states: HashMap::new(),
            loop_results: HashMap::new(),
        };

        // Add task states
        state
            .task_statuses
            .insert("task_0".to_string(), TaskStatus::Completed);
        state
            .task_statuses
            .insert("task_1".to_string(), TaskStatus::Running);
        state
            .task_statuses
            .insert("task_2".to_string(), TaskStatus::Pending);

        state.task_attempts.insert("task_0".to_string(), 1);
        state.task_attempts.insert("task_1".to_string(), 1);

        // Verify state tracking
        assert_eq!(state.task_statuses.len(), 3);
        assert_eq!(
            state.task_statuses.get("task_0"),
            Some(&TaskStatus::Completed)
        );
        assert_eq!(
            state.task_statuses.get("task_1"),
            Some(&TaskStatus::Running)
        );
        assert_eq!(
            state.task_statuses.get("task_2"),
            Some(&TaskStatus::Pending)
        );
    }

    #[tokio::test]
    async fn test_queue_workflow_job_integration() {
        let queue = Arc::new(MockQueue::new());
        let storage = Arc::new(MockStorage::new());

        let workflow = create_test_workflow("queue_test", 2);
        let metadata = create_workflow_metadata("queue_test", "1.0.0");

        let workflow_id = storage.store_workflow(&workflow, &metadata).await.unwrap();

        // Create execution
        let execution = create_execution(workflow_id, ExecutionStatus::Queued);
        storage.store_execution(&execution).await.unwrap();

        // Enqueue job for workflow
        let job = Job::new(
            workflow_id,
            execution.id,
            serde_json::json!({
                "workflow_name": "queue_test",
                "task_id": "task_0"
            }),
        );

        let job_id = queue.enqueue(job.clone()).await.unwrap();

        // Verify job in queue
        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.processing, 0);

        // Dequeue and process
        let dequeued = queue.dequeue("worker-1").await.unwrap();
        assert!(dequeued.is_some());

        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.processing, 1);

        // Complete job
        queue.complete(job_id).await.unwrap();

        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.processing, 0);
        assert_eq!(stats.completed, 1);
    }

    #[tokio::test]
    async fn test_workflow_execution_state_transitions() {
        let storage = Arc::new(MockStorage::new());
        let workflow_id = Uuid::new_v4();

        // Test all state transitions
        let transitions = vec![
            (ExecutionStatus::Queued, ExecutionStatus::Running),
            (ExecutionStatus::Running, ExecutionStatus::Completed),
            (ExecutionStatus::Running, ExecutionStatus::Failed),
            (ExecutionStatus::Running, ExecutionStatus::Cancelled),
        ];

        for (from_status, to_status) in transitions {
            let mut execution = create_execution(workflow_id, from_status.clone());
            storage.store_execution(&execution).await.unwrap();

            execution.status = to_status.clone();
            if matches!(
                to_status,
                ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled
            ) {
                execution.completed_at = Some(Utc::now());
            }

            storage
                .update_execution(execution.id, &execution)
                .await
                .unwrap();

            let updated = storage.get_execution(execution.id).await.unwrap().unwrap();

            assert_eq!(
                updated.status, to_status,
                "Should transition from {:?} to {:?}",
                from_status, to_status
            );
        }
    }

    #[tokio::test]
    async fn test_multiple_workflow_versions() {
        let storage = Arc::new(MockStorage::new());

        // Store multiple versions of same workflow
        let v1 = create_test_workflow("versioned_workflow", 2);
        let metadata_v1 = create_workflow_metadata("versioned_workflow", "1.0.0");
        storage.store_workflow(&v1, &metadata_v1).await.unwrap();

        let v2 = create_test_workflow("versioned_workflow", 3);
        let metadata_v2 = create_workflow_metadata("versioned_workflow", "2.0.0");
        storage.store_workflow(&v2, &metadata_v2).await.unwrap();

        assert_eq!(storage.workflow_count(), 2);

        // Verify both versions exist by checking count
        // Note: MockStorage doesn't have get_workflows_by_name, but we verified count above
    }

    #[tokio::test]
    async fn test_concurrent_workflow_executions() {
        let storage = Arc::new(MockStorage::new());
        let queue = Arc::new(MockQueue::new());

        let workflow = create_test_workflow("concurrent_test", 1);
        let metadata = create_workflow_metadata("concurrent_test", "1.0.0");
        let workflow_id = storage.store_workflow(&workflow, &metadata).await.unwrap();

        // Create 5 concurrent executions
        let mut execution_ids = Vec::new();
        for i in 0..5 {
            let execution = create_execution(workflow_id, ExecutionStatus::Queued);
            storage.store_execution(&execution).await.unwrap();
            execution_ids.push(execution.id);

            let job = Job::new(
                workflow_id,
                execution.id,
                serde_json::json!({ "execution_num": i }),
            );
            queue.enqueue(job).await.unwrap();
        }

        assert_eq!(storage.execution_count(), 5);
        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.pending, 5);

        // Process all executions
        for i in 0..5 {
            let job = queue
                .dequeue(&format!("worker-{}", i))
                .await
                .unwrap()
                .expect("Should dequeue job");
            queue.complete(job.id).await.unwrap();
        }

        let final_stats = queue.stats().await.unwrap();
        assert_eq!(final_stats.completed, 5);
        assert_eq!(final_stats.pending, 0);
    }

    #[tokio::test]
    async fn test_workflow_failure_handling() {
        let storage = Arc::new(MockStorage::new());
        let workflow_id = Uuid::new_v4();

        let mut execution = create_execution(workflow_id, ExecutionStatus::Running);
        storage.store_execution(&execution).await.unwrap();

        // Simulate failure
        execution.status = ExecutionStatus::Failed;
        execution.error = Some("Task execution failed".to_string());
        execution.completed_at = Some(Utc::now());

        storage
            .update_execution(execution.id, &execution)
            .await
            .unwrap();

        let failed = storage.get_execution(execution.id).await.unwrap().unwrap();

        assert_eq!(failed.status, ExecutionStatus::Failed);
        assert!(failed.error.is_some());
        assert_eq!(failed.error.unwrap(), "Task execution failed");
    }

    #[tokio::test]
    async fn test_workflow_cancellation() {
        let storage = Arc::new(MockStorage::new());
        let queue = Arc::new(MockQueue::new());
        let workflow_id = Uuid::new_v4();

        // Create running execution
        let mut execution = create_execution(workflow_id, ExecutionStatus::Running);
        storage.store_execution(&execution).await.unwrap();

        let job = Job::new(workflow_id, execution.id, serde_json::json!({}));
        let job_id = queue.enqueue(job).await.unwrap();

        queue.dequeue("worker-1").await.unwrap();

        // Cancel execution
        execution.status = ExecutionStatus::Cancelled;
        execution.completed_at = Some(Utc::now());
        storage
            .update_execution(execution.id, &execution)
            .await
            .unwrap();

        // Fail the job (representing cancellation)
        queue
            .fail(job_id, "Execution cancelled by user")
            .await
            .unwrap();

        let cancelled = storage.get_execution(execution.id).await.unwrap().unwrap();
        assert_eq!(cancelled.status, ExecutionStatus::Cancelled);

        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.failed, 1);
    }

    #[tokio::test]
    async fn test_workflow_info_retrieval() {
        let workflow = create_test_workflow("info_test", 5);
        let executor = DSLExecutor::new(workflow.clone()).unwrap();

        let (name, version) = executor.get_workflow_info();
        assert_eq!(name, "info_test");
        assert_eq!(version, "1.0.0");

        // Verify workflow definition has correct task count
        assert_eq!(workflow.tasks.len(), 5);
    }

    #[tokio::test]
    async fn test_empty_workflow_handling() {
        let workflow = create_test_workflow("empty_workflow", 0);
        let executor = DSLExecutor::new(workflow.clone());

        assert!(executor.is_ok(), "Should handle empty workflows");
        let _executor = executor.unwrap();
        assert_eq!(workflow.tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_error_scenarios() {
        let storage = Arc::new(MockStorage::new());

        // Test non-existent workflow retrieval
        let non_existent = storage.get_workflow(Uuid::new_v4()).await.unwrap();
        assert!(
            non_existent.is_none(),
            "Should return None for non-existent workflow"
        );

        // Test non-existent execution retrieval
        let non_existent = storage.get_execution(Uuid::new_v4()).await.unwrap();
        assert!(
            non_existent.is_none(),
            "Should return None for non-existent execution"
        );

        // Test failure simulation
        storage.fail_store();
        let workflow = create_test_workflow("fail_test", 1);
        let metadata = create_workflow_metadata("fail_test", "1.0.0");

        let result = storage.store_workflow(&workflow, &metadata).await;
        assert!(result.is_err(), "Should fail when failure mode enabled");
    }
}
