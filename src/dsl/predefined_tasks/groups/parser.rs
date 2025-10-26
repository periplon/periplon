//! Task Group Parser
//!
//! This module handles parsing of `.taskgroup.yaml` files into TaskGroup structures
//! with comprehensive validation of group structure and task compatibility.

use super::schema::{
    GroupDependency, GroupHooks, PrebuiltWorkflow, SharedConfig, TaskGroup, TaskGroupMetadata,
    TaskGroupSpec, TaskGroupTask,
};
use crate::dsl::schema::InputSpec;
use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Errors that can occur during task group parsing
#[derive(Debug, Error)]
pub enum ParseError {
    /// YAML deserialization error
    #[error("Failed to parse task group YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidField { field: String, reason: String },

    /// Validation error
    #[error("Task group validation failed: {0}")]
    ValidationError(String),

    /// Task compatibility error
    #[error("Task compatibility issue: {0}")]
    CompatibilityError(String),

    /// Duplicate definition
    #[error("Duplicate {item_type} name: {name}")]
    DuplicateName { item_type: String, name: String },

    /// Circular dependency detected
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// Version constraint error
    #[error("Invalid version constraint '{constraint}': {reason}")]
    InvalidVersion { constraint: String, reason: String },
}

/// Parse a task group from YAML string
///
/// # Arguments
///
/// * `yaml_content` - YAML string containing the task group definition
///
/// # Returns
///
/// Parsed `TaskGroup` or error
///
/// # Example
///
/// ```
/// use periplon_sdk::dsl::predefined_tasks::groups::parse_task_group;
///
/// let yaml = r#"
/// apiVersion: "taskgroup/v1"
/// kind: "TaskGroup"
/// metadata:
///   name: "integration-suite"
///   version: "2.0.0"
///   description: "Suite of integration tasks"
/// spec:
///   tasks:
///     - name: "google-drive-upload"
///       version: "^1.0.0"
///       required: true
/// "#;
///
/// let group = parse_task_group(yaml).unwrap();
/// assert_eq!(group.metadata.name, "integration-suite");
/// ```
pub fn parse_task_group(yaml_content: &str) -> Result<TaskGroup, ParseError> {
    // Parse YAML into TaskGroup
    let group: TaskGroup = serde_yaml::from_str(yaml_content)?;

    // Validate the parsed task group
    validate_task_group(&group)?;

    Ok(group)
}

/// Validate a parsed task group
fn validate_task_group(group: &TaskGroup) -> Result<(), ParseError> {
    // Validate metadata
    validate_metadata(&group.metadata)?;

    // Validate spec
    validate_spec(&group.spec)?;

    // Cross-validate task compatibility
    validate_task_compatibility(&group.spec)?;

    Ok(())
}

