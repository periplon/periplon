//! Task Group Schema Definitions
//!
//! This module defines the type structures for task groups - collections of related tasks
//! that work together as cohesive units. Task groups enable bundling multiple tasks into
//! integration suites, feature bundles, or multi-step workflows.

use crate::dsl::schema::{InputSpec, PermissionsSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// API version for task group format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum TaskGroupApiVersion {
    /// Version 1 of the task group API
    #[serde(rename = "taskgroup/v1")]
    #[default]
    V1,
}


/// Kind identifier for task groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[derive(Default)]
pub enum TaskGroupKind {
    /// Task group definition
    #[default]
    TaskGroup,
}


/// Complete task group definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGroup {
    /// API version
    #[serde(rename = "apiVersion")]
    pub api_version: TaskGroupApiVersion,

    /// Kind of resource
    pub kind: TaskGroupKind,

    /// Task group metadata
    pub metadata: TaskGroupMetadata,

    /// Task group specification
    pub spec: TaskGroupSpec,
}

/// Metadata for a task group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGroupMetadata {
    /// Task group name (unique identifier)
    pub name: String,

    /// Semantic version (e.g., "2.0.0")
    pub version: String,

    /// Author/organization
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Task group description
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

/// Specification for a task group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGroupSpec {
    /// Tasks included in this group
    pub tasks: Vec<TaskGroupTask>,

    /// Shared configuration for all tasks in the group
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_config: Option<SharedConfig>,

    /// Pre-configured workflows using these tasks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflows: Vec<PrebuiltWorkflow>,

    /// Group-level dependencies
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<GroupDependency>,

    /// Installation and usage hooks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<GroupHooks>,
}

/// Task reference within a task group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGroupTask {
    /// Task name
    pub name: String,

    /// Version constraint (e.g., "^1.2.0", "~1.5.0")
    pub version: String,

    /// Whether this task is required for the group to function
    #[serde(default = "default_required")]
    pub required: bool,

    /// Optional description of the task's role in the group
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_required() -> bool {
    true
}

/// Shared configuration applied to all tasks in the group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedConfig {
    /// Shared input parameters for all tasks
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, InputSpec>,

    /// Shared permission settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissions: Option<PermissionsSpec>,

    /// Shared environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub environment: HashMap<String, String>,

    /// Shared maximum turns for all agents
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
}

/// Pre-configured workflow template within a task group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrebuiltWorkflow {
    /// Workflow name
    pub name: String,

    /// Workflow description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Tasks in the workflow (raw YAML value to avoid circular dependencies)
    pub tasks: serde_yaml::Value,

    /// Input parameters for the workflow
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, InputSpec>,

    /// Output definitions for the workflow
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, String>,
}

/// Group-level dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupDependency {
    /// Dependency name
    pub name: String,

    /// Version constraint (e.g., "^1.0.0")
    pub version: String,

    /// Repository URL if not in standard sources
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}

/// Lifecycle hooks for task groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupHooks {
    /// Hooks to run after installing the group
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_install: Vec<Hook>,

    /// Hooks to run before using any task from the group
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_use: Vec<Hook>,

    /// Hooks to run after uninstalling the group
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_uninstall: Vec<Hook>,
}

/// Individual hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Hook {
    /// Execute a shell command
    #[serde(rename = "command")]
    Command {
        /// Command to execute
        command: String,
        /// Optional working directory
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
    },

    /// Validate a condition
    #[serde(rename = "validate")]
    Validate {
        /// Condition to check (e.g., "env.API_KEY")
        check: String,
        /// Error message if validation fails
        message: String,
    },

    /// Display a message
    #[serde(rename = "message")]
    Message {
        /// Message content
        content: String,
        /// Message level (info, warning, error)
        #[serde(default = "default_level")]
        level: String,
    },
}

fn default_level() -> String {
    "info".to_string()
}

/// Task group reference for importing into workflows
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskGroupReference {
    /// Group name
    pub name: String,

    /// Version constraint (e.g., "^2.0.0", "latest")
    pub version: String,

    /// Optional workflow name within the group (for uses_workflow syntax)
    pub workflow: Option<String>,
}

impl TaskGroupReference {
    /// Parse a task group reference string
    ///
    /// Supports two formats:
    /// - "group-name@version" - Reference to a task group
    /// - "group-name@version#workflow-name" - Reference to a specific workflow in the group
    pub fn parse(reference: &str) -> Result<Self, String> {
        // Split on '#' to check for workflow reference
        let parts: Vec<&str> = reference.split('#').collect();

        if parts.len() > 2 {
            return Err(format!(
                "Invalid task group reference '{}'. Expected format: 'group-name@version' or 'group-name@version#workflow'",
                reference
            ));
        }

        let workflow = if parts.len() == 2 {
            let workflow_name = parts[1].trim();
            if workflow_name.is_empty() {
                return Err("Workflow name cannot be empty".to_string());
            }
            Some(workflow_name.to_string())
        } else {
            None
        };

        // Parse the group@version part
        let group_version: Vec<&str> = parts[0].split('@').collect();

        if group_version.len() != 2 {
            return Err(format!(
                "Invalid task group reference '{}'. Expected format: 'group-name@version'",
                parts[0]
            ));
        }

        let name = group_version[0].trim();
        let version = group_version[1].trim();

        if name.is_empty() {
            return Err("Task group name cannot be empty".to_string());
        }

        if version.is_empty() {
            return Err("Version cannot be empty".to_string());
        }

        Ok(TaskGroupReference {
            name: name.to_string(),
            version: version.to_string(),
            workflow,
        })
    }
}

