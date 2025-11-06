//! Semantic Validator for DSL Workflows
//!
//! This module provides validation functionality to ensure workflows are semantically correct,
//! including checking for circular dependencies, valid agent references, tool availability,
//! and variable reference validation.

use crate::dsl::schema::{CollectionSource, DSLWorkflow, LoopSpec, TaskSpec};
use crate::dsl::variables::extract_variable_references;
use crate::error::{Error, Result};
use std::collections::{HashMap, HashSet};

// Loop safety constants
const MAX_LOOP_ITERATIONS: usize = 10_000;
const MAX_COLLECTION_SIZE: usize = 100_000;
const MAX_PARALLEL_ITERATIONS: usize = 100;

/// Validation errors and warnings collected during workflow validation
#[derive(Debug, Clone)]
pub struct ValidationErrors {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl ValidationErrors {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    fn into_result(self) -> Result<()> {
        // Print warnings to stderr
        if self.has_warnings() {
            eprintln!("Workflow validation warnings:");
            for warning in &self.warnings {
                eprintln!("  ⚠️  {}", warning);
            }
        }

        // Errors cause failure
        if self.has_errors() {
            Err(Error::InvalidInput(format!(
                "Workflow validation failed:\n{}",
                self.errors.join("\n")
            )))
        } else {
            Ok(())
        }
    }
}

/// Validate a DSL workflow for semantic correctness
///
/// # Arguments
///
/// * `workflow` - The workflow to validate
///
/// # Returns
///
/// Result indicating success or validation errors
///
/// # Validation Checks
///
/// - Agent references are valid
/// - Task dependencies exist
/// - No circular dependencies
/// - Tool availability
/// - Permission modes are valid
pub fn validate_workflow(workflow: &DSLWorkflow) -> Result<()> {
    let mut errors = ValidationErrors::new();

    // Validate agent references in tasks
    validate_agent_references(workflow, &mut errors);

    // Validate task dependencies
    validate_task_dependencies(workflow, &mut errors);

    // Check for circular dependencies
    validate_no_circular_dependencies(workflow, &mut errors);

    // Validate tool references
    validate_tool_references(workflow, &mut errors);

    // Validate permission modes
    validate_permission_modes(workflow, &mut errors);

    // Validate workflow stages
    validate_workflow_stages(workflow, &mut errors);

    // Validate loop specifications
    validate_loop_specs(workflow, &mut errors);

    // Validate subflow references
    validate_subflow_references(workflow, &mut errors);

    // Validate variable references
    validate_variable_references(workflow, &mut errors);

    // Validate task group imports
    validate_imports(workflow, &mut errors);

    // Validate uses_workflow references
    validate_uses_workflow_references(workflow, &mut errors);

    // Validate notification configurations
    validate_notification_configurations(workflow, &mut errors);

    errors.into_result()
}

/// Validate that all agent references in tasks exist
fn validate_agent_references(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    for (task_name, task_spec) in &workflow.tasks {
        if let Some(agent_name) = &task_spec.agent {
            if !workflow.agents.contains_key(agent_name) {
                errors.add_error(format!(
                    "Task '{}' references non-existent agent '{}'",
                    task_name, agent_name
                ));
            }
        }

        // Validate subtasks recursively
        validate_subtask_agent_references(&task_spec.subtasks, workflow, errors);
    }
}

/// Validate agent references in subtasks recursively
fn validate_subtask_agent_references(
    subtasks: &[HashMap<String, TaskSpec>],
    workflow: &DSLWorkflow,
    errors: &mut ValidationErrors,
) {
    for subtask_map in subtasks {
        for (subtask_name, subtask_spec) in subtask_map {
            if let Some(agent_name) = &subtask_spec.agent {
                if !workflow.agents.contains_key(agent_name) {
                    errors.add_error(format!(
                        "Subtask '{}' references non-existent agent '{}'",
                        subtask_name, agent_name
                    ));
                }
            }
            // Recursively validate nested subtasks
            validate_subtask_agent_references(&subtask_spec.subtasks, workflow, errors);
        }
    }
}

/// Validate that all task dependencies exist
fn validate_task_dependencies(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    let task_names: HashSet<_> = workflow.tasks.keys().cloned().collect();

    for (task_name, task_spec) in &workflow.tasks {
        for dep in &task_spec.depends_on {
            if !task_names.contains(dep) {
                errors.add_error(format!(
                    "Task '{}' depends on non-existent task '{}'",
                    task_name, dep
                ));
            }
        }

        for parallel_task in &task_spec.parallel_with {
            if !task_names.contains(parallel_task) {
                errors.add_error(format!(
                    "Task '{}' has parallel_with reference to non-existent task '{}'",
                    task_name, parallel_task
                ));
            }
        }
    }
}

/// Validate that there are no circular dependencies in tasks
fn validate_no_circular_dependencies(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    let task_names: Vec<_> = workflow.tasks.keys().cloned().collect();

    for start_task in &task_names {
        if has_circular_dependency(start_task, workflow, &mut HashSet::new()) {
            errors.add_error(format!(
                "Circular dependency detected involving task '{}'",
                start_task
            ));
        }
    }
}

/// Check if a task has circular dependencies using depth-first search
fn has_circular_dependency(
    task_name: &str,
    workflow: &DSLWorkflow,
    visited: &mut HashSet<String>,
) -> bool {
    if visited.contains(task_name) {
        return true;
    }

    visited.insert(task_name.to_string());

    if let Some(task_spec) = workflow.tasks.get(task_name) {
        for dep in &task_spec.depends_on {
            if has_circular_dependency(dep, workflow, visited) {
                return true;
            }
        }
    }

    visited.remove(task_name);
    false
}

/// Validate that tools referenced by agents are valid
fn validate_tool_references(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    const VALID_TOOLS: &[&str] = &[
        "Read",
        "Write",
        "Edit",
        "Bash",
        "Grep",
        "Glob",
        "WebSearch",
        "WebFetch",
        "Task",
        "TodoWrite",
        "Skill",
        "SlashCommand",
    ];

    for (agent_name, agent_spec) in &workflow.agents {
        for tool in &agent_spec.tools {
            if !VALID_TOOLS.contains(&tool.as_str()) {
                errors.add_error(format!(
                    "Agent '{}' references invalid tool '{}'",
                    agent_name, tool
                ));
            }
        }
    }
}

/// Validate that permission modes are valid
fn validate_permission_modes(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    const VALID_MODES: &[&str] = &["default", "acceptEdits", "plan", "bypassPermissions"];

    for (agent_name, agent_spec) in &workflow.agents {
        let mode = &agent_spec.permissions.mode;
        if !VALID_MODES.contains(&mode.as_str()) {
            errors.add_error(format!(
                "Agent '{}' has invalid permission mode '{}'. Valid modes: {}",
                agent_name,
                mode,
                VALID_MODES.join(", ")
            ));
        }
    }
}

/// Validate workflow stages
fn validate_workflow_stages(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    for (workflow_name, workflow_spec) in &workflow.workflows {
        let stage_names: HashSet<_> = workflow_spec
            .steps
            .iter()
            .map(|s| s.stage.as_str())
            .collect();

        for stage in &workflow_spec.steps {
            // Validate stage dependencies exist
            for dep in &stage.depends_on {
                if !stage_names.contains(dep.as_str()) {
                    errors.add_error(format!(
                        "Workflow '{}', stage '{}' depends on non-existent stage '{}'",
                        workflow_name, stage.stage, dep
                    ));
                }
            }

            // Validate agents in stages exist
            for agent_name in &stage.agents {
                if !workflow.agents.contains_key(agent_name) {
                    errors.add_error(format!(
                        "Workflow '{}', stage '{}' references non-existent agent '{}'",
                        workflow_name, stage.stage, agent_name
                    ));
                }
            }
        }
    }
}

/// Validate loop specifications in tasks
fn validate_loop_specs(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    for (task_name, task_spec) in &workflow.tasks {
        if let Some(loop_spec) = &task_spec.loop_spec {
            validate_loop_spec(task_name, loop_spec, errors);
        }

        // Validate subtasks recursively
        validate_subtask_loop_specs(task_name, &task_spec.subtasks, errors);
    }
}

/// Validate loop specifications in subtasks recursively
fn validate_subtask_loop_specs(
    parent_task: &str,
    subtasks: &[HashMap<String, TaskSpec>],
    errors: &mut ValidationErrors,
) {
    for subtask_map in subtasks {
        for (subtask_name, subtask_spec) in subtask_map {
            if let Some(loop_spec) = &subtask_spec.loop_spec {
                validate_loop_spec(
                    &format!("{}.{}", parent_task, subtask_name),
                    loop_spec,
                    errors,
                );
            }
            // Recursively validate nested subtasks
            validate_subtask_loop_specs(subtask_name, &subtask_spec.subtasks, errors);
        }
    }
}

/// Validate a single loop specification
fn validate_loop_spec(task_name: &str, loop_spec: &LoopSpec, errors: &mut ValidationErrors) {
    match loop_spec {
        LoopSpec::ForEach {
            collection,
            max_parallel,
            ..
        } => {
            // Validate collection source
            validate_collection_source(task_name, collection, errors);

            // Validate parallel limits
            if let Some(max) = max_parallel {
                if *max > MAX_PARALLEL_ITERATIONS {
                    errors.add_error(format!(
                        "Task '{}': max_parallel ({}) exceeds safety limit ({})",
                        task_name, max, MAX_PARALLEL_ITERATIONS
                    ));
                }
                if *max == 0 {
                    errors.add_error(format!(
                        "Task '{}': max_parallel must be greater than 0",
                        task_name
                    ));
                }
            }
        }
        LoopSpec::While {
            max_iterations,
            delay_between_secs,
            ..
        }
        | LoopSpec::RepeatUntil {
            max_iterations,
            delay_between_secs,
            ..
        } => {
            // Validate iteration limits
            if *max_iterations > MAX_LOOP_ITERATIONS {
                errors.add_error(format!(
                    "Task '{}': max_iterations ({}) exceeds safety limit ({})",
                    task_name, max_iterations, MAX_LOOP_ITERATIONS
                ));
            }
            if *max_iterations == 0 {
                errors.add_error(format!(
                    "Task '{}': max_iterations must be greater than 0",
                    task_name
                ));
            }

            // Warn about zero delay (tight loop risk)
            if let Some(delay) = delay_between_secs {
                if *delay == 0 && *max_iterations > 100 {
                    errors.add_warning(format!(
                        "Task '{}': Zero delay with {} iterations may cause tight loop and high CPU usage. Consider adding delay_between_secs.",
                        task_name, max_iterations
                    ));
                }
            } else if *max_iterations > 100 {
                errors.add_warning(format!(
                    "Task '{}': No delay specified with {} iterations may cause tight loop. Consider adding delay_between_secs.",
                    task_name, max_iterations
                ));
            }
        }
        LoopSpec::Repeat {
            count,
            max_parallel,
            ..
        } => {
            // Validate count (now a direct usize value)
            if *count > MAX_LOOP_ITERATIONS {
                errors.add_error(format!(
                    "Task '{}': repeat count ({}) exceeds safety limit ({})",
                    task_name, count, MAX_LOOP_ITERATIONS
                ));
            }
            if *count == 0 {
                errors.add_error(format!(
                    "Task '{}': repeat count must be greater than 0",
                    task_name
                ));
            }

            // Validate parallel limits
            if let Some(max) = max_parallel {
                if *max > MAX_PARALLEL_ITERATIONS {
                    errors.add_error(format!(
                        "Task '{}': max_parallel ({}) exceeds safety limit ({})",
                        task_name, max, MAX_PARALLEL_ITERATIONS
                    ));
                }
                if *max == 0 {
                    errors.add_error(format!(
                        "Task '{}': max_parallel must be greater than 0",
                        task_name
                    ));
                }
            }
        }
    }
}

/// Validate a collection source
fn validate_collection_source(
    task_name: &str,
    collection: &CollectionSource,
    errors: &mut ValidationErrors,
) {
    match collection {
        CollectionSource::State { key } => {
            if key.is_empty() {
                errors.add_error(format!(
                    "Task '{}': collection state key cannot be empty",
                    task_name
                ));
            }
        }
        CollectionSource::File { path, .. } => {
            if path.is_empty() {
                errors.add_error(format!(
                    "Task '{}': collection file path cannot be empty",
                    task_name
                ));
            }
        }
        CollectionSource::Range { start, end, step } => {
            if start >= end {
                errors.add_error(format!(
                    "Task '{}': range start ({}) must be less than end ({})",
                    task_name, start, end
                ));
            }

            if let Some(s) = step {
                if *s == 0 {
                    errors.add_error(format!("Task '{}': range step cannot be zero", task_name));
                }
                if *s < 0 {
                    errors.add_error(format!("Task '{}': range step must be positive", task_name));
                }
            }

            // Check if range would exceed max collection size
            let range_size = if let Some(s) = step {
                ((end - start) / s) as usize
            } else {
                (end - start) as usize
            };

            if range_size > MAX_COLLECTION_SIZE {
                errors.add_error(format!(
                    "Task '{}': range would generate {} items, exceeding safety limit ({})",
                    task_name, range_size, MAX_COLLECTION_SIZE
                ));
            }
        }
        CollectionSource::Inline { items } => {
            if items.is_empty() {
                errors.add_error(format!(
                    "Task '{}': inline collection cannot be empty",
                    task_name
                ));
            }

            if items.len() > MAX_COLLECTION_SIZE {
                errors.add_error(format!(
                    "Task '{}': inline collection has {} items, exceeding safety limit ({})",
                    task_name,
                    items.len(),
                    MAX_COLLECTION_SIZE
                ));
            }
        }
        CollectionSource::Http { url, method, .. } => {
            if url.is_empty() {
                errors.add_error(format!(
                    "Task '{}': HTTP collection URL cannot be empty",
                    task_name
                ));
            }

            // Validate URL format
            if !url.starts_with("http://") && !url.starts_with("https://") {
                errors.add_error(format!(
                    "Task '{}': HTTP collection URL must start with http:// or https://",
                    task_name
                ));
            }

            // Validate HTTP method
            let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH"];
            if !valid_methods.contains(&method.to_uppercase().as_str()) {
                errors.add_error(format!(
                    "Task '{}': HTTP method '{}' is not supported (use GET, POST, PUT, DELETE, or PATCH)",
                    task_name, method
                ));
            }
        }
    }
}

/// Validate subflow references and usage
fn validate_subflow_references(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    // Validate tasks that reference subflows or predefined tasks
    for (task_name, task_spec) in &workflow.tasks {
        // Count how many execution types are specified
        let execution_types: Vec<&str> = vec![
            if task_spec.agent.is_some() {
                "agent"
            } else {
                ""
            },
            if task_spec.subflow.is_some() {
                "subflow"
            } else {
                ""
            },
            if task_spec.uses.is_some() { "uses" } else { "" },
            if task_spec.embed.is_some() {
                "embed"
            } else {
                ""
            },
            if task_spec.script.is_some() {
                "script"
            } else {
                ""
            },
            if task_spec.command.is_some() {
                "command"
            } else {
                ""
            },
            if task_spec.http.is_some() { "http" } else { "" },
            if task_spec.mcp_tool.is_some() {
                "mcp_tool"
            } else {
                ""
            },
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();

        // Check for mutual exclusivity
        if execution_types.len() > 1 {
            errors.add_error(format!(
                "Task '{}' specifies multiple execution types: {}. Only one can be specified.",
                task_name,
                execution_types.join(", ")
            ));
        }

        // Validate specific execution types
        if let Some(subflow_id) = &task_spec.subflow {
            // Validate subflow exists
            if !workflow.subflows.contains_key(subflow_id) {
                errors.add_error(format!(
                    "Task '{}' references non-existent subflow '{}'",
                    task_name, subflow_id
                ));
            } else {
                // Validate inputs match subflow requirements
                let subflow = &workflow.subflows[subflow_id];
                validate_subflow_inputs(task_name, task_spec, subflow, errors);
            }
        }

        // Validate predefined task references
        if let Some(uses_ref) = &task_spec.uses {
            validate_predefined_task_reference(task_name, uses_ref, errors);
        }

        // Validate embedded predefined tasks
        if let Some(embed_ref) = &task_spec.embed {
            validate_predefined_task_reference(task_name, embed_ref, errors);

            // Overrides only make sense for embedded tasks
            if task_spec.overrides.is_some() && task_spec.embed.is_none() {
                errors.add_warning(format!(
                    "Task '{}' specifies 'overrides' but not 'embed'. Overrides are only used with embedded tasks.",
                    task_name
                ));
            }
        }

        // Check if task has no execution type at all
        if execution_types.is_empty() {
            // Check for other valid execution patterns
            let has_execution_pattern = task_spec.has_execution_type()
                || !task_spec.subtasks.is_empty()
                || task_spec.loop_spec.is_some();

            if !has_execution_pattern {
                errors.add_error(format!(
                    "Task '{}' must specify an execution type: agent, subflow, uses, embed, script, command, http, mcp_tool, uses_workflow, or have subtasks/loop",
                    task_name
                ));
            }
        }
    }

    // Validate subflow definitions
    for (subflow_id, subflow_spec) in &workflow.subflows {
        // If subflow is inline (no source), it must have agents or tasks
        if subflow_spec.source.is_none()
            && subflow_spec.agents.is_empty()
            && subflow_spec.tasks.is_empty()
        {
            errors.add_error(format!(
                "Inline subflow '{}' must define at least one agent or task",
                subflow_id
            ));
        }

        // Validate agents in inline subflows
        for (agent_id, agent_spec) in &subflow_spec.agents {
            // Validate tools
            const VALID_TOOLS: &[&str] = &[
                "Read",
                "Write",
                "Edit",
                "Bash",
                "Grep",
                "Glob",
                "WebSearch",
                "WebFetch",
                "Task",
                "TodoWrite",
                "Skill",
                "SlashCommand",
            ];
            for tool in &agent_spec.tools {
                if !VALID_TOOLS.contains(&tool.as_str()) {
                    errors.add_error(format!(
                        "Subflow '{}' agent '{}' references invalid tool '{}'",
                        subflow_id, agent_id, tool
                    ));
                }
            }

            // Validate permission mode
            const VALID_MODES: &[&str] = &["default", "acceptEdits", "plan", "bypassPermissions"];
            if !VALID_MODES.contains(&agent_spec.permissions.mode.as_str()) {
                errors.add_error(format!(
                    "Subflow '{}' agent '{}' has invalid permission mode '{}'",
                    subflow_id, agent_id, agent_spec.permissions.mode
                ));
            }
        }

        // Validate tasks in inline subflows
        for (task_id, task_spec) in &subflow_spec.tasks {
            // Validate agent references within the subflow
            if let Some(agent_id) = &task_spec.agent {
                if !subflow_spec.agents.contains_key(agent_id) {
                    errors.add_error(format!(
                        "Subflow '{}' task '{}' references non-existent agent '{}'",
                        subflow_id, task_id, agent_id
                    ));
                }
            }
        }
    }
}

/// Validate a predefined task reference
fn validate_predefined_task_reference(
    task_name: &str,
    task_ref: &str,
    errors: &mut ValidationErrors,
) {
    // Parse the task reference to ensure it's in the correct format
    match crate::dsl::predefined_tasks::schema::TaskReference::parse(task_ref) {
        Ok(_) => {
            // Reference is valid format (name@version)
            // Note: We don't check if the task exists here, as that requires
            // loading from the filesystem. That will be done at execution time.
        }
        Err(e) => {
            errors.add_error(format!(
                "Task '{}' has invalid predefined task reference '{}': {}",
                task_name, task_ref, e
            ));
        }
    }
}

/// Validate that task inputs match subflow requirements
fn validate_subflow_inputs(
    task_name: &str,
    task_spec: &TaskSpec,
    subflow: &crate::dsl::schema::SubflowSpec,
    errors: &mut ValidationErrors,
) {
    // Check that all required inputs are provided
    for (input_name, input_spec) in &subflow.inputs {
        if input_spec.required && !task_spec.inputs.contains_key(input_name) {
            // Check if there's a default value
            if input_spec.default.is_none() {
                errors.add_error(format!(
                    "Task '{}' missing required input '{}' for subflow",
                    task_name, input_name
                ));
            }
        }
    }

    // Warn about unexpected inputs
    for input_name in task_spec.inputs.keys() {
        if !subflow.inputs.contains_key(input_name) {
            errors.add_warning(format!(
                "Task '{}' provides input '{}' which is not defined in the subflow",
                task_name, input_name
            ));
        }
    }
}

/// Validate variable references in the workflow
fn validate_variable_references(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    // Collect all defined variables by scope (using owned Strings)
    let mut defined_vars: HashSet<String> = HashSet::new();

    // Workflow-level variables
    for var_name in workflow.inputs.keys() {
        defined_vars.insert(format!("workflow.{}", var_name));
    }
    for var_name in workflow.outputs.keys() {
        defined_vars.insert(format!("workflow.{}", var_name));
    }

    // Agent-level variables
    for (agent_id, agent_spec) in &workflow.agents {
        for var_name in agent_spec.inputs.keys() {
            defined_vars.insert(format!("agent.{}", var_name));
        }
        for var_name in agent_spec.outputs.keys() {
            defined_vars.insert(format!("agent.{}", var_name));
        }

        // Validate variable references in agent descriptions and prompts
        validate_string_variable_refs(
            &agent_spec.description,
            &defined_vars,
            &format!("Agent '{}' description", agent_id),
            errors,
        );
        if let Some(prompt) = &agent_spec.system_prompt {
            validate_string_variable_refs(
                prompt,
                &defined_vars,
                &format!("Agent '{}' system_prompt", agent_id),
                errors,
            );
        }
    }

    // Task-level variables
    for (task_id, task_spec) in &workflow.tasks {
        // Clone defined_vars for this task to add task-specific variables
        let mut task_defined_vars = defined_vars.clone();

        for var_name in task_spec.outputs.keys() {
            task_defined_vars.insert(format!("task.{}", var_name));
        }

        // If task has a loop, add loop variables
        if let Some(loop_spec) = &task_spec.loop_spec {
            // loop_index is always available
            task_defined_vars.insert("task.loop_index".to_string());

            // Add iterator variable if specified
            let iterator = match loop_spec {
                LoopSpec::ForEach { iterator, .. } => Some(iterator.clone()),
                LoopSpec::Repeat { iterator, .. } => iterator.clone(),
                LoopSpec::While { .. } | LoopSpec::RepeatUntil { .. } => None,
            };
            if let Some(iter_name) = iterator {
                task_defined_vars.insert(format!("task.{}", iter_name));
            }
        }

        // Validate variable references in task descriptions
        validate_string_variable_refs(
            &task_spec.description,
            &task_defined_vars,
            &format!("Task '{}' description", task_id),
            errors,
        );

        // Validate variable references in task inputs (values might contain variable refs)
        for (input_name, input_value) in &task_spec.inputs {
            if let Some(string_value) = input_value.as_str() {
                validate_string_variable_refs(
                    string_value,
                    &defined_vars,
                    &format!("Task '{}' input '{}'", task_id, input_name),
                    errors,
                );
            }
        }
    }

    // Validate subflow variables
    for subflow_spec in workflow.subflows.values() {
        for var_name in subflow_spec.inputs.keys() {
            defined_vars.insert(format!("subflow.{}", var_name));
        }
        for var_name in subflow_spec.outputs.keys() {
            defined_vars.insert(format!("subflow.{}", var_name));
        }
    }
}

/// Validate variable references in a string
fn validate_string_variable_refs(
    text: &str,
    defined_vars: &HashSet<String>,
    context: &str,
    errors: &mut ValidationErrors,
) {
    let refs = extract_variable_references(text);

    for var_ref in refs {
        // Check qualified references (scope.var)
        if var_ref.contains('.') {
            if !defined_vars.contains(&var_ref) {
                // Check if it's a valid scope prefix
                let scope = var_ref.split('.').next().unwrap();
                match scope {
                    "workflow" | "agent" | "task" | "subflow" | "loop" => {
                        errors.add_warning(format!(
                            "{}: Variable '{}' is not defined (will be checked at runtime for loop variables)",
                            context, var_ref
                        ));
                    }
                    _ => {
                        errors.add_error(format!(
                            "{}: Invalid scope '{}' in variable reference '{}'",
                            context, scope, var_ref
                        ));
                    }
                }
            }
        } else {
            // Unqualified references - check if exists in any scope (workflow, agent, task, loop)
            let workflow_qualified = format!("workflow.{}", var_ref);
            let agent_qualified = format!("agent.{}", var_ref);
            let task_qualified = format!("task.{}", var_ref);
            let subflow_qualified = format!("subflow.{}", var_ref);
            let loop_qualified = format!("loop.{}", var_ref);

            if !defined_vars.contains(&workflow_qualified)
                && !defined_vars.contains(&agent_qualified)
                && !defined_vars.contains(&task_qualified)
                && !defined_vars.contains(&subflow_qualified)
                && !defined_vars.contains(&loop_qualified)
            {
                errors.add_warning(format!(
                    "{}: Unqualified variable '{}' may not be defined at runtime. \
                     Consider using qualified reference (e.g., workflow.{}, agent.{}, task.{}, or loop.{})",
                    context, var_ref, var_ref, var_ref, var_ref, var_ref
                ));
            }
        }
    }
}

/// Validate task group imports
///
/// Checks that:
/// - Namespace identifiers are valid (alphanumeric, dash, underscore, no leading digit)
/// - Group references follow the format "name@version"
/// - No duplicate namespaces (already enforced by HashMap)
fn validate_imports(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    use crate::dsl::schema::WorkflowImport;

    for (namespace, group_ref) in &workflow.imports {
        // Validate namespace format
        if !WorkflowImport::validate_namespace(namespace) {
            errors.add_error(format!(
                "Invalid namespace identifier '{}': must be alphanumeric with dash/underscore, \
                 and not start with a digit",
                namespace
            ));
        }

        // Validate group reference format
        if WorkflowImport::parse_group_reference(group_ref).is_none() {
            errors.add_error(format!(
                "Invalid group reference '{}' in namespace '{}': must be in format 'name@version'",
                group_ref, namespace
            ));
        }
    }
}

/// Validate uses_workflow references in tasks
///
/// Checks that:
/// - uses_workflow is mutually exclusive with other execution types
/// - Format is "namespace:workflow_name"
/// - Namespace exists in workflow imports
fn validate_uses_workflow_references(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    use crate::dsl::schema::{TaskSpec, WorkflowImport};

    for (task_id, task) in &workflow.tasks {
        if let Some(ref workflow_ref) = task.uses_workflow {
            // Check mutual exclusivity with other execution types
            if task.execution_type_count() > 1 {
                errors.add_error(format!(
                    "Task '{}': uses_workflow is mutually exclusive with other execution types (agent, subflow, uses, embed, script, command, http, mcp_tool)",
                    task_id
                ));
            }

            // Validate format: "namespace:workflow_name"
            if let Some((namespace, _workflow_name)) =
                TaskSpec::parse_workflow_reference(workflow_ref)
            {
                // Check namespace exists in imports
                if !workflow.imports.contains_key(namespace) {
                    errors.add_error(format!(
                        "Task '{}': namespace '{}' in uses_workflow '{}' not found in imports. \
                         Available namespaces: {:?}",
                        task_id,
                        namespace,
                        workflow_ref,
                        workflow.imports.keys().collect::<Vec<_>>()
                    ));
                }

                // Validate namespace format (should match import validation)
                if !WorkflowImport::validate_namespace(namespace) {
                    errors.add_error(format!(
                        "Task '{}': invalid namespace '{}' in uses_workflow '{}'. \
                         Namespace must be alphanumeric with dash/underscore and not start with a digit",
                        task_id, namespace, workflow_ref
                    ));
                }
            } else {
                errors.add_error(format!(
                    "Task '{}': invalid uses_workflow reference '{}'. \
                     Expected format: 'namespace:workflow_name'",
                    task_id, workflow_ref
                ));
            }
        }
    }

    // Also check nested subtasks
    for (parent_id, parent_task) in &workflow.tasks {
        validate_subtasks_uses_workflow(parent_id, parent_task, workflow, errors);
    }
}

/// Recursively validate uses_workflow in subtasks
fn validate_subtasks_uses_workflow(
    parent_id: &str,
    parent_task: &crate::dsl::schema::TaskSpec,
    workflow: &DSLWorkflow,
    errors: &mut ValidationErrors,
) {
    use crate::dsl::schema::TaskSpec;

    for subtask_map in &parent_task.subtasks {
        for (subtask_id, subtask) in subtask_map {
            let full_id = format!("{}.{}", parent_id, subtask_id);

            if let Some(ref workflow_ref) = subtask.uses_workflow {
                // Check mutual exclusivity
                if subtask.execution_type_count() > 1 {
                    errors.add_error(format!(
                        "Subtask '{}': uses_workflow is mutually exclusive with other execution types",
                        full_id
                    ));
                }

                // Validate format
                if let Some((namespace, _)) = TaskSpec::parse_workflow_reference(workflow_ref) {
                    if !workflow.imports.contains_key(namespace) {
                        errors.add_error(format!(
                            "Subtask '{}': namespace '{}' not found in imports",
                            full_id, namespace
                        ));
                    }
                } else {
                    errors.add_error(format!(
                        "Subtask '{}': invalid uses_workflow reference '{}'",
                        full_id, workflow_ref
                    ));
                }
            }

            // Recurse into nested subtasks
            validate_subtasks_uses_workflow(&full_id, subtask, workflow, errors);
        }
    }
}

/// Validate notification configurations throughout the workflow
fn validate_notification_configurations(workflow: &DSLWorkflow, errors: &mut ValidationErrors) {
    // Collect all defined secrets for validation
    let defined_secrets: HashSet<String> = workflow.secrets.keys().cloned().collect();

    // Validate workflow-level notification defaults
    if let Some(notif_config) = &workflow.notifications {
        for (idx, channel) in notif_config.default_channels.iter().enumerate() {
            validate_notification_channel(
                channel,
                &format!("Workflow default_channels[{}]", idx),
                &defined_secrets,
                workflow,
                errors,
            );
        }
    }

    // Validate task-level notifications
    for (task_id, task_spec) in &workflow.tasks {
        // Validate on_complete notifications
        if let Some(on_complete) = &task_spec.on_complete {
            if let Some(notif_spec) = &on_complete.notify {
                validate_notification_spec(
                    notif_spec,
                    &format!("Task '{}' on_complete", task_id),
                    &defined_secrets,
                    workflow,
                    errors,
                );
            }
        }

        // Recursively validate subtask notifications
        validate_subtask_notifications(
            task_id,
            &task_spec.subtasks,
            &defined_secrets,
            workflow,
            errors,
        );
    }
}

/// Recursively validate notifications in subtasks
fn validate_subtask_notifications(
    parent_id: &str,
    subtasks: &[HashMap<String, crate::dsl::schema::TaskSpec>],
    defined_secrets: &HashSet<String>,
    workflow: &DSLWorkflow,
    errors: &mut ValidationErrors,
) {
    for subtask_map in subtasks {
        for (subtask_id, subtask_spec) in subtask_map {
            let full_id = format!("{}.{}", parent_id, subtask_id);

            // Validate on_complete notifications
            if let Some(on_complete) = &subtask_spec.on_complete {
                if let Some(notif_spec) = &on_complete.notify {
                    validate_notification_spec(
                        notif_spec,
                        &format!("Subtask '{}' on_complete", full_id),
                        defined_secrets,
                        workflow,
                        errors,
                    );
                }
            }

            // Recurse into nested subtasks
            validate_subtask_notifications(
                &full_id,
                &subtask_spec.subtasks,
                defined_secrets,
                workflow,
                errors,
            );
        }
    }
}

/// Validate a notification specification (Simple or Structured)
fn validate_notification_spec(
    notif_spec: &crate::dsl::schema::NotificationSpec,
    context: &str,
    defined_secrets: &HashSet<String>,
    workflow: &DSLWorkflow,
    errors: &mut ValidationErrors,
) {
    use crate::dsl::schema::NotificationSpec;

    match notif_spec {
        NotificationSpec::Simple(message) => {
            // Simple notifications just need a non-empty message
            if message.trim().is_empty() {
                errors.add_error(format!("{}: notification message cannot be empty", context));
            }
        }
        NotificationSpec::Structured {
            message,
            channels,
            priority,
            metadata,
            ..
        } => {
            // Validate message is not empty
            if message.trim().is_empty() {
                errors.add_error(format!("{}: notification message cannot be empty", context));
            }

            // Validate at least one channel is specified
            if channels.is_empty() {
                errors.add_error(format!(
                    "{}: structured notification must specify at least one channel",
                    context
                ));
            }

            // Validate priority if specified
            if let Some(priority_val) = priority {
                // Priority validation happens during deserialization via enum
                // But we can add additional checks here if needed
                use crate::dsl::schema::NotificationPriority;
                match priority_val {
                    NotificationPriority::Low
                    | NotificationPriority::Normal
                    | NotificationPriority::High
                    | NotificationPriority::Critical => {
                        // Valid priority
                    }
                }
            }

            // Validate each channel
            for (idx, channel) in channels.iter().enumerate() {
                validate_notification_channel(
                    channel,
                    &format!("{}channels[{}]", context, idx),
                    defined_secrets,
                    workflow,
                    errors,
                );
            }

            // Validate metadata keys are reasonable
            for key in metadata.keys() {
                if key.trim().is_empty() {
                    errors.add_error(format!("{}: metadata key cannot be empty", context));
                }
            }
        }
    }
}

/// Validate a notification channel configuration
fn validate_notification_channel(
    channel: &crate::dsl::schema::NotificationChannel,
    context: &str,
    defined_secrets: &HashSet<String>,
    _workflow: &DSLWorkflow,
    errors: &mut ValidationErrors,
) {
    use crate::dsl::schema::{
        FileNotificationFormat, HttpMethod, NotificationChannel, SlackMethod,
    };

    match channel {
        NotificationChannel::Console { .. } => {
            // Console channel has no required fields to validate
            // colored and timestamp are booleans with defaults
        }

        NotificationChannel::Ntfy {
            server,
            topic,
            priority,
            tags,
            ..
        } => {
            // Validate server URL
            if !server.starts_with("http://") && !server.starts_with("https://") {
                errors.add_error(format!(
                    "{}: Ntfy server '{}' must be a valid HTTP(S) URL",
                    context, server
                ));
            }

            // Validate topic is not empty
            if topic.trim().is_empty() {
                errors.add_error(format!("{}: Ntfy topic cannot be empty", context));
            }

            // Validate priority range (1-5)
            if let Some(p) = priority {
                if *p < 1 || *p > 5 {
                    errors.add_error(format!(
                        "{}: Ntfy priority must be between 1 and 5, got {}",
                        context, p
                    ));
                }
            }

            // Warn if too many tags
            if tags.len() > 10 {
                errors.add_warning(format!(
                    "{}: Ntfy supports up to 5 tags, {} specified may be truncated",
                    context,
                    tags.len()
                ));
            }
        }

        NotificationChannel::Slack {
            credential,
            channel: slack_channel,
            method,
            ..
        } => {
            // Validate credential secret reference
            validate_secret_reference(credential, context, "credential", defined_secrets, errors);

            // Validate channel format
            if !slack_channel.starts_with('#') && !slack_channel.starts_with('@') {
                errors.add_warning(format!(
                    "{}: Slack channel '{}' should typically start with # (channel) or @ (user)",
                    context, slack_channel
                ));
            }

            // Validate method
            match method {
                SlackMethod::Webhook | SlackMethod::Bot => {
                    // Both are valid
                }
            }
        }

        NotificationChannel::Discord {
            webhook_url, embed, ..
        } => {
            // Validate webhook URL (can be secret reference or direct URL)
            if webhook_url.starts_with("${secret.") {
                validate_secret_reference(
                    webhook_url,
                    context,
                    "webhook_url",
                    defined_secrets,
                    errors,
                );
            } else if !webhook_url.starts_with("https://discord.com/api/webhooks/")
                && !webhook_url.starts_with("https://discordapp.com/api/webhooks/")
            {
                errors.add_error(format!(
                    "{}: Discord webhook URL must be a valid Discord webhook URL or a secret reference",
                    context
                ));
            }

            // Validate embed if present
            if let Some(embed_data) = embed {
                // Validate color is valid RGB
                if let Some(color) = embed_data.color {
                    if color > 0xFFFFFF {
                        errors.add_error(format!(
                            "{}: Discord embed color {} exceeds maximum RGB value (16777215)",
                            context, color
                        ));
                    }
                }

                // Validate field limits
                if embed_data.fields.len() > 25 {
                    errors.add_error(format!(
                        "{}: Discord embed can have maximum 25 fields, got {}",
                        context,
                        embed_data.fields.len()
                    ));
                }

                // Validate individual field lengths
                for (field_idx, field) in embed_data.fields.iter().enumerate() {
                    if field.name.len() > 256 {
                        errors.add_error(format!(
                            "{}: Discord embed field[{}] name exceeds 256 characters",
                            context, field_idx
                        ));
                    }
                    if field.value.len() > 1024 {
                        errors.add_error(format!(
                            "{}: Discord embed field[{}] value exceeds 1024 characters",
                            context, field_idx
                        ));
                    }
                }

                // Validate title length
                if let Some(title) = &embed_data.title {
                    if title.len() > 256 {
                        errors.add_error(format!(
                            "{}: Discord embed title exceeds 256 characters",
                            context
                        ));
                    }
                }

                // Validate description length
                if let Some(desc) = &embed_data.description {
                    if desc.len() > 4096 {
                        errors.add_error(format!(
                            "{}: Discord embed description exceeds 4096 characters",
                            context
                        ));
                    }
                }
            }
        }

        NotificationChannel::Webhook {
            url,
            method,
            auth,
            retry,
            ..
        } => {
            // Validate URL (can be secret reference or direct URL)
            if url.starts_with("${secret.") {
                validate_secret_reference(url, context, "url", defined_secrets, errors);
            } else if !url.starts_with("http://") && !url.starts_with("https://") {
                errors.add_error(format!(
                    "{}: Webhook URL '{}' must be a valid HTTP(S) URL or a secret reference",
                    context, url
                ));
            }

            // Validate HTTP method
            match method {
                HttpMethod::Get
                | HttpMethod::Post
                | HttpMethod::Put
                | HttpMethod::Patch
                | HttpMethod::Delete
                | HttpMethod::Head
                | HttpMethod::Options => {
                    // All valid
                }
            }

            // Validate auth if present
            if let Some(auth_config) = auth {
                match auth_config {
                    crate::dsl::schema::HttpAuth::Bearer { token } => {
                        validate_secret_reference(
                            token,
                            context,
                            "auth.token",
                            defined_secrets,
                            errors,
                        );
                    }
                    crate::dsl::schema::HttpAuth::Basic { username, password } => {
                        validate_secret_reference(
                            username,
                            context,
                            "auth.username",
                            defined_secrets,
                            errors,
                        );
                        validate_secret_reference(
                            password,
                            context,
                            "auth.password",
                            defined_secrets,
                            errors,
                        );
                    }
                    crate::dsl::schema::HttpAuth::ApiKey { header, key } => {
                        if header.trim().is_empty() {
                            errors.add_error(format!(
                                "{}: auth API key header name cannot be empty",
                                context
                            ));
                        }
                        validate_secret_reference(
                            key,
                            context,
                            "auth.key",
                            defined_secrets,
                            errors,
                        );
                    }
                    crate::dsl::schema::HttpAuth::Custom { headers } => {
                        // Validate custom headers
                        if headers.is_empty() {
                            errors.add_error(format!(
                                "{}: auth.custom headers cannot be empty",
                                context
                            ));
                        }
                        for (header_name, header_value) in headers.iter() {
                            if header_name.trim().is_empty() {
                                errors.add_error(format!(
                                    "{}: auth.custom header name cannot be empty",
                                    context
                                ));
                            }
                            validate_secret_reference(
                                header_value,
                                context,
                                &format!("auth.custom.{}", header_name),
                                defined_secrets,
                                errors,
                            );
                        }
                    }
                }
            }

            // Validate retry configuration if present
            if let Some(retry_config) = retry {
                if retry_config.max_attempts == 0 {
                    errors.add_error(format!(
                        "{}: retry max_attempts must be greater than 0",
                        context
                    ));
                }
                if retry_config.max_attempts > 10 {
                    errors.add_warning(format!(
                        "{}: retry max_attempts of {} is very high, consider reducing it",
                        context, retry_config.max_attempts
                    ));
                }
                if retry_config.delay_secs == 0 && retry_config.exponential_backoff {
                    errors.add_warning(format!(
                        "{}: exponential backoff with 0 initial delay may not be effective",
                        context
                    ));
                }
            }
        }

        NotificationChannel::File { path, format, .. } => {
            // Validate path is not empty
            if path.trim().is_empty() {
                errors.add_error(format!("{}: File path cannot be empty", context));
            }

            // Validate format
            match format {
                FileNotificationFormat::Text
                | FileNotificationFormat::Json
                | FileNotificationFormat::JsonLines => {
                    // All valid
                }
            }

            // Warn about absolute vs relative paths
            if path.starts_with('/') || path.contains(":\\") {
                errors.add_warning(format!(
                    "{}: Using absolute path '{}', consider using relative paths for portability",
                    context, path
                ));
            }
        }

        NotificationChannel::Email {
            to,
            subject,
            smtp,
            cc,
            bcc,
            ..
        } => {
            // Validate at least one recipient
            if to.is_empty() {
                errors.add_error(format!(
                    "{}: Email must have at least one 'to' recipient",
                    context
                ));
            }

            // Validate email addresses
            for (idx, email) in to.iter().enumerate() {
                if !is_valid_email_format(email) {
                    errors.add_error(format!(
                        "{}: Invalid email address in 'to[{}]': '{}'",
                        context, idx, email
                    ));
                }
            }

            for (idx, email) in cc.iter().enumerate() {
                if !is_valid_email_format(email) {
                    errors.add_error(format!(
                        "{}: Invalid email address in 'cc[{}]': '{}'",
                        context, idx, email
                    ));
                }
            }

            for (idx, email) in bcc.iter().enumerate() {
                if !is_valid_email_format(email) {
                    errors.add_error(format!(
                        "{}: Invalid email address in 'bcc[{}]': '{}'",
                        context, idx, email
                    ));
                }
            }

            // Validate subject is not empty
            if let Some(subj) = subject {
                if subj.trim().is_empty() {
                    errors.add_warning(format!("{}: Email subject is empty", context));
                }
            }

            // Validate SMTP configuration
            if smtp.host.trim().is_empty() {
                errors.add_error(format!("{}: SMTP host cannot be empty", context));
            }

            if smtp.port == 0 {
                errors.add_error(format!(
                    "{}: SMTP port cannot be 0 (must be 1-65535)",
                    context
                ));
            }

            // Validate common SMTP ports
            let common_ports = [25, 465, 587, 2525];
            if !common_ports.contains(&smtp.port) {
                errors.add_warning(format!(
                    "{}: SMTP port {} is not a standard port (25, 465, 587, 2525)",
                    context, smtp.port
                ));
            }

            // Validate credentials
            validate_secret_reference(
                &smtp.username,
                context,
                "smtp.username",
                defined_secrets,
                errors,
            );
            validate_secret_reference(
                &smtp.password,
                context,
                "smtp.password",
                defined_secrets,
                errors,
            );

            // Validate from address
            if !is_valid_email_format(&smtp.from) {
                errors.add_error(format!(
                    "{}: Invalid 'from' email address: '{}'",
                    context, smtp.from
                ));
            }

            // Warn about TLS
            if !smtp.use_tls && smtp.port == 587 {
                errors.add_warning(format!(
                    "{}: Port 587 typically requires TLS, but use_tls is false",
                    context
                ));
            }
        }

        NotificationChannel::Teams {
            webhook_url,
            theme_color,
            facts,
            ..
        } => {
            // Validate webhook URL
            validate_secret_reference(webhook_url, context, "webhook_url", defined_secrets, errors);
            if !webhook_url.starts_with("http://")
                && !webhook_url.starts_with("https://")
                && !webhook_url.contains("${secret.")
            {
                errors.add_error(format!(
                    "{}: Teams webhook URL must be a valid HTTP(S) URL or secret reference",
                    context
                ));
            }

            // Validate theme color format if provided
            if let Some(color) = theme_color {
                if !color.is_empty() && !color.starts_with('#') {
                    errors.add_warning(format!(
                        "{}: Teams theme_color should be in hex format (e.g., #FF5733)",
                        context
                    ));
                }
            }

            // No validation needed for facts - they're just key-value pairs
            let _ = facts;
        }

        NotificationChannel::Telegram {
            bot_token,
            chat_id,
            parse_mode,
            ..
        } => {
            // Validate bot token
            validate_secret_reference(bot_token, context, "bot_token", defined_secrets, errors);

            // Validate chat_id is not empty
            if chat_id.trim().is_empty() {
                errors.add_error(format!("{}: Telegram chat_id cannot be empty", context));
            }

            // parse_mode is an enum, so it's always valid
            let _ = parse_mode;
        }

        NotificationChannel::PagerDuty {
            integration_key,
            action,
            severity,
            ..
        } => {
            // Validate integration key
            validate_secret_reference(
                integration_key,
                context,
                "integration_key",
                defined_secrets,
                errors,
            );

            // action and severity are enums, so they're always valid
            let _ = (action, severity);
        }
    }
}

/// Validate a secret reference in the format ${secret.name}
fn validate_secret_reference(
    value: &str,
    context: &str,
    field_name: &str,
    defined_secrets: &HashSet<String>,
    errors: &mut ValidationErrors,
) {
    // Check if it's a secret reference
    if value.starts_with("${secret.") && value.ends_with('}') {
        // Extract secret name
        let secret_name = &value[9..value.len() - 1]; // Remove "${secret." prefix and "}" suffix

        if secret_name.is_empty() {
            errors.add_error(format!(
                "{}: {} has empty secret name in reference '{}'",
                context, field_name, value
            ));
        } else if !defined_secrets.contains(secret_name) {
            errors.add_error(format!(
                "{}: {} references undefined secret '{}'. Available secrets: {:?}",
                context,
                field_name,
                secret_name,
                defined_secrets.iter().collect::<Vec<_>>()
            ));
        }
    } else if value.starts_with("${secret.") || value.contains("${secret.") {
        // Malformed secret reference
        errors.add_error(format!(
            "{}: {} has malformed secret reference '{}'. Expected format: ${{secret.name}}",
            context, field_name, value
        ));
    }
    // If it doesn't look like a secret reference, we don't validate it
    // (it might be a direct credential, which is allowed but not recommended)
}

/// Basic email format validation
fn is_valid_email_format(email: &str) -> bool {
    // Basic validation: contains @ and has characters before and after it
    // This is intentionally simple to avoid complex regex
    let parts: Vec<&str> = email.split('@').collect();
    parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() && parts[1].contains('.')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Provider;
    use crate::dsl::schema::{AgentSpec, PermissionsSpec};

    fn create_test_workflow() -> DSLWorkflow {
        DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        }
    }

    #[test]
    fn test_validate_minimal_workflow() {
        let workflow = create_test_workflow();
        assert!(validate_workflow(&workflow).is_ok());
    }

    #[test]
    fn test_validate_invalid_agent_reference() {
        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let task = TaskSpec {
            description: "Test task".to_string(),
            agent: Some("non_existent_agent".to_string()),
            ..Default::default()
        };

        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("non-existent agent"));
    }

