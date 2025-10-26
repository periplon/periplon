// Filesystem queue backend implementation

#[cfg(feature = "server")]
use async_trait::async_trait;
#[cfg(feature = "server")]
use chrono::Utc;
#[cfg(feature = "server")]
use serde_json;
#[cfg(feature = "server")]
use std::ffi::OsStr;
#[cfg(feature = "server")]
use std::fs::OpenOptions;
#[cfg(feature = "server")]
use std::io::Write;
#[cfg(feature = "server")]
use std::path::{Path, PathBuf};
#[cfg(feature = "server")]
use std::time::Duration;
#[cfg(feature = "server")]
use tokio::fs;
#[cfg(feature = "server")]
use uuid::Uuid;

#[cfg(feature = "server")]
use super::traits::*;

#[cfg(feature = "server")]
pub struct FilesystemQueue {
    queue_dir: PathBuf,
    _poll_interval: Duration,
    lock_timeout: Duration,
}

#[cfg(feature = "server")]
impl FilesystemQueue {
    pub async fn new(
        queue_dir: PathBuf,
        poll_interval_ms: u64,
        lock_timeout_secs: u64,
    ) -> Result<Self> {
        let queue = Self {
            queue_dir,
            _poll_interval: Duration::from_millis(poll_interval_ms),
            lock_timeout: Duration::from_secs(lock_timeout_secs),
        };

        // Create queue directories
        queue.ensure_directories().await?;

        Ok(queue)
    }

    async fn ensure_directories(&self) -> Result<()> {
        let dirs = ["pending", "processing", "completed", "failed"];

        for dir in dirs {
            let path = self.queue_dir.join(dir);
            fs::create_dir_all(&path)
                .await
                .map_err(|e| QueueError::IoError(e.to_string()))?;
        }

        Ok(())
    }

    async fn try_lock(&self, job_id: &Uuid, worker_id: &str) -> Result<bool> {
        let lock_path = self
            .queue_dir
            .join("pending")
            .join(format!("{}.lock", job_id));

        // Try to create lock file exclusively
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                // Write worker ID and timestamp
                let lock_data = format!("{}:{}", worker_id, Utc::now().to_rfc3339());
                file.write_all(lock_data.as_bytes())
                    .map_err(|e| QueueError::LockError(e.to_string()))?;
                file.sync_all()
                    .map_err(|e| QueueError::LockError(e.to_string()))?;
                Ok(true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                // Check if lock is stale
                if self.is_lock_stale(&lock_path).await? {
                    // Remove stale lock and retry (use Box::pin to avoid infinite recursion)
                    fs::remove_file(&lock_path)
                        .await
                        .map_err(|e| QueueError::IoError(e.to_string()))?;
                    Box::pin(self.try_lock(job_id, worker_id)).await
                } else {
                    Ok(false)
                }
            }
            Err(e) => Err(QueueError::LockError(e.to_string())),
        }
    }

    async fn is_lock_stale(&self, lock_path: &Path) -> Result<bool> {
        let metadata = fs::metadata(lock_path)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        let modified = metadata
            .modified()
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        let age = std::time::SystemTime::now()
            .duration_since(modified)
            .map_err(|e| QueueError::Error(e.to_string()))?;

        Ok(age > self.lock_timeout)
    }

    async fn move_job(&self, job_id: Uuid, from_dir: &str, to_dir: &str) -> Result<()> {
        let from_path = self
            .queue_dir
            .join(from_dir)
            .join(format!("{}.json", job_id));
        let to_path = self.queue_dir.join(to_dir).join(format!("{}.json", job_id));

        fs::rename(&from_path, &to_path)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        // Remove lock file if exists
        let lock_path = self
            .queue_dir
            .join(from_dir)
            .join(format!("{}.lock", job_id));
        if lock_path.exists() {
            let _ = fs::remove_file(&lock_path).await;
        }

        Ok(())
    }
}

#[cfg(feature = "server")]
#[async_trait]
impl WorkQueue for FilesystemQueue {
    async fn enqueue(&self, job: Job) -> Result<Uuid> {
        let id = job.id;
        let pending_path = self.queue_dir.join("pending").join(format!("{}.json", id));

        let json = serde_json::to_string_pretty(&job)
            .map_err(|e| QueueError::SerializationError(e.to_string()))?;

        fs::write(&pending_path, json)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        Ok(id)
    }