impl fmt::Display for TaskGroupReference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref workflow) = self.workflow {
            write!(f, "{}@{}#{}", self.name, self.version, workflow)
        } else {
            write!(f, "{}@{}", self.name, self.version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_group_reference_parse_basic() {
        let reference = TaskGroupReference::parse("google-workspace-suite@2.0.0").unwrap();
        assert_eq!(reference.name, "google-workspace-suite");
        assert_eq!(reference.version, "2.0.0");
        assert_eq!(reference.workflow, None);
    }

    #[test]
    fn test_task_group_reference_parse_with_workflow() {
        let reference =
            TaskGroupReference::parse("google-workspace-suite@2.0.0#backup-to-drive").unwrap();
        assert_eq!(reference.name, "google-workspace-suite");
        assert_eq!(reference.version, "2.0.0");
        assert_eq!(reference.workflow, Some("backup-to-drive".to_string()));
    }

    #[test]
    fn test_task_group_reference_parse_with_spaces() {
        let reference = TaskGroupReference::parse("  my-group  @  1.0.0  #  workflow  ").unwrap();
        assert_eq!(reference.name, "my-group");
        assert_eq!(reference.version, "1.0.0");
        assert_eq!(reference.workflow, Some("workflow".to_string()));
    }

    #[test]
    fn test_task_group_reference_parse_invalid_no_at() {
        let result = TaskGroupReference::parse("google-workspace-suite");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected format"));
    }

    #[test]
    fn test_task_group_reference_parse_invalid_empty_name() {
        let result = TaskGroupReference::parse("@2.0.0");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("name cannot be empty"));
    }

    #[test]
    fn test_task_group_reference_parse_invalid_empty_version() {
        let result = TaskGroupReference::parse("group@");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Version cannot be empty"));
    }

    #[test]
    fn test_task_group_reference_parse_invalid_empty_workflow() {
        let result = TaskGroupReference::parse("group@1.0.0#");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Workflow name cannot be empty"));
    }

    #[test]
    fn test_task_group_reference_parse_invalid_too_many_hashes() {
        let result = TaskGroupReference::parse("group@1.0.0#workflow#extra");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Expected format"));
    }

    #[test]
    fn test_task_group_reference_to_string_basic() {
        let reference = TaskGroupReference {
            name: "my-group".to_string(),
            version: "2.1.0".to_string(),
            workflow: None,
        };
        assert_eq!(reference.to_string(), "my-group@2.1.0");
    }

    #[test]
    fn test_task_group_reference_to_string_with_workflow() {
        let reference = TaskGroupReference {
            name: "my-group".to_string(),
            version: "2.1.0".to_string(),
            workflow: Some("my-workflow".to_string()),
        };
        assert_eq!(reference.to_string(), "my-group@2.1.0#my-workflow");
    }

    #[test]
    fn test_shared_config_defaults() {
        let config = SharedConfig {
            inputs: HashMap::new(),
            permissions: None,
            environment: HashMap::new(),
            max_turns: None,
        };
        assert!(config.inputs.is_empty());
        assert!(config.permissions.is_none());
        assert!(config.environment.is_empty());
        assert!(config.max_turns.is_none());
    }

    #[test]
    fn test_task_group_task_required_default() {
        let json = r#"{
            "name": "test-task",
            "version": "1.0.0"
        }"#;
        let task: TaskGroupTask = serde_json::from_str(json).unwrap();
        assert!(task.required); // Should default to true
    }

    #[test]
    fn test_task_group_task_required_explicit() {
        let json = r#"{
            "name": "test-task",
            "version": "1.0.0",
            "required": false
        }"#;
        let task: TaskGroupTask = serde_json::from_str(json).unwrap();
        assert!(!task.required);
    }

    #[test]
    fn test_hook_command_deserialization() {
        let json = r#"{
            "type": "command",
            "command": "echo hello",
            "cwd": "/tmp"
        }"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        match hook {
            Hook::Command { command, cwd } => {
                assert_eq!(command, "echo hello");
                assert_eq!(cwd, Some("/tmp".to_string()));
            }
            _ => panic!("Expected Hook::Command"),
        }
    }

    #[test]
    fn test_hook_validate_deserialization() {
        let json = r#"{
            "type": "validate",
            "check": "env.API_KEY",
            "message": "API_KEY required"
        }"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        match hook {
            Hook::Validate { check, message } => {
                assert_eq!(check, "env.API_KEY");
                assert_eq!(message, "API_KEY required");
            }
            _ => panic!("Expected Hook::Validate"),
        }
    }

    #[test]
    fn test_hook_message_deserialization() {
        let json = r#"{
            "type": "message",
            "content": "Installation complete"
        }"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        match hook {
            Hook::Message { content, level } => {
                assert_eq!(content, "Installation complete");
                assert_eq!(level, "info"); // Default level
            }
            _ => panic!("Expected Hook::Message"),
        }
    }

    #[test]
    fn test_hook_message_with_level() {
        let json = r#"{
            "type": "message",
            "content": "Warning message",
            "level": "warning"
        }"#;
        let hook: Hook = serde_json::from_str(json).unwrap();
        match hook {
            Hook::Message { content, level } => {
                assert_eq!(content, "Warning message");
                assert_eq!(level, "warning");
            }
            _ => panic!("Expected Hook::Message"),
        }
    }
}
