// Filesystem storage backend implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::{DateTime, Utc};
#[cfg(feature = "server")]
use serde_json;
#[cfg(feature = "server")]
use std::path::PathBuf;
#[cfg(feature = "server")]
use tokio::fs;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::*;
#[cfg(feature = "server")]
use crate::dsl::schema::DSLWorkflow;

#[cfg(feature = "server")]
pub struct FilesystemStorage {
    base_path: PathBuf,
    workflows_dir: String,
    executions_dir: String,
    checkpoints_dir: String,
    logs_dir: String,
}

#[cfg(feature = "server")]
impl FilesystemStorage {
    pub async fn new(
        base_path: PathBuf,
        workflows_dir: String,
        executions_dir: String,
        checkpoints_dir: String,
        logs_dir: String,
    ) -> Result<Self> {
        let storage = Self {
            base_path,
            workflows_dir,
            executions_dir,
            checkpoints_dir,
            logs_dir,
        };

        // Create directory structure
        storage.ensure_directories().await?;

        Ok(storage)
    }

    async fn ensure_directories(&self) -> Result<()> {
        let dirs = [
            &self.workflows_dir,
            &self.executions_dir,
            &self.checkpoints_dir,
            &self.logs_dir,
        ];

        for dir in dirs {
            let path = self.base_path.join(dir);
            fs::create_dir_all(&path)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    fn workflow_dir(&self, id: Uuid) -> PathBuf {
        self.base_path
            .join(&self.workflows_dir)
            .join(id.to_string())
    }

    fn execution_dir(&self, id: Uuid) -> PathBuf {
        self.base_path
            .join(&self.executions_dir)
            .join(id.to_string())
    }

    fn checkpoint_dir(&self, execution_id: Uuid) -> PathBuf {
        self.base_path
            .join(&self.checkpoints_dir)
            .join(execution_id.to_string())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl WorkflowStorage for FilesystemStorage {
    async fn store_workflow(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<Uuid> {
        let id = metadata.id;
        let workflow_dir = self.workflow_dir(id);

        // Create directory
        fs::create_dir_all(&workflow_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Write metadata
        let metadata_path = workflow_dir.join("metadata.json");
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(&metadata_path, metadata_json)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        // Write workflow YAML
        let definition_path = workflow_dir.join("definition.yaml");
        let yaml = serde_yaml::to_string(&workflow)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(&definition_path, yaml)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(id)
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let workflow_dir = self.workflow_dir(id);

        if !workflow_dir.exists() {
            return Ok(None);
        }

        // Read metadata
        let metadata_path = workflow_dir.join("metadata.json");
        let metadata_json = fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let metadata: WorkflowMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Read workflow YAML
        let definition_path = workflow_dir.join("definition.yaml");
        let yaml = fs::read_to_string(&definition_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let workflow: DSLWorkflow = serde_yaml::from_str(&yaml)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        Ok(Some((workflow, metadata)))
    }

    async fn update_workflow(
        &self,
        id: Uuid,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        let workflow_dir = self.workflow_dir(id);

        if !workflow_dir.exists() {
            return Err(StorageError::NotFound(format!("Workflow {} not found", id)));
        }

        self.store_workflow(workflow, metadata).await?;
        Ok(())
    }

    async fn delete_workflow(&self, id: Uuid) -> Result<()> {
        let workflow_dir = self.workflow_dir(id);

        if !workflow_dir.exists() {
            return Err(StorageError::NotFound(format!("Workflow {} not found", id)));
        }

        fs::remove_dir_all(&workflow_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn list_workflows(
        &self,
        filter: &WorkflowFilter,
    ) -> Result<Vec<(DSLWorkflow, WorkflowMetadata)>> {
        let workflows_path = self.base_path.join(&self.workflows_dir);
        let mut entries = fs::read_dir(&workflows_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let mut workflows = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            if !entry
                .file_type()
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?
                .is_dir()
            {
                continue;
            }

            let id_str = entry.file_name().to_string_lossy().to_string();
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some((workflow, metadata)) = self.get_workflow(id).await? {
                    // Apply filters
                    let mut matches = true;

                    if let Some(ref name) = filter.name {
                        if !metadata.name.contains(name) {
                            matches = false;
                        }
                    }

                    if !filter.tags.is_empty()
                        && !filter.tags.iter().any(|tag| metadata.tags.contains(tag)) {
                            matches = false;
                        }

                    if let Some(ref created_by) = filter.created_by {
                        if metadata.created_by.as_ref() != Some(created_by) {
                            matches = false;
                        }
                    }

                    if let Some(is_active) = filter.is_active {
                        if metadata.is_active != is_active {
                            matches = false;
                        }
                    }

                    if matches {
                        workflows.push((workflow, metadata));
                    }
                }
            }
        }

        // Apply limit/offset
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(workflows.len());

        workflows = workflows.into_iter().skip(offset).take(limit).collect();

        Ok(workflows)
    }

    async fn get_workflow_version(
        &self,
        id: Uuid,
        version: &str,
    ) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let workflow_dir = self.workflow_dir(id);
        let version_path = workflow_dir
            .join("versions")
            .join(format!("{}.yaml", version));

        if !version_path.exists() {
            return Ok(None);
        }

        // Read metadata (from main)
        let metadata_path = workflow_dir.join("metadata.json");
        let metadata_json = fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let metadata: WorkflowMetadata = serde_json::from_str(&metadata_json)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Read version YAML
        let yaml = fs::read_to_string(&version_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let workflow: DSLWorkflow = serde_yaml::from_str(&yaml)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        Ok(Some((workflow, metadata)))
    }

    async fn store_workflow_version(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        let workflow_dir = self.workflow_dir(metadata.id);
        let versions_dir = workflow_dir.join("versions");

        fs::create_dir_all(&versions_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let version_path = versions_dir.join(format!("{}.yaml", metadata.version));
        let yaml = serde_yaml::to_string(&workflow)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(&version_path, yaml)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ExecutionStorage for FilesystemStorage {
    async fn store_execution(&self, execution: &Execution) -> Result<Uuid> {
        let id = execution.id;
        let execution_dir = self.execution_dir(id);

        fs::create_dir_all(&execution_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let metadata_path = execution_dir.join("metadata.json");
        let json = serde_json::to_string_pretty(&execution)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(&metadata_path, json)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(id)
    }

    async fn get_execution(&self, id: Uuid) -> Result<Option<Execution>> {
        let execution_dir = self.execution_dir(id);

        if !execution_dir.exists() {
            return Ok(None);
        }

        let metadata_path = execution_dir.join("metadata.json");
        let json = fs::read_to_string(&metadata_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let execution: Execution = serde_json::from_str(&json)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        Ok(Some(execution))
    }

    async fn update_execution(&self, id: Uuid, execution: &Execution) -> Result<()> {
        let execution_dir = self.execution_dir(id);

        if !execution_dir.exists() {
            return Err(StorageError::NotFound(format!(
                "Execution {} not found",
                id
            )));
        }

        self.store_execution(execution).await?;
        Ok(())
    }

    async fn list_executions(&self, filter: &ExecutionFilter) -> Result<Vec<Execution>> {
        let executions_path = self.base_path.join(&self.executions_dir);
        let mut entries = fs::read_dir(&executions_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let mut executions = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            if !entry
                .file_type()
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?
                .is_dir()
            {
                continue;
            }

            let id_str = entry.file_name().to_string_lossy().to_string();
            if let Ok(id) = Uuid::parse_str(&id_str) {
                if let Some(execution) = self.get_execution(id).await? {
                    let mut matches = true;

                    if let Some(workflow_id) = filter.workflow_id {
                        if execution.workflow_id != workflow_id {
                            matches = false;
                        }
                    }

                    if let Some(ref status) = filter.status {
                        if &execution.status != status {
                            matches = false;
                        }
                    }

                    if matches {
                        executions.push(execution);
                    }
                }
            }
        }

        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(executions.len());

        executions = executions.into_iter().skip(offset).take(limit).collect();

        Ok(executions)
    }

    async fn store_execution_log(&self, log: &ExecutionLog) -> Result<()> {
        let execution_dir = self.execution_dir(log.execution_id);
        let logs_dir = execution_dir.join("logs");

        fs::create_dir_all(&logs_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let log_file = logs_dir.join("current.jsonl");
        let log_line = serde_json::to_string(&log)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Append to log file
        let mut content = if log_file.exists() {
            fs::read_to_string(&log_file)
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?
        } else {
            String::new()
        };

        content.push_str(&log_line);
        content.push('\n');

        fs::write(&log_file, content)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn get_execution_logs(
        &self,
        execution_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionLog>> {
        let execution_dir = self.execution_dir(execution_id);
        let log_file = execution_dir.join("logs").join("current.jsonl");

        if !log_file.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_file)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let mut logs: Vec<ExecutionLog> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();

        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        Ok(logs)
    }

    async fn delete_execution(&self, id: Uuid) -> Result<()> {
        let execution_dir = self.execution_dir(id);

        if !execution_dir.exists() {
            return Err(StorageError::NotFound(format!(
                "Execution {} not found",
                id
            )));
        }

        fs::remove_dir_all(&execution_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl CheckpointStorage for FilesystemStorage {
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Uuid> {
        let checkpoint_dir = self.checkpoint_dir(checkpoint.execution_id);

        fs::create_dir_all(&checkpoint_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let checkpoint_file = checkpoint_dir.join(format!("{}.json", checkpoint.checkpoint_name));
        let json = serde_json::to_string_pretty(&checkpoint)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        fs::write(&checkpoint_file, json)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        Ok(checkpoint.id)
    }

    async fn get_checkpoint(&self, execution_id: Uuid, name: &str) -> Result<Option<Checkpoint>> {
        let checkpoint_dir = self.checkpoint_dir(execution_id);
        let checkpoint_file = checkpoint_dir.join(format!("{}.json", name));

        if !checkpoint_file.exists() {
            return Ok(None);
        }

        let json = fs::read_to_string(&checkpoint_file)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        let checkpoint: Checkpoint = serde_json::from_str(&json)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        Ok(Some(checkpoint))
    }

    async fn list_checkpoints(&self, execution_id: Uuid) -> Result<Vec<Checkpoint>> {
        let checkpoint_dir = self.checkpoint_dir(execution_id);

        if !checkpoint_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = fs::read_dir(&checkpoint_dir)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let mut checkpoints = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?
        {
            if entry
                .file_type()
                .await
                .map_err(|e| StorageError::IoError(e.to_string()))?
                .is_file()
            {
                let path = entry.path();
                let json = fs::read_to_string(&path)
                    .await
                    .map_err(|e| StorageError::IoError(e.to_string()))?;
                if let Ok(checkpoint) = serde_json::from_str::<Checkpoint>(&json) {
                    checkpoints.push(checkpoint);
                }
            }
        }

        Ok(checkpoints)
    }

    async fn delete_checkpoint(&self, id: Uuid) -> Result<()> {
        // Note: This is a simplified implementation
        // In a real system, you'd need to search for the checkpoint
        Err(StorageError::NotFound(format!(
            "Checkpoint {} not found",
            id
        )))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ScheduleStorage for FilesystemStorage {
    async fn store_schedule(&self, _schedule: &Schedule) -> Result<Uuid> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_schedule(&self, _id: Uuid) -> Result<Option<Schedule>> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn update_schedule(&self, _id: Uuid, _schedule: &Schedule) -> Result<()> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn delete_schedule(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn list_schedules(&self, _filter: &ScheduleFilter) -> Result<Vec<Schedule>> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_due_schedules(&self, _before: DateTime<Utc>) -> Result<Vec<Schedule>> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn store_schedule_run(&self, _run: &ScheduleRun) -> Result<Uuid> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_schedule_runs(
        &self,
        _schedule_id: Uuid,
        _limit: Option<usize>,
    ) -> Result<Vec<ScheduleRun>> {
        Err(StorageError::IoError(
            "Schedule storage not implemented for filesystem backend".to_string(),
        ))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl OrganizationStorage for FilesystemStorage {
    async fn store_organization(&self, _organization: &Organization) -> Result<Uuid> {
        Err(StorageError::IoError(
            "Organization storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_organization(&self, _id: Uuid) -> Result<Option<Organization>> {
        Err(StorageError::IoError(
            "Organization storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_organization_by_slug(&self, _slug: &str) -> Result<Option<Organization>> {
        Err(StorageError::IoError(
            "Organization storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn update_organization(&self, _id: Uuid, _organization: &Organization) -> Result<()> {
        Err(StorageError::IoError(
            "Organization storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn delete_organization(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::IoError(
            "Organization storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn list_organizations(&self, _filter: &OrganizationFilter) -> Result<Vec<Organization>> {
        Err(StorageError::IoError(
            "Organization storage not implemented for filesystem backend".to_string(),
        ))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl TeamStorage for FilesystemStorage {
    async fn store_team(&self, _team: &Team) -> Result<Uuid> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_team(&self, _id: Uuid) -> Result<Option<Team>> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn update_team(&self, _id: Uuid, _team: &Team) -> Result<()> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn delete_team(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn list_teams(&self, _filter: &TeamFilter) -> Result<Vec<Team>> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn add_team_member(&self, _member: &TeamMember) -> Result<Uuid> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn remove_team_member(&self, _team_id: Uuid, _user_id: Uuid) -> Result<()> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_team_members(&self, _team_id: Uuid) -> Result<Vec<TeamMember>> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_user_teams(&self, _user_id: Uuid) -> Result<Vec<Team>> {
        Err(StorageError::IoError(
            "Team storage not implemented for filesystem backend".to_string(),
        ))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ApiKeyStorage for FilesystemStorage {
    async fn store_api_key(&self, _api_key: &ApiKey) -> Result<Uuid> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_api_key(&self, _id: Uuid) -> Result<Option<ApiKey>> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn get_api_key_by_hash(&self, _key_hash: &str) -> Result<Option<ApiKey>> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn update_api_key(&self, _id: Uuid, _api_key: &ApiKey) -> Result<()> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn revoke_api_key(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn delete_api_key(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn list_api_keys(&self, _filter: &ApiKeyFilter) -> Result<Vec<ApiKey>> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }

    async fn update_last_used(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::IoError(
            "API key storage not implemented for filesystem backend".to_string(),
        ))
    }
}
