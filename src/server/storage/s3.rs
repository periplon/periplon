// S3 storage backend implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use aws_sdk_s3::{primitives::ByteStream, Client as S3Client};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::{
    ApiKey, ApiKeyFilter, ApiKeyStorage, Checkpoint, CheckpointStorage, Execution, ExecutionFilter,
    ExecutionLog, ExecutionStorage, Organization, OrganizationFilter, OrganizationStorage, Result,
    Schedule, ScheduleFilter, ScheduleRun, ScheduleStorage, StorageError, Team,
    TeamFilter, TeamMember, TeamStorage, WorkflowFilter, WorkflowMetadata, WorkflowStorage,
};
#[cfg(feature = "server")]
use crate::dsl::schema::DSLWorkflow;
#[cfg(feature = "server")]
use chrono::{DateTime, Utc};

/// S3-based storage backend
///
/// Structure:
/// - {prefix}/workflows/{id}.json - Workflow definition and metadata
/// - {prefix}/workflows/{id}/versions/{version}.json - Workflow versions
/// - {prefix}/executions/{id}.json - Execution metadata
/// - {prefix}/executions/{id}/logs/{timestamp}.json - Execution logs
/// - {prefix}/checkpoints/{execution_id}/{name}.json - Checkpoints
#[cfg(feature = "server")]
pub struct S3Storage {
    client: S3Client,
    bucket: String,
    prefix: String,
}

#[cfg(feature = "server")]
impl S3Storage {
    pub async fn new(
        endpoint: String,
        region: String,
        bucket: String,
        access_key_id: String,
        secret_access_key: String,
        prefix: Option<String>,
    ) -> Result<Self> {
        use aws_sdk_s3::config::{Credentials, Region};

        let creds = Credentials::new(access_key_id, secret_access_key, None, None, "s3-storage");

        let region = Region::new(region);

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .region(region)
            .credentials_provider(creds)
            .load()
            .await;

        let mut s3_config = aws_sdk_s3::config::Builder::from(&config);

        if !endpoint.is_empty() {
            s3_config = s3_config.endpoint_url(endpoint);
        }

        let client = S3Client::from_conf(s3_config.build());

        Ok(Self {
            client,
            bucket,
            prefix: prefix.unwrap_or_else(|| "dsl-storage".to_string()),
        })
    }

    pub fn from_client(client: S3Client, bucket: String, prefix: Option<String>) -> Self {
        Self {
            client,
            bucket,
            prefix: prefix.unwrap_or_else(|| "dsl-storage".to_string()),
        }
    }

    fn workflow_key(&self, id: Uuid) -> String {
        format!("{}/workflows/{}.json", self.prefix, id)
    }

    fn workflow_version_key(&self, id: Uuid, version: &str) -> String {
        format!("{}/workflows/{}/versions/{}.json", self.prefix, id, version)
    }

    fn execution_key(&self, id: Uuid) -> String {
        format!("{}/executions/{}.json", self.prefix, id)
    }

    fn execution_log_key(&self, execution_id: Uuid, timestamp: i64) -> String {
        format!(
            "{}/executions/{}/logs/{}.json",
            self.prefix, execution_id, timestamp
        )
    }

    fn checkpoint_key(&self, execution_id: Uuid, name: &str) -> String {
        format!("{}/checkpoints/{}/{}.json", self.prefix, execution_id, name)
    }

    async fn put_json<T: serde::Serialize>(&self, key: &str, data: &T) -> Result<()> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(json.into_bytes()))
            .content_type("application/json")
            .send()
            .await
            .map_err(|e| StorageError::S3Error(format!("Failed to put object: {}", e)))?;

        Ok(())
    }

    async fn get_json<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        match self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(output) => {
                let bytes = output
                    .body
                    .collect()
                    .await
                    .map_err(|e| StorageError::S3Error(format!("Failed to read object: {}", e)))?
                    .into_bytes();

                let data: T = serde_json::from_slice(&bytes)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                Ok(Some(data))
            }
            Err(e) => {
                if e.to_string().contains("NoSuchKey") {
                    Ok(None)
                } else {
                    Err(StorageError::S3Error(format!(
                        "Failed to get object: {}",
                        e
                    )))
                }
            }
        }
    }

    async fn delete_object(&self, key: &str) -> Result<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::S3Error(format!("Failed to delete object: {}", e)))?;

        Ok(())
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.bucket)
                .prefix(prefix);

            if let Some(token) = continuation_token {
                request = request.continuation_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| StorageError::S3Error(format!("Failed to list objects: {}", e)))?;

            if let Some(contents) = response.contents {
                for object in contents {
                    if let Some(key) = object.key {
                        keys.push(key);
                    }
                }
            }

            if response.is_truncated == Some(true) {
                continuation_token = response.next_continuation_token;
            } else {
                break;
            }
        }

        Ok(keys)
    }
}

