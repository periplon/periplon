// Redis-based queue implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use redis::{aio::ConnectionManager, AsyncCommands};
#[cfg(feature = "server")]
use std::time::Duration;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::{Job, QueueError, QueueStats, Result, WorkQueue};

/// Redis-based work queue implementation
///
/// Uses Redis data structures:
/// - Sorted set for pending jobs (sorted by priority and created_at)
/// - Hash for job data
/// - Hash for job locks (worker_id -> timestamp)
/// - Sets for completed/failed job IDs
#[cfg(feature = "server")]
pub struct RedisQueue {
    conn: ConnectionManager,
    queue_name: String,
}

#[cfg(feature = "server")]
impl RedisQueue {
    pub async fn new(redis_url: &str, queue_name: Option<String>) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| QueueError::Error(format!("Failed to connect to Redis: {}", e)))?;

        let conn = ConnectionManager::new(client).await.map_err(|e| {
            QueueError::Error(format!("Failed to create connection manager: {}", e))
        })?;

        Ok(Self {
            conn,
            queue_name: queue_name.unwrap_or_else(|| "dsl:queue".to_string()),
        })
    }

    pub fn from_connection(conn: ConnectionManager, queue_name: Option<String>) -> Self {
        Self {
            conn,
            queue_name: queue_name.unwrap_or_else(|| "dsl:queue".to_string()),
        }
    }

    // Redis key helpers
    fn pending_key(&self) -> String {
        format!("{}:pending", self.queue_name)
    }

    fn processing_key(&self) -> String {
        format!("{}:processing", self.queue_name)
    }

    fn completed_key(&self) -> String {
        format!("{}:completed", self.queue_name)
    }

    fn failed_key(&self) -> String {
        format!("{}:failed", self.queue_name)
    }

    fn job_key(&self, job_id: Uuid) -> String {
        format!("{}:job:{}", self.queue_name, job_id)
    }

    fn lock_key(&self, job_id: Uuid) -> String {
        format!("{}:lock:{}", self.queue_name, job_id)
    }

    fn heartbeat_key(&self, job_id: Uuid) -> String {
        format!("{}:heartbeat:{}", self.queue_name, job_id)
    }

    // Calculate priority score for sorted set
    // Higher priority and older jobs get higher scores (dequeued first)
    fn calculate_score(job: &Job) -> f64 {
        // Base score from priority (higher priority = higher score)
        let priority_score = job.priority as f64 * 1_000_000.0;

        // Add timestamp component (older = higher score)
        // Use negative timestamp so older jobs have higher scores
        let timestamp_score = -(job.created_at.timestamp() as f64);

        priority_score + timestamp_score
    }

    #[allow(dead_code)]
    async fn _save_job(&mut self, job: &Job) -> Result<()> {
        let job_key = self.job_key(job.id);
        let job_json = serde_json::to_string(job)
            .map_err(|e| QueueError::SerializationError(e.to_string()))?;

        self.conn
            .set::<_, _, ()>(&job_key, job_json)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to save job: {}", e)))?;

        Ok(())
    }

    #[allow(dead_code)]
    async fn _load_job(&mut self, job_id: Uuid) -> Result<Option<Job>> {
        let job_key = self.job_key(job_id);

        let job_json: Option<String> = self
            .conn
            .get(&job_key)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to load job: {}", e)))?;

        match job_json {
            Some(json) => {
                let job: Job = serde_json::from_str(&json)
                    .map_err(|e| QueueError::SerializationError(e.to_string()))?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }

    #[allow(dead_code)]
    async fn _acquire_lock(
        &mut self,
        job_id: Uuid,
        worker_id: &str,
        ttl_secs: u64,
    ) -> Result<bool> {
        let lock_key = self.lock_key(job_id);
        let heartbeat_key = self.heartbeat_key(job_id);

        // Try to acquire lock using SET NX (set if not exists)
        let acquired: bool = self
            .conn
            .set_nx(&lock_key, worker_id)
            .await
            .map_err(|e| QueueError::LockError(format!("Failed to acquire lock: {}", e)))?;

        if acquired {
            // Set expiration on lock
            self.conn
                .expire::<_, ()>(&lock_key, ttl_secs as i64)
                .await
                .map_err(|e| {
                    QueueError::LockError(format!("Failed to set lock expiration: {}", e))
                })?;

            // Set initial heartbeat
            let now = Utc::now().timestamp();
            self.conn
                .set_ex::<_, _, ()>(&heartbeat_key, now, ttl_secs)
                .await
                .map_err(|e| QueueError::Error(format!("Failed to set heartbeat: {}", e)))?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[allow(dead_code)]
    async fn _release_lock(&mut self, job_id: Uuid) -> Result<()> {
        let lock_key = self.lock_key(job_id);
        let heartbeat_key = self.heartbeat_key(job_id);

        self.conn
            .del::<_, ()>(&[&lock_key, &heartbeat_key])
            .await
            .map_err(|e| QueueError::LockError(format!("Failed to release lock: {}", e)))?;

        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl WorkQueue for RedisQueue {
    async fn enqueue(&self, job: Job) -> Result<Uuid> {
        let mut conn = self.conn.clone();
        let job_id = job.id;
        let score = Self::calculate_score(&job);

        // Save job data
        let job_key = self.job_key(job_id);
        let job_json = serde_json::to_string(&job)
            .map_err(|e| QueueError::SerializationError(e.to_string()))?;

        conn.set::<_, _, ()>(&job_key, job_json)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to save job: {}", e)))?;

        // Add to pending sorted set
        conn.zadd::<_, _, _, ()>(&self.pending_key(), job_id.to_string(), score)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to enqueue job: {}", e)))?;

        Ok(job_id)
    }

    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        let mut conn = self.conn.clone();
        let pending_key = self.pending_key();
        let processing_key = self.processing_key();

        // Get highest priority job (atomic operation using ZPOPMAX)
        let results: Vec<(String, f64)> = conn
            .zpopmax(&pending_key, 1)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to dequeue job: {}", e)))?;

        let result: Option<(String, f64)> = results.into_iter().next();

        if let Some((job_id_str, _score)) = result {
            let job_id = Uuid::parse_str(&job_id_str)
                .map_err(|e| QueueError::Error(format!("Invalid job ID: {}", e)))?;

            // Load job data
            let job_key = self.job_key(job_id);
            let job_json: Option<String> = conn
                .get(&job_key)
                .await
                .map_err(|e| QueueError::Error(format!("Failed to load job: {}", e)))?;

            if let Some(json) = job_json {
                let mut job: Job = serde_json::from_str(&json)
                    .map_err(|e| QueueError::SerializationError(e.to_string()))?;

                // Try to acquire lock (300 second TTL)
                let lock_key = self.lock_key(job_id);
                let heartbeat_key = self.heartbeat_key(job_id);

                let acquired: bool = conn
                    .set_nx(&lock_key, worker_id)
                    .await
                    .map_err(|e| QueueError::LockError(format!("Failed to acquire lock: {}", e)))?;

                if acquired {
                    // Set expiration on lock
                    conn.expire::<_, ()>(&lock_key, 300).await.map_err(|e| {
                        QueueError::LockError(format!("Failed to set lock expiration: {}", e))
                    })?;

                    // Set initial heartbeat
                    let now = Utc::now().timestamp();
                    conn.set_ex::<_, _, ()>(&heartbeat_key, now, 300)
                        .await
                        .map_err(|e| {
                            QueueError::Error(format!("Failed to set heartbeat: {}", e))
                        })?;

                    // Add to processing set
                    conn.sadd::<_, _, ()>(&processing_key, job_id.to_string())
                        .await
                        .map_err(|e| {
                            QueueError::Error(format!("Failed to mark job as processing: {}", e))
                        })?;

                    // Increment attempts
                    job.attempts += 1;
                    let job_json = serde_json::to_string(&job)
                        .map_err(|e| QueueError::SerializationError(e.to_string()))?;
                    conn.set::<_, _, ()>(&job_key, job_json)
                        .await
                        .map_err(|e| QueueError::Error(format!("Failed to save job: {}", e)))?;

                    Ok(Some(job))
                } else {
                    // Failed to acquire lock, re-enqueue
                    let score = Self::calculate_score(&job);
                    conn.zadd::<_, _, _, ()>(&pending_key, job_id.to_string(), score)
                        .await
                        .map_err(|e| {
                            QueueError::Error(format!("Failed to re-enqueue job: {}", e))
                        })?;

                    Ok(None)
                }
            } else {
                // Job data not found, skip
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn complete(&self, job_id: Uuid) -> Result<()> {
        let mut conn = self.conn.clone();
        let processing_key = self.processing_key();
        let completed_key = self.completed_key();

        // Remove from processing
        conn.srem::<_, _, ()>(&processing_key, job_id.to_string())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to remove from processing: {}", e)))?;

        // Add to completed
        conn.sadd::<_, _, ()>(&completed_key, job_id.to_string())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to mark as completed: {}", e)))?;

        // Release lock
        let lock_key = self.lock_key(job_id);
        let heartbeat_key = self.heartbeat_key(job_id);
        conn.del::<_, ()>(&[&lock_key, &heartbeat_key])
            .await
            .map_err(|e| QueueError::LockError(format!("Failed to release lock: {}", e)))?;

        Ok(())
    }

    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        let processing_key = self.processing_key();
        let failed_key = self.failed_key();

        // Remove from processing
        conn.srem::<_, _, ()>(&processing_key, job_id.to_string())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to remove from processing: {}", e)))?;

        // Add to failed
        conn.sadd::<_, _, ()>(&failed_key, job_id.to_string())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to mark as failed: {}", e)))?;

        // Store error message
        let error_key = format!("{}:error", self.job_key(job_id));
        conn.set_ex::<_, _, ()>(&error_key, error, 86400) // Keep for 24 hours
            .await
            .map_err(|e| QueueError::Error(format!("Failed to store error: {}", e)))?;

        // Release lock
        let lock_key = self.lock_key(job_id);
        let heartbeat_key = self.heartbeat_key(job_id);
        conn.del::<_, ()>(&[&lock_key, &heartbeat_key])
            .await
            .map_err(|e| QueueError::LockError(format!("Failed to release lock: {}", e)))?;

        Ok(())
    }

    async fn requeue(&self, job_id: Uuid, delay: Option<Duration>) -> Result<()> {
        let mut conn = self.conn.clone();
        let processing_key = self.processing_key();
        let pending_key = self.pending_key();

        // Load job
        let job_key = self.job_key(job_id);
        let job_json: Option<String> = conn
            .get(&job_key)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to load job: {}", e)))?;

        if let Some(json) = job_json {
            let mut job: Job = serde_json::from_str(&json)
                .map_err(|e| QueueError::SerializationError(e.to_string()))?;

            // Check if job has retries left
            if job.attempts >= job.max_retries {
                return self.fail(job_id, "Max retries exceeded").await;
            }

            // Update scheduled time if delay is specified
            if let Some(delay) = delay {
                job.scheduled_at = Some(Utc::now() + chrono::Duration::from_std(delay).unwrap());
            }

            // Save updated job
            let job_json = serde_json::to_string(&job)
                .map_err(|e| QueueError::SerializationError(e.to_string()))?;
            conn.set::<_, _, ()>(&job_key, job_json)
                .await
                .map_err(|e| QueueError::Error(format!("Failed to save job: {}", e)))?;

            // Remove from processing
            conn.srem::<_, _, ()>(&processing_key, job_id.to_string())
                .await
                .map_err(|e| {
                    QueueError::Error(format!("Failed to remove from processing: {}", e))
                })?;

            // Add back to pending
            let score = Self::calculate_score(&job);
            conn.zadd::<_, _, _, ()>(&pending_key, job_id.to_string(), score)
                .await
                .map_err(|e| QueueError::Error(format!("Failed to requeue job: {}", e)))?;

            // Release lock
            let lock_key = self.lock_key(job_id);
            let heartbeat_key = self.heartbeat_key(job_id);
            conn.del::<_, ()>(&[&lock_key, &heartbeat_key])
                .await
                .map_err(|e| QueueError::LockError(format!("Failed to release lock: {}", e)))?;

            Ok(())
        } else {
            Err(QueueError::NotFound(job_id))
        }
    }

    async fn heartbeat(&self, job_id: Uuid, _worker_id: &str) -> Result<()> {
        let mut conn = self.conn.clone();
        let heartbeat_key = self.heartbeat_key(job_id);

        // Update heartbeat timestamp
        let now = Utc::now().timestamp();
        conn.set_ex::<_, _, ()>(&heartbeat_key, now, 300) // 5 minute TTL
            .await
            .map_err(|e| QueueError::Error(format!("Failed to update heartbeat: {}", e)))?;

        Ok(())
    }

    async fn release_stale_jobs(&self, timeout: Duration) -> Result<Vec<Uuid>> {
        let mut conn = self.conn.clone();
        let processing_key = self.processing_key();
        let pending_key = self.pending_key();

        // Get all processing job IDs
        let job_ids: Vec<String> = conn
            .smembers(&processing_key)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to get processing jobs: {}", e)))?;

        let mut released = Vec::new();
        let timeout_secs = timeout.as_secs() as i64;
        let now = Utc::now().timestamp();

        for job_id_str in job_ids {
            let job_id = match Uuid::parse_str(&job_id_str) {
                Ok(id) => id,
                Err(_) => continue,
            };

            // Check heartbeat
            let heartbeat_key = self.heartbeat_key(job_id);
            let last_heartbeat: Option<i64> = conn
                .get(&heartbeat_key)
                .await
                .map_err(|e| QueueError::Error(format!("Failed to get heartbeat: {}", e)))?;

            let is_stale = match last_heartbeat {
                Some(ts) => (now - ts) > timeout_secs,
                None => true, // No heartbeat found
            };

            if is_stale {
                // Load job data
                let job_key = self.job_key(job_id);
                let job_json: Option<String> = conn
                    .get(&job_key)
                    .await
                    .map_err(|e| QueueError::Error(format!("Failed to load job: {}", e)))?;

                if let Some(json) = job_json {
                    let job: Job = serde_json::from_str(&json)
                        .map_err(|e| QueueError::SerializationError(e.to_string()))?;

                    // Release lock
                    let lock_key = self.lock_key(job_id);
                    let heartbeat_key = self.heartbeat_key(job_id);
                    conn.del::<_, ()>(&[&lock_key, &heartbeat_key])
                        .await
                        .map_err(|e| {
                            QueueError::LockError(format!("Failed to release lock: {}", e))
                        })?;

                    // Remove from processing
                    conn.srem::<_, _, ()>(&processing_key, job_id.to_string())
                        .await
                        .map_err(|e| {
                            QueueError::Error(format!("Failed to remove from processing: {}", e))
                        })?;

                    // Add back to pending
                    let score = Self::calculate_score(&job);
                    conn.zadd::<_, _, _, ()>(&pending_key, job_id.to_string(), score)
                        .await
                        .map_err(|e| {
                            QueueError::Error(format!("Failed to re-enqueue stale job: {}", e))
                        })?;

                    released.push(job_id);
                }
            }
        }

        Ok(released)
    }

    async fn stats(&self) -> Result<QueueStats> {
        let mut conn = self.conn.clone();

        let pending: usize = conn
            .zcard(self.pending_key())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to get pending count: {}", e)))?;

        let processing: usize = conn
            .scard(self.processing_key())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to get processing count: {}", e)))?;

        let completed: usize = conn
            .scard(self.completed_key())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to get completed count: {}", e)))?;

        let failed: usize = conn
            .scard(self.failed_key())
            .await
            .map_err(|e| QueueError::Error(format!("Failed to get failed count: {}", e)))?;

        Ok(QueueStats {
            pending,
            processing,
            completed,
            failed,
        })
    }

    async fn get_job(&self, job_id: Uuid) -> Result<Option<Job>> {
        let mut conn = self.conn.clone();
        let job_key = self.job_key(job_id);

        let job_json: Option<String> = conn
            .get(&job_key)
            .await
            .map_err(|e| QueueError::Error(format!("Failed to load job: {}", e)))?;

        match job_json {
            Some(json) => {
                let job: Job = serde_json::from_str(&json)
                    .map_err(|e| QueueError::SerializationError(e.to_string()))?;
                Ok(Some(job))
            }
            None => Ok(None),
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Redis instance
    async fn test_redis_queue() {
        // This test requires a running Redis instance
        // Run with: cargo test --features server -- --ignored
        let queue = RedisQueue::new("redis://localhost", None).await.unwrap();

        let workflow_id = Uuid::new_v4();
        let execution_id = Uuid::new_v4();
        let job = Job::new(
            workflow_id,
            execution_id,
            serde_json::json!({"test": "data"}),
        );

        // Enqueue
        let job_id = queue.enqueue(job.clone()).await.unwrap();
        assert_eq!(job_id, job.id);

        // Dequeue
        let dequeued = queue.dequeue("worker-1").await.unwrap();
        assert!(dequeued.is_some());
        let dequeued_job = dequeued.unwrap();
        assert_eq!(dequeued_job.id, job.id);

        // Complete
        queue.complete(job_id).await.unwrap();

        // Stats
        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.completed, 1);
    }
}
