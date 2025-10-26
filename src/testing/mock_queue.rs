//! Mock Work Queue for Testing
//!
//! Provides a configurable in-memory work queue for testing worker processing,
//! job scheduling, retry logic, and queue operations without external dependencies.

#[cfg(feature = "server")]
use crate::server::queue::{Job, QueueError, QueueStats, Result, WorkQueue};
#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use std::collections::{HashMap, VecDeque};
#[cfg(feature = "server")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "server")]
use std::time::Duration;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
#[derive(Clone)]
pub struct MockQueue {
    state: Arc<Mutex<QueueState>>,
}

#[cfg(feature = "server")]
struct QueueState {
    pending: VecDeque<Job>,
    processing: HashMap<Uuid, (Job, String)>, // job_id -> (job, worker_id)
    completed: Vec<Job>,
    failed: Vec<(Job, String)>, // (job, error_message)
    enqueue_count: usize,
    dequeue_count: usize,
    complete_count: usize,
    fail_count: usize,
    requeue_count: usize,
    heartbeat_count: usize,
    should_fail_dequeue: bool,
    should_fail_enqueue: bool,
}

#[cfg(feature = "server")]
impl MockQueue {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(QueueState {
                pending: VecDeque::new(),
                processing: HashMap::new(),
                completed: Vec::new(),
                failed: Vec::new(),
                enqueue_count: 0,
                dequeue_count: 0,
                complete_count: 0,
                fail_count: 0,
                requeue_count: 0,
                heartbeat_count: 0,
                should_fail_dequeue: false,
                should_fail_enqueue: false,
            })),
        }
    }

    /// Configure the queue to fail dequeue operations
    pub fn fail_dequeue(&self) {
        let mut state = self.state.lock().unwrap();
        state.should_fail_dequeue = true;
    }

    /// Configure the queue to fail enqueue operations
    pub fn fail_enqueue(&self) {
        let mut state = self.state.lock().unwrap();
        state.should_fail_enqueue = true;
    }

    /// Get number of pending jobs
    pub fn pending_count(&self) -> usize {
        self.state.lock().unwrap().pending.len()
    }

    /// Get number of processing jobs
    pub fn processing_count(&self) -> usize {
        self.state.lock().unwrap().processing.len()
    }

    /// Get number of completed jobs
    pub fn completed_count(&self) -> usize {
        self.state.lock().unwrap().completed.len()
    }

    /// Get number of failed jobs
    pub fn failed_count(&self) -> usize {
        self.state.lock().unwrap().failed.len()
    }

    /// Get total number of enqueue operations
    pub fn enqueue_count(&self) -> usize {
        self.state.lock().unwrap().enqueue_count
    }

    /// Get total number of dequeue operations
    pub fn dequeue_count(&self) -> usize {
        self.state.lock().unwrap().dequeue_count
    }

    /// Get total number of complete operations
    pub fn complete_count(&self) -> usize {
        self.state.lock().unwrap().complete_count
    }

    /// Get total number of fail operations
    pub fn fail_count(&self) -> usize {
        self.state.lock().unwrap().fail_count
    }

    /// Get total number of requeue operations
    pub fn requeue_count(&self) -> usize {
        self.state.lock().unwrap().requeue_count
    }

    /// Get total number of heartbeat operations
    pub fn heartbeat_count(&self) -> usize {
        self.state.lock().unwrap().heartbeat_count
    }

    /// Get all pending jobs
    pub fn get_pending(&self) -> Vec<Job> {
        self.state.lock().unwrap().pending.iter().cloned().collect()
    }

    /// Get all completed jobs
    pub fn get_completed(&self) -> Vec<Job> {
        self.state.lock().unwrap().completed.clone()
    }

    /// Get all failed jobs with error messages
    pub fn get_failed(&self) -> Vec<(Job, String)> {
        self.state.lock().unwrap().failed.clone()
    }

    /// Clear all state
    pub fn clear(&self) {
        let mut state = self.state.lock().unwrap();
        state.pending.clear();
        state.processing.clear();
        state.completed.clear();
        state.failed.clear();
        state.enqueue_count = 0;
        state.dequeue_count = 0;
        state.complete_count = 0;
        state.fail_count = 0;
        state.requeue_count = 0;
        state.heartbeat_count = 0;
        state.should_fail_dequeue = false;
        state.should_fail_enqueue = false;
    }
}

#[cfg(feature = "server")]
impl Default for MockQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl WorkQueue for MockQueue {
    async fn enqueue(&self, job: Job) -> Result<Uuid> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_enqueue {
            return Err(QueueError::Error("Enqueue failure".to_string()));
        }

        state.enqueue_count += 1;
        let job_id = job.id;
        state.pending.push_back(job);

