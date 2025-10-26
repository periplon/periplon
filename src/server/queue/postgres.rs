// PostgreSQL queue backend implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use sqlx::{PgPool, Row};
#[cfg(feature = "server")]
use std::time::Duration;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::*;

#[cfg(feature = "server")]
pub struct PostgresQueue {
    pool: PgPool,
    _poll_interval_ms: u64,
    _max_retries: u32,
}

#[cfg(feature = "server")]
impl PostgresQueue {
    pub async fn new(database_url: &str, poll_interval_ms: u64, max_retries: u32) -> Result<Self> {
        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| QueueError::Error(e.to_string()))?;

        Ok(Self {
            pool,
            _poll_interval_ms: poll_interval_ms,
            _max_retries: max_retries,
        })
    }

    pub fn from_pool(pool: PgPool, poll_interval_ms: u64, max_retries: u32) -> Self {
        Self {
            pool,
            _poll_interval_ms: poll_interval_ms,
            _max_retries: max_retries,
        }
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl WorkQueue for PostgresQueue {
    async fn enqueue(&self, job: Job) -> Result<Uuid> {
        let id = job.id;

        sqlx::query(
            r#"
            INSERT INTO execution_queue (
                id, workflow_id, execution_id, priority, payload,
                status, created_at, scheduled_at, attempts, max_retries
            )
            VALUES ($1, $2, $3, $4, $5, 'pending', $6, $7, $8, $9)
            "#,
        )
        .bind(id)
        .bind(job.workflow_id)
        .bind(job.execution_id)
        .bind(job.priority)
        .bind(&job.payload)
        .bind(job.created_at)
        .bind(job.scheduled_at)
        .bind(job.attempts as i32)
        .bind(job.max_retries as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        Ok(id)
    }

    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        // Use SELECT FOR UPDATE SKIP LOCKED for efficient, non-blocking dequeue
        // This is a PostgreSQL-specific optimization that allows multiple workers
        // to dequeue jobs concurrently without blocking each other
        let row = sqlx::query(
            r#"
            UPDATE execution_queue
            SET
                status = 'processing',
                worker_id = $1,
                claimed_at = NOW(),
                updated_at = NOW(),
                attempts = attempts + 1
            WHERE id = (
                SELECT id
                FROM execution_queue
                WHERE status = 'pending'
                AND (scheduled_at IS NULL OR scheduled_at <= NOW())
                ORDER BY priority DESC, created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING
                id, workflow_id, execution_id, priority, payload,
                created_at, scheduled_at, attempts, max_retries
            "#,
        )
        .bind(worker_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Job {
                id: row.get("id"),
                workflow_id: row.get("workflow_id"),
                execution_id: row.get("execution_id"),
                priority: row.get("priority"),
                payload: row.get("payload"),
                created_at: row.get("created_at"),
                attempts: row.get::<i32, _>("attempts") as u32,
                max_retries: row.get::<i32, _>("max_retries") as u32,
                scheduled_at: row.get("scheduled_at"),
            }))
        } else {
            Ok(None)
        }
    }

    async fn complete(&self, job_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE execution_queue
            SET
                status = 'completed',
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(QueueError::NotFound(job_id));
        }

        Ok(())
    }

    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE execution_queue
            SET
                status = 'failed',
                error = $2,
                completed_at = NOW(),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .bind(error)
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(QueueError::NotFound(job_id));
        }

        Ok(())
    }

    async fn requeue(&self, job_id: Uuid, delay: Option<Duration>) -> Result<()> {
        let scheduled_at = delay.map(|d| Utc::now() + chrono::Duration::from_std(d).unwrap());

        let result = sqlx::query(
            r#"
            UPDATE execution_queue
            SET
                status = 'pending',
                worker_id = NULL,
                claimed_at = NULL,
                scheduled_at = COALESCE($2, scheduled_at),
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .bind(scheduled_at)
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(QueueError::NotFound(job_id));
        }

        Ok(())
    }

    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE execution_queue
            SET updated_at = NOW()
            WHERE id = $1 AND worker_id = $2 AND status = 'processing'
            "#,
        )
        .bind(job_id)
        .bind(worker_id)
        .execute(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(QueueError::Error(format!(
                "Job {} not found or not owned by worker {}",
                job_id, worker_id
            )));
        }

        Ok(())
    }

    async fn release_stale_jobs(&self, timeout: Duration) -> Result<Vec<Uuid>> {
        let timeout_secs = timeout.as_secs() as i64;

        let rows = sqlx::query(
            r#"
            UPDATE execution_queue
            SET
                status = 'pending',
                worker_id = NULL,
                claimed_at = NULL,
                updated_at = NOW()
            WHERE status = 'processing'
            AND updated_at < NOW() - INTERVAL '1 second' * $1
            RETURNING id
            "#,
        )
        .bind(timeout_secs)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        Ok(rows.into_iter().map(|row| row.get("id")).collect())
    }

    async fn stats(&self) -> Result<QueueStats> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending') as pending,
                COUNT(*) FILTER (WHERE status = 'processing') as processing,
                COUNT(*) FILTER (WHERE status = 'completed') as completed,
                COUNT(*) FILTER (WHERE status = 'failed') as failed
            FROM execution_queue
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        Ok(QueueStats {
            pending: row
                .try_get::<Option<i64>, _>("pending")
                .ok()
                .flatten()
                .unwrap_or(0) as usize,
            processing: row
                .try_get::<Option<i64>, _>("processing")
                .ok()
                .flatten()
                .unwrap_or(0) as usize,
            completed: row
                .try_get::<Option<i64>, _>("completed")
                .ok()
                .flatten()
                .unwrap_or(0) as usize,
            failed: row
                .try_get::<Option<i64>, _>("failed")
                .ok()
                .flatten()
                .unwrap_or(0) as usize,
        })
    }

    async fn get_job(&self, job_id: Uuid) -> Result<Option<Job>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, workflow_id, execution_id, priority, payload,
                created_at, scheduled_at, attempts, max_retries
            FROM execution_queue
            WHERE id = $1
            "#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QueueError::Error(e.to_string()))?;

        if let Some(row) = row {
            Ok(Some(Job {
                id: row.get("id"),
                workflow_id: row.get("workflow_id"),
                execution_id: row.get("execution_id"),
                priority: row.get("priority"),
                payload: row.get("payload"),
                created_at: row.get("created_at"),
                attempts: row.get::<i32, _>("attempts") as u32,
                max_retries: row.get::<i32, _>("max_retries") as u32,
                scheduled_at: row.get("scheduled_at"),
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {

    #[tokio::test]
    #[ignore] // Requires PostgreSQL database
    async fn test_postgres_queue() {
        // This test requires a running PostgreSQL instance
        // Run with: cargo test --features server -- --ignored
    }
}
