//! Mock Storage for Testing
//!
//! Provides a configurable in-memory storage implementation for testing worker
//! processing, execution tracking, and workflow management without external dependencies.

#[cfg(feature = "server")]
use crate::dsl::schema::DSLWorkflow;
#[cfg(feature = "server")]
use crate::server::storage::{
    Checkpoint, CheckpointStorage, Execution, ExecutionFilter, ExecutionLog, ExecutionStatus,
    ExecutionStorage, Result, Schedule, ScheduleFilter, ScheduleRun, ScheduleStorage, StorageError,
    WorkflowFilter, WorkflowMetadata, WorkflowStorage,
};
#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use std::collections::HashMap;
#[cfg(feature = "server")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct MockStorage {
    state: Arc<Mutex<StorageState>>,
}

#[cfg(feature = "server")]
struct StorageState {
    workflows: HashMap<Uuid, (DSLWorkflow, WorkflowMetadata)>,
    executions: HashMap<Uuid, Execution>,
    execution_logs: HashMap<Uuid, Vec<ExecutionLog>>,
    checkpoints: HashMap<Uuid, Checkpoint>,
    schedules: HashMap<Uuid, Schedule>,
    schedule_runs: HashMap<Uuid, Vec<ScheduleRun>>,
    should_fail_get: bool,
    should_fail_store: bool,
}

#[cfg(feature = "server")]
impl MockStorage {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(StorageState {
                workflows: HashMap::new(),
                executions: HashMap::new(),
                execution_logs: HashMap::new(),
                checkpoints: HashMap::new(),
                schedules: HashMap::new(),
                schedule_runs: HashMap::new(),
                should_fail_get: false,
                should_fail_store: false,
            })),
        }
    }

    /// Configure storage to fail get operations
    pub fn fail_get(&self) {
        let mut state = self.state.lock().unwrap();
        state.should_fail_get = true;
    }

    /// Configure storage to fail store operations
    pub fn fail_store(&self) {
        let mut state = self.state.lock().unwrap();
        state.should_fail_store = true;
    }

    /// Get number of stored workflows
    pub fn workflow_count(&self) -> usize {
        self.state.lock().unwrap().workflows.len()
    }

    /// Get number of stored executions
    pub fn execution_count(&self) -> usize {
        self.state.lock().unwrap().executions.len()
    }

    /// Get all executions
    pub fn get_all_executions(&self) -> Vec<Execution> {
        self.state
            .lock()
            .unwrap()
            .executions
            .values()
            .cloned()
            .collect()
    }

    /// Get executions by status
    pub fn get_executions_by_status(&self, status: ExecutionStatus) -> Vec<Execution> {
        self.state
            .lock()
            .unwrap()
            .executions
            .values()
            .filter(|e| e.status == status)
            .cloned()
            .collect()
    }

    /// Clear all state
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.workflows.clear();
        state.executions.clear();
        state.execution_logs.clear();
        state.checkpoints.clear();
        state.schedules.clear();
        state.schedule_runs.clear();
        state.should_fail_get = false;
        state.should_fail_store = false;
    }

    /// Get number of stored schedules
    pub fn schedule_count(&self) -> usize {
        self.state.lock().unwrap().schedules.len()
    }
}

