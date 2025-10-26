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