        Ok(job_id)
    }

    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        let mut state = self.state.lock().unwrap();

        if state.should_fail_dequeue {
            return Err(QueueError::Error("Dequeue failure".to_string()));
        }

        state.dequeue_count += 1;

        if let Some(mut job) = state.pending.pop_front() {
            job.attempts += 1;
            state.processing.insert(job.id, (job.clone(), worker_id.to_string()));
            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    async fn complete(&self, job_id: Uuid) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.complete_count += 1;

        if let Some((job, _)) = state.processing.remove(&job_id) {
            state.completed.push(job);
            Ok(())
        } else {
            Err(QueueError::NotFound(job_id))
        }
    }

    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.fail_count += 1;

        if let Some((job, _)) = state.processing.remove(&job_id) {
            state.failed.push((job, error.to_string()));
            Ok(())
        } else {
            Err(QueueError::NotFound(job_id))
        }
    }

    async fn requeue(&self, job_id: Uuid, _delay: Option<Duration>) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.requeue_count += 1;

        if let Some((job, _)) = state.processing.remove(&job_id) {
            state.pending.push_back(job);
            Ok(())
        } else {
            Err(QueueError::NotFound(job_id))
        }
    }

    async fn heartbeat(&self, job_id: Uuid, _worker_id: &str) -> Result<()> {
        let mut state = self.state.lock().unwrap();
        state.heartbeat_count += 1;

        if state.processing.contains_key(&job_id) {
            Ok(())
        } else {
            Err(QueueError::NotFound(job_id))
        }
    }

    async fn stats(&self) -> Result<QueueStats> {
        let state = self.state.lock().unwrap();
        Ok(QueueStats {
            pending: state.pending.len(),
            processing: state.processing.len(),
            completed: state.completed.len(),
            failed: state.failed.len(),
        })
    }

    async fn release_stale_jobs(&self, _timeout: Duration) -> Result<Vec<Uuid>> {
        // For testing, we don't track heartbeat timestamps
        // Just return empty list
        Ok(Vec::new())
    }

    async fn get_job(&self, job_id: Uuid) -> Result<Option<Job>> {
        let state = self.state.lock().unwrap();

        // Check pending
        if let Some(job) = state.pending.iter().find(|j| j.id == job_id) {
            return Ok(Some(job.clone()));
        }

        // Check processing
        if let Some((job, _)) = state.processing.get(&job_id) {
            return Ok(Some(job.clone()));
        }

        // Check completed
        if let Some(job) = state.completed.iter().find(|j| j.id == job_id) {
            return Ok(Some(job.clone()));
        }

        // Check failed
        if let Some((job, _)) = state.failed.iter().find(|(j, _)| j.id == job_id) {
            return Ok(Some(job.clone()));
        }

        Ok(None)
    }
}

#[cfg(all(test, feature = "server"))]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_enqueue_dequeue() {
        let queue = MockQueue::new();
        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({}));
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();
        assert_eq!(queue.pending_count(), 1);

        let dequeued = queue.dequeue("worker-1").await.unwrap().unwrap();
        assert_eq!(dequeued.id, job_id);
        assert_eq!(queue.processing_count(), 1);
        assert_eq!(queue.pending_count(), 0);
    }

    #[tokio::test]
    async fn test_complete() {
        let queue = MockQueue::new();
        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({}));
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();
        queue.dequeue("worker-1").await.unwrap();
        queue.complete(job_id).await.unwrap();

        assert_eq!(queue.completed_count(), 1);
        assert_eq!(queue.processing_count(), 0);
    }

    #[tokio::test]
    async fn test_fail() {
        let queue = MockQueue::new();
        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({}));
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();
        queue.dequeue("worker-1").await.unwrap();
        queue.fail(job_id, "Test error").await.unwrap();

        assert_eq!(queue.failed_count(), 1);
        let failed = queue.get_failed();
        assert_eq!(failed[0].1, "Test error");
    }

    #[tokio::test]
    async fn test_requeue() {
        let queue = MockQueue::new();
        let job = Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({}));
        let job_id = job.id;

        queue.enqueue(job).await.unwrap();
        queue.dequeue("worker-1").await.unwrap();
        queue.requeue(job_id, None).await.unwrap();

        assert_eq!(queue.pending_count(), 1);
        assert_eq!(queue.processing_count(), 0);
        assert_eq!(queue.requeue_count(), 1);
    }

    #[tokio::test]
    async fn test_stats() {
        let queue = MockQueue::new();

        for _ in 0..3 {
            queue.enqueue(Job::new(Uuid::new_v4(), Uuid::new_v4(), json!({}))).await.unwrap();
        }

        let job = queue.dequeue("worker-1").await.unwrap().unwrap();
        queue.complete(job.id).await.unwrap();

        let stats = queue.stats().await.unwrap();
        assert_eq!(stats.pending, 2);
        assert_eq!(stats.processing, 0);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.failed, 0);
    }
}