#[cfg(feature = "server")]
impl Default for MockStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl WorkflowStorage for MockStorage {
    async fn store_workflow(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<Uuid> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError("Store failure".to_string()));
        }

        state
            .workflows
            .insert(metadata.id, (workflow.clone(), metadata.clone()));
        Ok(metadata.id)
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let state = self.state.lock().unwrap();

        if state.should_fail_get {
            return Err(StorageError::IoError("Get failure".to_string()));
        }

        Ok(state.workflows.get(&id).cloned())
    }

    async fn update_workflow(
        &self,
        id: Uuid,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError("Update failure".to_string()));
        }

        if let std::collections::hash_map::Entry::Occupied(mut e) = state.workflows.entry(id) {
            e.insert((workflow.clone(), metadata.clone()));
            Ok(())
        } else {
            Err(StorageError::NotFound(format!("Workflow {} not found", id)))
        }
    }

    async fn delete_workflow(&self, id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.workflows.remove(&id).is_some() {
            Ok(())
        } else {
            Err(StorageError::NotFound(format!("Workflow {} not found", id)))
        }
    }

    async fn list_workflows(
        &self,
        filter: &WorkflowFilter,
    ) -> Result<Vec<(DSLWorkflow, WorkflowMetadata)>> {
        let state = self.state.lock().unwrap();

        let mut workflows: Vec<_> = state.workflows.values().cloned().collect();

        // Apply filters
        if let Some(ref name) = filter.name {
            workflows.retain(|(_, m)| m.name == *name);
        }

        if let Some(is_active) = filter.is_active {
            workflows.retain(|(_, m)| m.is_active == is_active);
        }

        if let Some(limit) = filter.limit {
            workflows.truncate(limit);
        }

        Ok(workflows)
    }

    async fn get_workflow_version(
        &self,
        id: Uuid,
        version: &str,
    ) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let state = self.state.lock().unwrap();

        Ok(state
            .workflows
            .get(&id)
            .filter(|(_, m)| m.version == version)
            .cloned())
    }

    async fn store_workflow_version(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        self.store_workflow(workflow, metadata).await?;
        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ExecutionStorage for MockStorage {
    async fn store_execution(&self, execution: &Execution) -> Result<Uuid> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError("Store execution failure".to_string()));
        }

        state.executions.insert(execution.id, execution.clone());
        Ok(execution.id)
    }

    async fn get_execution(&self, id: Uuid) -> Result<Option<Execution>> {
        let state = self.state.lock().unwrap();

        if state.should_fail_get {
            return Err(StorageError::IoError("Get execution failure".to_string()));
        }

        Ok(state.executions.get(&id).cloned())
    }

    async fn update_execution(&self, id: Uuid, execution: &Execution) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError(
                "Update execution failure".to_string(),
            ));
        }

        if let std::collections::hash_map::Entry::Occupied(mut e) = state.executions.entry(id) {
            e.insert(execution.clone());
            Ok(())
        } else {
            Err(StorageError::NotFound(format!(
                "Execution {} not found",
                id
            )))
        }
    }

    async fn list_executions(&self, filter: &ExecutionFilter) -> Result<Vec<Execution>> {
        let state = self.state.lock().unwrap();

        let mut executions: Vec<_> = state.executions.values().cloned().collect();

        // Apply filters
        if let Some(workflow_id) = filter.workflow_id {
            executions.retain(|e| e.workflow_id == workflow_id);
        }

        if let Some(ref status) = filter.status {
            executions.retain(|e| e.status == *status);
        }

        if let Some(ref triggered_by) = filter.triggered_by {
            executions.retain(|e| e.triggered_by.as_ref() == Some(triggered_by));
        }

        if let Some(limit) = filter.limit {
            executions.truncate(limit);
        }

        Ok(executions)
    }

    async fn store_execution_log(&self, log: &ExecutionLog) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        let logs = state.execution_logs.entry(log.execution_id).or_default();
        logs.push(log.clone());
        Ok(())
    }

    async fn get_execution_logs(
        &self,
        execution_id: Uuid,
        _limit: Option<usize>,
    ) -> Result<Vec<ExecutionLog>> {
        let state = self.state.lock().unwrap();
        Ok(state
            .execution_logs
            .get(&execution_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn delete_execution(&self, id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.executions.remove(&id).is_some() {
            state.execution_logs.remove(&id);
            Ok(())
        } else {
            Err(StorageError::NotFound(format!(
                "Execution {} not found",
                id
            )))
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl CheckpointStorage for MockStorage {
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Uuid> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError("Save checkpoint failure".to_string()));
        }

        state.checkpoints.insert(checkpoint.id, checkpoint.clone());
        Ok(checkpoint.id)
    }

    async fn get_checkpoint(&self, execution_id: Uuid, name: &str) -> Result<Option<Checkpoint>> {
        let state = self.state.lock().unwrap();

        if state.should_fail_get {
            return Err(StorageError::IoError("Get checkpoint failure".to_string()));
        }

        Ok(state
            .checkpoints
            .values()
            .find(|c| c.execution_id == execution_id && c.checkpoint_name == name)
            .cloned())
    }

    async fn list_checkpoints(&self, execution_id: Uuid) -> Result<Vec<Checkpoint>> {
        let state = self.state.lock().unwrap();

        Ok(state
            .checkpoints
            .values()
            .filter(|c| c.execution_id == execution_id)
            .cloned()
            .collect())
    }

    async fn delete_checkpoint(&self, id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.checkpoints.remove(&id);
        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ScheduleStorage for MockStorage {
    async fn store_schedule(&self, schedule: &Schedule) -> Result<Uuid> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError("Store schedule failure".to_string()));
        }

        state.schedules.insert(schedule.id, schedule.clone());
        Ok(schedule.id)
    }

    async fn get_schedule(&self, id: Uuid) -> Result<Option<Schedule>> {
        let state = self.state.lock().unwrap();

        if state.should_fail_get {
            return Err(StorageError::IoError("Get schedule failure".to_string()));
        }

        Ok(state.schedules.get(&id).cloned())
    }

    async fn update_schedule(&self, id: Uuid, schedule: &Schedule) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError("Update schedule failure".to_string()));
        }

        if let std::collections::hash_map::Entry::Occupied(mut e) = state.schedules.entry(id) {
            e.insert(schedule.clone());
            Ok(())
        } else {
            Err(StorageError::NotFound(format!("Schedule {} not found", id)))
        }
    }

    async fn delete_schedule(&self, id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();

        if state.schedules.remove(&id).is_some() {
            state.schedule_runs.remove(&id);
            Ok(())
        } else {
            Err(StorageError::NotFound(format!("Schedule {} not found", id)))
        }
    }

    async fn list_schedules(&self, filter: &ScheduleFilter) -> Result<Vec<Schedule>> {
        let state = self.state.lock().unwrap();

        let mut schedules: Vec<_> = state.schedules.values().cloned().collect();

        // Apply filters
        if let Some(workflow_id) = filter.workflow_id {
            schedules.retain(|s| s.workflow_id == workflow_id);
        }

        if let Some(is_active) = filter.is_active {
            schedules.retain(|s| s.is_active == is_active);
        }

        if let Some(ref created_by) = filter.created_by {
            schedules.retain(|s| s.created_by.as_ref() == Some(created_by));
        }

        if let Some(limit) = filter.limit {
            schedules.truncate(limit);
        }

        Ok(schedules)
    }

    async fn get_due_schedules(
        &self,
        before: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Schedule>> {
        let state = self.state.lock().unwrap();

        let schedules: Vec<_> = state
            .schedules
            .values()
            .filter(|s| s.is_active && s.next_run_at.map(|next| next <= before).unwrap_or(false))
            .cloned()
            .collect();

        Ok(schedules)
    }

    async fn store_schedule_run(&self, run: &ScheduleRun) -> Result<Uuid> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_store {
            return Err(StorageError::IoError(
                "Store schedule run failure".to_string(),
            ));
        }

        let runs = state.schedule_runs.entry(run.schedule_id).or_default();
        runs.push(run.clone());
        Ok(run.id)
    }

    async fn get_schedule_runs(
        &self,
        schedule_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<ScheduleRun>> {
        let state = self.state.lock().unwrap();

        let mut runs = state
            .schedule_runs
            .get(&schedule_id)
            .cloned()
            .unwrap_or_default();

        if let Some(limit) = limit {
            runs.truncate(limit);
        }

        Ok(runs)
    }
}

