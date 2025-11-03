//! Task Source Abstraction
//!
//! This module provides a unified abstraction for discovering and loading predefined tasks
//! from various sources (local filesystem, git repositories, registries, etc.).

pub mod config;
pub mod git;
pub mod local;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::dsl::predefined_tasks::{PredefinedTask, PredefinedTaskMetadata};
use crate::error::Result;

pub use config::{SourceConfig, TaskSourcesConfig, UpdatePolicy};
pub use git::GitTaskSource;
pub use local::LocalTaskSource;

/// Represents a source of predefined tasks
#[async_trait]
pub trait TaskSource: Send + Sync {
    /// Unique name of this source
    fn name(&self) -> &str;

    /// Source type identifier
    fn source_type(&self) -> SourceType;

    /// Priority for resolution (higher = searched first)
    fn priority(&self) -> u8;

    /// Whether this source is trusted
    fn is_trusted(&self) -> bool;

    /// Discover all tasks from this source
    async fn discover_tasks(&mut self) -> Result<Vec<TaskMetadata>>;

    /// Load a specific task by name and optional version
    async fn load_task(&mut self, name: &str, version: Option<&str>) -> Result<PredefinedTask>;

    /// Update/refresh the source (git pull, registry sync, etc.)
    async fn update(&mut self) -> Result<UpdateResult>;

    /// Check if source is available/healthy
    async fn health_check(&self) -> Result<HealthStatus>;
}

/// Source type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Local,
    Git,
    Registry,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Local => write!(f, "local"),
            SourceType::Git => write!(f, "git"),
            SourceType::Registry => write!(f, "registry"),
        }
    }
}

/// Task metadata for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub source_name: String,
    pub source_type: SourceType,
}

impl From<(&PredefinedTaskMetadata, &str, SourceType)> for TaskMetadata {
    fn from(
        (metadata, source_name, source_type): (&PredefinedTaskMetadata, &str, SourceType),
    ) -> Self {
        Self {
            name: metadata.name.clone(),
            version: metadata.version.clone(),
            description: metadata.description.clone(),
            author: metadata.author.clone(),
            tags: metadata.tags.clone(),
            source_name: source_name.to_string(),
            source_type,
        }
    }
}

/// Update operation result
#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub updated: bool,
    pub message: String,
    pub new_tasks: usize,
    pub updated_tasks: usize,
}

/// Health status for a source
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub available: bool,
    pub message: Option<String>,
    pub last_check: DateTime<Utc>,
}

