//! Predefined Task Resolver
//!
//! This module handles resolving task references and instantiating them as TaskSpec instances.

use super::loader::{LoadError, TaskLoader};
use super::schema::{PredefinedTask, TaskReference};
use crate::dsl::schema::{AgentSpec, TaskSpec};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during task resolution
#[derive(Debug, Error)]
pub enum ResolveError {
    /// Task load error
    #[error("Failed to load task: {0}")]
    LoadError(#[from] LoadError),

    /// Invalid task reference
    #[error("Invalid task reference: {0}")]
    InvalidReference(String),

    /// Missing required input
    #[error("Missing required input '{input}' for task '{task}'")]
    MissingRequiredInput { task: String, input: String },

    /// Invalid input type
    #[error(
        "Invalid type for input '{input}' in task '{task}': expected {expected}, got {actual}"
    )]
    InvalidInputType {
        task: String,
        input: String,
        expected: String,
        actual: String,
    },

    /// Input validation failed
    #[error("Input validation failed for '{input}' in task '{task}': {reason}")]
    ValidationFailed {
        task: String,
        input: String,
        reason: String,
    },

    /// Variable substitution error
    #[error("Variable substitution failed: {0}")]
    VariableError(String),
}

/// Task resolver for resolving and instantiating predefined tasks
pub struct TaskResolver {
    loader: TaskLoader,
}

impl TaskResolver {
    /// Create a new task resolver with default loader
    pub fn new() -> Self {
        TaskResolver {
            loader: TaskLoader::new(),
        }
    }

    /// Create a task resolver with a custom loader
    pub fn with_loader(loader: TaskLoader) -> Self {
        TaskResolver { loader }
    }

    /// Resolve a task reference and create an agent + task spec
    ///
    /// This is the main entry point for resolving `uses:` references in workflows.
    ///
    /// Returns a tuple of (agent_id, AgentSpec, TaskSpec) that can be added to the workflow.
    pub fn resolve(
        &mut self,
        task_ref_str: &str,
        task_inputs: &HashMap<String, serde_json::Value>,
        task_outputs: &HashMap<String, crate::dsl::schema::OutputSpec>,
    ) -> Result<(String, AgentSpec, TaskSpec), ResolveError> {
        // Parse the task reference
        let task_ref =
            TaskReference::parse(task_ref_str).map_err(ResolveError::InvalidReference)?;

        // Load the predefined task
        let predefined_task = self.loader.load(&task_ref)?;

        // Validate inputs
        validate_inputs(&predefined_task, task_inputs)?;

        // Create agent spec from template
        let agent_spec = create_agent_from_template(&predefined_task, task_inputs)?;

        // Create task spec
        let task_spec = create_task_spec(&predefined_task, task_inputs, task_outputs)?;

        // Generate unique agent ID
        let agent_id = format!(
            "{}_{}",
            predefined_task.metadata.name,
            task_ref.version.replace('.', "_")
        );

        Ok((agent_id, agent_spec, task_spec))
    }
}

impl Default for TaskResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to resolve a task reference
pub fn resolve_task_reference(
    task_ref: &str,
    inputs: &HashMap<String, serde_json::Value>,
    outputs: &HashMap<String, crate::dsl::schema::OutputSpec>,
) -> Result<(String, AgentSpec, TaskSpec), ResolveError> {
    let mut resolver = TaskResolver::new();
    resolver.resolve(task_ref, inputs, outputs)
}

/// Validate that provided inputs match the task's requirements
fn validate_inputs(
    task: &PredefinedTask,
    provided_inputs: &HashMap<String, serde_json::Value>,
) -> Result<(), ResolveError> {
    // Check all required inputs are provided
    for (input_name, input_spec) in &task.spec.inputs {
        if input_spec.base.required {
            // Check if input is provided or has a default
            if !provided_inputs.contains_key(input_name) && input_spec.base.default.is_none() {
                return Err(ResolveError::MissingRequiredInput {
                    task: task.metadata.name.clone(),
                    input: input_name.clone(),
                });
            }
        }

        // If input is provided, validate it
        if let Some(value) = provided_inputs.get(input_name) {
            validate_input_value(
                &task.metadata.name,
                input_name,
                value,
                &input_spec.base.param_type,
                &input_spec.validation,
            )?;
        }
    }

    Ok(())
}