// ============================================================================
// WorkflowStorage Implementation
// ============================================================================

#[cfg(feature = "server")]
#[async_trait]
impl WorkflowStorage for S3Storage {
    async fn store_workflow(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<Uuid> {
        let key = self.workflow_key(metadata.id);

        let data = serde_json::json!({
            "workflow": workflow,
            "metadata": metadata,
        });

        self.put_json(&key, &data).await?;

        // Also store as a version
        self.store_workflow_version(workflow, metadata).await?;

        Ok(metadata.id)
    }

    async fn get_workflow(&self, id: Uuid) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let key = self.workflow_key(id);

        match self.get_json::<serde_json::Value>(&key).await? {
            Some(data) => {
                let workflow: DSLWorkflow = serde_json::from_value(data["workflow"].clone())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                let metadata: WorkflowMetadata =
                    serde_json::from_value(data["metadata"].clone())
                        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some((workflow, metadata)))
            }
            None => Ok(None),
        }
    }

    async fn update_workflow(
        &self,
        id: Uuid,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        // Check if workflow exists
        if self.get_workflow(id).await?.is_none() {
            return Err(StorageError::NotFound(format!("Workflow {} not found", id)));
        }

        let key = self.workflow_key(id);

        let data = serde_json::json!({
            "workflow": workflow,
            "metadata": metadata,
        });

        self.put_json(&key, &data).await?;

        // Store new version
        self.store_workflow_version(workflow, metadata).await?;

        Ok(())
    }

    async fn delete_workflow(&self, id: Uuid) -> Result<()> {
        let key = self.workflow_key(id);

        // Check if exists
        if self.get_workflow(id).await?.is_none() {
            return Err(StorageError::NotFound(format!("Workflow {} not found", id)));
        }

        self.delete_object(&key).await?;

        // Also delete all versions
        let versions_prefix = format!("{}/workflows/{}/versions/", self.prefix, id);
        let version_keys = self.list_objects(&versions_prefix).await?;

        for version_key in version_keys {
            let _ = self.delete_object(&version_key).await; // Ignore errors
        }

        Ok(())
    }

    async fn list_workflows(
        &self,
        filter: &WorkflowFilter,
    ) -> Result<Vec<(DSLWorkflow, WorkflowMetadata)>> {
        let prefix = format!("{}/workflows/", self.prefix);
        let keys = self.list_objects(&prefix).await?;

        let mut results = Vec::new();

        for key in keys {
            // Skip version files
            if key.contains("/versions/") {
                continue;
            }

            if let Some(data) = self.get_json::<serde_json::Value>(&key).await? {
                let workflow: DSLWorkflow = serde_json::from_value(data["workflow"].clone())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                let metadata: WorkflowMetadata =
                    serde_json::from_value(data["metadata"].clone())
                        .map_err(|e| StorageError::SerializationError(e.to_string()))?;

                // Apply filters
                if let Some(ref name) = filter.name {
                    if !metadata.name.contains(name) {
                        continue;
                    }
                }

                if let Some(is_active) = filter.is_active {
                    if metadata.is_active != is_active {
                        continue;
                    }
                }

                if let Some(ref created_by) = filter.created_by {
                    if metadata.created_by.as_ref() != Some(created_by) {
                        continue;
                    }
                }

                if !filter.tags.is_empty() {
                    let has_tag = filter.tags.iter().any(|t| metadata.tags.contains(t));
                    if !has_tag {
                        continue;
                    }
                }

                if let Some(created_after) = filter.created_after {
                    if metadata.created_at < created_after {
                        continue;
                    }
                }

                if let Some(created_before) = filter.created_before {
                    if metadata.created_at > created_before {
                        continue;
                    }
                }

                results.push((workflow, metadata));
            }
        }

        // Sort by created_at (newest first)
        results.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);

        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    async fn get_workflow_version(
        &self,
        id: Uuid,
        version: &str,
    ) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>> {
        let key = self.workflow_version_key(id, version);

        match self.get_json::<serde_json::Value>(&key).await? {
            Some(data) => {
                let workflow: DSLWorkflow = serde_json::from_value(data["workflow"].clone())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                let metadata: WorkflowMetadata =
                    serde_json::from_value(data["metadata"].clone())
                        .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                Ok(Some((workflow, metadata)))
            }
            None => Ok(None),
        }
    }

    async fn store_workflow_version(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()> {
        let key = self.workflow_version_key(metadata.id, &metadata.version);

        let data = serde_json::json!({
            "workflow": workflow,
            "metadata": metadata,
        });

        self.put_json(&key, &data).await
    }
}