// Note: We implement the core storage traits needed for testing.
// Full Storage trait implementation requires many additional traits (OrganizationStorage,
// TeamStorage, ApiKeyStorage, etc.) which can be added as needed.

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_workflow() -> (DSLWorkflow, WorkflowMetadata) {
        let workflow = DSLWorkflow {
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
            description: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_by: None,
            tags: vec![],
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
            triggered_by: None,
            trigger_type: "manual".to_string(),
            input_params: None,
            result: None,
            error: None,
            retry_count: 0,
            parent_execution_id: None,
        }
    }

    #[tokio::test]
    async fn test_store_and_get_workflow() {
        let storage = MockStorage::new();
        let (workflow, metadata) = create_test_workflow();
        let workflow_id = metadata.id;

        storage.store_workflow(&workflow, &metadata).await.unwrap();
        assert_eq!(storage.workflow_count(), 1);

        let retrieved = storage.get_workflow(workflow_id).await.unwrap().unwrap();
        assert_eq!(retrieved.1.name, "test-workflow");
    }

    #[tokio::test]
    async fn test_create_and_get_execution() {
        let storage = MockStorage::new();
        let execution = create_test_execution(Uuid::new_v4());
        let execution_id = execution.id;

        storage.store_execution(&execution).await.unwrap();
        assert_eq!(storage.execution_count(), 1);

        let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Queued);
    }

    #[tokio::test]
    async fn test_update_execution() {
        let storage = MockStorage::new();
        let mut execution = create_test_execution(Uuid::new_v4());
        let execution_id = execution.id;

        storage.store_execution(&execution).await.unwrap();

        execution.status = ExecutionStatus::Running;
        storage
            .update_execution(execution_id, &execution)
            .await
            .unwrap();

        let retrieved = storage.get_execution(execution_id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, ExecutionStatus::Running);
    }

    #[tokio::test]
    async fn test_list_executions_by_status() {
        let storage = MockStorage::new();
        let workflow_id = Uuid::new_v4();

        for _ in 0..3 {
            let mut exec = create_test_execution(workflow_id);
            exec.status = ExecutionStatus::Running;
            storage.store_execution(&exec).await.unwrap();
        }

        for _ in 0..2 {
            let exec = create_test_execution(workflow_id);
            storage.store_execution(&exec).await.unwrap();
        }

        let running = storage.get_executions_by_status(ExecutionStatus::Running);
        assert_eq!(running.len(), 3);

        let queued = storage.get_executions_by_status(ExecutionStatus::Queued);
        assert_eq!(queued.len(), 2);
    }
}
