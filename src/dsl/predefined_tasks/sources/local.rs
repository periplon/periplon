//! Local Filesystem Task Source
//!
//! Discovers and loads predefined tasks from local directories.

use async_trait::async_trait;
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::dsl::predefined_tasks::{load_predefined_task, PredefinedTask};
use crate::error::Result;

use super::config::expand_path;
use super::{HealthStatus, SourceType, TaskMetadata, TaskSource, UpdateResult};

/// Local filesystem task source
pub struct LocalTaskSource {
    name: String,
    path: PathBuf,
    priority: u8,
}

impl LocalTaskSource {
    /// Create a new local task source
    pub fn new(name: String, path: impl AsRef<Path>, priority: u8) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy();
        let path = expand_path(&path_str)?;
        Ok(Self {
            name,
            path,
            priority,
        })
    }

    /// Scan directory for .task.yaml files recursively
    async fn scan_tasks(&self) -> Result<Vec<PathBuf>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let mut task_files = Vec::new();
        self.visit_dirs(&self.path, &mut task_files)?;
        Ok(task_files)
    }

    /// Recursively visit directories to find task files
    fn visit_dirs(&self, dir: &Path, task_files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.visit_dirs(&path, task_files)?;
            } else if self.is_task_file(&path) {
                task_files.push(path);
            }
        }

        Ok(())
    }

    /// Check if a file is a task definition file
    fn is_task_file(&self, path: &Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("yaml")
            && path
                .file_name()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.ends_with(".task.yaml"))
    }
}

#[async_trait]
impl TaskSource for LocalTaskSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_type(&self) -> SourceType {
        SourceType::Local
    }

    fn priority(&self) -> u8 {
        self.priority
    }

    fn is_trusted(&self) -> bool {
        true // Local sources are always trusted
    }

    async fn discover_tasks(&mut self) -> Result<Vec<TaskMetadata>> {
        let task_files = self.scan_tasks().await?;
        let mut metadata = Vec::new();

        for task_file in task_files {
            match load_predefined_task(&task_file) {
                Ok(task) => {
                    metadata.push(TaskMetadata::from((
                        &task.metadata,
                        self.name.as_str(),
                        SourceType::Local,
                    )));
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse task file {}: {}",
                        task_file.display(),
                        e
                    );
                }
            }
        }

        Ok(metadata)
    }

    async fn load_task(&mut self, name: &str, version: Option<&str>) -> Result<PredefinedTask> {
        let task_files = self.scan_tasks().await?;

        // Try to find exact match first
        for task_file in &task_files {
            if let Ok(task) = load_predefined_task(task_file) {
                if task.metadata.name == name {
                    if let Some(ver) = version {
                        if task.metadata.version == ver {
                            return Ok(task);
                        }
                    } else {
                        // No version specified, return first match
                        return Ok(task);
                    }
                }
            }
        }

        Err(crate::error::Error::TaskNotFound {
            name: name.to_string(),
            version: version.map(String::from),
            source_name: self.name.clone(),
        })
    }

    async fn update(&mut self) -> Result<UpdateResult> {
        // Local sources don't need updating
        Ok(UpdateResult {
            updated: false,
            message: format!("Local source does not require updates: {}", self.name),
            new_tasks: 0,
            updated_tasks: 0,
        })
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let available = self.path.exists();

        Ok(HealthStatus {
            available,
            message: if available {
                None
            } else {
                Some(format!("Path does not exist: {}", self.path.display()))
            },
            last_check: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_source_discovery() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        // Create test task
        let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
  description: "Test task"
spec:
  agent_template:
    description: "Test task"
  inputs:
    input1:
      type: string
      required: true
"#;
        std::fs::write(tasks_dir.join("test.task.yaml"), task_yaml).unwrap();

        let mut source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();

        let tasks = source.discover_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "test-task");
        assert_eq!(tasks[0].version, "1.0.0");
    }

    #[tokio::test]
    async fn test_load_task_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "my-task"
  version: "2.0.0"
spec:
  agent_template:
    description: "My task"
  inputs: {}
"#;
        std::fs::write(tasks_dir.join("my-task.task.yaml"), task_yaml).unwrap();

        let mut source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();

        let task = source.load_task("my-task", None).await.unwrap();
        assert_eq!(task.metadata.name, "my-task");
        assert_eq!(task.metadata.version, "2.0.0");
    }

    #[tokio::test]
    async fn test_load_task_by_name_and_version() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "versioned-task"
  version: "1.5.0"
spec:
  agent_template:
    description: "Versioned task"
  inputs: {}
"#;
        std::fs::write(tasks_dir.join("versioned.task.yaml"), task_yaml).unwrap();

        let mut source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();

        let task = source
            .load_task("versioned-task", Some("1.5.0"))
            .await
            .unwrap();
        assert_eq!(task.metadata.version, "1.5.0");

        // Wrong version should fail
        let result = source.load_task("versioned-task", Some("2.0.0")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_nonexistent_path() {
        let source = LocalTaskSource::new("test".to_string(), "/nonexistent/path", 10).unwrap();

        let health = source.health_check().await.unwrap();
        assert!(!health.available);
        assert!(health.message.is_some());
    }

    #[tokio::test]
    async fn test_nonexistent_path_discovery() {
        let mut source = LocalTaskSource::new("test".to_string(), "/nonexistent/path", 10).unwrap();

        // Should return empty list for nonexistent path
        let tasks = source.discover_tasks().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        let nested_dir = tasks_dir.join("subdir");
        std::fs::create_dir_all(&nested_dir).unwrap();

        // Create task in nested directory
        let task_yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "nested-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Nested task"
  inputs: {}
"#;
        std::fs::write(nested_dir.join("nested.task.yaml"), task_yaml).unwrap();

        let mut source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();

        let tasks = source.discover_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].name, "nested-task");
    }

    #[tokio::test]
    async fn test_update_operation() {
        let temp_dir = TempDir::new().unwrap();
        let tasks_dir = temp_dir.path().join("tasks");
        std::fs::create_dir(&tasks_dir).unwrap();

        let mut source = LocalTaskSource::new("test".to_string(), &tasks_dir, 10).unwrap();

        // Update should succeed but report no changes
        let result = source.update().await.unwrap();
        assert!(!result.updated);
        assert_eq!(result.new_tasks, 0);
        assert_eq!(result.updated_tasks, 0);
        assert!(result.message.contains("does not require updates"));
    }

    #[tokio::test]
    async fn test_trait_methods() {
        let temp_dir = TempDir::new().unwrap();
        let source = LocalTaskSource::new("my-source".to_string(), temp_dir.path(), 5).unwrap();

        assert_eq!(source.name(), "my-source");
        assert_eq!(source.source_type(), SourceType::Local);
        assert_eq!(source.priority(), 5);
        assert!(source.is_trusted());
    }

    #[tokio::test]
    async fn test_path_is_file_not_directory() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("not-a-directory.txt");
        std::fs::write(&file_path, "content").unwrap();

        let mut source = LocalTaskSource::new("test".to_string(), &file_path, 10).unwrap();

        // Should return empty list when path is a file, not a directory
        let tasks = source.discover_tasks().await.unwrap();
        assert_eq!(tasks.len(), 0);
    }
}