/// Information about a configured source
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub name: String,
    pub source_type: SourceType,
    pub priority: u8,
    pub trusted: bool,
    pub enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_display_local() {
        let source_type = SourceType::Local;
        assert_eq!(source_type.to_string(), "local");
    }

    #[test]
    fn test_source_type_display_git() {
        let source_type = SourceType::Git;
        assert_eq!(source_type.to_string(), "git");
    }

    #[test]
    fn test_source_type_display_registry() {
        let source_type = SourceType::Registry;
        assert_eq!(source_type.to_string(), "registry");
    }

    #[test]
    fn test_source_type_serialize() {
        let local = SourceType::Local;
        let json = serde_json::to_string(&local).unwrap();
        assert_eq!(json, "\"local\"");

        let git = SourceType::Git;
        let json = serde_json::to_string(&git).unwrap();
        assert_eq!(json, "\"git\"");

        let registry = SourceType::Registry;
        let json = serde_json::to_string(&registry).unwrap();
        assert_eq!(json, "\"registry\"");
    }

    #[test]
    fn test_source_type_deserialize() {
        let local: SourceType = serde_json::from_str("\"local\"").unwrap();
        assert_eq!(local, SourceType::Local);

        let git: SourceType = serde_json::from_str("\"git\"").unwrap();
        assert_eq!(git, SourceType::Git);

        let registry: SourceType = serde_json::from_str("\"registry\"").unwrap();
        assert_eq!(registry, SourceType::Registry);
    }

    #[test]
    fn test_task_metadata_from_conversion() {
        let predefined_metadata = PredefinedTaskMetadata {
            name: "test-task".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test task".to_string()),
            author: Some("Test Author".to_string()),
            license: Some("MIT".to_string()),
            repository: Some("https://github.com/test/task".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
        };

        let task_metadata: TaskMetadata =
            (&predefined_metadata, "my-source", SourceType::Local).into();

        assert_eq!(task_metadata.name, "test-task");
        assert_eq!(task_metadata.version, "1.0.0");
        assert_eq!(task_metadata.description, Some("A test task".to_string()));
        assert_eq!(task_metadata.author, Some("Test Author".to_string()));
        assert_eq!(task_metadata.tags.len(), 2);
        assert_eq!(task_metadata.source_name, "my-source");
        assert_eq!(task_metadata.source_type, SourceType::Local);
    }

    #[test]
    fn test_task_metadata_from_conversion_minimal() {
        let predefined_metadata = PredefinedTaskMetadata {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            description: None,
            author: None,
            license: None,
            repository: None,
            tags: vec![],
        };

        let task_metadata: TaskMetadata =
            (&predefined_metadata, "git-repo", SourceType::Git).into();

        assert_eq!(task_metadata.name, "minimal");
        assert_eq!(task_metadata.version, "0.1.0");
        assert!(task_metadata.description.is_none());
        assert!(task_metadata.author.is_none());
        assert!(task_metadata.tags.is_empty());
        assert_eq!(task_metadata.source_name, "git-repo");
        assert_eq!(task_metadata.source_type, SourceType::Git);
    }

    #[test]
    fn test_task_metadata_construction() {
        let metadata = TaskMetadata {
            name: "my-task".to_string(),
            version: "2.0.0".to_string(),
            description: Some("Description".to_string()),
            author: Some("Author Name".to_string()),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            source_name: "registry".to_string(),
            source_type: SourceType::Registry,
        };

        assert_eq!(metadata.name, "my-task");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_update_result_construction() {
        let result = UpdateResult {
            updated: true,
            message: "Updated successfully".to_string(),
            new_tasks: 5,
            updated_tasks: 3,
        };

        assert!(result.updated);
        assert_eq!(result.message, "Updated successfully");
        assert_eq!(result.new_tasks, 5);
        assert_eq!(result.updated_tasks, 3);
    }

    #[test]
    fn test_health_status_construction() {
        let now = Utc::now();
        let status = HealthStatus {
            available: true,
            message: Some("All systems operational".to_string()),
            last_check: now,
        };

        assert!(status.available);
        assert!(status.message.is_some());
        assert_eq!(status.last_check, now);
    }

    #[test]
    fn test_health_status_unavailable() {
        let now = Utc::now();
        let status = HealthStatus {
            available: false,
            message: Some("Connection failed".to_string()),
            last_check: now,
        };

        assert!(!status.available);
        assert_eq!(status.message, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_source_info_construction() {
        let info = SourceInfo {
            name: "my-source".to_string(),
            source_type: SourceType::Git,
            priority: 10,
            trusted: true,
            enabled: true,
        };

        assert_eq!(info.name, "my-source");
        assert_eq!(info.source_type, SourceType::Git);
        assert_eq!(info.priority, 10);
        assert!(info.trusted);
        assert!(info.enabled);
    }

    #[test]
    fn test_source_info_untrusted_disabled() {
        let info = SourceInfo {
            name: "untrusted-source".to_string(),
            source_type: SourceType::Registry,
            priority: 1,
            trusted: false,
            enabled: false,
        };

        assert!(!info.trusted);
        assert!(!info.enabled);
    }

    #[test]
    fn test_task_metadata_serialization_roundtrip() {
        let original = TaskMetadata {
            name: "roundtrip-test".to_string(),
            version: "3.0.0".to_string(),
            description: Some("Test description".to_string()),
            author: Some("Test Author".to_string()),
            tags: vec!["test".to_string()],
            source_name: "test-source".to_string(),
            source_type: SourceType::Local,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TaskMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, original.name);
        assert_eq!(deserialized.version, original.version);
        assert_eq!(deserialized.source_type, original.source_type);
    }
}