    #[test]
    fn test_validate_circular_dependency() {
        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let task1 = TaskSpec {
            description: "Task 1".to_string(),
            depends_on: vec!["task2".to_string()],
            ..Default::default()
        };

        let task2 = TaskSpec {
            description: "Task 2".to_string(),
            depends_on: vec!["task1".to_string()],
            ..Default::default()
        };

        workflow.tasks.insert("task1".to_string(), task1);
        workflow.tasks.insert("task2".to_string(), task2);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Circular dependency"));
    }

    #[test]
    fn test_validate_invalid_tool() {
        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let agent = AgentSpec {
            provider: None,
            description: "Test agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec!["InvalidTool".to_string()],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        };

        workflow.agents.insert("agent1".to_string(), agent);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid tool"));
    }

    #[test]
    fn test_validate_invalid_permission_mode() {
        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let agent = AgentSpec {
            provider: None,
            description: "Test agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec!["Read".to_string()],
            permissions: PermissionsSpec {
                mode: "invalid_mode".to_string(),
                allowed_directories: vec![],
            },
            max_turns: None,
        };

        workflow.agents.insert("agent1".to_string(), agent);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid permission mode"));
    }

    #[test]
    fn test_validate_loop_repeat_exceeds_max_iterations() {
        use crate::dsl::schema::LoopSpec;

        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let task = TaskSpec {
            description: "Test loop task".to_string(),
            loop_spec: Some(LoopSpec::Repeat {
                count: 20_000, // exceeds MAX_LOOP_ITERATIONS
                iterator: None,
                parallel: false,
                max_parallel: None,
            }),
            ..Default::default()
        };

        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds safety limit"));
    }

    #[test]
    fn test_validate_loop_foreach_with_valid_range() {
        use crate::dsl::schema::{CollectionSource, LoopSpec};

        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let task = TaskSpec {
            description: "Test loop task".to_string(),
            loop_spec: Some(LoopSpec::ForEach {
                collection: CollectionSource::Range {
                    start: 0,
                    end: 10,
                    step: Some(1),
                },
                iterator: "i".to_string(),
                parallel: false,
                max_parallel: None,
            }),
            ..Default::default()
        };

        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_loop_invalid_range() {
        use crate::dsl::schema::{CollectionSource, LoopSpec};

        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let task = TaskSpec {
            description: "Test loop task".to_string(),
            loop_spec: Some(LoopSpec::ForEach {
                collection: CollectionSource::Range {
                    start: 10,
                    end: 5, // invalid: start > end
                    step: Some(1),
                },
                iterator: "i".to_string(),
                parallel: false,
                max_parallel: None,
            }),
            ..Default::default()
        };

        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be less than end"));
    }

    #[test]
    fn test_validate_loop_max_parallel_exceeds_limit() {
        use crate::dsl::schema::{CollectionSource, LoopSpec};

        let mut workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: None,
            create_cwd: None,
            secrets: HashMap::new(),
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            agents: HashMap::new(),
            tasks: HashMap::new(),
            workflows: HashMap::new(),
            tools: None,
            communication: None,
            mcp_servers: HashMap::new(),
            subflows: HashMap::new(),
            imports: HashMap::new(),
            notifications: None,
            limits: None,
        };

        let task = TaskSpec {
            description: "Test loop task".to_string(),
            loop_spec: Some(LoopSpec::ForEach {
                collection: CollectionSource::Inline {
                    items: vec![serde_json::json!(1), serde_json::json!(2)],
                },
                iterator: "i".to_string(),
                parallel: true,
                max_parallel: Some(200), // exceeds MAX_PARALLEL_ITERATIONS
            }),
            ..Default::default()
        };

        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds safety limit"));
    }

    #[test]
    fn test_validate_imports_valid() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());
        workflow.imports.insert(
            "slack-api".to_string(),
            "slack-integrations@2.1.0".to_string(),
        );
        workflow
            .imports
            .insert("github_actions".to_string(), "github-ci@3.0.0".to_string());

