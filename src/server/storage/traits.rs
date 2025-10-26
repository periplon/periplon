// Storage layer traits - all backends implement these

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::{DateTime, Utc};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use thiserror::Error;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use crate::dsl::schema::DSLWorkflow;

#[cfg(feature = "server")]
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("S3 error: {0}")]
    S3Error(String),
}

#[cfg(feature = "server")]
pub type Result<T> = std::result::Result<T, StorageError>;

// ============================================================================
// Workflow Storage
// ============================================================================

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub tags: Vec<String>,
    pub is_active: bool,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowFilter {
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub created_by: Option<String>,
    pub is_active: Option<bool>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
impl Default for WorkflowFilter {
    fn default() -> Self {
        Self {
            name: None,
            tags: Vec::new(),
            created_by: None,
            is_active: None,
            created_after: None,
            created_before: None,
            limit: Some(100),
            offset: None,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
pub trait WorkflowStorage: Send + Sync {
    /// Store a new workflow
    async fn store_workflow(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<Uuid>;

    /// Get workflow by ID
    async fn get_workflow(&self, id: Uuid) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>>;

    /// Update existing workflow
    async fn update_workflow(
        &self,
        id: Uuid,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()>;

    /// Delete workflow
    async fn delete_workflow(&self, id: Uuid) -> Result<()>;

    /// List workflows with filtering
    async fn list_workflows(
        &self,
        filter: &WorkflowFilter,
    ) -> Result<Vec<(DSLWorkflow, WorkflowMetadata)>>;

    /// Get specific workflow version
    async fn get_workflow_version(
        &self,
        id: Uuid,
        version: &str,
    ) -> Result<Option<(DSLWorkflow, WorkflowMetadata)>>;

    /// Store a new version of a workflow
    async fn store_workflow_version(
        &self,
        workflow: &DSLWorkflow,
        metadata: &WorkflowMetadata,
    ) -> Result<()>;
}

// ============================================================================
// Execution Storage
// ============================================================================

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_version: String,
    pub status: ExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub triggered_by: Option<String>,
    pub trigger_type: String,
    pub input_params: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub retry_count: u32,
    pub parent_execution_id: Option<Uuid>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionLog {
    pub id: Option<i64>,
    pub execution_id: Uuid,
    pub task_execution_id: Option<Uuid>,
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
    pub metadata: Option<serde_json::Value>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionFilter {
    pub workflow_id: Option<Uuid>,
    pub status: Option<ExecutionStatus>,
    pub triggered_by: Option<String>,
    pub started_after: Option<DateTime<Utc>>,
    pub started_before: Option<DateTime<Utc>>,
    pub completed_after: Option<DateTime<Utc>>,
    pub completed_before: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
impl Default for ExecutionFilter {
    fn default() -> Self {
        Self {
            workflow_id: None,
            status: None,
            triggered_by: None,
            started_after: None,
            started_before: None,
            completed_after: None,
            completed_before: None,
            limit: Some(100),
            offset: None,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
pub trait ExecutionStorage: Send + Sync {
    /// Store a new execution
    async fn store_execution(&self, execution: &Execution) -> Result<Uuid>;

    /// Get execution by ID
    async fn get_execution(&self, id: Uuid) -> Result<Option<Execution>>;

    /// Update execution
    async fn update_execution(&self, id: Uuid, execution: &Execution) -> Result<()>;

    /// List executions with filtering
    async fn list_executions(&self, filter: &ExecutionFilter) -> Result<Vec<Execution>>;

    /// Store execution log entry
    async fn store_execution_log(&self, log: &ExecutionLog) -> Result<()>;

    /// Get execution logs
    async fn get_execution_logs(
        &self,
        execution_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<ExecutionLog>>;

    /// Delete execution
    async fn delete_execution(&self, id: Uuid) -> Result<()>;
}

// ============================================================================
// Checkpoint Storage
// ============================================================================

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub execution_id: Uuid,
    pub checkpoint_name: String,
    pub state: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "server")]
#[async_trait]
pub trait CheckpointStorage: Send + Sync {
    /// Store a checkpoint
    async fn store_checkpoint(&self, checkpoint: &Checkpoint) -> Result<Uuid>;

    /// Get checkpoint by execution ID and name
    async fn get_checkpoint(&self, execution_id: Uuid, name: &str) -> Result<Option<Checkpoint>>;

    /// List all checkpoints for an execution
    async fn list_checkpoints(&self, execution_id: Uuid) -> Result<Vec<Checkpoint>>;

    /// Delete checkpoint
    async fn delete_checkpoint(&self, id: Uuid) -> Result<()>;
}

// ============================================================================
// Schedule Storage
// ============================================================================

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub cron_expression: String,
    pub timezone: String,
    pub is_active: bool,
    pub input_params: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScheduleRunStatus {
    Scheduled,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleRun {
    pub id: Uuid,
    pub schedule_id: Uuid,
    pub execution_id: Option<Uuid>,
    pub scheduled_for: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub status: ScheduleRunStatus,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleFilter {
    pub workflow_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub created_by: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
impl Default for ScheduleFilter {
    fn default() -> Self {
        Self {
            workflow_id: None,
            is_active: None,
            created_by: None,
            limit: Some(100),
            offset: None,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
pub trait ScheduleStorage: Send + Sync {
    /// Store a new schedule
    async fn store_schedule(&self, schedule: &Schedule) -> Result<Uuid>;

    /// Get schedule by ID
    async fn get_schedule(&self, id: Uuid) -> Result<Option<Schedule>>;

    /// Update schedule
    async fn update_schedule(&self, id: Uuid, schedule: &Schedule) -> Result<()>;

    /// Delete schedule
    async fn delete_schedule(&self, id: Uuid) -> Result<()>;

    /// List schedules with filtering
    async fn list_schedules(&self, filter: &ScheduleFilter) -> Result<Vec<Schedule>>;

    /// Get schedules due for execution
    async fn get_due_schedules(&self, before: DateTime<Utc>) -> Result<Vec<Schedule>>;

    /// Store schedule run record
    async fn store_schedule_run(&self, run: &ScheduleRun) -> Result<Uuid>;

    /// Get schedule run history
    async fn get_schedule_runs(
        &self,
        schedule_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<ScheduleRun>>;
}

// ============================================================================
// Organization Storage
// ============================================================================

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub logo_url: Option<String>,
    pub plan: String,
    pub settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationFilter {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub is_active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
impl Default for OrganizationFilter {
    fn default() -> Self {
        Self {
            name: None,
            slug: None,
            is_active: None,
            limit: Some(100),
            offset: None,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
pub trait OrganizationStorage: Send + Sync {
    /// Store a new organization
    async fn store_organization(&self, organization: &Organization) -> Result<Uuid>;

    /// Get organization by ID
    async fn get_organization(&self, id: Uuid) -> Result<Option<Organization>>;

    /// Get organization by slug
    async fn get_organization_by_slug(&self, slug: &str) -> Result<Option<Organization>>;

    /// Update organization
    async fn update_organization(&self, id: Uuid, organization: &Organization) -> Result<()>;

    /// Delete organization
    async fn delete_organization(&self, id: Uuid) -> Result<()>;

    /// List organizations with filtering
    async fn list_organizations(&self, filter: &OrganizationFilter) -> Result<Vec<Organization>>;
}

// ============================================================================
// Team Storage
// ============================================================================

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub added_at: DateTime<Utc>,
    pub added_by: Option<Uuid>,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamFilter {
    pub organization_id: Option<Uuid>,
    pub name: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
impl Default for TeamFilter {
    fn default() -> Self {
        Self {
            organization_id: None,
            name: None,
            limit: Some(100),
            offset: None,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
pub trait TeamStorage: Send + Sync {
    /// Store a new team
    async fn store_team(&self, team: &Team) -> Result<Uuid>;

    /// Get team by ID
    async fn get_team(&self, id: Uuid) -> Result<Option<Team>>;

    /// Update team
    async fn update_team(&self, id: Uuid, team: &Team) -> Result<()>;

    /// Delete team
    async fn delete_team(&self, id: Uuid) -> Result<()>;

    /// List teams with filtering
    async fn list_teams(&self, filter: &TeamFilter) -> Result<Vec<Team>>;

    /// Add user to team
    async fn add_team_member(&self, member: &TeamMember) -> Result<Uuid>;

    /// Remove user from team
    async fn remove_team_member(&self, team_id: Uuid, user_id: Uuid) -> Result<()>;

    /// Get team members
    async fn get_team_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>>;

    /// Get user's teams
    async fn get_user_teams(&self, user_id: Uuid) -> Result<Vec<Team>>;
}

// ============================================================================
// API Key Storage
// ============================================================================

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyFilter {
    pub user_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[cfg(feature = "server")]
impl Default for ApiKeyFilter {
    fn default() -> Self {
        Self {
            user_id: None,
            is_active: None,
            limit: Some(100),
            offset: None,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
pub trait ApiKeyStorage: Send + Sync {
    /// Store a new API key
    async fn store_api_key(&self, api_key: &ApiKey) -> Result<Uuid>;

    /// Get API key by ID
    async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>>;

    /// Get API key by key hash
    async fn get_api_key_by_hash(&self, key_hash: &str) -> Result<Option<ApiKey>>;

    /// Update API key
    async fn update_api_key(&self, id: Uuid, api_key: &ApiKey) -> Result<()>;

    /// Revoke API key (set is_active = false)
    async fn revoke_api_key(&self, id: Uuid) -> Result<()>;

    /// Delete API key
    async fn delete_api_key(&self, id: Uuid) -> Result<()>;

    /// List API keys with filtering
    async fn list_api_keys(&self, filter: &ApiKeyFilter) -> Result<Vec<ApiKey>>;

    /// Update last used timestamp
    async fn update_last_used(&self, id: Uuid) -> Result<()>;
}

// ============================================================================
// Combined Storage Trait
// ============================================================================

#[cfg(feature = "server")]
pub trait Storage:
    WorkflowStorage
    + ExecutionStorage
    + CheckpointStorage
    + ScheduleStorage
    + OrganizationStorage
    + TeamStorage
    + ApiKeyStorage
    + Send
    + Sync
{
}

// Blanket implementation for any type that implements all traits
#[cfg(feature = "server")]
impl<T> Storage for T where
    T: WorkflowStorage
        + ExecutionStorage
        + CheckpointStorage
        + ScheduleStorage
        + OrganizationStorage
        + TeamStorage
        + ApiKeyStorage
        + Send
        + Sync
{
}
