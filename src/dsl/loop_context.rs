//! Loop Context and Variable Substitution
//!
//! This module provides context management for loop iterations, including variable
//! substitution in task descriptions, conditions, and other task parameters.

use crate::dsl::schema::{ConditionSpec, DoneCriterion, TaskSpec};
use serde_json::Value;
use std::collections::HashMap;

/// Loop iteration context with variable substitution
#[derive(Debug, Clone)]
pub struct LoopContext {
    /// Current iteration number (0-based)
    pub iteration: usize,
    /// Variables available in this loop context
    pub variables: HashMap<String, Value>,
    /// Parent context for nested loops
    pub parent_context: Option<Box<LoopContext>>,
}

impl LoopContext {
    /// Create a new loop context
    pub fn new(iteration: usize) -> Self {
        Self {
            iteration,
            variables: HashMap::new(),
            parent_context: None,
        }
    }

    /// Create a loop context with a parent (for nested loops)
    pub fn with_parent(iteration: usize, parent: LoopContext) -> Self {
        Self {
            iteration,
            variables: HashMap::new(),
            parent_context: Some(Box::new(parent)),
        }
    }

    /// Set a variable in this context
    pub fn set_variable(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Get a variable from this context or parent contexts
    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name).or_else(|| {
            self.parent_context
                .as_ref()
                .and_then(|p| p.get_variable(name))
        })
    }

    /// Substitute {{variable}} and ${task.variable} placeholders in text
    ///
    /// Supports:
    /// - {{variable_name}} - Replace with variable value
    /// - {{iteration}} - Replace with current iteration number
    /// - ${task.variable_name} - Replace with variable value (same as {{variable_name}})
    /// - ${task.iteration} - Replace with current iteration number
    pub fn substitute_variables(&self, text: &str) -> String {
        let mut result = text.to_string();

        // Replace {{iteration}} with current iteration
        result = result.replace("{{iteration}}", &self.iteration.to_string());

        // Also support ${task.iteration} syntax
        result = result.replace("${task.iteration}", &self.iteration.to_string());

        // Replace {{variable}} with variable values
        for (key, value) in &self.variables {
            let placeholder_curly = format!("{{{{{}}}}}", key);
            let placeholder_dollar = format!("${{task.{}}}", key);
            let value_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            result = result.replace(&placeholder_curly, &value_str);
            result = result.replace(&placeholder_dollar, &value_str);
        }

        // Check parent context for nested loops
        if let Some(parent) = &self.parent_context {
            result = parent.substitute_variables(&result);
        }

        result
    }
}

/// Substitute variables in a task spec before execution
///
/// This creates a modified clone of the task with all variable placeholders replaced.
pub fn substitute_task_variables(task: &TaskSpec, context: &LoopContext) -> TaskSpec {
    let mut modified_task = task.clone();

    // Substitute in description
    modified_task.description = context.substitute_variables(&modified_task.description);

    // Substitute in output path
    if let Some(output) = &modified_task.output {
        modified_task.output = Some(context.substitute_variables(output));
    }

    // Substitute in conditions
    if let Some(condition) = &modified_task.condition {
        modified_task.condition = Some(substitute_condition_variables(condition, context));
    }

    // Substitute in definition_of_done criteria
    if let Some(dod) = &mut modified_task.definition_of_done {
        for criterion in &mut dod.criteria {
            substitute_criterion_variables(criterion, context);
        }
    }

    // Substitute in script content
    if let Some(script) = &mut modified_task.script {
        if let Some(content) = &script.content {
            script.content = Some(context.substitute_variables(content));
        }
        if let Some(file) = &script.file {
            script.file = Some(context.substitute_variables(file));
        }
        if let Some(working_dir) = &script.working_dir {
            script.working_dir = Some(context.substitute_variables(working_dir));
        }
        // Substitute in environment variables
        script.env = script
            .env
            .iter()
            .map(|(k, v)| (k.clone(), context.substitute_variables(v)))
            .collect();
    }

    // Substitute in task inputs
    modified_task.inputs = modified_task
        .inputs
        .iter()
        .map(|(k, v)| (k.clone(), substitute_json_value(v, context)))
        .collect();

    // Recursively substitute in subtasks
    modified_task.subtasks = modified_task
        .subtasks
        .iter()
        .map(|subtask_map| {
            subtask_map
                .iter()
                .map(|(name, spec)| (name.clone(), substitute_task_variables(spec, context)))
                .collect()
        })
        .collect();

    modified_task
}

/// Substitute variables in a condition specification
fn substitute_condition_variables(
    condition: &ConditionSpec,
    context: &LoopContext,
) -> ConditionSpec {
    match condition {
        ConditionSpec::Single(cond) => {
            ConditionSpec::Single(substitute_single_condition(cond, context))
        }
        ConditionSpec::And { and } => ConditionSpec::And {
            and: and
                .iter()
                .map(|c| substitute_condition_variables(c, context))
                .collect(),
        },
        ConditionSpec::Or { or } => ConditionSpec::Or {
            or: or
                .iter()
                .map(|c| substitute_condition_variables(c, context))
                .collect(),
        },
        ConditionSpec::Not { not } => ConditionSpec::Not {
            not: Box::new(substitute_condition_variables(not, context)),
        },
    }
}