/// Validate a single input value
fn validate_input_value(
    task_name: &str,
    input_name: &str,
    value: &serde_json::Value,
    expected_type: &str,
    validation: &Option<super::schema::InputValidation>,
) -> Result<(), ResolveError> {
    // Type validation
    let actual_type = match value {
        serde_json::Value::String(_) => "string",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
        serde_json::Value::Null => "null",
    };

    if expected_type != actual_type && expected_type != "secret" {
        return Err(ResolveError::InvalidInputType {
            task: task_name.to_string(),
            input: input_name.to_string(),
            expected: expected_type.to_string(),
            actual: actual_type.to_string(),
        });
    }

    // Additional validation rules
    if let Some(validation) = validation {
        // String validation
        if expected_type == "string" {
            if let serde_json::Value::String(s) = value {
                // Pattern validation
                if let Some(pattern) = &validation.pattern {
                    let re =
                        regex::Regex::new(pattern).map_err(|_| ResolveError::ValidationFailed {
                            task: task_name.to_string(),
                            input: input_name.to_string(),
                            reason: format!("Invalid regex pattern: {}", pattern),
                        })?;

                    if !re.is_match(s) {
                        return Err(ResolveError::ValidationFailed {
                            task: task_name.to_string(),
                            input: input_name.to_string(),
                            reason: format!("Value does not match pattern: {}", pattern),
                        });
                    }
                }

                // Length validation
                if let Some(min_len) = validation.min_length {
                    if s.len() < min_len {
                        return Err(ResolveError::ValidationFailed {
                            task: task_name.to_string(),
                            input: input_name.to_string(),
                            reason: format!(
                                "String length {} is less than minimum {}",
                                s.len(),
                                min_len
                            ),
                        });
                    }
                }

                if let Some(max_len) = validation.max_length {
                    if s.len() > max_len {
                        return Err(ResolveError::ValidationFailed {
                            task: task_name.to_string(),
                            input: input_name.to_string(),
                            reason: format!(
                                "String length {} exceeds maximum {}",
                                s.len(),
                                max_len
                            ),
                        });
                    }
                }
            }
        }

        // Number validation
        if expected_type == "number" {
            if let serde_json::Value::Number(n) = value {
                if let Some(n_f64) = n.as_f64() {
                    if let Some(min) = validation.min {
                        if n_f64 < min {
                            return Err(ResolveError::ValidationFailed {
                                task: task_name.to_string(),
                                input: input_name.to_string(),
                                reason: format!("Value {} is less than minimum {}", n_f64, min),
                            });
                        }
                    }

                    if let Some(max) = validation.max {
                        if n_f64 > max {
                            return Err(ResolveError::ValidationFailed {
                                task: task_name.to_string(),
                                input: input_name.to_string(),
                                reason: format!("Value {} exceeds maximum {}", n_f64, max),
                            });
                        }
                    }
                }
            }
        }

        // Enum validation (allowed_values)
        if !validation.allowed_values.is_empty() && !validation.allowed_values.contains(value) {
            return Err(ResolveError::ValidationFailed {
                task: task_name.to_string(),
                input: input_name.to_string(),
                reason: format!(
                    "Value not in allowed values: {:?}",
                    validation.allowed_values
                ),
            });
        }
    }

    Ok(())
}

/// Create an AgentSpec from a predefined task template
fn create_agent_from_template(
    task: &PredefinedTask,
    inputs: &HashMap<String, serde_json::Value>,
) -> Result<AgentSpec, ResolveError> {
    let template = &task.spec.agent_template;

    // Substitute variables in description
    let description = substitute_template_variables(&template.description, task, inputs)?;

    // Substitute variables in system prompt if present
    let system_prompt = if let Some(prompt) = &template.system_prompt {
        Some(substitute_template_variables(prompt, task, inputs)?)
    } else {
        None
    };

    Ok(AgentSpec {
        description,
        model: template.model.clone(),
        system_prompt,
        cwd: None,
        create_cwd: None,
        inputs: HashMap::new(),
        outputs: HashMap::new(),
        tools: template.tools.clone(),
        permissions: template.permissions.clone(),
        max_turns: template.max_turns,
    })
}

/// Create a TaskSpec from a predefined task
fn create_task_spec(
    task: &PredefinedTask,
    inputs: &HashMap<String, serde_json::Value>,
    outputs: &HashMap<String, crate::dsl::schema::OutputSpec>,
) -> Result<TaskSpec, ResolveError> {
    // Merge provided inputs with defaults
    let mut final_inputs = HashMap::new();

    for (input_name, input_spec) in &task.spec.inputs {
        if let Some(value) = inputs.get(input_name) {
            final_inputs.insert(input_name.clone(), value.clone());
        } else if let Some(default) = &input_spec.base.default {
            final_inputs.insert(input_name.clone(), default.clone());
        }
    }

    // Use provided outputs or task's default outputs
    let final_outputs = if outputs.is_empty() {
        task.spec
            .outputs
            .iter()
            .map(|(k, v)| (k.clone(), v.base.clone()))
            .collect()
    } else {
        outputs.clone()
    };

    Ok(TaskSpec {
        description: substitute_template_variables(
            &task.spec.agent_template.description,
            task,
            inputs,
        )?,
        agent: None, // Will be set by executor
        inputs: final_inputs,
        outputs: final_outputs,
        ..Default::default()
    })
}