// ============================================================================
// ExecutionStorage Implementation
// ============================================================================

#[cfg(feature = "server")]
#[async_trait]
impl ExecutionStorage for S3Storage {
    async fn store_execution(&self, execution: &Execution) -> Result<Uuid> {
        let key = self.execution_key(execution.id);
        self.put_json(&key, execution).await?;
        Ok(execution.id)
    }

    async fn get_execution(&self, id: Uuid) -> Result<Option<Execution>> {
        let key = self.execution_key(id);
        self.get_json(&key).await
    }

    async fn update_execution(&self, id: Uuid, execution: &Execution) -> Result<()> {
        // Check if exists
        if self.get_execution(id).await?.is_none() {
            return Err(StorageError::NotFound(format!(
                "Execution {} not found",
                id
            )));
        }

        let key = self.execution_key(id);
        self.put_json(&key, execution).await
    }

    async fn list_executions(&self, filter: &ExecutionFilter) -> Result<Vec<Execution>> {
        let prefix = format!("{}/executions/", self.prefix);
        let keys = self.list_objects(&prefix).await?;

        let mut results = Vec::new();

        for key in keys {
            // Skip log files
            if key.contains("/logs/") {
                continue;
            }

            if let Some(execution) = self.get_json::<Execution>(&key).await? {
                // Apply filters
                if let Some(workflow_id) = filter.workflow_id {
                    if execution.workflow_id != workflow_id {
                        continue;
                    }
                }

                if let Some(ref status) = filter.status {
                    if &execution.status != status {
                        continue;
                    }
                }

                if let Some(ref triggered_by) = filter.triggered_by {
                    if execution.triggered_by.as_ref() != Some(triggered_by) {
                        continue;
                    }
                }

                if let Some(started_after) = filter.started_after {
                    if execution.started_at.is_none_or(|t| t < started_after) {
                        continue;
                    }
                }

                if let Some(started_before) = filter.started_before {
                    if execution.started_at.is_none_or(|t| t > started_before) {
                        continue;
                    }
                }

                if let Some(completed_after) = filter.completed_after {
                    if execution.completed_at.is_none_or(|t| t < completed_after) {
                        continue;
                    }
                }

                if let Some(completed_before) = filter.completed_before {
                    if execution.completed_at.is_none_or(|t| t > completed_before) {
                        continue;
                    }
                }

                results.push(execution);
            }
        }

        // Sort by created_at (newest first)
        results.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        // Apply pagination
        let offset = filter.offset.unwrap_or(0);
        let limit = filter.limit.unwrap_or(100);

        Ok(results.into_iter().skip(offset).take(limit).collect())
    }

    async fn store_execution_log(&self, log: &ExecutionLog) -> Result<()> {
        let key = self.execution_log_key(log.execution_id, log.timestamp.timestamp());
        self.put_json(&key, log).await
    }

    async fn get_execution_logs(
        &self,
        execution_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionLog>> {
        let prefix = format!("{}/executions/{}/logs/", self.prefix, execution_id);
        let keys = self.list_objects(&prefix).await?;

        let mut logs = Vec::new();

        for key in keys {
            if let Some(log) = self.get_json::<ExecutionLog>(&key).await? {
                logs.push(log);
            }
        }

        // Sort by timestamp
        logs.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Apply limit
        if let Some(limit) = limit {
            logs.truncate(limit);
        }

        Ok(logs)
    }

    async fn delete_execution(&self, id: Uuid) -> Result<()> {
        // Check if exists
        if self.get_execution(id).await?.is_none() {
            return Err(StorageError::NotFound(format!(
                "Execution {} not found",
                id
            )));
        }

        let key = self.execution_key(id);
        self.delete_object(&key).await?;

        // Delete logs
        let logs_prefix = format!("{}/executions/{}/logs/", self.prefix, id);
        let log_keys = self.list_objects(&logs_prefix).await?;

        for log_key in log_keys {
            let _ = self.delete_object(&log_key).await; // Ignore errors
        }

        Ok(())
    }
}

// ============================================================================
// CheckpointStorage Implementation
// ============================================================================

#[cfg(feature = "server")]
#[async_trait]
impl CheckpointStorage for S3Storage {
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Uuid> {
        let key = self.checkpoint_key(checkpoint.execution_id, &checkpoint.checkpoint_name);
        self.put_json(&key, checkpoint).await?;
        Ok(checkpoint.id)
    }

    async fn get_checkpoint(&self, execution_id: Uuid, name: &str) -> Result<Option<Checkpoint>> {
        let key = self.checkpoint_key(execution_id, name);
        self.get_json(&key).await
    }