/// Validate task group metadata
fn validate_metadata(metadata: &TaskGroupMetadata) -> Result<(), ParseError> {
    // Validate name (must be non-empty, lowercase, hyphen-separated)
    if metadata.name.is_empty() {
        return Err(ParseError::MissingField("metadata.name".to_string()));
    }

    if !is_valid_group_name(&metadata.name) {
        return Err(ParseError::InvalidField {
            field: "metadata.name".to_string(),
            reason: "Task group name must be lowercase with hyphens (e.g., 'my-group')".to_string(),
        });
    }

    // Validate version (must be valid semver)
    if metadata.version.is_empty() {
        return Err(ParseError::MissingField("metadata.version".to_string()));
    }

    if !is_valid_semver(&metadata.version) {
        return Err(ParseError::InvalidField {
            field: "metadata.version".to_string(),
            reason: "Version must be valid semantic version (e.g., '2.0.0')".to_string(),
        });
    }

    // Validate license if present
    if let Some(license) = &metadata.license {
        if license.is_empty() {
            return Err(ParseError::InvalidField {
                field: "metadata.license".to_string(),
                reason: "License cannot be empty if specified".to_string(),
            });
        }
    }

    // Validate repository URL if present
    if let Some(repo) = &metadata.repository {
        if repo.is_empty() {
            return Err(ParseError::InvalidField {
                field: "metadata.repository".to_string(),
                reason: "Repository URL cannot be empty if specified".to_string(),
            });
        }
    }

    // Validate tags
    for tag in &metadata.tags {
        if tag.is_empty() {
            return Err(ParseError::InvalidField {
                field: "metadata.tags".to_string(),
                reason: "Tags cannot contain empty strings".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate task group spec
fn validate_spec(spec: &TaskGroupSpec) -> Result<(), ParseError> {
    // Validate that there is at least one task
    if spec.tasks.is_empty() {
        return Err(ParseError::ValidationError(
            "Task group must contain at least one task".to_string(),
        ));
    }

    // Check for duplicate task names
    let mut task_names = HashSet::new();
    for task in &spec.tasks {
        if !task_names.insert(&task.name) {
            return Err(ParseError::DuplicateName {
                item_type: "task".to_string(),
                name: task.name.clone(),
            });
        }
    }

    // Validate each task
    for task in &spec.tasks {
        validate_task(task)?;
    }

    // Validate shared config if present
    if let Some(shared_config) = &spec.shared_config {
        validate_shared_config(shared_config)?;
    }

    // Validate workflows
    let mut workflow_names = HashSet::new();
    for workflow in &spec.workflows {
        if !workflow_names.insert(&workflow.name) {
            return Err(ParseError::DuplicateName {
                item_type: "workflow".to_string(),
                name: workflow.name.clone(),
            });
        }
        validate_workflow(workflow)?;
    }

    // Validate dependencies
    let mut dep_names = HashSet::new();
    for dep in &spec.dependencies {
        if !dep_names.insert(&dep.name) {
            return Err(ParseError::DuplicateName {
                item_type: "dependency".to_string(),
                name: dep.name.clone(),
            });
        }
        validate_dependency(dep)?;
    }

    // Check for circular dependencies
    detect_circular_dependencies(&spec.dependencies)?;

    // Validate hooks if present
    if let Some(hooks) = &spec.hooks {
        validate_hooks(hooks)?;
    }

    Ok(())
}

/// Validate a single task reference
fn validate_task(task: &TaskGroupTask) -> Result<(), ParseError> {
    // Validate task name
    if task.name.is_empty() {
        return Err(ParseError::MissingField("task.name".to_string()));
    }

    if !is_valid_task_name(&task.name) {
        return Err(ParseError::InvalidField {
            field: format!("tasks.{}.name", task.name),
            reason: "Task name must be lowercase with hyphens".to_string(),
        });
    }

    // Validate version constraint
    if task.version.is_empty() {
        return Err(ParseError::MissingField(format!(
            "tasks.{}.version",
            task.name
        )));
    }

    if !is_valid_version_constraint(&task.version) {
        return Err(ParseError::InvalidVersion {
            constraint: task.version.clone(),
            reason: "Must be valid semver constraint (e.g., '^1.0.0', '~2.1.0', '1.2.3')"
                .to_string(),
        });
    }

    Ok(())
}

/// Validate shared configuration
fn validate_shared_config(config: &SharedConfig) -> Result<(), ParseError> {
    // Validate shared inputs
    for (name, input) in &config.inputs {
        validate_input_name(name)?;
        validate_input_spec(name, input)?;
    }

    // Validate environment variables
    for (key, value) in &config.environment {
        if key.is_empty() {
            return Err(ParseError::InvalidField {
                field: "shared_config.environment".to_string(),
                reason: "Environment variable name cannot be empty".to_string(),
            });
        }
        if value.is_empty() {
            return Err(ParseError::InvalidField {
                field: format!("shared_config.environment.{}", key),
                reason: "Environment variable value cannot be empty".to_string(),
            });
        }
    }

    // Validate max_turns if present
    if let Some(max_turns) = config.max_turns {
        if max_turns == 0 {
            return Err(ParseError::InvalidField {
                field: "shared_config.max_turns".to_string(),
                reason: "max_turns must be greater than 0".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate a prebuilt workflow
fn validate_workflow(workflow: &PrebuiltWorkflow) -> Result<(), ParseError> {
    // Validate workflow name
    if workflow.name.is_empty() {
        return Err(ParseError::MissingField("workflow.name".to_string()));
    }

    if !is_valid_workflow_name(&workflow.name) {
        return Err(ParseError::InvalidField {
            field: format!("workflows.{}.name", workflow.name),
            reason: "Workflow name must be lowercase with hyphens".to_string(),
        });
    }

    // Validate tasks field is non-empty
    if workflow.tasks.is_null() {
        return Err(ParseError::MissingField(format!(
            "workflows.{}.tasks",
            workflow.name
        )));
    }

    // Validate workflow inputs
    for (name, input) in &workflow.inputs {
        validate_input_name(name)?;
        validate_input_spec(name, input)?;
    }

    // Validate workflow outputs
    for (name, output_path) in &workflow.outputs {
        if name.is_empty() {
            return Err(ParseError::InvalidField {
                field: format!("workflows.{}.outputs", workflow.name),
                reason: "Output name cannot be empty".to_string(),
            });
        }
        if output_path.is_empty() {
            return Err(ParseError::InvalidField {
                field: format!("workflows.{}.outputs.{}", workflow.name, name),
                reason: "Output path cannot be empty".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate a group dependency
fn validate_dependency(dep: &GroupDependency) -> Result<(), ParseError> {
    // Validate dependency name
    if dep.name.is_empty() {
        return Err(ParseError::MissingField("dependency.name".to_string()));
    }

    if !is_valid_group_name(&dep.name) {
        return Err(ParseError::InvalidField {
            field: format!("dependencies.{}.name", dep.name),
            reason: "Dependency name must be lowercase with hyphens".to_string(),
        });
    }

    // Validate version constraint
    if dep.version.is_empty() {
        return Err(ParseError::MissingField(format!(
            "dependencies.{}.version",
            dep.name
        )));
    }

    if !is_valid_version_constraint(&dep.version) {
        return Err(ParseError::InvalidVersion {
            constraint: dep.version.clone(),
            reason: "Must be valid semver constraint".to_string(),
        });
    }

    // Validate repository URL if present
    if let Some(repo) = &dep.repository {
        if repo.is_empty() {
            return Err(ParseError::InvalidField {
                field: format!("dependencies.{}.repository", dep.name),
                reason: "Repository URL cannot be empty if specified".to_string(),
            });
        }
    }

    Ok(())
}

/// Validate group hooks
fn validate_hooks(hooks: &GroupHooks) -> Result<(), ParseError> {
    // Validate post_install hooks
    for (i, hook) in hooks.post_install.iter().enumerate() {
        validate_hook(hook, &format!("hooks.post_install[{}]", i))?;
    }

    // Validate pre_use hooks
    for (i, hook) in hooks.pre_use.iter().enumerate() {
        validate_hook(hook, &format!("hooks.pre_use[{}]", i))?;
    }

    // Validate post_uninstall hooks
    for (i, hook) in hooks.post_uninstall.iter().enumerate() {
        validate_hook(hook, &format!("hooks.post_uninstall[{}]", i))?;
    }

    Ok(())
}

/// Validate a single hook
fn validate_hook(hook: &super::schema::Hook, path: &str) -> Result<(), ParseError> {
    use super::schema::Hook;

    match hook {
        Hook::Command { command, .. } => {
            if command.is_empty() {
                return Err(ParseError::InvalidField {
                    field: format!("{}.command", path),
                    reason: "Command cannot be empty".to_string(),
                });
            }
        }
        Hook::Validate { check, message } => {
            if check.is_empty() {
                return Err(ParseError::InvalidField {
                    field: format!("{}.check", path),
                    reason: "Check expression cannot be empty".to_string(),
                });
            }
            if message.is_empty() {
                return Err(ParseError::InvalidField {
                    field: format!("{}.message", path),
                    reason: "Validation message cannot be empty".to_string(),
                });
            }
        }
        Hook::Message { content, level } => {
            if content.is_empty() {
                return Err(ParseError::InvalidField {
                    field: format!("{}.content", path),
                    reason: "Message content cannot be empty".to_string(),
                });
            }
            let valid_levels = ["info", "warning", "error"];
            if !valid_levels.contains(&level.as_str()) {
                return Err(ParseError::InvalidField {
                    field: format!("{}.level", path),
                    reason: format!("Level must be one of: {}", valid_levels.join(", ")),
                });
            }
        }
    }

    Ok(())
}

/// Validate task compatibility across the group
fn validate_task_compatibility(spec: &TaskGroupSpec) -> Result<(), ParseError> {
    // Check for variable type conflicts in shared config
    if let Some(shared_config) = &spec.shared_config {
        validate_input_type_consistency(&shared_config.inputs)?;
    }

    // Validate workflow references to tasks in the group
    let task_names: HashSet<_> = spec.tasks.iter().map(|t| t.name.as_str()).collect();

    for workflow in &spec.workflows {
        validate_workflow_task_references(workflow, &task_names)?;
    }

    Ok(())
}

/// Validate input type consistency
fn validate_input_type_consistency(inputs: &HashMap<String, InputSpec>) -> Result<(), ParseError> {
    for (name, input) in inputs {
        // Validate type is one of the supported types
        let valid_types = ["string", "number", "boolean", "object", "array", "secret"];
        if !valid_types.contains(&input.param_type.as_str()) {
            return Err(ParseError::InvalidField {
                field: format!("inputs.{}.type", name),
                reason: format!("Type must be one of: {}", valid_types.join(", ")),
            });
        }
    }

    Ok(())
}

/// Validate workflow task references
fn validate_workflow_task_references(
    workflow: &PrebuiltWorkflow,
    available_tasks: &HashSet<&str>,
) -> Result<(), ParseError> {
    // Parse the tasks YAML value to check task references
    if let serde_yaml::Value::Mapping(tasks) = &workflow.tasks {
        for (task_id, task_spec) in tasks {
            if let serde_yaml::Value::String(_id) = task_id {
                // Check if the task spec contains an 'agent' field that references a task
                if let serde_yaml::Value::Mapping(spec) = task_spec {
                    if let Some(serde_yaml::Value::String(agent)) = spec.get("agent") {
                        // If the agent name matches a task in the group, that's valid
                        // This is a basic check - more sophisticated validation can be added later
                        if !available_tasks.contains(agent.as_str()) {
                            // This is not necessarily an error - the agent might be defined elsewhere
                            // For now, we'll just note it but not fail
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Detect circular dependencies in group dependencies
fn detect_circular_dependencies(dependencies: &[GroupDependency]) -> Result<(), ParseError> {
    // Build dependency graph
    let _dep_map: HashMap<&str, Vec<&str>> = dependencies
        .iter()
        .map(|d| (d.name.as_str(), Vec::new()))
        .collect();

    // TODO: Implement self-dependency and circular dependency detection
    // For now, we only prepare the structure for future enhancement
    // Full transitive dependency checking would require loading the dependency manifests

    Ok(())
}

/// Validate input parameter name
fn validate_input_name(name: &str) -> Result<(), ParseError> {
    if name.is_empty() {
        return Err(ParseError::ValidationError(
            "Input name cannot be empty".to_string(),
        ));
    }

    // Input names should be valid identifiers (alphanumeric + underscore + hyphen)
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ParseError::InvalidField {
            field: format!("inputs.{}", name),
            reason:
                "Input name must contain only alphanumeric characters, underscores, and hyphens"
                    .to_string(),
        });
    }

    Ok(())
}

/// Validate input specification
fn validate_input_spec(name: &str, spec: &InputSpec) -> Result<(), ParseError> {
    // Validate type is non-empty
    if spec.param_type.is_empty() {
        return Err(ParseError::MissingField(format!("inputs.{}.type", name)));
    }

    // Validate type is one of: string, number, boolean, object, array, secret
    let valid_types = ["string", "number", "boolean", "object", "array", "secret"];
    if !valid_types.contains(&spec.param_type.as_str()) {
        return Err(ParseError::InvalidField {
            field: format!("inputs.{}.type", name),
            reason: format!("Type must be one of: {}", valid_types.join(", ")),
        });
    }

    Ok(())
}

/// Check if a group name is valid (lowercase, hyphen-separated)
fn is_valid_group_name(name: &str) -> bool {
    is_valid_identifier(name)
}

/// Check if a task name is valid (lowercase, hyphen-separated)
fn is_valid_task_name(name: &str) -> bool {
    is_valid_identifier(name)
}

/// Check if a workflow name is valid (lowercase, hyphen-separated)
fn is_valid_workflow_name(name: &str) -> bool {
    is_valid_identifier(name)
}

/// Check if an identifier is valid (lowercase, hyphen-separated)
fn is_valid_identifier(name: &str) -> bool {
    // Identifier must:
    // - Be non-empty
    // - Contain only lowercase letters, numbers, and hyphens
    // - Not start or end with a hyphen
    // - Not contain consecutive hyphens

    if name.is_empty() {
        return false;
    }

    if name.starts_with('-') || name.ends_with('-') {
        return false;
    }

    if name.contains("--") {
        return false;
    }

    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Check if a version is valid semver
fn is_valid_semver(version: &str) -> bool {
    // Basic semver validation: MAJOR.MINOR.PATCH
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Check if a version constraint is valid
fn is_valid_version_constraint(constraint: &str) -> bool {
    // Valid constraints:
    // - Exact: "1.2.3"
    // - Caret: "^1.2.3"
    // - Tilde: "~1.2.3"
    // - Greater than: ">1.2.3", ">=1.2.3"
    // - Less than: "<1.2.3", "<=1.2.3"
    // - Range: ">=1.2.3 <2.0.0"
    // - Wildcard: "1.x", "1.2.x", "*"
    // - Latest: "latest"

    if constraint == "latest" || constraint == "*" {
        return true;
    }

    // Strip leading operator
    let version = constraint
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim();

    // Check for range (contains space)
    if version.contains(' ') {
        return version.split_whitespace().all(is_valid_version_constraint);
    }

    // Check for wildcard
    if version.contains('x') || version.contains('X') {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() > 3 {
            return false;
        }
        return parts
            .iter()
            .all(|p| *p == "x" || *p == "X" || p.parse::<u32>().is_ok());
    }

    // Validate as semver
    is_valid_semver(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_task_group() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "integration-suite"
  version: "2.0.0"
  description: "Suite of integration tasks"
  author: "Example Corp"
  license: "MIT"
  tags: ["integration", "testing"]
spec:
  tasks:
    - name: "google-drive-upload"
      version: "^1.0.0"
      required: true
      description: "Upload files to Google Drive"
    - name: "slack-notify"
      version: "~2.1.0"
      required: false
  shared_config:
    inputs:
      api_key:
        type: string
        required: true
        description: "API key for authentication"
    permissions:
      mode: "acceptEdits"
    environment:
      ENV: "production"
    max_turns: 10
"#;

        let group = parse_task_group(yaml).unwrap();
        assert_eq!(group.metadata.name, "integration-suite");
        assert_eq!(group.metadata.version, "2.0.0");
        assert_eq!(group.spec.tasks.len(), 2);
        assert!(group.spec.shared_config.is_some());
    }

    #[test]
    fn test_parse_missing_name() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("name"));
    }

    #[test]
    fn test_parse_invalid_group_name() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "InvalidGroupName"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lowercase"));
    }

    #[test]
    fn test_parse_empty_tasks() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "empty-group"
  version: "1.0.0"
spec:
  tasks: []
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one task"));
    }

    #[test]
    fn test_parse_duplicate_task_names() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "duplicate-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
    - name: "test-task"
      version: "2.0.0"
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier("my-task"));
        assert!(is_valid_identifier("task123"));
        assert!(is_valid_identifier("google-drive-upload"));

        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("-task"));
        assert!(!is_valid_identifier("task-"));
        assert!(!is_valid_identifier("task--name"));
        assert!(!is_valid_identifier("MyTask"));
        assert!(!is_valid_identifier("my_task"));
    }

    #[test]
    fn test_is_valid_semver() {
        assert!(is_valid_semver("1.0.0"));
        assert!(is_valid_semver("2.1.3"));
        assert!(is_valid_semver("0.0.1"));

        assert!(!is_valid_semver("1.0"));
        assert!(!is_valid_semver("1"));
        assert!(!is_valid_semver("v1.0.0"));
        assert!(!is_valid_semver("1.0.0-alpha"));
    }

    #[test]
    fn test_is_valid_version_constraint() {
        // Exact versions
        assert!(is_valid_version_constraint("1.2.3"));

        // Caret and tilde
        assert!(is_valid_version_constraint("^1.2.3"));
        assert!(is_valid_version_constraint("~1.2.3"));

        // Comparisons
        assert!(is_valid_version_constraint(">1.2.3"));
        assert!(is_valid_version_constraint(">=1.2.3"));
        assert!(is_valid_version_constraint("<2.0.0"));
        assert!(is_valid_version_constraint("<=2.0.0"));

        // Special values
        assert!(is_valid_version_constraint("latest"));
        assert!(is_valid_version_constraint("*"));

        // Wildcards
        assert!(is_valid_version_constraint("1.x"));
        assert!(is_valid_version_constraint("1.2.x"));

        // Invalid
        assert!(!is_valid_version_constraint(""));
        assert!(!is_valid_version_constraint("invalid"));
        assert!(!is_valid_version_constraint("1.2.3.4"));
    }

    #[test]
    fn test_validate_shared_config() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "test-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
  shared_config:
    inputs:
      api_key:
        type: string
        required: true
    max_turns: 0
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("max_turns must be greater than 0"));
    }

    #[test]
    fn test_validate_workflow() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "test-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
  workflows:
    - name: "test-workflow"
      description: "Test workflow"
      tasks:
        task1:
          description: "Test task"
          agent: "test-agent"
      inputs:
        param1:
          type: string
          required: true
      outputs:
        result: "./output.txt"
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_hooks() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "test-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
  hooks:
    post_install:
      - type: command
        command: "echo 'Installed'"
      - type: validate
        check: "env.API_KEY"
        message: "API_KEY is required"
      - type: message
        content: "Installation complete"
        level: info
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_hook_level() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "test-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
  hooks:
    post_install:
      - type: message
        content: "Test"
        level: invalid
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Level must be"));
    }

    #[test]
    fn test_validate_dependency() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "test-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
  dependencies:
    - name: "other-group"
      version: "^2.0.0"
      optional: false
    - name: "optional-group"
      version: "~1.5.0"
      optional: true
      repository: "https://example.com/repo"
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_duplicate_workflow_names() {
        let yaml = r#"
apiVersion: "taskgroup/v1"
kind: "TaskGroup"
metadata:
  name: "test-group"
  version: "1.0.0"
spec:
  tasks:
    - name: "test-task"
      version: "1.0.0"
  workflows:
    - name: "workflow"
      tasks: {}
    - name: "workflow"
      tasks: {}
"#;

        let result = parse_task_group(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }
}
