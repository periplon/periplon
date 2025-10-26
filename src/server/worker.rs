// Background worker for executing workflows

#[cfg(feature = "server")]
use std::sync::Arc;
#[cfg(feature = "server")]
use std::time::Duration;
#[cfg(feature = "server")]
use tokio::time::sleep;
#[cfg(feature = "server")]
use tracing::{error, info, warn};
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::queue::{Job, WorkQueue};
#[cfg(feature = "server")]
use super::storage::{ExecutionStatus, Storage};
#[cfg(feature = "server")]
use crate::dsl::schema::DSLWorkflow;
#[cfg(feature = "server")]
use crate::dsl::DSLExecutor;

#[cfg(feature = "server")]
pub struct Worker {
    worker_id: String,
    queue: Arc<dyn WorkQueue>,
    storage: Arc<dyn Storage>,
    concurrency: usize,
    poll_interval: Duration,
    _heartbeat_interval: Duration,
}

#[cfg(feature = "server")]
impl Worker {
    pub fn new(
        worker_id: String,
        queue: Arc<dyn WorkQueue>,
        storage: Arc<dyn Storage>,
        concurrency: usize,
    ) -> Self {
        Self {
            worker_id,
            queue,
            storage,
            concurrency,
            poll_interval: Duration::from_secs(1),
            _heartbeat_interval: Duration::from_secs(30),
        }
    }

    /// Start the worker (runs indefinitely)
    pub async fn run(&self) {
        info!(
            "Worker {} starting with concurrency {}",
            self.worker_id, self.concurrency
        );

        loop {
            // Try to dequeue a job
            match self.queue.dequeue(&self.worker_id).await {
                Ok(Some(job)) => {
                    info!("Worker {} processing job {}", self.worker_id, job.id);

                    // Process job
                    if let Err(e) = self.process_job(job).await {
                        error!("Worker {} failed to process job: {}", self.worker_id, e);
                    }
                }
                Ok(None) => {
                    // No jobs available, sleep
                    sleep(self.poll_interval).await;
                }
                Err(e) => {
                    error!("Worker {} failed to dequeue: {}", self.worker_id, e);
                    sleep(self.poll_interval).await;
                }
            }
        }
    }

    async fn process_job(&self, job: Job) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let execution_id = job.execution_id;
        let job_id = job.id;

        // Get execution from storage
        let execution = self
            .storage
            .get_execution(execution_id)
            .await?
            .ok_or_else(|| format!("Execution {} not found", execution_id))?;

        // Get workflow from storage
        let (workflow, _metadata) = self
            .storage
            .get_workflow(execution.workflow_id)
            .await?
            .ok_or_else(|| format!("Workflow {} not found", execution.workflow_id))?;

        // Update execution status to running
        let mut updated_execution = execution.clone();
        updated_execution.status = ExecutionStatus::Running;
        updated_execution.started_at = Some(chrono::Utc::now());
        self.storage
            .update_execution(execution_id, &updated_execution)
            .await?;

        // Start heartbeat task
        let queue = Arc::clone(&self.queue);
        let worker_id = self.worker_id.clone();
        let heartbeat_handle = tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(30)).await;
                if let Err(e) = queue.heartbeat(job_id, &worker_id).await {
                    warn!("Heartbeat failed: {}", e);
                }
            }
        });

        // Execute workflow
        let result = self.execute_workflow(workflow, execution_id).await;

        // Cancel heartbeat
        heartbeat_handle.abort();

        // Update execution based on result
        match result {
            Ok(output) => {
                let mut final_execution = updated_execution;
                final_execution.status = ExecutionStatus::Completed;
                final_execution.completed_at = Some(chrono::Utc::now());
                final_execution.result = Some(output);
                self.storage
                    .update_execution(execution_id, &final_execution)
                    .await?;

                // Mark job as complete
                self.queue.complete(job_id).await?;

                info!("Worker {} completed job {}", self.worker_id, job_id);
            }
            Err(e) => {
                let error_msg = e.to_string();

                let mut final_execution = updated_execution;
                final_execution.status = ExecutionStatus::Failed;
                final_execution.completed_at = Some(chrono::Utc::now());
                final_execution.error = Some(error_msg.clone());
                self.storage
                    .update_execution(execution_id, &final_execution)
                    .await?;

                // Check if should retry
                if job.attempts < job.max_retries {
                    warn!(
                        "Worker {} requeueing job {} (attempt {}/{})",
                        self.worker_id, job_id, job.attempts, job.max_retries
                    );
                    self.queue
                        .requeue(job_id, Some(Duration::from_secs(60)))
                        .await?;
                } else {
                    error!(
                        "Worker {} failed job {} after {} attempts",
                        self.worker_id, job_id, job.attempts
                    );
                    self.queue.fail(job_id, &error_msg).await?;
                }
            }
        }

        Ok(())
    }

    async fn execute_workflow(
        &self,
        workflow: DSLWorkflow,
        _execution_id: Uuid,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Create executor
        let mut executor = DSLExecutor::new(workflow)?;

        // Execute workflow
        executor.execute().await?;

        // Get final state
        let state = executor.get_state();

        // Convert to JSON
        let output = if let Some(state) = state {
            serde_json::to_value(state)?
        } else {
            serde_json::json!({
                "status": "completed",
                "message": "Workflow executed successfully"
            })
        };

        Ok(output)
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {

    #[test]
    fn test_worker_creation() {
        // Test is placeholder - actual testing would require mock implementations
    }
}