    async fn list_checkpoints(&self, execution_id: Uuid) -> Result<Vec<Checkpoint>> {
        let prefix = format!("{}/checkpoints/{}/", self.prefix, execution_id);
        let keys = self.list_objects(&prefix).await?;

        let mut checkpoints = Vec::new();

        for key in keys {
            if let Some(checkpoint) = self.get_json::<Checkpoint>(&key).await? {
                checkpoints.push(checkpoint);
            }
        }

        // Sort by created_at
        checkpoints.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        Ok(checkpoints)
    }

    async fn delete_checkpoint(&self, id: Uuid) -> Result<()> {
        // Need to find the checkpoint by ID (less efficient, but maintains interface)
        let prefix = format!("{}/checkpoints/", self.prefix);
        let keys = self.list_objects(&prefix).await?;

        for key in keys {
            if let Some(checkpoint) = self.get_json::<Checkpoint>(&key).await? {
                if checkpoint.id == id {
                    self.delete_object(&key).await?;
                    return Ok(());
                }
            }
        }

        Err(StorageError::NotFound(format!(
            "Checkpoint {} not found",
            id
        )))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ScheduleStorage for S3Storage {
    async fn store_schedule(&self, _schedule: &Schedule) -> Result<Uuid> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_schedule(&self, _id: Uuid) -> Result<Option<Schedule>> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn update_schedule(&self, _id: Uuid, _schedule: &Schedule) -> Result<()> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn delete_schedule(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn list_schedules(&self, _filter: &ScheduleFilter) -> Result<Vec<Schedule>> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_due_schedules(&self, _before: DateTime<Utc>) -> Result<Vec<Schedule>> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn store_schedule_run(&self, _run: &ScheduleRun) -> Result<Uuid> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_schedule_runs(
        &self,
        _schedule_id: Uuid,
        _limit: Option<usize>,
    ) -> Result<Vec<ScheduleRun>> {
        Err(StorageError::S3Error(
            "Schedule storage not implemented for S3 backend".to_string(),
        ))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl OrganizationStorage for S3Storage {
    async fn store_organization(&self, _organization: &Organization) -> Result<Uuid> {
        Err(StorageError::S3Error(
            "Organization storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_organization(&self, _id: Uuid) -> Result<Option<Organization>> {
        Err(StorageError::S3Error(
            "Organization storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_organization_by_slug(&self, _slug: &str) -> Result<Option<Organization>> {
        Err(StorageError::S3Error(
            "Organization storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn update_organization(&self, _id: Uuid, _organization: &Organization) -> Result<()> {
        Err(StorageError::S3Error(
            "Organization storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn delete_organization(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::S3Error(
            "Organization storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn list_organizations(&self, _filter: &OrganizationFilter) -> Result<Vec<Organization>> {
        Err(StorageError::S3Error(
            "Organization storage not implemented for S3 backend".to_string(),
        ))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl TeamStorage for S3Storage {
    async fn store_team(&self, _team: &Team) -> Result<Uuid> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_team(&self, _id: Uuid) -> Result<Option<Team>> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn update_team(&self, _id: Uuid, _team: &Team) -> Result<()> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn delete_team(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn list_teams(&self, _filter: &TeamFilter) -> Result<Vec<Team>> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn add_team_member(&self, _member: &TeamMember) -> Result<Uuid> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn remove_team_member(&self, _team_id: Uuid, _user_id: Uuid) -> Result<()> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_team_members(&self, _team_id: Uuid) -> Result<Vec<TeamMember>> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_user_teams(&self, _user_id: Uuid) -> Result<Vec<Team>> {
        Err(StorageError::S3Error(
            "Team storage not implemented for S3 backend".to_string(),
        ))
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl ApiKeyStorage for S3Storage {
    async fn store_api_key(&self, _api_key: &ApiKey) -> Result<Uuid> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_api_key(&self, _id: Uuid) -> Result<Option<ApiKey>> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn get_api_key_by_hash(&self, _key_hash: &str) -> Result<Option<ApiKey>> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn update_api_key(&self, _id: Uuid, _api_key: &ApiKey) -> Result<()> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn revoke_api_key(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn delete_api_key(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn list_api_keys(&self, _filter: &ApiKeyFilter) -> Result<Vec<ApiKey>> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }

    async fn update_last_used(&self, _id: Uuid) -> Result<()> {
        Err(StorageError::S3Error(
            "API key storage not implemented for S3 backend".to_string(),
        ))
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    

    #[tokio::test]
    #[ignore] // Requires S3 credentials and bucket
    async fn test_s3_storage() {
        // This test requires AWS credentials and a test bucket
        // Run with: cargo test --features server -- --ignored
    }
}
