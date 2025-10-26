//! Predefined Task Parser
//!
//! This module handles parsing of `.task.yaml` files into PredefinedTask structures.

use super::schema::PredefinedTask;
use thiserror::Error;

/// Errors that can occur during task parsing
#[derive(Debug, Error)]
pub enum ParseError {
    /// YAML deserialization error
    #[error("Failed to parse task YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidField { field: String, reason: String },

    /// Validation error
    #[error("Task validation failed: {0}")]
    ValidationError(String),
}

/// Parse a predefined task from YAML string
///
/// # Arguments
///
/// * `yaml_content` - YAML string containing the task definition
///
/// # Returns
///
/// Parsed `PredefinedTask` or error
///
/// # Example
///
/// ```
/// use periplon_sdk::dsl::predefined_tasks::parse_predefined_task;
///
/// let yaml = r#"
/// apiVersion: "task/v1"
/// kind: "PredefinedTask"
/// metadata:
///   name: "example-task"
///   version: "1.0.0"
///   description: "An example task"
/// spec:
///   agent_template:
///     description: "Do something"
///     tools: ["Read", "Write"]
///   inputs:
///     input1:
///       type: string
///       required: true
/// "#;
///
/// let task = parse_predefined_task(yaml).unwrap();
/// assert_eq!(task.metadata.name, "example-task");
/// ```
pub fn parse_predefined_task(yaml_content: &str) -> Result<PredefinedTask, ParseError> {
    // Parse YAML into PredefinedTask
    let task: PredefinedTask = serde_yaml::from_str(yaml_content)?;

    // Validate the parsed task
    validate_task(&task)?;

    Ok(task)
}

/// Validate a parsed predefined task
fn validate_task(task: &PredefinedTask) -> Result<(), ParseError> {
    // Validate metadata
    validate_metadata(&task.metadata)?;

    // Validate spec
    validate_spec(&task.spec)?;

    Ok(())
}

/// Validate task metadata
fn validate_metadata(metadata: &super::schema::PredefinedTaskMetadata) -> Result<(), ParseError> {
    // Validate name (must be non-empty, lowercase, hyphen-separated)
    if metadata.name.is_empty() {
        return Err(ParseError::MissingField("metadata.name".to_string()));
    }

    if !is_valid_task_name(&metadata.name) {
        return Err(ParseError::InvalidField {
            field: "metadata.name".to_string(),
            reason: "Task name must be lowercase with hyphens (e.g., 'my-task')".to_string(),
        });
    }

    // Validate version (must be valid semver for Phase 3+, for now just non-empty)
    if metadata.version.is_empty() {
        return Err(ParseError::MissingField("metadata.version".to_string()));
    }

    Ok(())
}

/// Validate task spec
fn validate_spec(spec: &super::schema::PredefinedTaskSpec) -> Result<(), ParseError> {
    // Validate agent template
    if spec.agent_template.description.is_empty() {
        return Err(ParseError::MissingField(
            "spec.agent_template.description".to_string(),
        ));
    }

    // Validate inputs
    for (name, input) in &spec.inputs {
        validate_input_name(name)?;
        validate_input_spec(name, input)?;
    }

    // Validate outputs
    for (name, output) in &spec.outputs {
        validate_output_name(name)?;
        validate_output_spec(name, output)?;
    }

    Ok(())
}