/// Substitute variables in a template string
fn substitute_template_variables(
    template: &str,
    task: &PredefinedTask,
    inputs: &HashMap<String, serde_json::Value>,
) -> Result<String, ResolveError> {
    let mut result = template.to_string();

    // Simple variable substitution for ${input.name}
    for (input_name, input_value) in inputs {
        let placeholder = format!("${{input.{}}}", input_name);
        let value_str = match input_value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            other => serde_json::to_string(other)
                .map_err(|e| ResolveError::VariableError(e.to_string()))?,
        };
        result = result.replace(&placeholder, &value_str);
    }

    // Also check for any unresolved ${input.X} variables and use defaults
    for (input_name, input_spec) in &task.spec.inputs {
        let placeholder = format!("${{input.{}}}", input_name);
        if result.contains(&placeholder) {
            if let Some(default) = &input_spec.base.default {
                let value_str = match default {
                    serde_json::Value::String(s) => s.clone(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    other => serde_json::to_string(other)
                        .map_err(|e| ResolveError::VariableError(e.to_string()))?,
                };
                result = result.replace(&placeholder, &value_str);
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::predefined_tasks::schema::*;
    use crate::dsl::schema::{InputSpec, PermissionsSpec};

    fn create_test_task() -> PredefinedTask {
        PredefinedTask {
            api_version: TaskApiVersion::V1,
            kind: TaskKind::PredefinedTask,
            metadata: PredefinedTaskMetadata {
                name: "test-task".to_string(),
                version: "1.0.0".to_string(),
                author: None,
                description: Some("Test task".to_string()),
                license: None,
                repository: None,
                tags: vec![],
            },
            spec: PredefinedTaskSpec {
                agent_template: AgentTemplate {
                    description: "Process ${input.file_path}".to_string(),
                    model: Some("claude-sonnet-4-5".to_string()),
                    system_prompt: None,
                    tools: vec!["Read".to_string()],
                    permissions: PermissionsSpec::default(),
                    max_turns: None,
                },
                inputs: {
                    let mut inputs = HashMap::new();
                    inputs.insert(
                        "file_path".to_string(),
                        PredefinedTaskInputSpec {
                            base: InputSpec {
                                param_type: "string".to_string(),
                                required: true,
                                default: None,
                                description: None,
                            },
                            validation: None,
                            source: None,
                        },
                    );
                    inputs
                },
                outputs: HashMap::new(),
                dependencies: vec![],
                examples: vec![],
            },
        }
    }

    #[test]
    fn test_validate_inputs_success() {
        let task = create_test_task();
        let mut inputs = HashMap::new();
        inputs.insert("file_path".to_string(), serde_json::json!("test.txt"));

        let result = validate_inputs(&task, &inputs);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_inputs_missing_required() {
        let task = create_test_task();
        let inputs = HashMap::new();

        let result = validate_inputs(&task, &inputs);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::MissingRequiredInput { .. }
        ));
    }

    #[test]
    fn test_validate_input_type_mismatch() {
        let task = create_test_task();
        let mut inputs = HashMap::new();
        inputs.insert("file_path".to_string(), serde_json::json!(123));

        let result = validate_inputs(&task, &inputs);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ResolveError::InvalidInputType { .. }
        ));
    }

    #[test]
    fn test_substitute_template_variables() {
        let task = create_test_task();
        let mut inputs = HashMap::new();
        inputs.insert("file_path".to_string(), serde_json::json!("report.pdf"));

        let result = substitute_template_variables("Process ${input.file_path}", &task, &inputs);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Process report.pdf");
    }

    #[test]
    fn test_create_agent_from_template() {
        let task = create_test_task();
        let mut inputs = HashMap::new();
        inputs.insert("file_path".to_string(), serde_json::json!("test.txt"));

        let result = create_agent_from_template(&task, &inputs);
        assert!(result.is_ok());

        let agent = result.unwrap();
        assert_eq!(agent.description, "Process test.txt");
        assert_eq!(agent.model, Some("claude-sonnet-4-5".to_string()));
        assert_eq!(agent.tools.len(), 1);
    }
}