        let result = validate_workflow(&workflow);
        assert!(result.is_ok(), "Valid imports should pass validation");
    }

    #[test]
    fn test_validate_imports_invalid_namespace() {
        let mut workflow = create_test_workflow();

        // Invalid: starts with digit
        workflow
            .imports
            .insert("123invalid".to_string(), "some-group@1.0.0".to_string());

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid namespace identifier"));
    }

    #[test]
    fn test_validate_imports_invalid_namespace_special_chars() {
        let mut workflow = create_test_workflow();

        // Invalid: contains special characters
        workflow.imports.insert(
            "google:workspace".to_string(),
            "some-group@1.0.0".to_string(),
        );

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid namespace identifier"));
    }

    #[test]
    fn test_validate_imports_invalid_group_reference() {
        let mut workflow = create_test_workflow();

        // Invalid: missing version
        workflow
            .imports
            .insert("google".to_string(), "google-workspace".to_string());

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid group reference"));
    }

    #[test]
    fn test_validate_imports_invalid_group_reference_format() {
        let mut workflow = create_test_workflow();

        // Invalid: multiple @ signs
        workflow
            .imports
            .insert("google".to_string(), "google@workspace@1.0.0".to_string());

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid group reference"));
    }

    #[test]
    fn test_validate_imports_empty_namespace() {
        let mut workflow = create_test_workflow();

        // Invalid: empty namespace
        workflow
            .imports
            .insert("".to_string(), "some-group@1.0.0".to_string());

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid namespace identifier"));
    }

    #[test]
    fn test_validate_imports_empty_group_name() {
        let mut workflow = create_test_workflow();

        // Invalid: empty group name
        workflow
            .imports
            .insert("google".to_string(), "@1.0.0".to_string());

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid group reference"));
    }

    #[test]
    fn test_validate_imports_empty_version() {
        let mut workflow = create_test_workflow();

        // Invalid: empty version
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@".to_string());

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid group reference"));
    }

    #[test]
    fn test_workflow_import_struct() {
        use crate::dsl::schema::WorkflowImport;

        // Test construction
        let import =
            WorkflowImport::new("google".to_string(), "google-workspace@1.0.0".to_string());
        assert_eq!(import.namespace, "google");
        assert_eq!(import.group_reference, "google-workspace@1.0.0");

        // Test from_entry
        let import2 = WorkflowImport::from_entry("slack", "slack-api@2.0.0");
        assert_eq!(import2.namespace, "slack");
        assert_eq!(import2.group_reference, "slack-api@2.0.0");
    }

    #[test]
    fn test_workflow_import_validate_namespace() {
        use crate::dsl::schema::WorkflowImport;

        // Valid namespaces
        assert!(WorkflowImport::validate_namespace("google"));
        assert!(WorkflowImport::validate_namespace("slack-api"));
        assert!(WorkflowImport::validate_namespace("github_actions"));
        assert!(WorkflowImport::validate_namespace("my-namespace_123"));

        // Invalid namespaces
        assert!(!WorkflowImport::validate_namespace(""));
        assert!(!WorkflowImport::validate_namespace("123invalid"));
        assert!(!WorkflowImport::validate_namespace("google:workspace"));
        assert!(!WorkflowImport::validate_namespace("slack.api"));
        assert!(!WorkflowImport::validate_namespace("name space"));
    }

    #[test]
    fn test_workflow_import_parse_group_reference() {
        use crate::dsl::schema::WorkflowImport;

        // Valid references
        let parsed = WorkflowImport::parse_group_reference("google-workspace@1.0.0");
        assert_eq!(
            parsed,
            Some(("google-workspace".to_string(), "1.0.0".to_string()))
        );

        let parsed2 = WorkflowImport::parse_group_reference("my-group@2.3.4-beta");
        assert_eq!(
            parsed2,
            Some(("my-group".to_string(), "2.3.4-beta".to_string()))
        );

        // Invalid references
        assert_eq!(WorkflowImport::parse_group_reference("no-version"), None);
        assert_eq!(WorkflowImport::parse_group_reference("@1.0.0"), None);
        assert_eq!(WorkflowImport::parse_group_reference("name@"), None);
        assert_eq!(
            WorkflowImport::parse_group_reference("multiple@at@signs"),
            None
        );
    }

    #[test]
    fn test_taskspec_parse_workflow_reference() {
        use crate::dsl::schema::TaskSpec;

        // Valid references
        assert_eq!(
            TaskSpec::parse_workflow_reference("google:upload-files"),
            Some(("google", "upload-files"))
        );
        assert_eq!(
            TaskSpec::parse_workflow_reference("slack-api:send-message"),
            Some(("slack-api", "send-message"))
        );
        assert_eq!(
            TaskSpec::parse_workflow_reference("ns_123:workflow_name"),
            Some(("ns_123", "workflow_name"))
        );

        // Invalid references
        assert_eq!(TaskSpec::parse_workflow_reference("no-colon"), None);
        assert_eq!(TaskSpec::parse_workflow_reference(":workflow"), None);
        assert_eq!(TaskSpec::parse_workflow_reference("namespace:"), None);
        assert_eq!(
            TaskSpec::parse_workflow_reference("multiple:colons:here"),
            None
        );
    }

    #[test]
    fn test_taskspec_execution_type_helpers() {
        use crate::dsl::schema::TaskSpec;

        // Test with no execution type
        let task = TaskSpec::default();
        assert!(!task.has_execution_type());
        assert_eq!(task.execution_type_count(), 0);

        // Test with agent
        let task = TaskSpec {
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };
        assert!(task.has_execution_type());
        assert_eq!(task.execution_type_count(), 1);

        // Test with uses_workflow
        let task = TaskSpec {
            uses_workflow: Some("google:upload".to_string()),
            ..Default::default()
        };
        assert!(task.has_execution_type());
        assert_eq!(task.execution_type_count(), 1);

        // Test with multiple (should be invalid)
        let task = TaskSpec {
            agent: Some("test_agent".to_string()),
            uses_workflow: Some("google:upload".to_string()),
            ..Default::default()
        };
        assert!(task.has_execution_type());
        assert_eq!(task.execution_type_count(), 2);
    }

    #[test]
    fn test_validate_uses_workflow_valid() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        // Add a dummy agent to satisfy validation (workflows need at least one agent)
        workflow.agents.insert(
            "processor".to_string(),
            AgentSpec {
                provider: None,
                description: "Processor agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        let task = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some("google:upload-files".to_string()),
            inputs: [("folder_id".to_string(), serde_json::json!("abc123"))]
                .iter()
                .cloned()
                .collect(),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        if let Err(e) = &result {
            eprintln!("Validation error: {:?}", e);
        }
        assert!(result.is_ok(), "Valid uses_workflow should pass validation");
    }

    #[test]
    fn test_validate_uses_workflow_invalid_format() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        // Invalid: no colon
        let task = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some("invalid-format".to_string()),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected format: 'namespace:workflow_name'"));
    }

    #[test]
    fn test_validate_uses_workflow_namespace_not_found() {
        let mut workflow = create_test_workflow();
        // No imports defined

        let task = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some("google:upload-files".to_string()),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("namespace 'google' in uses_workflow"));
        assert!(err_msg.contains("not found in imports"));
    }

    #[test]
    fn test_validate_uses_workflow_mutual_exclusivity_agent() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        // Invalid: both agent and uses_workflow
        let task = TaskSpec {
            description: "Upload files".to_string(),
            agent: Some("test_agent".to_string()),
            uses_workflow: Some("google:upload-files".to_string()),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("mutually exclusive"));
    }

    #[test]
    fn test_validate_uses_workflow_mutual_exclusivity_subflow() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        // Invalid: both subflow and uses_workflow
        let task = TaskSpec {
            description: "Upload files".to_string(),
            subflow: Some("my-subflow".to_string()),
            uses_workflow: Some("google:upload-files".to_string()),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("mutually exclusive"));
    }

    #[test]
    fn test_validate_uses_workflow_mutual_exclusivity_uses() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        // Invalid: both uses and uses_workflow
        let task = TaskSpec {
            description: "Upload files".to_string(),
            uses: Some("some-task@1.0.0".to_string()),
            uses_workflow: Some("google:upload-files".to_string()),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("mutually exclusive"));
    }

    #[test]
    fn test_validate_uses_workflow_in_subtasks() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        let subtask = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some("google:upload-files".to_string()),
            ..Default::default()
        };

        let mut subtask_map = HashMap::new();
        subtask_map.insert("upload_subtask".to_string(), subtask);

        let parent_task = TaskSpec {
            description: "Parent task".to_string(),
            subtasks: vec![subtask_map],
            ..Default::default()
        };

        workflow.tasks.insert("parent".to_string(), parent_task);

        let result = validate_workflow(&workflow);
        assert!(result.is_ok(), "Valid uses_workflow in subtask should pass");
    }

    #[test]
    fn test_validate_uses_workflow_invalid_namespace_in_subtask() {
        let mut workflow = create_test_workflow();
        // No imports defined

        let subtask = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some("missing:upload-files".to_string()),
            ..Default::default()
        };

        let mut subtask_map = HashMap::new();
        subtask_map.insert("upload_subtask".to_string(), subtask);

        let parent_task = TaskSpec {
            description: "Parent task".to_string(),
            subtasks: vec![subtask_map],
            ..Default::default()
        };

        workflow.tasks.insert("parent".to_string(), parent_task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("namespace 'missing' not found"));
    }

    #[test]
    fn test_validate_uses_workflow_empty_namespace() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        let task = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some(":upload-files".to_string()),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected format: 'namespace:workflow_name'"));
    }

    #[test]
    fn test_validate_uses_workflow_empty_workflow_name() {
        let mut workflow = create_test_workflow();
        workflow
            .imports
            .insert("google".to_string(), "google-workspace@1.0.0".to_string());

        let task = TaskSpec {
            description: "Upload files".to_string(),
            uses_workflow: Some("google:".to_string()),
            ..Default::default()
        };
        workflow.tasks.insert("upload".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected format: 'namespace:workflow_name'"));
    }

    // ========================================================================
    // Notification Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_simple_notification_valid() {
        use crate::dsl::schema::{ActionSpec, NotificationSpec};

        let mut workflow = create_test_workflow();

        // Add a dummy agent so task has valid execution type
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        let task = TaskSpec {
            description: "Test task".to_string(),
            agent: Some("test_agent".to_string()),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Simple("Task completed".to_string())),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_simple_notification_empty_message() {
        use crate::dsl::schema::{ActionSpec, NotificationSpec};

        let mut workflow = create_test_workflow();

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Simple("   ".to_string())),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("notification message cannot be empty"));
    }

    #[test]
    fn test_validate_structured_notification_no_channels() {
        use crate::dsl::schema::{ActionSpec, NotificationSpec};
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test message".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![], // Empty channels - should fail
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must specify at least one channel"));
    }

    #[test]
    fn test_validate_ntfy_channel_valid() {
        use crate::dsl::schema::{ActionSpec, NotificationChannel, NotificationSpec};
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();

        // Add a dummy agent so task has valid execution type
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        let task = TaskSpec {
            description: "Test task".to_string(),
            agent: Some("test_agent".to_string()),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Ntfy {
                        server: "https://ntfy.sh".to_string(),
                        topic: "test-topic".to_string(),
                        title: None,
                        priority: Some(3),
                        tags: vec!["test".to_string()],
                        click_url: None,
                        attach_url: None,
                        markdown: false,
                        auth_token: None,
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_ntfy_channel_invalid_priority() {
        use crate::dsl::schema::{ActionSpec, NotificationChannel, NotificationSpec};
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Ntfy {
                        server: "https://ntfy.sh".to_string(),
                        topic: "test-topic".to_string(),
                        title: None,
                        priority: Some(10), // Invalid: must be 1-5
                        tags: vec![],
                        click_url: None,
                        attach_url: None,
                        markdown: false,
                        auth_token: None,
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("priority must be between 1 and 5"));
    }

    #[test]
    fn test_validate_slack_channel_with_secret() {
        use crate::dsl::schema::{ActionSpec, NotificationChannel, NotificationSpec, SlackMethod};
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();
        use crate::dsl::schema::{SecretSource, SecretSpec};
        workflow.secrets.insert(
            "slack_token".to_string(),
            SecretSpec {
                source: SecretSource::Value {
                    value: "encrypted".to_string(),
                },
                description: None,
            },
        );

        // Add a dummy agent so task has valid execution type
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        let task = TaskSpec {
            description: "Test task".to_string(),
            agent: Some("test_agent".to_string()),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Slack {
                        credential: "${secret.slack_token}".to_string(),
                        channel: "#general".to_string(),
                        method: SlackMethod::Webhook,
                        attachments: vec![],
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_slack_channel_undefined_secret() {
        use crate::dsl::schema::{ActionSpec, NotificationChannel, NotificationSpec, SlackMethod};
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();
        // No secrets defined

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Slack {
                        credential: "${secret.undefined_token}".to_string(),
                        channel: "#general".to_string(),
                        method: SlackMethod::Webhook,
                        attachments: vec![],
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("references undefined secret"));
    }

    #[test]
    fn test_validate_discord_embed_too_many_fields() {
        use crate::dsl::schema::{
            ActionSpec, DiscordEmbed, DiscordField, NotificationChannel, NotificationSpec,
        };
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();

        // Create more than 25 fields
        let fields: Vec<DiscordField> = (0..30)
            .map(|i| DiscordField {
                name: format!("Field {}", i),
                value: format!("Value {}", i),
                inline: false,
            })
            .collect();

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Discord {
                        webhook_url: "https://discord.com/api/webhooks/123/token".to_string(),
                        username: None,
                        avatar_url: None,
                        tts: false,
                        embed: Some(DiscordEmbed {
                            title: None,
                            description: None,
                            color: None,
                            fields,
                            footer: None,
                            timestamp: None,
                        }),
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("can have maximum 25 fields"));
    }

    #[test]
    fn test_validate_email_channel_valid() {
        use crate::dsl::schema::{
            ActionSpec, NotificationChannel, NotificationSpec, SecretSource, SecretSpec, SmtpConfig,
        };
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();
        workflow.secrets.insert(
            "smtp_user".to_string(),
            SecretSpec {
                source: SecretSource::Value {
                    value: "user".to_string(),
                },
                description: None,
            },
        );
        workflow.secrets.insert(
            "smtp_pass".to_string(),
            SecretSpec {
                source: SecretSource::Value {
                    value: "pass".to_string(),
                },
                description: None,
            },
        );

        // Add a dummy agent so task has valid execution type
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        let task = TaskSpec {
            description: "Test task".to_string(),
            agent: Some("test_agent".to_string()),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Email {
                        to: vec!["user@example.com".to_string()],
                        cc: vec![],
                        bcc: vec![],
                        subject: Some("Test Subject".to_string()),
                        smtp: SmtpConfig {
                            host: "smtp.example.com".to_string(),
                            port: 587,
                            username: "${secret.smtp_user}".to_string(),
                            password: "${secret.smtp_pass}".to_string(),
                            from: "noreply@example.com".to_string(),
                            use_tls: true,
                        },
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_email_channel_invalid_address() {
        use crate::dsl::schema::{
            ActionSpec, NotificationChannel, NotificationSpec, SecretSource, SecretSpec, SmtpConfig,
        };
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();
        workflow.secrets.insert(
            "smtp_user".to_string(),
            SecretSpec {
                source: SecretSource::Value {
                    value: "user".to_string(),
                },
                description: None,
            },
        );
        workflow.secrets.insert(
            "smtp_pass".to_string(),
            SecretSpec {
                source: SecretSource::Value {
                    value: "pass".to_string(),
                },
                description: None,
            },
        );

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Email {
                        to: vec!["invalid-email".to_string()], // Invalid email format
                        cc: vec![],
                        bcc: vec![],
                        subject: Some("Test".to_string()),
                        smtp: SmtpConfig {
                            host: "smtp.example.com".to_string(),
                            port: 587,
                            username: "${secret.smtp_user}".to_string(),
                            password: "${secret.smtp_pass}".to_string(),
                            from: "noreply@example.com".to_string(),
                            use_tls: true,
                        },
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid email address"));
    }

    #[test]
    fn test_validate_webhook_retry_invalid() {
        use crate::dsl::schema::{
            ActionSpec, HttpMethod, NotificationChannel, NotificationSpec, RetryConfig,
        };
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::Webhook {
                        url: "https://example.com/webhook".to_string(),
                        method: HttpMethod::Post,
                        headers: HashMap::new(),
                        auth: None,
                        body_template: None,
                        timeout_secs: None,
                        retry: Some(RetryConfig {
                            max_attempts: 0, // Invalid: must be > 0
                            delay_secs: 5,
                            exponential_backoff: false,
                        }),
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("max_attempts must be greater than 0"));
    }

    #[test]
    fn test_validate_file_channel_empty_path() {
        use crate::dsl::schema::{
            ActionSpec, FileNotificationFormat, NotificationChannel, NotificationSpec,
        };
        use std::collections::HashMap;

        let mut workflow = create_test_workflow();

        let task = TaskSpec {
            description: "Test task".to_string(),
            on_complete: Some(ActionSpec {
                notify: Some(NotificationSpec::Structured {
                    message: "Test".to_string(),
                    title: None,
                    priority: None,
                    channels: vec![NotificationChannel::File {
                        path: "   ".to_string(), // Empty path
                        append: true,
                        timestamp: true,
                        format: FileNotificationFormat::Json,
                    }],
                    metadata: HashMap::new(),
                }),
            }),
            ..Default::default()
        };
        workflow.tasks.insert("task1".to_string(), task);

        let result = validate_workflow(&workflow);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("path cannot be empty"));
    }

    #[test]
    fn test_validate_workflow_level_notification_defaults() {
        use crate::dsl::schema::{NotificationChannel, NotificationDefaults};

        let mut workflow = create_test_workflow();

        workflow.notifications = Some(NotificationDefaults {
            notify_on_start: false,
            notify_on_completion: true,
            notify_on_failure: true,
            notify_on_workflow_completion: true,
            default_channels: vec![NotificationChannel::Console {
                colored: true,
                timestamp: true,
            }],
        });

        let result = validate_workflow(&workflow);
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_valid_email_format() {
        assert!(is_valid_email_format("user@example.com"));
        assert!(is_valid_email_format("test.user@example.co.uk"));
        assert!(is_valid_email_format("admin+tag@domain.org"));

        assert!(!is_valid_email_format("invalid"));
        assert!(!is_valid_email_format("@example.com"));
        assert!(!is_valid_email_format("user@"));
        assert!(!is_valid_email_format("user@@example.com"));
        assert!(!is_valid_email_format("user@nodomain"));
    }

    #[test]
    fn test_get_execution_type() {
        use crate::dsl::schema::{CommandSpec, ScriptLanguage, ScriptSpec, TaskSpec};

        // No execution type
        let task = TaskSpec::default();
        assert!(!task.has_execution_type());

        // Single execution type - agent
        let task = TaskSpec {
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };
        assert!(task.has_execution_type());

        // Single execution type - subflow
        let task = TaskSpec {
            subflow: Some("test_subflow".to_string()),
            ..Default::default()
        };
        assert!(task.has_execution_type());

        // Single execution type - script
        let task = TaskSpec {
            script: Some(ScriptSpec {
                language: ScriptLanguage::Python,
                content: Some("print('test')".to_string()),
                file: None,
                working_dir: None,
                env: HashMap::new(),
                timeout_secs: None,
            }),
            ..Default::default()
        };
        assert!(task.has_execution_type());

        // Single execution type - command
        let task = TaskSpec {
            command: Some(CommandSpec {
                executable: "echo".to_string(),
                args: vec!["test".to_string()],
                working_dir: None,
                env: HashMap::new(),
                timeout_secs: None,
                capture_stdout: true,
                capture_stderr: true,
            }),
            ..Default::default()
        };
        assert!(task.has_execution_type());

        // Multiple execution types (also has execution type)
        let task = TaskSpec {
            agent: Some("test_agent".to_string()),
            subflow: Some("test_subflow".to_string()),
            ..Default::default()
        };
        assert!(task.has_execution_type());
    }

    #[test]
    fn test_subtask_execution_type_same_as_parent_valid() {
        use crate::dsl::schema::{AgentSpec, PermissionsSpec, TaskSpec};

        let mut workflow = create_test_workflow();

        // Create an agent
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        // Parent task with agent execution type
        let mut parent_task = TaskSpec {
            description: "Parent task".to_string(),
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };

        // Subtask with same execution type (agent)
        let subtask = TaskSpec {
            description: "Subtask".to_string(),
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };

        let mut subtask_map = HashMap::new();
        subtask_map.insert("subtask1".to_string(), subtask);
        parent_task.subtasks.push(subtask_map);

        workflow.tasks.insert("parent".to_string(), parent_task);

        // Should be valid
        let result = validate_workflow(&workflow);
        assert!(
            result.is_ok(),
            "Expected validation to pass but got: {:?}",
            result
        );
    }

    #[test]
    fn test_subtask_execution_type_inherits_from_parent() {
        use crate::dsl::schema::{AgentSpec, PermissionsSpec, TaskSpec};

        let mut workflow = create_test_workflow();

        // Create an agent
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        // Parent task with agent execution type
        let mut parent_task = TaskSpec {
            description: "Parent task".to_string(),
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };

        // Subtask with NO execution type (will inherit)
        let subtask = TaskSpec {
            description: "Subtask".to_string(),
            ..Default::default()
        };

        let mut subtask_map = HashMap::new();
        subtask_map.insert("subtask1".to_string(), subtask);
        parent_task.subtasks.push(subtask_map);

        workflow.tasks.insert("parent".to_string(), parent_task);

        // Should be valid - subtask inherits agent from parent
        let result = validate_workflow(&workflow);
        assert!(
            result.is_ok(),
            "Expected validation to pass but got: {:?}",
            result
        );
    }

    #[test]
    fn test_subtask_execution_type_different_from_parent_allowed() {
        use crate::dsl::schema::{AgentSpec, PermissionsSpec, SubflowSpec, TaskSpec};

        let mut workflow = create_test_workflow();

        // Create an agent
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        // Create an agent for the subflow
        let subflow_agent = AgentSpec {
            provider: None,
            description: "Subflow agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec![],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        };

        let mut subflow_agents = HashMap::new();
        subflow_agents.insert("subflow_agent".to_string(), subflow_agent);

        let subflow_task = TaskSpec {
            description: "Subflow task".to_string(),
            agent: Some("subflow_agent".to_string()),
            ..Default::default()
        };

        let mut subflow_tasks = HashMap::new();
        subflow_tasks.insert("task1".to_string(), subflow_task);

        // Create a subflow
        workflow.subflows.insert(
            "test_subflow".to_string(),
            SubflowSpec {
                description: Some("Test subflow".to_string()),
                source: None,
                agents: subflow_agents,
                tasks: subflow_tasks,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            },
        );

        // Parent task with agent execution type
        let mut parent_task = TaskSpec {
            description: "Parent task".to_string(),
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };

        // Subtask with DIFFERENT execution type (subflow) - this is now allowed
        let subtask = TaskSpec {
            description: "Subtask".to_string(),
            subflow: Some("test_subflow".to_string()),
            ..Default::default()
        };

        let mut subtask_map = HashMap::new();
        subtask_map.insert("subtask1".to_string(), subtask);
        parent_task.subtasks.push(subtask_map);

        workflow.tasks.insert("parent".to_string(), parent_task);

        // Should now pass validation - subtasks can override with different execution types
        let result = validate_workflow(&workflow);
        assert!(
            result.is_ok(),
            "Expected validation to pass but got: {:?}",
            result
        );
    }

    #[test]
    fn test_subtask_execution_type_different_nested_allowed() {
        use crate::dsl::schema::{
            AgentSpec, PermissionsSpec, ScriptLanguage, ScriptSpec, TaskSpec,
        };

        let mut workflow = create_test_workflow();

        // Create an agent
        workflow.agents.insert(
            "test_agent".to_string(),
            AgentSpec {
                provider: None,
                description: "Test agent".to_string(),
                model: None,
                system_prompt: None,
                cwd: None,
                create_cwd: None,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
                tools: vec![],
                permissions: PermissionsSpec::default(),
                max_turns: None,
            },
        );

        // Parent task with agent execution type
        let mut parent_task = TaskSpec {
            description: "Parent task".to_string(),
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };

        // Intermediate subtask with same type
        let mut intermediate_subtask = TaskSpec {
            description: "Intermediate subtask".to_string(),
            agent: Some("test_agent".to_string()),
            ..Default::default()
        };

        // Nested subtask with DIFFERENT execution type (script) - this is now allowed
        let nested_subtask = TaskSpec {
            description: "Nested subtask".to_string(),
            script: Some(ScriptSpec {
                language: ScriptLanguage::Python,
                content: Some("print('test')".to_string()),
                file: None,
                working_dir: None,
                env: HashMap::new(),
                timeout_secs: None,
            }),
            ..Default::default()
        };

        let mut nested_subtask_map = HashMap::new();
        nested_subtask_map.insert("nested_subtask".to_string(), nested_subtask);
        intermediate_subtask.subtasks.push(nested_subtask_map);

        let mut subtask_map = HashMap::new();
        subtask_map.insert("intermediate".to_string(), intermediate_subtask);
        parent_task.subtasks.push(subtask_map);

        workflow.tasks.insert("parent".to_string(), parent_task);

        // Should now pass validation - subtasks can override with different execution types
        let result = validate_workflow(&workflow);
        assert!(
            result.is_ok(),
            "Expected validation to pass but got: {:?}",
            result
        );
    }

    #[test]
    fn test_subtask_with_own_execution_type_does_not_inherit_agent() {
        use crate::dsl::schema::{ScriptLanguage, ScriptSpec, TaskSpec};

        // Create parent task with agent
        let parent_task = TaskSpec {
            description: "Parent task".to_string(),
            agent: Some("parent_agent".to_string()),
            ..Default::default()
        };

        // Create subtask with script execution type
        let mut subtask = TaskSpec {
            description: "Subtask".to_string(),
            script: Some(ScriptSpec {
                language: ScriptLanguage::Python,
                content: Some("print('test')".to_string()),
                file: None,
                working_dir: None,
                env: HashMap::new(),
                timeout_secs: None,
            }),
            ..Default::default()
        };

        // Apply inheritance
        subtask.inherit_from_parent(&parent_task);

        // Subtask should NOT inherit agent because it has its own execution type
        assert_eq!(
            subtask.agent, None,
            "Subtask should not inherit agent when it has its own execution type"
        );
        assert!(
            subtask.script.is_some(),
            "Subtask should retain its script execution type"
        );
    }

    #[test]
    fn test_subtask_without_execution_type_inherits_agent() {
        use crate::dsl::schema::TaskSpec;

        // Create parent task with agent
        let parent_task = TaskSpec {
            description: "Parent task".to_string(),
            agent: Some("parent_agent".to_string()),
            ..Default::default()
        };

        // Create subtask with NO execution type
        let mut subtask = TaskSpec {
            description: "Subtask".to_string(),
            ..Default::default()
        };

        // Apply inheritance
        subtask.inherit_from_parent(&parent_task);

        // Subtask SHOULD inherit agent because it has no execution type
        assert_eq!(
            subtask.agent,
            Some("parent_agent".to_string()),
            "Subtask should inherit agent when it has no execution type"
        );
    }

    #[test]
    fn test_parent_no_execution_type_subtask_any_type_valid() {
        use crate::dsl::schema::{AgentSpec, PermissionsSpec, SubflowSpec, TaskSpec};

        let mut workflow = create_test_workflow();

        // Create an agent for the subflow
        let subflow_agent = AgentSpec {
            provider: None,
            description: "Subflow agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec![],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        };

        let mut subflow_agents = HashMap::new();
        subflow_agents.insert("subflow_agent".to_string(), subflow_agent);

        let subflow_task = TaskSpec {
            description: "Subflow task".to_string(),
            agent: Some("subflow_agent".to_string()),
            ..Default::default()
        };

        let mut subflow_tasks = HashMap::new();
        subflow_tasks.insert("task1".to_string(), subflow_task);

        // Create a subflow with agents and tasks
        workflow.subflows.insert(
            "test_subflow".to_string(),
            SubflowSpec {
                description: Some("Test subflow".to_string()),
                source: None,
                agents: subflow_agents,
                tasks: subflow_tasks,
                inputs: HashMap::new(),
                outputs: HashMap::new(),
            },
        );

        // Parent task with NO execution type (organizational parent)
        let mut parent_task = TaskSpec {
            description: "Parent task".to_string(),
            ..Default::default()
        };

        // Subtask with subflow execution type
        let subtask = TaskSpec {
            description: "Subtask".to_string(),
            subflow: Some("test_subflow".to_string()),
            ..Default::default()
        };

        let mut subtask_map = HashMap::new();
        subtask_map.insert("subtask1".to_string(), subtask);
        parent_task.subtasks.push(subtask_map);

        workflow.tasks.insert("parent".to_string(), parent_task);

        // Should be valid - parent has no execution type, so subtask can use any
        let result = validate_workflow(&workflow);
        assert!(
            result.is_ok(),
            "Expected validation to pass but got: {:?}",
            result
        );
    }
}
