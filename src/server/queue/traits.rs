// Queue system traits

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::{DateTime, Utc};
#[cfg(feature = "server")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use std::time::Duration;
#[cfg(feature = "server")]
use thiserror::Error;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Queue error: {0}")]
    Error(String),

    #[error("Job not found: {0}")]
    NotFound(Uuid),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Lock error: {0}")]
    LockError(String),
}

#[cfg(feature = "server")]
pub type Result<T> = std::result::Result<T, QueueError>;

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub execution_id: Uuid,
    pub priority: i32,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub attempts: u32,
    pub max_retries: u32,
    pub scheduled_at: Option<DateTime<Utc>>,
}

#[cfg(feature = "server")]
impl Job {
    pub fn new(workflow_id: Uuid, execution_id: Uuid, payload: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            workflow_id,
            execution_id,
            priority: 0,
            payload,
            created_at: Utc::now(),
            attempts: 0,
            max_retries: 3,
            scheduled_at: None,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    pub fn with_schedule(mut self, scheduled_at: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(scheduled_at);
        self
    }
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
}

#[cfg(feature = "server")]
#[async_trait]
pub trait WorkQueue: Send + Sync {
    /// Enqueue a new job
    async fn enqueue(&self, job: Job) -> Result<Uuid>;

    /// Dequeue next available job (with locking)
    /// Returns None if no jobs are available
    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>>;

    /// Mark job as completed
    async fn complete(&self, job_id: Uuid) -> Result<()>;

    /// Mark job as failed
    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()>;

    /// Requeue job for retry (with optional delay)
    async fn requeue(&self, job_id: Uuid, delay: Option<Duration>) -> Result<()>;

    /// Send heartbeat to keep job locked
    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()>;

    /// Release stuck jobs (dead worker recovery)
    /// Returns list of released job IDs
    async fn release_stale_jobs(&self, timeout: Duration) -> Result<Vec<Uuid>>;

    /// Get queue statistics
    async fn stats(&self) -> Result<QueueStats>;

    /// Get job by ID
    async fn get_job(&self, job_id: Uuid) -> Result<Option<Job>>;
}