/// Validate input parameter name
fn validate_input_name(name: &str) -> Result<(), ParseError> {
    if name.is_empty() {
        return Err(ParseError::ValidationError(
            "Input name cannot be empty".to_string(),
        ));
    }

    // Input names should be valid identifiers (alphanumeric + underscore)
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
fn validate_input_spec(
    name: &str,
    spec: &super::schema::PredefinedTaskInputSpec,
) -> Result<(), ParseError> {
    // Validate type is non-empty
    if spec.base.param_type.is_empty() {
        return Err(ParseError::MissingField(format!("inputs.{}.type", name)));
    }

    // Validate type is one of: string, number, boolean, object, array, secret
    let valid_types = ["string", "number", "boolean", "object", "array", "secret"];
    if !valid_types.contains(&spec.base.param_type.as_str()) {
        return Err(ParseError::InvalidField {
            field: format!("inputs.{}.type", name),
            reason: format!("Type must be one of: {}", valid_types.join(", ")),
        });
    }

    // If validation rules are present, validate them
    if let Some(validation) = &spec.validation {
        validate_input_validation(name, &spec.base.param_type, validation)?;
    }

    Ok(())
}

/// Validate input validation rules
fn validate_input_validation(
    name: &str,
    input_type: &str,
    validation: &super::schema::InputValidation,
) -> Result<(), ParseError> {
    // Pattern validation only applies to strings
    if validation.pattern.is_some() && input_type != "string" {
        return Err(ParseError::InvalidField {
            field: format!("inputs.{}.validation.pattern", name),
            reason: "Pattern validation only applies to string types".to_string(),
        });
    }

    // Min/max validation only applies to numbers
    if (validation.min.is_some() || validation.max.is_some()) && input_type != "number" {
        return Err(ParseError::InvalidField {
            field: format!("inputs.{}.validation", name),
            reason: "Min/max validation only applies to number types".to_string(),
        });
    }

    // Length validation applies to strings and arrays
    if (validation.min_length.is_some() || validation.max_length.is_some())
        && input_type != "string"
        && input_type != "array"
    {
        return Err(ParseError::InvalidField {
            field: format!("inputs.{}.validation", name),
            reason: "Length validation only applies to string and array types".to_string(),
        });
    }

    Ok(())
}

/// Validate output parameter name
fn validate_output_name(name: &str) -> Result<(), ParseError> {
    if name.is_empty() {
        return Err(ParseError::ValidationError(
            "Output name cannot be empty".to_string(),
        ));
    }

    // Output names should be valid identifiers
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ParseError::InvalidField {
            field: format!("outputs.{}", name),
            reason:
                "Output name must contain only alphanumeric characters, underscores, and hyphens"
                    .to_string(),
        });
    }

    Ok(())
}

/// Validate output specification
fn validate_output_spec(
    _name: &str,
    _spec: &super::schema::PredefinedTaskOutputSpec,
) -> Result<(), ParseError> {
    // Output validation is minimal for Phase 1
    // More comprehensive validation will be added in later phases
    Ok(())
}

/// Check if a task name is valid (lowercase, hyphen-separated)
fn is_valid_task_name(name: &str) -> bool {
    // Task name must:
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_task() {
        let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "example-task"
  version: "1.0.0"
  description: "An example task"
  tags: ["example", "test"]
spec:
  agent_template:
    description: "Do something useful"
    model: "claude-sonnet-4-5"
    tools: ["Read", "Write"]
    permissions:
      mode: "acceptEdits"
  inputs:
    file_path:
      type: string
      required: true
      description: "Path to file"
  outputs:
    result:
      type: string
      description: "Result output"
      source:
        type: state
        key: "result_value"
"#;

        let task = parse_predefined_task(yaml).unwrap();
        assert_eq!(task.metadata.name, "example-task");
        assert_eq!(task.metadata.version, "1.0.0");
        assert_eq!(task.spec.inputs.len(), 1);
        assert_eq!(task.spec.outputs.len(), 1);
    }

    #[test]
    fn test_parse_missing_name() {
        let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  version: "1.0.0"
spec:
  agent_template:
    description: "Do something"
"#;

        let result = parse_predefined_task(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_task_name() {
        let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "InvalidTaskName"
  version: "1.0.0"
spec:
  agent_template:
    description: "Do something"
"#;

        let result = parse_predefined_task(yaml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lowercase"));
    }

    #[test]
    fn test_parse_missing_agent_description() {
        let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "example-task"
  version: "1.0.0"
spec:
  agent_template:
    tools: ["Read"]
"#;

        let result = parse_predefined_task(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_task_name() {
        assert!(is_valid_task_name("my-task"));
        assert!(is_valid_task_name("task123"));
        assert!(is_valid_task_name("google-drive-upload"));

        assert!(!is_valid_task_name(""));
        assert!(!is_valid_task_name("-task"));
        assert!(!is_valid_task_name("task-"));
        assert!(!is_valid_task_name("task--name"));
        assert!(!is_valid_task_name("MyTask"));
        assert!(!is_valid_task_name("my_task"));
    }

    #[test]
    fn test_validate_input_type() {
        let yaml = r#"
apiVersion: "task/v1"
kind: "PredefinedTask"
metadata:
  name: "test-task"
  version: "1.0.0"
spec:
  agent_template:
    description: "Test"
  inputs:
    param1:
      type: invalid_type
      required: true
"#;

        let result = parse_predefined_task(yaml);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Type must be one of"));
    }
}