    async fn dequeue(&self, worker_id: &str) -> Result<Option<Job>> {
        let pending_dir = self.queue_dir.join("pending");
        let mut entries = fs::read_dir(&pending_dir)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?
        {
            let path = entry.path();

            // Skip lock files
            if path.extension() == Some(OsStr::new("lock")) {
                continue;
            }

            let file_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| Uuid::parse_str(s).ok());

            if let Some(job_id) = file_name {
                // Try to acquire lock
                if self.try_lock(&job_id, worker_id).await? {
                    // Read job file
                    let contents = fs::read_to_string(&path)
                        .await
                        .map_err(|e| QueueError::IoError(e.to_string()))?;
                    let mut job: Job = serde_json::from_str(&contents)
                        .map_err(|e| QueueError::SerializationError(e.to_string()))?;

                    // Check if job is scheduled for future
                    if let Some(scheduled_at) = job.scheduled_at {
                        if scheduled_at > Utc::now() {
                            // Release lock and skip
                            let lock_path = pending_dir.join(format!("{}.lock", job_id));
                            let _ = fs::remove_file(&lock_path).await;
                            continue;
                        }
                    }

                    // Increment attempts
                    job.attempts += 1;

                    // Move to processing
                    self.move_job(job_id, "pending", "processing").await?;

                    // Write updated job
                    let processing_path = self
                        .queue_dir
                        .join("processing")
                        .join(format!("{}.json", job_id));
                    let json = serde_json::to_string_pretty(&job)
                        .map_err(|e| QueueError::SerializationError(e.to_string()))?;
                    fs::write(&processing_path, json)
                        .await
                        .map_err(|e| QueueError::IoError(e.to_string()))?;

                    return Ok(Some(job));
                }
            }
        }

        Ok(None)
    }

    async fn complete(&self, job_id: Uuid) -> Result<()> {
        self.move_job(job_id, "processing", "completed").await
    }

    async fn fail(&self, job_id: Uuid, error: &str) -> Result<()> {
        // Write error file
        let error_path = self
            .queue_dir
            .join("failed")
            .join(format!("{}.error", job_id));
        fs::write(&error_path, error)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        self.move_job(job_id, "processing", "failed").await
    }

    async fn requeue(&self, job_id: Uuid, delay: Option<Duration>) -> Result<()> {
        // Read job from processing or failed
        let processing_path = self
            .queue_dir
            .join("processing")
            .join(format!("{}.json", job_id));
        let failed_path = self
            .queue_dir
            .join("failed")
            .join(format!("{}.json", job_id));

        let (path, from_dir) = if processing_path.exists() {
            (processing_path, "processing")
        } else if failed_path.exists() {
            (failed_path, "failed")
        } else {
            return Err(QueueError::NotFound(job_id));
        };

        let contents = fs::read_to_string(&path)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;
        let mut job: Job = serde_json::from_str(&contents)
            .map_err(|e| QueueError::SerializationError(e.to_string()))?;

        // Apply delay if specified
        if let Some(delay) = delay {
            job.scheduled_at = Some(Utc::now() + chrono::Duration::from_std(delay).unwrap());
        }

        // Move back to pending
        self.move_job(job_id, from_dir, "pending").await?;

        // Write updated job
        let pending_path = self
            .queue_dir
            .join("pending")
            .join(format!("{}.json", job_id));
        let json = serde_json::to_string_pretty(&job)
            .map_err(|e| QueueError::SerializationError(e.to_string()))?;
        fs::write(&pending_path, json)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn heartbeat(&self, job_id: Uuid, worker_id: &str) -> Result<()> {
        let lock_path = self
            .queue_dir
            .join("processing")
            .join(format!("{}.lock", job_id));

        // Update lock file timestamp
        let lock_data = format!("{}:{}", worker_id, Utc::now().to_rfc3339());
        fs::write(&lock_path, lock_data)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        Ok(())
    }

    async fn release_stale_jobs(&self, _timeout: Duration) -> Result<Vec<Uuid>> {
        let processing_dir = self.queue_dir.join("processing");
        let mut entries = fs::read_dir(&processing_dir)
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?;

        let mut released = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| QueueError::IoError(e.to_string()))?
        {
            let path = entry.path();

            if path.extension() == Some(OsStr::new("lock"))
                && self.is_lock_stale(&path).await? {
                    // Extract job ID from lock file name
                    if let Some(job_id_str) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(job_id) = Uuid::parse_str(job_id_str) {
                            // Move back to pending
                            self.move_job(job_id, "processing", "pending").await?;
                            released.push(job_id);
                        }
                    }
                }
        }

        Ok(released)
    }

    async fn stats(&self) -> Result<QueueStats> {
        let mut stats = QueueStats {
            pending: 0,
            processing: 0,
            completed: 0,
            failed: 0,
        };

        // Count files in each directory
        for (dir_name, counter) in [
            ("pending", &mut stats.pending),
            ("processing", &mut stats.processing),
            ("completed", &mut stats.completed),
            ("failed", &mut stats.failed),
        ] {
            let dir_path = self.queue_dir.join(dir_name);
            if let Ok(mut entries) = fs::read_dir(&dir_path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if entry.path().extension() == Some(OsStr::new("json")) {
                        *counter += 1;
                    }
                }
            }
        }

        Ok(stats)
    }

    async fn get_job(&self, job_id: Uuid) -> Result<Option<Job>> {
        // Check all directories
        for dir in ["pending", "processing", "completed", "failed"] {
            let path = self.queue_dir.join(dir).join(format!("{}.json", job_id));

            if path.exists() {
                let contents = fs::read_to_string(&path)
                    .await
                    .map_err(|e| QueueError::IoError(e.to_string()))?;
                let job: Job = serde_json::from_str(&contents)
                    .map_err(|e| QueueError::SerializationError(e.to_string()))?;
                return Ok(Some(job));
            }
        }

        Ok(None)
    }
}
