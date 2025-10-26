//! Predefined Task Schema Definitions
//!
//! This module defines the type structures for predefined tasks, including metadata,
//! input/output specifications, and agent templates.

use crate::dsl::schema::{InputSpec, OutputSpec, PermissionsSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// API version for predefined task format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TaskApiVersion {
    /// Version 1 of the task API
    #[serde(rename = "task/v1")]
    #[default]
    V1,
}

/// Kind identifier for predefined tasks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum TaskKind {
    /// Predefined task definition
    #[default]
    PredefinedTask,
}

/// Complete predefined task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedTask {
    /// API version
    #[serde(rename = "apiVersion")]
    pub api_version: TaskApiVersion,

    /// Kind of resource
    pub kind: TaskKind,

    /// Task metadata
    pub metadata: PredefinedTaskMetadata,

    /// Task specification
    pub spec: PredefinedTaskSpec,
}

/// Metadata for a predefined task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedTaskMetadata {
    /// Task name (unique identifier)
    pub name: String,

    /// Semantic version (e.g., "1.2.0")
    pub version: String,

    /// Author/organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Task description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// License identifier (e.g., "MIT", "Apache-2.0")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Repository URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Tags for categorization and discovery
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Specification for a predefined task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedTaskSpec {
    /// Agent template that will be instantiated when this task is used
    pub agent_template: AgentTemplate,

    /// Input parameter definitions with validation
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, PredefinedTaskInputSpec>,

    /// Output definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, PredefinedTaskOutputSpec>,

    /// Task dependencies (Phase 3+)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<TaskDependency>,

    /// Example usages for documentation
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<TaskExample>,
}

/// Agent template for instantiation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    /// Agent description (supports variable interpolation)
    pub description: String,

    /// Model to use (e.g., "claude-sonnet-4-5")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,

    /// System prompt (supports variable interpolation)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,

    /// Allowed tools for this agent
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,

    /// Permission settings
    #[serde(default)]
    pub permissions: PermissionsSpec,

    /// Maximum number of turns
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
}

/// Input specification for predefined tasks (extends base InputSpec)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedTaskInputSpec {
    /// Base input specification
    #[serde(flatten)]
    pub base: InputSpec,

    /// Validation rules
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<InputValidation>,

    /// Source for default value (e.g., environment variable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Input validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputValidation {
    /// Regex pattern for string validation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    /// Minimum value for numbers
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Maximum value for numbers
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    /// Minimum length for strings/arrays
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length for strings/arrays
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Allowed values (enum)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_values: Vec<serde_json::Value>,
}

/// Output specification for predefined tasks (extends base OutputSpec)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredefinedTaskOutputSpec {
    /// Base output specification
    #[serde(flatten)]
    pub base: OutputSpec,

    /// Output type for documentation
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub output_type: Option<String>,
}

/// Task dependency specification (Phase 3+)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    /// Dependency task name
    pub name: String,

    /// Version constraint (e.g., "^1.2.0")
    pub version: String,

    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// Example usage for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExample {
    /// Example name
    pub name: String,

    /// Example description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Example input values
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Task reference in workflow (what goes in TaskSpec.uses field)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskReference {
    /// Task name
    pub name: String,

    /// Version constraint (e.g., "1.2.0", "^1.2.0", "latest")
    /// For Phase 1, only exact versions are supported
    pub version: String,
}

impl TaskReference {
    /// Parse a task reference string (e.g., "google-drive-upload@1.2.0")
    pub fn parse(reference: &str) -> Result<Self, String> {
        let parts: Vec<&str> = reference.split('@').collect();

        if parts.len() != 2 {
            return Err(format!(
                "Invalid task reference '{}'. Expected format: 'task-name@version'",
                reference
            ));
        }

        let name = parts[0].trim();
        let version = parts[1].trim();

        if name.is_empty() {
            return Err("Task name cannot be empty".to_string());
        }

        if version.is_empty() {
            return Err("Version cannot be empty".to_string());
        }

        Ok(TaskReference {
            name: name.to_string(),
            version: version.to_string(),
        })
    }
}

impl fmt::Display for TaskReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_reference_parse_valid() {
        let reference = TaskReference::parse("google-drive-upload@1.2.0").unwrap();
        assert_eq!(reference.name, "google-drive-upload");
        assert_eq!(reference.version, "1.2.0");
    }

    #[test]
    fn test_task_reference_parse_with_spaces() {
        let reference = TaskReference::parse("  my-task  @  1.0.0  ").unwrap();
        assert_eq!(reference.name, "my-task");
        assert_eq!(reference.version, "1.0.0");
    }

    #[test]
    fn test_task_reference_parse_invalid_no_at() {
        let result = TaskReference::parse("google-drive-upload");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected format"));
    }

    #[test]
    fn test_task_reference_parse_invalid_empty_name() {
        let result = TaskReference::parse("@1.2.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_task_reference_parse_invalid_empty_version() {
        let result = TaskReference::parse("task@");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Version cannot be empty"));
    }

    #[test]
    fn test_task_reference_to_string() {
        let reference = TaskReference {
            name: "my-task".to_string(),
            version: "2.1.0".to_string(),
        };
        assert_eq!(reference.to_string(), "my-task@2.1.0");
    }
}