/// Substitute variables in a single condition
fn substitute_single_condition(
    condition: &crate::dsl::schema::Condition,
    context: &LoopContext,
) -> crate::dsl::schema::Condition {
    use crate::dsl::schema::Condition;

    match condition {
        Condition::TaskStatus { task, status } => Condition::TaskStatus {
            task: context.substitute_variables(task),
            status: status.clone(),
        },
        Condition::StateEquals { key, value } => Condition::StateEquals {
            key: context.substitute_variables(key),
            value: substitute_json_value(value, context),
        },
        Condition::StateExists { key } => Condition::StateExists {
            key: context.substitute_variables(key),
        },
        Condition::Always => Condition::Always,
        Condition::Never => Condition::Never,
    }
}

/// Substitute variables in a JSON value
fn substitute_json_value(value: &Value, context: &LoopContext) -> Value {
    match value {
        Value::String(s) => Value::String(context.substitute_variables(s)),
        Value::Array(arr) => Value::Array(
            arr.iter()
                .map(|v| substitute_json_value(v, context))
                .collect(),
        ),
        Value::Object(obj) => Value::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), substitute_json_value(v, context)))
                .collect(),
        ),
        _ => value.clone(),
    }
}

/// Substitute variables in a definition of done criterion
fn substitute_criterion_variables(criterion: &mut DoneCriterion, context: &LoopContext) {
    match criterion {
        DoneCriterion::FileExists { path, description } => {
            *path = context.substitute_variables(path);
            *description = context.substitute_variables(description);
        }
        DoneCriterion::FileContains {
            path,
            pattern,
            description,
        } => {
            *path = context.substitute_variables(path);
            *pattern = context.substitute_variables(pattern);
            *description = context.substitute_variables(description);
        }
        DoneCriterion::FileNotContains {
            path,
            pattern,
            description,
        } => {
            *path = context.substitute_variables(path);
            *pattern = context.substitute_variables(pattern);
            *description = context.substitute_variables(description);
        }
        DoneCriterion::CommandSucceeds {
            command,
            args,
            description,
            working_dir,
        } => {
            *command = context.substitute_variables(command);
            *args = args
                .iter()
                .map(|arg| context.substitute_variables(arg))
                .collect();
            *description = context.substitute_variables(description);
            if let Some(dir) = working_dir {
                *working_dir = Some(context.substitute_variables(dir));
            }
        }
        DoneCriterion::OutputMatches {
            pattern,
            description,
            ..
        } => {
            *pattern = context.substitute_variables(pattern);
            *description = context.substitute_variables(description);
        }
        DoneCriterion::DirectoryExists { path, description } => {
            *path = context.substitute_variables(path);
            *description = context.substitute_variables(description);
        }
        DoneCriterion::TestsPassed {
            command,
            args,
            description,
        } => {
            *command = context.substitute_variables(command);
            *args = args
                .iter()
                .map(|arg| context.substitute_variables(arg))
                .collect();
            *description = context.substitute_variables(description);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop_context_variable_substitution() {
        let mut context = LoopContext::new(5);
        context.set_variable("file".to_string(), Value::String("data.json".to_string()));

        let text = "Processing file {{file}} on iteration {{iteration}}";
        let result = context.substitute_variables(text);

        assert_eq!(result, "Processing file data.json on iteration 5");
    }

    #[test]
    fn test_loop_context_nested_variables() {
        let mut parent = LoopContext::new(2);
        parent.set_variable("row".to_string(), Value::Number(10.into()));

        let mut child = LoopContext::with_parent(3, parent);
        child.set_variable("col".to_string(), Value::Number(20.into()));

        let text = "Row {{row}}, Col {{col}}, Iteration {{iteration}}";
        let result = child.substitute_variables(text);

        assert_eq!(result, "Row 10, Col 20, Iteration 3");
    }

    #[test]
    fn test_substitute_task_variables() {
        let mut context = LoopContext::new(0);
        context.set_variable("id".to_string(), Value::Number(42.into()));

        let task = TaskSpec {
            description: "Process item {{id}}".to_string(),
            output: Some("output_{{id}}.txt".to_string()),
            ..Default::default()
        };

        let substituted = substitute_task_variables(&task, &context);

        assert_eq!(substituted.description, "Process item 42");
        assert_eq!(substituted.output, Some("output_42.txt".to_string()));
    }

    #[test]
    fn test_variable_get_from_parent() {
        let mut parent = LoopContext::new(0);
        parent.set_variable(
            "parent_var".to_string(),
            Value::String("parent".to_string()),
        );

        let child = LoopContext::with_parent(1, parent);

        assert_eq!(
            child.get_variable("parent_var"),
            Some(&Value::String("parent".to_string()))
        );
    }

    #[test]
    fn test_dollar_sign_task_syntax() {
        let mut context = LoopContext::new(5);
        context.set_variable("feature".to_string(), Value::String("yaml_editor".to_string()));

        let text = "Testing feature: ${task.feature} on iteration ${task.iteration}";
        let result = context.substitute_variables(text);

        assert_eq!(result, "Testing feature: yaml_editor on iteration 5");
    }

    #[test]
    fn test_mixed_syntax() {
        let mut context = LoopContext::new(3);
        context.set_variable("name".to_string(), Value::String("test".to_string()));

        // Both syntaxes should work
        let text1 = "Name: {{name}}";
        let result1 = context.substitute_variables(text1);
        assert_eq!(result1, "Name: test");

        let text2 = "Name: ${task.name}";
        let result2 = context.substitute_variables(text2);
        assert_eq!(result2, "Name: test");

        let text3 = "Name: {{name}} or ${task.name}";
        let result3 = context.substitute_variables(text3);
        assert_eq!(result3, "Name: test or test");
    }
}
