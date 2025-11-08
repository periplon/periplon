//! DSL Executor
//!
//! This module provides the execution engine for DSL workflows, managing agent lifecycle,
//! task scheduling, and workflow orchestration.

use crate::adapters::primary::PeriplonSDKClient;
use crate::dsl::hooks::{ErrorRecovery, HooksExecutor};
use crate::dsl::loop_context::{substitute_task_variables, LoopContext};
use crate::dsl::message_bus::MessageBus;
use crate::dsl::notifications::{NotificationContext, NotificationManager};
use crate::dsl::schema::{AgentSpec, CollectionSource, DSLWorkflow, FileFormat, LoopSpec};
use crate::dsl::state::{StatePersistence, WorkflowState};
use crate::dsl::task_graph::{TaskGraph, TaskStatus};
use crate::error::{Error, Result};
use crate::options::AgentOptions;
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, Semaphore};

/// Execution context for loop and task execution
/// Groups commonly-passed parameters to avoid too_many_arguments clippy warnings
struct ExecutionContext<'a> {
    workflow_inputs: &'a HashMap<String, serde_json::Value>,
    agents: &'a Arc<Mutex<HashMap<String, PeriplonSDKClient>>>,
    task_graph: &'a Arc<Mutex<TaskGraph>>,
    state: &'a Arc<Mutex<Option<WorkflowState>>>,
    workflow_name: &'a Arc<String>,
    json_output: bool,
}

/// DSL Executor for running workflows
pub struct DSLExecutor {
    workflow: DSLWorkflow,
    agents: HashMap<String, PeriplonSDKClient>,
    task_graph: TaskGraph,
    message_bus: Arc<MessageBus>,
    state: Option<WorkflowState>,
    state_persistence: Option<StatePersistence>,
    resolved_inputs: HashMap<String, serde_json::Value>,
    notification_manager: Arc<NotificationManager>,
    workflow_start_time: Option<Instant>,
    json_output: bool,

    // Debugging infrastructure
    debugger: Option<Arc<Mutex<crate::dsl::debugger::DebuggerState>>>,
    inspector: Option<Arc<crate::dsl::debugger::Inspector>>,
}

impl DSLExecutor {
    /// Create a new DSL executor
    ///
    /// # Arguments
    ///
    /// * `workflow` - The workflow to execute
    ///
    /// # Returns
    ///
    /// Result containing the executor or an error
    pub fn new(workflow: DSLWorkflow) -> Result<Self> {
        // Resolve workflow inputs (use defaults)
        let resolved_inputs = Self::resolve_workflow_inputs(&workflow);

        // Initialize notification manager
        let notification_manager = Arc::new(NotificationManager::new());

        Ok(DSLExecutor {
            workflow,
            agents: HashMap::new(),
            task_graph: TaskGraph::new(),
            message_bus: Arc::new(MessageBus::new()),
            state: None,
            state_persistence: None,
            resolved_inputs,
            notification_manager,
            workflow_start_time: None,
            json_output: false,
            debugger: None,
            inspector: None,
        })
    }

    /// Get a reference to the task graph (for testing and inspection)
    pub fn task_graph(&self) -> &TaskGraph {
        &self.task_graph
    }

    /// Enable debugging mode
    ///
    /// Returns self with debugging infrastructure initialized
    pub fn with_debugger(mut self) -> Self {
        let debugger = Arc::new(Mutex::new(crate::dsl::debugger::DebuggerState::new()));
        let state = Arc::new(Mutex::new(self.state.clone()));
        let inspector = Arc::new(crate::dsl::debugger::Inspector::new(
            debugger.clone(),
            state,
        ));

        self.debugger = Some(debugger);
        self.inspector = Some(inspector);
        self
    }

    /// Get debugger reference (if debugging is enabled)
    pub fn debugger(&self) -> Option<&Arc<Mutex<crate::dsl::debugger::DebuggerState>>> {
        self.debugger.as_ref()
    }

    /// Get inspector reference (if debugging is enabled)
    pub fn inspector(&self) -> Option<&Arc<crate::dsl::debugger::Inspector>> {
        self.inspector.as_ref()
    }

    /// Check if debugging is enabled
    pub fn is_debug_mode(&self) -> bool {
        self.debugger.is_some()
    }

    /// Resolve workflow inputs by extracting default values
    fn resolve_workflow_inputs(workflow: &DSLWorkflow) -> HashMap<String, serde_json::Value> {
        let mut resolved = HashMap::new();

        for (key, input_spec) in &workflow.inputs {
            if let Some(default_value) = &input_spec.default {
                resolved.insert(key.clone(), default_value.clone());
            }
        }

        resolved
    }

    /// Substitute workflow and task variables in a string
    ///
    /// Supports:
    /// - ${workflow.variable_name} - Replace with workflow input value
    /// - ${task.variable_name} - Replace with task input value
    /// - {{workflow.variable_name}} - Replace with workflow input value (curly syntax)
    /// - {{task.variable_name}} - Replace with task input value (curly syntax)
    fn substitute_variables(
        text: &str,
        workflow_inputs: &HashMap<String, serde_json::Value>,
        task_inputs: &HashMap<String, serde_json::Value>,
    ) -> String {
        let mut result = text.to_string();

        // Replace ${workflow.variable} with workflow input values
        for (key, value) in workflow_inputs {
            let placeholder_dollar = format!("${{workflow.{}}}", key);
            let placeholder_curly = format!("{{{{workflow.{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            result = result.replace(&placeholder_dollar, &value_str);
            result = result.replace(&placeholder_curly, &value_str);
        }

        // Replace ${task.variable} with task input values
        for (key, value) in task_inputs {
            let placeholder_dollar = format!("${{task.{}}}", key);
            let placeholder_curly = format!("{{{{task.{}}}}}", key);
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            result = result.replace(&placeholder_dollar, &value_str);
            result = result.replace(&placeholder_curly, &value_str);
        }

        result
    }

    /// Substitute variables including task outputs
    ///
    /// Supports all variable types including:
    /// - {{task.task_name.output}} - Replace with task output
    /// - ${task.task_name.output} - Replace with task output (dollar syntax)
    fn substitute_variables_with_state(
        text: &str,
        workflow_inputs: &HashMap<String, serde_json::Value>,
        task_inputs: &HashMap<String, serde_json::Value>,
        workflow_state: Option<&crate::dsl::state::WorkflowState>,
    ) -> String {
        let mut result = Self::substitute_variables(text, workflow_inputs, task_inputs);

        // Replace task output references if state is available
        if let Some(state) = workflow_state {
            // Use regex to find all {{task.task_name.output}} and ${task.task_name.output} patterns
            use regex::Regex;

            // Pattern for {{task.name.output}}
            if let Ok(re) = Regex::new(r"\{\{task\.([a-zA-Z0-9_-]+)\.output\}\}") {
                result = re
                    .replace_all(&result, |caps: &regex::Captures| {
                        let task_name = &caps[1];
                        if let Some(task_output) = state.get_task_output(task_name) {
                            task_output.content.clone()
                        } else {
                            caps[0].to_string() // Keep original if not found
                        }
                    })
                    .to_string();
            }

            // Pattern for ${task.name.output}
            if let Ok(re) = Regex::new(r"\$\{task\.([a-zA-Z0-9_-]+)\.output\}") {
                result = re
                    .replace_all(&result, |caps: &regex::Captures| {
                        let task_name = &caps[1];
                        if let Some(task_output) = state.get_task_output(task_name) {
                            task_output.content.clone()
                        } else {
                            caps[0].to_string() // Keep original if not found
                        }
                    })
                    .to_string();
            }
        }

        result
    }

    /// Create a notification context with task metadata
    ///
    /// # Arguments
    ///
    /// * `task_id` - ID of the task
    /// * `task_status` - Status of the task (completed, failed, etc.)
    /// * `duration` - Task execution duration
    /// * `error_msg` - Optional error message
    fn create_notification_context(
        &self,
        task_id: &str,
        task_status: &str,
        duration: Option<std::time::Duration>,
        error_msg: Option<&str>,
    ) -> NotificationContext {
        let mut context = NotificationContext::new();

        // Add workflow-level variables
        for (key, value) in &self.resolved_inputs {
            let value_str = match value {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => serde_json::to_string(value).unwrap_or_default(),
            };
            context = context.with_workflow_var(key, value_str);
        }

        // Add task metadata
        context = context
            .with_metadata("task_id", task_id)
            .with_metadata("task_status", task_status)
            .with_metadata("workflow_name", &self.workflow.name);

        if let Some(dur) = duration {
            context = context.with_metadata("duration_secs", dur.as_secs().to_string());
            context = context.with_metadata("duration_human", format!("{:.2}s", dur.as_secs_f64()));
        }

        if let Some(err) = error_msg {
            context = context.with_metadata("error", err);
        }

        // Add secrets from workflow (just the keys, not values)
        for secret_name in self.workflow.secrets.keys() {
            // Secrets are resolved at runtime via environment or other sources
            // For now, we'll just mark them as available in the context
            context = context.with_secret(secret_name, format!("${{secret.{}}}", secret_name));
        }

        context
    }

    /// Enable state persistence for this workflow
    ///
    /// # Arguments
    ///
    /// * `state_dir` - Directory to store state files (default: ".workflow_states")
    pub fn enable_state_persistence(&mut self, state_dir: Option<&str>) -> Result<()> {
        let persistence = if let Some(dir) = state_dir {
            StatePersistence::new(dir)?
        } else {
            StatePersistence::default()
        };

        self.state_persistence = Some(persistence);

        // Try to load existing state for resume
        if let Some(ref persistence) = self.state_persistence {
            if persistence.has_state(&self.workflow.name) {
                println!(
                    "Found existing state for workflow '{}' - will resume if possible",
                    self.workflow.name
                );
            }
        }

        Ok(())
    }

    /// Set JSON output mode
    ///
    /// When enabled, messages are output in full JSON format for programmatic processing.
    /// When disabled (default), messages are formatted for interactive terminal display
    /// with colors and condensed output.
    ///
    /// # Arguments
    ///
    /// * `json` - True for JSON mode, false for interactive mode
    pub fn set_json_output(&mut self, json: bool) {
        self.json_output = json;
    }

    /// Try to resume from saved state
    ///
    /// Returns true if workflow was resumed, false if starting fresh
    pub fn try_resume(&mut self) -> Result<bool> {
        if let Some(ref persistence) = self.state_persistence {
            if persistence.has_state(&self.workflow.name) {
                let saved_state = persistence.load_state(&self.workflow.name)?;

                if saved_state.can_resume() {
                    println!(
                        "Resuming workflow '{}' from checkpoint (progress: {:.1}%)",
                        self.workflow.name,
                        saved_state.get_progress() * 100.0
                    );

                    self.state = Some(saved_state);
                    return Ok(true);
                } else {
                    println!(
                        "Cannot resume workflow '{}' - status: {:?}",
                        self.workflow.name, saved_state.status
                    );
                }
            }
        }

        Ok(false)
    }

    /// Checkpoint the current workflow state
    fn checkpoint_state(&mut self) -> Result<()> {
        if let (Some(ref mut state), Some(ref persistence)) =
            (&mut self.state, &self.state_persistence)
        {
            persistence.save_state(state)?;
        }
        Ok(())
    }

    /// Get a reference to the message bus
    pub fn message_bus(&self) -> Arc<MessageBus> {
        self.message_bus.clone()
    }

    /// Get the current workflow state
    pub fn get_state(&self) -> Option<&WorkflowState> {
        self.state.as_ref()
    }

    /// Initialize the executor by creating agents and building the task graph
    pub async fn initialize(&mut self) -> Result<()> {
        // Register agents with message bus
        for agent_name in self.workflow.agents.keys() {
            self.message_bus.register_agent(agent_name.clone()).await?;
        }

        // Create communication channels from workflow configuration
        if let Some(comm_config) = &self.workflow.communication {
            for (channel_name, channel_spec) in &comm_config.channels {
                self.message_bus
                    .create_channel(
                        channel_name.clone(),
                        channel_spec.description.clone(),
                        channel_spec.participants.clone(),
                        channel_spec.message_format.clone(),
                    )
                    .await?;
            }
        }

        // Create variable context for interpolation
        let mut var_context = crate::dsl::variables::VariableContext::new();
        for (key, value) in &self.resolved_inputs {
            var_context.insert(&crate::dsl::variables::Scope::Workflow, key, value.clone());
        }

        // Create agent instances
        for (name, spec) in &self.workflow.agents {
            let options = self.agent_spec_to_options(
                spec,
                self.workflow.cwd.as_deref(),
                self.workflow.create_cwd,
                &var_context,
            )?;
            let mut client = PeriplonSDKClient::new(options);
            client.connect(None).await?;
            self.agents.insert(name.clone(), client);
        }

        // Build task graph with hierarchical tasks flattened
        // Collect tasks first to avoid borrow checker issues
        let tasks: Vec<(String, crate::dsl::schema::TaskSpec)> = self
            .workflow
            .tasks
            .iter()
            .map(|(name, spec)| (name.clone(), spec.clone()))
            .collect();

        for (name, spec) in tasks {
            self.add_hierarchical_task(&name, &spec, None)?;
        }

        println!(
            "Initialized {} agents and {} channels",
            self.message_bus.agent_count().await,
            self.message_bus.channel_count().await
        );

        // Initialize workflow state if persistence is enabled and not resuming
        if self.state_persistence.is_some() && self.state.is_none() {
            let mut state =
                WorkflowState::new(self.workflow.name.clone(), self.workflow.version.clone());

            // Register all tasks in state
            for task_id in self.task_graph.topological_sort()? {
                state.update_task_status(&task_id, TaskStatus::Pending);
            }

            self.state = Some(state);
            println!("Initialized workflow state tracking");
        }

        Ok(())
    }

    /// Add a task and its subtasks to the graph recursively
    ///
    /// # Arguments
    ///
    /// * `task_name` - Name of the task
    /// * `task_spec` - Task specification
    /// * `parent_name` - Optional parent task name for dependency tracking
    fn add_hierarchical_task(
        &mut self,
        task_name: &str,
        task_spec: &crate::dsl::schema::TaskSpec,
        parent_name: Option<&str>,
    ) -> Result<()> {
        // Create a modified spec for this task (without subtasks, as they'll be separate)
        let mut task_spec_flat = task_spec.clone();

        // If this task has subtasks, we'll add dependencies to them
        let has_subtasks = !task_spec.subtasks.is_empty();

        // Build a map of sibling task names for dependency resolution
        let sibling_names: std::collections::HashSet<String> = if let Some(parent) = parent_name {
            // Get parent task spec to find all siblings
            if let Some(parent_task) = self.workflow.tasks.get(parent) {
                parent_task
                    .subtasks
                    .iter()
                    .flat_map(|map| map.keys())
                    .map(|s| s.to_string())
                    .collect()
            } else {
                std::collections::HashSet::new()
            }
        } else {
            std::collections::HashSet::new()
        };

        // Resolve dependency names to full hierarchical names for sibling references
        if let Some(parent) = parent_name {
            task_spec_flat.depends_on = task_spec_flat
                .depends_on
                .iter()
                .map(|dep| {
                    if sibling_names.contains(dep) {
                        format!("{}.{}", parent, dep)
                    } else {
                        dep.clone()
                    }
                })
                .collect();

            // Also resolve parallel_with references
            task_spec_flat.parallel_with = task_spec_flat
                .parallel_with
                .iter()
                .map(|task| {
                    if sibling_names.contains(task) {
                        format!("{}.{}", parent, task)
                    } else {
                        task.clone()
                    }
                })
                .collect();
        }

        // Clear subtasks from the flat spec (they'll be added as separate tasks)
        // EXCEPT if this task has a loop_spec - in that case we need the subtasks
        // to execute them within each loop iteration
        let has_loop = task_spec_flat.loop_spec.is_some();
        if !has_loop {
            task_spec_flat.subtasks.clear();
        }

        // Determine if this is an executable task
        // A task is executable if it has an execution type AND no subtasks
        // OR if it has no execution type AND no subtasks (leaf task)
        // OR if it has a loop_spec (loops with subtasks need to execute the subtasks within each iteration)
        // Tasks with subtasks but no loop are NOT executable (they're organizational parents)
        let is_executable = !has_subtasks || has_loop;

        // If this task has a parent, add parent as dependency only if parent is executable
        if let Some(parent) = parent_name {
            // Check if parent task is executable by looking it up
            // A parent is only executable if it has no subtasks
            let parent_is_executable = self
                .workflow
                .tasks
                .get(parent)
                .map(|parent_spec| parent_spec.subtasks.is_empty())
                .unwrap_or(true);

            if parent_is_executable && !task_spec_flat.depends_on.contains(&parent.to_string()) {
                task_spec_flat.depends_on.push(parent.to_string());
            }
        }

        // Resolve dependencies on non-executable parent tasks to their leaf subtasks
        let mut resolved_dependencies = Vec::new();
        for dep in &task_spec_flat.depends_on {
            if let Some(dep_task) = self.workflow.tasks.get(dep) {
                // Check if the dependency is a non-executable parent (has subtasks but no loop)
                // Tasks with loops are executable even if they have subtasks
                let dep_has_loop = dep_task.loop_spec.is_some();
                let is_non_executable_parent = !dep_task.subtasks.is_empty() && !dep_has_loop;

                if is_non_executable_parent {
                    // Resolve to leaf subtasks - we need to collect all leaf tasks of this parent
                    // For now, add all immediate subtasks (this will be further resolved recursively)
                    for subtask_map in &dep_task.subtasks {
                        for subtask_name in subtask_map.keys() {
                            resolved_dependencies.push(format!("{}.{}", dep, subtask_name));
                        }
                    }
                } else {
                    // Task is executable (either has no subtasks, or has loop+subtasks)
                    resolved_dependencies.push(dep.clone());
                }
            } else {
                // Dependency not found in workflow tasks, keep as-is (might be a hierarchical reference)
                resolved_dependencies.push(dep.clone());
            }
        }
        task_spec_flat.depends_on = resolved_dependencies;

        // Add this task to the graph only if it's an executable task
        if is_executable {
            self.task_graph
                .add_task(task_name.to_string(), task_spec_flat);
        }

        // Recursively add subtasks
        // EXCEPT if the parent has a loop - in that case, subtasks are managed by the loop executor
        if has_subtasks && !has_loop {
            for subtask_map in &task_spec.subtasks {
                for (subtask_name, subtask_spec) in subtask_map {
                    let full_subtask_name = format!("{}.{}", task_name, subtask_name);

                    // Apply inheritance from parent task to subtask
                    let mut inherited_subtask_spec = subtask_spec.clone();
                    inherited_subtask_spec.inherit_from_parent(task_spec);

                    // Subtasks depend on their parent task by default
                    self.add_hierarchical_task(
                        &full_subtask_name,
                        &inherited_subtask_spec,
                        Some(task_name),
                    )?;
                }
            }
        }

        Ok(())
    }

    // ========================================================================
    // Debug Helper Methods
    // ========================================================================

    /// Check if should pause at task (for debugging)
    async fn check_debug_pause(&self, task_id: &str) -> bool {
        if let Some(ref debugger) = self.debugger {
            let dbg = debugger.lock().await;
            dbg.should_pause(task_id)
        } else {
            false
        }
    }

    /// Record task entry in debugger
    async fn debug_enter_task(&self, task_id: &str, parent_task: Option<&str>) {
        if let Some(ref debugger) = self.debugger {
            let mut dbg = debugger.lock().await;
            dbg.enter_task(task_id.to_string(), parent_task.map(|s| s.to_string()));
        }
    }

    /// Record task exit in debugger
    async fn debug_exit_task(&self) {
        if let Some(ref debugger) = self.debugger {
            let mut dbg = debugger.lock().await;
            dbg.exit_task();
        }
    }

    /// Create execution snapshot
    #[allow(dead_code)]
    async fn debug_create_snapshot(&self, description: String) {
        if let Some(ref debugger) = self.debugger {
            // Get current state
            if let Some(ref state) = self.state {
                let mut dbg = debugger.lock().await;
                dbg.create_snapshot(state, description);
            }
        }
    }

    /// Record side effect (file operation, state change, etc.)
    #[allow(dead_code)]
    async fn debug_record_side_effect(
        &self,
        task_id: &str,
        effect_type: crate::dsl::debugger::SideEffectType,
        compensation: Arc<dyn crate::dsl::debugger::CompensationStrategy>,
    ) {
        if let Some(ref debugger) = self.debugger {
            let mut dbg = debugger.lock().await;
            dbg.side_effects
                .record(task_id.to_string(), effect_type, compensation);
        }
    }

    /// Wait for user input when paused (for interactive debugging)
    async fn debug_wait_for_continue(&self) {
        if let Some(ref debugger) = self.debugger {
            loop {
                let should_wait = {
                    let dbg = debugger.lock().await;
                    matches!(
                        dbg.mode,
                        crate::dsl::debugger::DebugMode::Paused
                            | crate::dsl::debugger::DebugMode::Suspended
                    )
                };

                if !should_wait {
                    break;
                }

                // Sleep briefly to avoid busy-waiting
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    /// Execute the workflow
    ///
    /// Executes tasks in topological order, respecting dependencies
    /// with support for parallel execution and hooks
    pub async fn execute(&mut self) -> Result<()> {
        // Run pre-workflow hooks if they exist
        if let Some(workflows) = self.workflow.workflows.values().next() {
            if let Some(hooks) = &workflows.hooks {
                if !hooks.pre_workflow.is_empty() {
                    HooksExecutor::execute_pre_workflow(&hooks.pre_workflow, &self.workflow.name)
                        .await?;
                }
            }
        }

        // Execute the workflow and handle errors
        let execution_result = self.execute_tasks().await;

        // Run post-workflow hooks if they exist (even on error)
        if let Some(workflows) = self.workflow.workflows.values().next() {
            if let Some(hooks) = &workflows.hooks {
                if !hooks.post_workflow.is_empty() {
                    let _ = HooksExecutor::execute_post_workflow(
                        &hooks.post_workflow,
                        &self.workflow.name,
                    )
                    .await;
                }
            }
        }

        // If execution failed, run error hooks
        if let Err(ref e) = execution_result {
            if let Some(workflows) = self.workflow.workflows.values().next() {
                if let Some(hooks) = &workflows.hooks {
                    if !hooks.on_error.is_empty() {
                        let _ = HooksExecutor::execute_error(
                            &hooks.on_error,
                            &self.workflow.name,
                            &e.to_string(),
                        )
                        .await;
                    }
                }
            }

            // Mark workflow as failed in state
            if let Some(ref mut state) = self.state {
                state.mark_failed();
                let _ = self.checkpoint_state();
            }
        } else {
            // Mark workflow as completed in state
            if let Some(ref mut state) = self.state {
                state.mark_completed();
                let _ = self.checkpoint_state();
            }
        }

        execution_result
    }

    /// Execute all tasks in the workflow
    async fn execute_tasks(&mut self) -> Result<()> {
        // Record workflow start time
        self.workflow_start_time = Some(Instant::now());

        // Initialize debugger for workflow execution
        if let Some(ref debugger) = self.debugger {
            let mut dbg = debugger.lock().await;
            dbg.start();
            println!("🐛 Debug mode enabled");
        }

        // Get execution order
        let order = self.task_graph.topological_sort()?;

        println!("Executing workflow: {}", self.workflow.name);
        println!("Task execution order: {:?}", order);

        // Create initial snapshot if debugging
        if self.is_debug_mode() {
            if let Some(ref state) = self.state {
                if let Some(ref debugger) = self.debugger {
                    let mut dbg = debugger.lock().await;
                    dbg.create_snapshot(state, "Workflow start".to_string());
                    println!("📸 Created initial snapshot");
                }
            }
        }

        // Send workflow start notification if configured
        if let Some(notif_config) = &self.workflow.notifications {
            if notif_config.notify_on_start && !notif_config.default_channels.is_empty() {
                let context = self.create_notification_context("workflow", "started", None, None);
                let spec = crate::dsl::schema::NotificationSpec::Structured {
                    message: format!("Workflow '{}' started", self.workflow.name),
                    title: Some(format!("Workflow Started: {}", self.workflow.name)),
                    priority: Some(crate::dsl::schema::NotificationPriority::Normal),
                    channels: notif_config.default_channels.clone(),
                    metadata: HashMap::new(),
                };
                if let Err(e) = self.notification_manager.send(&spec, &context).await {
                    eprintln!("Warning: Failed to send workflow start notification: {}", e);
                }
            }
        }

        // Wrap agents, task graph, and resolved inputs in Arc for parallel access
        let agents = Arc::new(Mutex::new(std::mem::take(&mut self.agents)));
        let task_graph = Arc::new(Mutex::new(std::mem::take(&mut self.task_graph)));
        let state = Arc::new(Mutex::new(self.state.take()));
        let workflow_inputs = Arc::new(self.resolved_inputs.clone());
        let state_persistence = Arc::new(self.state_persistence.clone());
        let workflow_name = Arc::new(self.workflow.name.clone());

        // Track which tasks we've already processed
        let mut processed = std::collections::HashSet::new();

        for task_id in order {
            // Skip if already processed (as part of parallel group)
            if processed.contains(&task_id) {
                continue;
            }

            // Skip if task was already completed (resume functionality)
            if let Some(ref workflow_state) = *state.lock().await {
                if workflow_state.get_task_status(&task_id) == Some(TaskStatus::Completed) {
                    println!("Skipping already completed task: {}", task_id);
                    processed.insert(task_id.clone());
                    continue;
                }
            }

            // Get parallel tasks
            let parallel_tasks = {
                let graph = task_graph.lock().await;
                graph.get_parallel_tasks(&task_id)
            };

            if parallel_tasks.is_empty() {
                // === DEBUG: Check breakpoints and pause if needed ===
                if self.is_debug_mode() {
                    // Check if should pause at this task
                    if self.check_debug_pause(&task_id).await {
                        println!("⏸️  Breakpoint hit at task: {}", task_id);

                        // Display debugger status
                        if let Some(ref debugger) = self.debugger {
                            let dbg = debugger.lock().await;
                            let status = dbg.status_summary();
                            println!("{}", status);
                        }

                        // Wait for user to continue
                        println!("⏸️  Execution paused. Waiting for continue...");
                        self.debug_wait_for_continue().await;
                        println!("▶️  Execution resumed");
                    }

                    // Create snapshot before task execution
                    if let Some(ref workflow_state) = *state.lock().await {
                        if let Some(ref debugger) = self.debugger {
                            let mut dbg = debugger.lock().await;
                            dbg.create_snapshot(
                                workflow_state,
                                format!("Before task: {}", task_id),
                            );
                            println!("📸 Snapshot created before task: {}", task_id);
                        }
                    }

                    // Record task entry
                    self.debug_enter_task(&task_id, None).await;
                }

                // Execute task sequentially
                let task_result = self
                    .execute_task_parallel(
                        &task_id,
                        agents.clone(),
                        task_graph.clone(),
                        state.clone(),
                        workflow_inputs.clone(),
                        state_persistence.clone(),
                    )
                    .await;

                // === DEBUG: Post-execution hooks ===
                if self.is_debug_mode() {
                    // Record task exit
                    self.debug_exit_task().await;

                    // Create snapshot after task execution
                    if let Some(ref workflow_state) = *state.lock().await {
                        if let Some(ref debugger) = self.debugger {
                            let mut dbg = debugger.lock().await;
                            let description = if task_result.is_ok() {
                                format!("After task: {} (success)", task_id)
                            } else {
                                format!("After task: {} (failed)", task_id)
                            };
                            dbg.create_snapshot(workflow_state, description);
                            println!("📸 Snapshot created after task: {}", task_id);
                        }
                    }
                }

                // Propagate errors
                task_result?;
                processed.insert(task_id.clone());
            } else {
                // Execute tasks in parallel using tokio::spawn
                println!(
                    "Executing tasks in parallel: {} with {:?}",
                    task_id, parallel_tasks
                );

                let mut handles = vec![];

                // Spawn main task
                {
                    let task_id = task_id.clone();
                    let agents = agents.clone();
                    let graph = task_graph.clone();
                    let workflow_state = state.clone();
                    let inputs = workflow_inputs.clone();
                    let persistence = state_persistence.clone();
                    let wf_name = workflow_name.clone();
                    let json_out = self.json_output;

                    let handle = tokio::spawn(async move {
                        execute_task_static(
                            task_id,
                            agents,
                            graph,
                            workflow_state,
                            inputs,
                            persistence,
                            wf_name,
                            json_out,
                        )
                        .await
                    });
                    handles.push(handle);
                }

                // Spawn parallel tasks
                for parallel_id in &parallel_tasks {
                    let status = {
                        let graph = task_graph.lock().await;
                        graph.get_task_status(parallel_id)
                    };

                    if status == Some(TaskStatus::Pending) {
                        let parallel_id = parallel_id.clone();
                        let agents = agents.clone();
                        let graph = task_graph.clone();
                        let workflow_state = state.clone();
                        let inputs = workflow_inputs.clone();
                        let persistence = state_persistence.clone();
                        let wf_name = workflow_name.clone();
                        let json_out = self.json_output;

                        let handle = tokio::spawn(async move {
                            execute_task_static(
                                parallel_id,
                                agents,
                                graph,
                                workflow_state,
                                inputs,
                                persistence,
                                wf_name,
                                json_out,
                            )
                            .await
                        });
                        handles.push(handle);
                    }
                }

                // Wait for all parallel tasks to complete
                for handle in handles {
                    match handle.await {
                        Ok(Ok(())) => {}
                        Ok(Err(e)) => return Err(e),
                        Err(e) => return Err(Error::InvalidInput(format!("Task panicked: {}", e))),
                    }
                }

                processed.insert(task_id.clone());
                for parallel_id in parallel_tasks {
                    processed.insert(parallel_id);
                }
            }
        }

        // Restore agents, task graph, and state
        self.agents = Arc::try_unwrap(agents)
            .map_err(|_| Error::InvalidInput("Failed to unwrap agents".to_string()))?
            .into_inner();
        self.task_graph = Arc::try_unwrap(task_graph)
            .map_err(|_| Error::InvalidInput("Failed to unwrap task graph".to_string()))?
            .into_inner();
        self.state = Arc::try_unwrap(state)
            .map_err(|_| Error::InvalidInput("Failed to unwrap state".to_string()))?
            .into_inner();

        // Checkpoint state after execution
        if self.state.is_some() {
            let _ = self.checkpoint_state();
        }

        println!("Workflow execution completed");

        // Send workflow completion notification if configured
        if let Some(notif_config) = &self.workflow.notifications {
            if notif_config.notify_on_workflow_completion
                && !notif_config.default_channels.is_empty()
            {
                let duration = self.workflow_start_time.map(|start| start.elapsed());
                let context =
                    self.create_notification_context("workflow", "completed", duration, None);
                let spec = crate::dsl::schema::NotificationSpec::Structured {
                    message: format!("Workflow '{}' completed successfully", self.workflow.name),
                    title: Some(format!("Workflow Completed: {}", self.workflow.name)),
                    priority: Some(crate::dsl::schema::NotificationPriority::Normal),
                    channels: notif_config.default_channels.clone(),
                    metadata: HashMap::new(),
                };
                if let Err(e) = self.notification_manager.send(&spec, &context).await {
                    eprintln!(
                        "Warning: Failed to send workflow completion notification: {}",
                        e
                    );
                }
            }
        }

        // === DEBUG: Finalize debugger ===
        if self.is_debug_mode() {
            // Create final snapshot
            if let Some(ref state) = self.state {
                if let Some(ref debugger) = self.debugger {
                    let mut dbg = debugger.lock().await;
                    dbg.create_snapshot(state, "Workflow completed".to_string());
                    println!("📸 Created final snapshot");

                    // Display final debug summary
                    let status = dbg.status_summary();
                    println!("\n🐛 Debug Session Summary:");
                    println!("{}", status);

                    // Display side effect summary
                    let side_effect_summary = dbg.side_effects.summary();
                    if !side_effect_summary.is_empty() {
                        println!("\n📝 Side Effects Summary:");
                        for (effect_type, count) in side_effect_summary {
                            println!("  {} x {}", count, effect_type);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a single task with shared state (for parallel execution)
    async fn execute_task_parallel(
        &self,
        task_id: &str,
        agents: Arc<Mutex<HashMap<String, PeriplonSDKClient>>>,
        task_graph: Arc<Mutex<TaskGraph>>,
        state: Arc<Mutex<Option<WorkflowState>>>,
        workflow_inputs: Arc<HashMap<String, serde_json::Value>>,
        state_persistence: Arc<Option<StatePersistence>>,
    ) -> Result<()> {
        let workflow_name = Arc::new(self.workflow.name.clone());
        execute_task_static(
            task_id.to_string(),
            agents,
            task_graph,
            state,
            workflow_inputs,
            state_persistence,
            workflow_name,
            self.json_output,
        )
        .await
    }

    /// Convert DSL AgentSpec to SDK AgentOptions
    ///
    /// # Arguments
    ///
    /// * `spec` - Agent specification
    /// * `workflow_cwd` - Workflow-level working directory (optional)
    /// * `workflow_create_cwd` - Workflow-level create_cwd flag (optional)
    ///
    /// # Working Directory Priority
    ///
    /// 1. Agent-level cwd (highest priority)
    /// 2. Workflow-level cwd
    /// 3. None (current directory - backwards compatible)
    ///
    /// # Create CWD Priority
    ///
    /// 1. Agent-level create_cwd (highest priority)
    /// 2. Workflow-level create_cwd
    /// 3. false (backwards compatible - don't create)
    fn agent_spec_to_options(
        &self,
        spec: &AgentSpec,
        workflow_cwd: Option<&str>,
        workflow_create_cwd: Option<bool>,
        var_context: &crate::dsl::variables::VariableContext,
    ) -> Result<AgentOptions> {
        let add_dirs = spec
            .permissions
            .allowed_directories
            .iter()
            .map(std::path::PathBuf::from)
            .collect();

        // Interpolate workflow cwd if present
        let interpolated_workflow_cwd = workflow_cwd.map(|cwd| {
            var_context
                .interpolate(cwd)
                .unwrap_or_else(|_| cwd.to_string())
        });

        // Interpolate agent cwd if present
        let interpolated_agent_cwd = spec
            .cwd
            .as_ref()
            .map(|cwd| var_context.interpolate(cwd).unwrap_or_else(|_| cwd.clone()));

        // Determine working directory with cascading defaults
        let cwd = interpolated_agent_cwd
            .as_deref()
            .or(interpolated_workflow_cwd.as_deref())
            .map(std::path::PathBuf::from);

        // Determine create_cwd with cascading defaults
        let create_cwd = spec.create_cwd.or(workflow_create_cwd).unwrap_or(false);

        let options = AgentOptions {
            allowed_tools: spec.tools.clone(),
            model: spec.model.clone(),
            max_turns: spec.max_turns,
            permission_mode: Some(spec.permissions.mode.clone()),
            add_dirs,
            cwd,
            create_cwd,
            ..Default::default()
        };
        Ok(options)
    }

    /// Shutdown the executor and disconnect all agents
    pub async fn shutdown(&mut self) -> Result<()> {
        println!("Shutting down executor...");

        for (name, agent) in &mut self.agents {
            println!("Disconnecting agent: {}", name);
            agent.disconnect().await?;
        }

        println!("Executor shutdown complete");
        Ok(())
    }

    /// Get workflow information
    pub fn get_workflow_info(&self) -> (&str, &str) {
        (&self.workflow.name, &self.workflow.version)
    }

    /// Get task count
    pub fn get_task_count(&self) -> usize {
        self.task_graph.task_count()
    }

    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        self.task_graph.is_complete()
    }
}

/// Static function for executing a task (used in parallel execution)
///
/// This function can be called from tokio::spawn and includes retry logic
#[allow(clippy::too_many_arguments)]
async fn execute_task_static(
    task_id: String,
    agents: Arc<Mutex<HashMap<String, PeriplonSDKClient>>>,
    task_graph: Arc<Mutex<TaskGraph>>,
    state: Arc<Mutex<Option<WorkflowState>>>,
    workflow_inputs: Arc<HashMap<String, serde_json::Value>>,
    state_persistence: Arc<Option<StatePersistence>>,
    workflow_name: Arc<String>,
    json_output: bool,
) -> Result<()> {
    // Get task spec and error recovery strategy
    let (spec, recovery_strategy) = {
        let graph = task_graph.lock().await;
        let task_node = graph
            .get_task(&task_id)
            .ok_or_else(|| Error::InvalidInput(format!("Task '{}' not found", task_id)))?;

        // Get error recovery strategy from task spec
        let strategy = ErrorRecovery::get_strategy_from_spec(task_node.spec.on_error.as_ref());

        (task_node.spec.clone(), strategy)
    };

    println!("Executing task: {} - {}", task_id, spec.description);

    // Check if this is a loop task
    if let Some(ref loop_spec) = spec.loop_spec {
        println!(
            "Task '{}' has loop specification - delegating to loop executor",
            task_id
        );
        let ctx = ExecutionContext {
            workflow_inputs: &workflow_inputs,
            agents: &agents,
            task_graph: &task_graph,
            state: &state,
            workflow_name: &workflow_name,
            json_output,
        };
        return execute_task_with_loop(&task_id, &spec, loop_spec, &ctx).await;
    }

    // Check condition if present
    if let Some(ref condition) = spec.condition {
        let condition_met = {
            let graph = task_graph.lock().await;
            let workflow_state = state.lock().await;
            let state_ref = workflow_state.as_ref();
            evaluate_condition(condition, &graph, state_ref)
        };

        if !condition_met {
            println!("Task '{}' condition not met - skipping", task_id);

            // Mark task as skipped
            {
                let mut graph = task_graph.lock().await;
                graph.update_task_status(&task_id, TaskStatus::Skipped)?;
            }
            if let Some(ref mut workflow_state) = *state.lock().await {
                workflow_state.update_task_status(&task_id, TaskStatus::Skipped);
            }

            return Ok(());
        }

        println!(
            "Task '{}' condition met - proceeding with execution",
            task_id
        );
    }

    // Attempt execution with retry logic (for errors)
    let mut error_attempt = 0;
    // Track DoD retries separately
    let mut dod_attempt = 0;
    // Store last DoD feedback for retry attempts
    let mut last_dod_feedback: Option<String> = None;

    loop {
        // Record attempt in state
        if let Some(ref mut workflow_state) = *state.lock().await {
            workflow_state.record_task_attempt(&task_id);
        }
        // Update status to running
        {
            let mut graph = task_graph.lock().await;
            graph.update_task_status(&task_id, TaskStatus::Running)?;
        }
        if let Some(ref mut workflow_state) = *state.lock().await {
            workflow_state.update_task_status(&task_id, TaskStatus::Running);
        }

        // Create variable context for interpolation
        let mut var_context = crate::dsl::variables::VariableContext::new();
        for (key, value) in workflow_inputs.iter() {
            var_context.insert(&crate::dsl::variables::Scope::Workflow, key, value.clone());
        }

        // Interpolate task description
        let interpolated_description = var_context
            .interpolate(&spec.description)
            .unwrap_or_else(|_| spec.description.clone());

        // Build task description with DoD feedback if this is a retry
        let task_description = if let Some(ref feedback) = last_dod_feedback {
            // Include the actual unmet criteria feedback
            format!("{}\n\n{}", interpolated_description, feedback)
        } else {
            interpolated_description
        };

        // Try to execute the task
        match execute_task_attempt(
            &task_id,
            &task_description,
            &spec,
            &workflow_inputs,
            &agents,
            error_attempt,
            &state,
            &workflow_name,
            json_output,
        )
        .await
        {
            Ok(task_output) => {
                // Task executed successfully - now check definition of done
                if let Some(ref dod) = spec.definition_of_done {
                    println!("Checking definition of done for task: {}", task_id);

                    // Use the variable context created earlier
                    let dod_results =
                        check_definition_of_done(dod, task_output.as_deref(), &var_context).await;
                    let all_met = dod_results.iter().all(|r| r.met);

                    if !all_met {
                        let mut unmet_feedback = format_unmet_criteria(&dod_results);

                        // Enhance feedback with permission hints
                        unmet_feedback = enhance_feedback_with_permission_hints(
                            unmet_feedback,
                            task_output.as_deref().unwrap_or(""),
                            &dod_results,
                            dod.auto_elevate_permissions,
                        );

                        println!("Definition of done not met for task '{}':", task_id);
                        println!("{}", unmet_feedback);

                        dod_attempt += 1;

                        if dod_attempt <= dod.max_retries {
                            println!(
                                "Retrying task '{}' (DoD attempt {}/{})",
                                task_id, dod_attempt, dod.max_retries
                            );

                            // Apply auto-elevation if configured and permission issue detected
                            if dod.auto_elevate_permissions
                                && detect_permission_issue(
                                    task_output.as_deref().unwrap_or(""),
                                    &dod_results,
                                )
                            {
                                println!(
                                    "  🔓 Auto-elevating permissions to 'bypassPermissions' for retry..."
                                );

                                // Actually update the agent's permission mode
                                if let Some(ref agent_id) = spec.agent {
                                    let mut agents_guard = agents.lock().await;
                                    if let Some(agent) = agents_guard.get_mut(agent_id) {
                                        if let Err(e) =
                                            agent.set_permission_mode("bypassPermissions").await
                                        {
                                            eprintln!(
                                                "Warning: Failed to elevate permissions: {}",
                                                e
                                            );
                                        } else {
                                            println!("  ✓ Permissions elevated successfully");
                                        }
                                    }
                                }
                            }

                            // Store feedback for next iteration
                            last_dod_feedback = Some(unmet_feedback);

                            // Continue loop to retry with feedback
                            continue;
                        } else {
                            println!(
                                "Task '{}' exhausted all DoD retries ({}/{})",
                                task_id, dod_attempt, dod.max_retries
                            );

                            if dod.fail_on_unmet {
                                // Mark as failed
                                {
                                    let mut graph = task_graph.lock().await;
                                    graph.update_task_status(&task_id, TaskStatus::Failed)?;
                                }
                                if let Some(ref mut workflow_state) = *state.lock().await {
                                    workflow_state.update_task_status(&task_id, TaskStatus::Failed);
                                    workflow_state.record_task_error(
                                        &task_id,
                                        &format!(
                                            "Definition of done not met after {} retries",
                                            dod.max_retries
                                        ),
                                    );
                                }
                                return Err(Error::InvalidInput(format!(
                                    "Task '{}' failed: definition of done not met after {} retries",
                                    task_id, dod.max_retries
                                )));
                            }
                            // Otherwise fall through to mark as completed despite unmet DoD
                        }
                    } else {
                        println!("✓ Definition of done met for task: {}", task_id);
                    }
                }

                // Success - update status and record result
                {
                    let mut graph = task_graph.lock().await;
                    graph.update_task_status(&task_id, TaskStatus::Completed)?;
                }
                if let Some(ref mut workflow_state) = *state.lock().await {
                    workflow_state.update_task_status(&task_id, TaskStatus::Completed);

                    // Record task result for workflow context
                    if let Some(ref output) = task_output {
                        workflow_state.record_task_result(&task_id, output);

                        // Store task output with metadata for reference by other tasks
                        use crate::dsl::schema::TruncationStrategy;
                        use crate::dsl::state::{OutputType, TaskOutput};
                        let task_output_obj = TaskOutput::new(
                            task_id.clone(),
                            OutputType::Combined,
                            output.clone(),
                            output.len(),
                            false, // not truncated
                            TruncationStrategy::Tail,
                        );
                        workflow_state.store_task_output(task_output_obj);
                    }
                }

                // Write output to file if output directive is present
                if let (Some(ref output_content), Some(ref output_path)) =
                    (&task_output, &spec.output)
                {
                    // Interpolate output path (supports workflow variables, task inputs, and task outputs)
                    let interpolated_path = {
                        let state_guard = state.lock().await;
                        let state_ref = state_guard.as_ref();
                        DSLExecutor::substitute_variables_with_state(
                            output_path,
                            &workflow_inputs,
                            &HashMap::new(),
                            state_ref,
                        )
                    };

                    if let Err(e) = write_task_output_to_file(&interpolated_path, output_content) {
                        eprintln!(
                            "Warning: Failed to write task output to '{}': {}",
                            interpolated_path, e
                        );
                    } else if !json_output {
                        println!("✓ Output written to: {}", interpolated_path);
                    }
                }

                // Checkpoint state after task completion
                if let (Some(ref workflow_state), Some(ref persistence)) =
                    (&*state.lock().await, &*state_persistence)
                {
                    if let Err(e) = persistence.save_state(workflow_state) {
                        eprintln!(
                            "Warning: Failed to checkpoint state after task '{}': {}",
                            task_id, e
                        );
                    }
                }

                println!("Task completed: {}", task_id);

                // Handle on_complete actions
                if let Some(on_complete) = &spec.on_complete {
                    if let Some(notify_spec) = &on_complete.notify {
                        // Extract message from NotificationSpec (Simple or Structured)
                        let message = match notify_spec {
                            crate::dsl::NotificationSpec::Simple(msg) => msg.clone(),
                            crate::dsl::NotificationSpec::Structured { message, .. } => {
                                message.clone()
                            }
                        };
                        println!("Notification: {}", message);
                    }
                }

                return Ok(());
            }
            Err(e) => {
                println!(
                    "Task '{}' failed (attempt {}): {}",
                    task_id,
                    error_attempt + 1,
                    e
                );

                // Record error in state
                if let Some(ref mut workflow_state) = *state.lock().await {
                    workflow_state.record_task_error(&task_id, &e.to_string());
                }

                error_attempt += 1;

                // Check if we should retry
                if !ErrorRecovery::should_retry(&recovery_strategy, error_attempt) {
                    // No more retries - mark as failed
                    {
                        let mut graph = task_graph.lock().await;
                        graph.update_task_status(&task_id, TaskStatus::Failed)?;
                    }
                    if let Some(ref mut workflow_state) = *state.lock().await {
                        workflow_state.update_task_status(&task_id, TaskStatus::Failed);
                    }

                    // Try fallback agent if available
                    if let Some(fallback_agent) =
                        ErrorRecovery::get_fallback_agent(&recovery_strategy)
                    {
                        println!("Attempting fallback with agent: {}", fallback_agent);

                        // Reset to Running status for fallback attempt
                        {
                            let mut graph = task_graph.lock().await;
                            graph.update_task_status(&task_id, TaskStatus::Running)?;
                        }
                        if let Some(ref mut workflow_state) = *state.lock().await {
                            workflow_state.update_task_status(&task_id, TaskStatus::Running);
                        }

                        // Execute with fallback agent
                        match execute_task_with_agent(
                            &task_id,
                            &spec,
                            &agents,
                            fallback_agent,
                            error_attempt,
                            json_output,
                        )
                        .await
                        {
                            Ok(()) => {
                                println!("Task completed with fallback agent: {}", task_id);

                                // Mark as completed
                                {
                                    let mut graph = task_graph.lock().await;
                                    graph.update_task_status(&task_id, TaskStatus::Completed)?;
                                }
                                if let Some(ref mut workflow_state) = *state.lock().await {
                                    workflow_state
                                        .update_task_status(&task_id, TaskStatus::Completed);
                                }

                                return Ok(());
                            }
                            Err(fallback_err) => {
                                println!("Fallback agent also failed: {}", fallback_err);

                                // Mark as failed again
                                {
                                    let mut graph = task_graph.lock().await;
                                    graph.update_task_status(&task_id, TaskStatus::Failed)?;
                                }
                                if let Some(ref mut workflow_state) = *state.lock().await {
                                    workflow_state.update_task_status(&task_id, TaskStatus::Failed);
                                    workflow_state.record_task_error(
                                        &task_id,
                                        &format!(
                                            "Primary and fallback failed: {} / {}",
                                            e, fallback_err
                                        ),
                                    );
                                }

                                return Err(fallback_err);
                            }
                        }
                    }

                    return Err(e);
                }

                // Calculate retry delay with optional exponential backoff
                let retry_delay = calculate_retry_delay(&recovery_strategy, error_attempt);
                println!(
                    "Retrying task '{}' (attempt {}) in {}s...",
                    task_id,
                    error_attempt + 1,
                    retry_delay
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(retry_delay)).await;
            }
        }
    }
}

/// Execute a script task
async fn execute_script_task(
    _task_id: &str,
    script_spec: &crate::dsl::schema::ScriptSpec,
    workflow_inputs: &HashMap<String, serde_json::Value>,
    task_inputs: &HashMap<String, serde_json::Value>,
    attempt: u32,
) -> Result<Option<String>> {
    use crate::dsl::schema::ScriptLanguage;
    use tokio::process::Command;

    // Determine the interpreter based on language
    let (interpreter, args): (&str, Vec<&str>) = match script_spec.language {
        ScriptLanguage::Python => ("python3", vec!["-c"]),
        ScriptLanguage::JavaScript => ("node", vec!["-e"]),
        ScriptLanguage::Bash => ("bash", vec!["-c"]),
        ScriptLanguage::Ruby => ("ruby", vec!["-e"]),
        ScriptLanguage::Perl => ("perl", vec!["-e"]),
    };

    // Get script content - either inline or from file
    let raw_script_content = if let Some(content) = &script_spec.content {
        content.clone()
    } else if let Some(file_path) = &script_spec.file {
        // Interpolate variables in file path
        let interpolated_file_path =
            DSLExecutor::substitute_variables(file_path, workflow_inputs, task_inputs);
        tokio::fs::read_to_string(&interpolated_file_path)
            .await
            .map_err(|e| {
                Error::InvalidInput(format!(
                    "Failed to read script file '{}': {}",
                    interpolated_file_path, e
                ))
            })?
    } else {
        return Err(Error::InvalidInput(
            "Script must specify either 'content' or 'file'".to_string(),
        ));
    };

    // Substitute workflow and task variables in script content
    let script_content =
        DSLExecutor::substitute_variables(&raw_script_content, workflow_inputs, task_inputs);

    // Build command
    let mut cmd = Command::new(interpreter);
    cmd.args(&args);
    cmd.arg(&script_content);

    // Set working directory if specified (with variable interpolation)
    if let Some(working_dir) = &script_spec.working_dir {
        let interpolated_working_dir =
            DSLExecutor::substitute_variables(working_dir, workflow_inputs, task_inputs);
        cmd.current_dir(interpolated_working_dir);
    }

    // Set environment variables (with variable interpolation)
    for (key, value) in &script_spec.env {
        let interpolated_value =
            DSLExecutor::substitute_variables(value, workflow_inputs, task_inputs);
        cmd.env(key, interpolated_value);
    }

    // Capture stdout and stderr
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    if attempt > 0 {
        println!(
            "  [Retry {}] Executing {} script...",
            attempt,
            format!("{:?}", script_spec.language).to_lowercase()
        );
    } else {
        println!(
            "  Executing {} script...",
            format!("{:?}", script_spec.language).to_lowercase()
        );
    }

    // Execute with timeout if specified
    let output = if let Some(timeout_secs) = script_spec.timeout_secs {
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output())
            .await
            .map_err(|_| {
                Error::InvalidInput(format!(
                    "Script execution timed out after {} seconds",
                    timeout_secs
                ))
            })?
            .map_err(|e| Error::InvalidInput(format!("Failed to execute script: {}", e)))?
    } else {
        cmd.output()
            .await
            .map_err(|e| Error::InvalidInput(format!("Failed to execute script: {}", e)))?
    };

    // Convert output to strings
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Print output
    if !stdout.is_empty() {
        print!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    // Check exit status
    if !output.status.success() {
        return Err(Error::InvalidInput(format!(
            "Script failed with exit code: {:?}",
            output.status.code()
        )));
    }

    // Return combined output
    let combined_output = format!("{}{}", stdout, stderr);
    Ok(if combined_output.is_empty() {
        None
    } else {
        Some(combined_output)
    })
}

/// Execute a command task
async fn execute_command_task(
    _task_id: &str,
    command_spec: &crate::dsl::schema::CommandSpec,
    workflow_inputs: &HashMap<String, serde_json::Value>,
    task_inputs: &HashMap<String, serde_json::Value>,
    attempt: u32,
) -> Result<Option<String>> {
    use tokio::process::Command;

    // Substitute variables in executable
    let executable =
        DSLExecutor::substitute_variables(&command_spec.executable, workflow_inputs, task_inputs);

    // Substitute variables in args
    let args: Vec<String> = command_spec
        .args
        .iter()
        .map(|arg| DSLExecutor::substitute_variables(arg, workflow_inputs, task_inputs))
        .collect();

    // Build command
    let mut cmd = Command::new(&executable);
    cmd.args(&args);

    // Set working directory if specified
    if let Some(working_dir) = &command_spec.working_dir {
        let working_dir =
            DSLExecutor::substitute_variables(working_dir, workflow_inputs, task_inputs);
        cmd.current_dir(working_dir);
    }

    // Set environment variables
    for (key, value) in &command_spec.env {
        let value = DSLExecutor::substitute_variables(value, workflow_inputs, task_inputs);
        cmd.env(key, value);
    }

    // Capture stdout and stderr based on spec
    if command_spec.capture_stdout {
        cmd.stdout(std::process::Stdio::piped());
    }
    if command_spec.capture_stderr {
        cmd.stderr(std::process::Stdio::piped());
    }

    if attempt > 0 {
        println!(
            "  [Retry {}] Executing command: {} {}",
            attempt,
            executable,
            args.join(" ")
        );
    } else {
        println!("  Executing command: {} {}", executable, args.join(" "));
    }

    // Execute with timeout if specified
    let output = if let Some(timeout_secs) = command_spec.timeout_secs {
        tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), cmd.output())
            .await
            .map_err(|_| {
                Error::InvalidInput(format!(
                    "Command execution timed out after {} seconds",
                    timeout_secs
                ))
            })?
            .map_err(|e| Error::InvalidInput(format!("Failed to execute command: {}", e)))?
    } else {
        cmd.output()
            .await
            .map_err(|e| Error::InvalidInput(format!("Failed to execute command: {}", e)))?
    };

    // Convert output to strings
    let stdout = if command_spec.capture_stdout {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::new()
    };
    let stderr = if command_spec.capture_stderr {
        String::from_utf8_lossy(&output.stderr).to_string()
    } else {
        String::new()
    };

    // Print output
    if !stdout.is_empty() {
        print!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }

    // Check exit status
    if !output.status.success() {
        return Err(Error::InvalidInput(format!(
            "Command '{}' failed with exit code: {:?}",
            executable,
            output.status.code()
        )));
    }

    // Return combined output
    let combined_output = format!("{}{}", stdout, stderr);
    Ok(if combined_output.is_empty() {
        None
    } else {
        Some(combined_output)
    })
}

/// Write task output to a file
fn write_task_output_to_file(path: &str, content: &str) -> Result<()> {
    use std::fs;
    use std::path::Path;

    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|e| {
            Error::InvalidInput(format!(
                "Failed to create directory '{}': {}",
                parent.display(),
                e
            ))
        })?;
    }

    // Write content to file
    fs::write(path, content)
        .map_err(|e| Error::InvalidInput(format!("Failed to write to file '{}': {}", path, e)))?;

    Ok(())
}

/// Execute an LLM task (direct API call)
async fn execute_llm_task(
    _task_id: &str,
    llm_spec: &crate::dsl::schema::LlmSpec,
    workflow_inputs: &HashMap<String, serde_json::Value>,
    task_inputs: &HashMap<String, serde_json::Value>,
    workflow_state: Option<&crate::dsl::state::WorkflowState>,
    attempt: u32,
) -> Result<Option<String>> {
    use crate::adapters::secondary::HttpLlmClient;
    use crate::ports::secondary::{LlmClient, LlmRequest};

    // Substitute variables in prompt (including task outputs)
    let prompt = DSLExecutor::substitute_variables_with_state(
        &llm_spec.prompt,
        workflow_inputs,
        task_inputs,
        workflow_state,
    );

    // Substitute variables in system prompt if present
    let system_prompt = llm_spec.system_prompt.as_ref().map(|sp| {
        DSLExecutor::substitute_variables_with_state(
            sp,
            workflow_inputs,
            task_inputs,
            workflow_state,
        )
    });

    // Substitute variables in endpoint if present
    let endpoint = llm_spec.endpoint.as_ref().map(|ep| {
        DSLExecutor::substitute_variables_with_state(
            ep,
            workflow_inputs,
            task_inputs,
            workflow_state,
        )
    });

    // Substitute variables in API key if present
    let api_key = llm_spec.api_key.as_ref().map(|key| {
        DSLExecutor::substitute_variables_with_state(
            key,
            workflow_inputs,
            task_inputs,
            workflow_state,
        )
    });

    if attempt > 0 {
        println!(
            "  [Retry {}] Executing LLM task with {:?} {}",
            attempt, llm_spec.provider, llm_spec.model
        );
    } else {
        println!(
            "  Executing LLM task with {:?} {}",
            llm_spec.provider, llm_spec.model
        );
    }

    // Build LLM request
    let request = LlmRequest {
        provider: llm_spec.provider.clone(),
        model: llm_spec.model.clone(),
        prompt,
        system_prompt,
        endpoint,
        api_key,
        temperature: llm_spec.temperature,
        max_tokens: llm_spec.max_tokens,
        top_p: llm_spec.top_p,
        top_k: llm_spec.top_k,
        stop: llm_spec.stop.clone(),
        timeout_secs: llm_spec.timeout_secs,
        extra_params: llm_spec.extra_params.clone(),
    };

    // Create HTTP LLM client
    let client = HttpLlmClient::new();

    // Execute LLM request
    let response = client
        .execute(request)
        .await
        .map_err(|e| Error::InvalidInput(format!("LLM execution failed: {}", e)))?;

    // Print response
    println!("\n{}", response.content);

    // Print token usage if available
    if let Some(usage) = &response.usage {
        println!(
            "\n  Token usage: {} input + {} output = {} total",
            usage.prompt_tokens, usage.completion_tokens, usage.total_tokens
        );
    }

    // Print finish reason if available
    if let Some(reason) = &response.finish_reason {
        println!("  Finish reason: {}", reason);
    }

    println!(); // Add blank line for readability

    // Return content as output
    Ok(Some(response.content))
}

/// Attempt to execute a task once
#[allow(clippy::too_many_arguments)]
async fn execute_task_attempt(
    _task_id: &str,
    task_description: &str,
    _spec: &crate::dsl::schema::TaskSpec,
    workflow_inputs: &HashMap<String, serde_json::Value>,
    agents: &Arc<Mutex<HashMap<String, PeriplonSDKClient>>>,
    attempt: u32,
    workflow_state: &Arc<Mutex<Option<crate::dsl::state::WorkflowState>>>,
    workflow_name: &str,
    json_output: bool,
) -> Result<Option<String>> {
    // Check what type of task this is and execute accordingly
    if let Some(script_spec) = &_spec.script {
        // Execute script task with variable substitution
        return execute_script_task(
            _task_id,
            script_spec,
            workflow_inputs,
            &_spec.inputs,
            attempt,
        )
        .await;
    }

    // Check if this is a command task
    if let Some(command_spec) = &_spec.command {
        // Execute command task
        return execute_command_task(
            _task_id,
            command_spec,
            workflow_inputs,
            &_spec.inputs,
            attempt,
        )
        .await;
    }

    // Check if this is an LLM task
    if let Some(llm_spec) = &_spec.llm {
        // Execute LLM task with state for task output references
        let state_ref = {
            let state_guard = workflow_state.lock().await;
            state_guard.as_ref().map(|s| s as *const _)
        };
        let state_opt = unsafe { state_ref.map(|ptr| &*ptr) };
        return execute_llm_task(
            _task_id,
            llm_spec,
            workflow_inputs,
            &_spec.inputs,
            state_opt,
            attempt,
        )
        .await;
    }

    // Default to agent-based execution
    let agent_name = _spec.agent.as_ref().ok_or_else(|| {
        Error::InvalidInput(format!("No agent specified for task '{}'", _task_id))
    })?;

    // Build workflow context for agent-based tasks (only if inject_context is true)
    let workflow_context = if _spec.inject_context {
        if let Some(ref state) = *workflow_state.lock().await {
            // Include context if there are completed tasks, or if output file is specified
            let has_workflow_context =
                !state.get_completed_tasks().is_empty() || !state.get_failed_tasks().is_empty();
            let has_output_file = _spec.output.is_some();

            if has_workflow_context || has_output_file {
                state.build_context_summary(workflow_name, None, _spec.output.as_deref())
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // Prepend context to task description for agent (only if inject_context is true)
    let enhanced_description = if _spec.inject_context && !workflow_context.is_empty() {
        format!("{}\n{}", workflow_context, task_description)
    } else {
        task_description.to_string()
    };

    // Execute task query
    let mut agents_guard = agents.lock().await;
    let agent = agents_guard
        .get_mut(agent_name)
        .ok_or_else(|| Error::InvalidInput(format!("Agent '{}' not found", agent_name)))?;

    agent.query(&enhanced_description).await?;

    // Process response and capture output
    let stream = agent.receive_response()?;
    futures::pin_mut!(stream);
    let mut output = String::new();

    while let Some(msg) = stream.next().await {
        // Capture output for DoD checking
        output.push_str(&format!("{:?}\n", msg));

        // Log messages using formatter
        if attempt > 0 {
            let prefix_str = format!("Retry {}", attempt);
            let formatted =
                crate::dsl::message_formatter::format_message(&msg, json_output, Some(&prefix_str));
            println!("{}", formatted);
        } else {
            let formatted = crate::dsl::message_formatter::format_message(&msg, json_output, None);
            println!("{}", formatted);
        }
    }

    Ok(if output.is_empty() {
        None
    } else {
        Some(output)
    })
}

/// Execute a task with a specific agent (for fallback support)
async fn execute_task_with_agent(
    _task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    agents: &Arc<Mutex<HashMap<String, PeriplonSDKClient>>>,
    agent_name: &str,
    attempt: u32,
    json_output: bool,
) -> Result<()> {
    // Execute task query with specified agent
    let mut agents_guard = agents.lock().await;
    let agent = agents_guard
        .get_mut(agent_name)
        .ok_or_else(|| Error::InvalidInput(format!("Agent '{}' not found", agent_name)))?;

    agent.query(&spec.description).await?;

    // Process response
    let stream = agent.receive_response()?;
    futures::pin_mut!(stream);
    while let Some(msg) = stream.next().await {
        // Log messages using formatter
        if attempt > 0 {
            let prefix_str = "Fallback".to_string();
            let formatted =
                crate::dsl::message_formatter::format_message(&msg, json_output, Some(&prefix_str));
            println!("{}", formatted);
        } else {
            let formatted = crate::dsl::message_formatter::format_message(&msg, json_output, None);
            println!("{}", formatted);
        }
    }

    Ok(())
}

/// Calculate retry delay with optional exponential backoff
///
/// # Arguments
///
/// * `recovery_strategy` - Error recovery strategy
/// * `attempt` - Current error attempt number (1-based, 1 = first retry)
///
/// # Returns
///
/// Delay in seconds
fn calculate_retry_delay(
    recovery_strategy: &crate::dsl::hooks::RecoveryStrategy,
    attempt: u32,
) -> u64 {
    use crate::dsl::hooks::RecoveryStrategy;

    match recovery_strategy {
        RecoveryStrategy::Retry { config, .. } => {
            let base_delay = config.as_ref().map(|c| c.retry_delay_secs).unwrap_or(1);

            let use_backoff = config
                .as_ref()
                .map(|c| c.exponential_backoff)
                .unwrap_or(false);

            if use_backoff {
                // Exponential backoff: base_delay * 2^(attempt-1)
                // For attempt 1 (first retry): base_delay * 2^0 = base_delay
                // For attempt 2 (second retry): base_delay * 2^1 = base_delay * 2
                // For attempt 3 (third retry): base_delay * 2^2 = base_delay * 4
                // Cap at 60 seconds to avoid extremely long delays
                let exponent = attempt.saturating_sub(1);
                let delay = base_delay * 2u64.pow(exponent);
                delay.min(60)
            } else {
                base_delay
            }
        }
        _ => 1, // Default 1 second for other strategies
    }
}

/// Evaluate a condition against the current workflow state
///
/// # Arguments
///
/// * `condition` - The condition to evaluate
/// * `task_graph` - Current task graph state
/// * `workflow_state` - Current workflow state (optional)
///
/// # Returns
///
/// true if the condition is met, false otherwise
fn evaluate_condition(
    condition: &crate::dsl::schema::ConditionSpec,
    task_graph: &TaskGraph,
    workflow_state: Option<&WorkflowState>,
) -> bool {
    use crate::dsl::schema::ConditionSpec;

    match condition {
        ConditionSpec::Single(cond) => evaluate_single_condition(cond, task_graph, workflow_state),
        ConditionSpec::And { and } => and
            .iter()
            .all(|c| evaluate_condition(c, task_graph, workflow_state)),
        ConditionSpec::Or { or } => or
            .iter()
            .any(|c| evaluate_condition(c, task_graph, workflow_state)),
        ConditionSpec::Not { not } => !evaluate_condition(not, task_graph, workflow_state),
    }
}

/// Evaluate a single condition
fn evaluate_single_condition(
    condition: &crate::dsl::schema::Condition,
    task_graph: &TaskGraph,
    workflow_state: Option<&WorkflowState>,
) -> bool {
    use crate::dsl::schema::{Condition, TaskStatusCondition};

    match condition {
        Condition::TaskStatus { task, status } => {
            // Get task status from task graph
            let task_status = task_graph.get_task_status(task);
            matches!(
                (task_status, status),
                (Some(TaskStatus::Completed), TaskStatusCondition::Completed)
                    | (Some(TaskStatus::Failed), TaskStatusCondition::Failed)
                    | (Some(TaskStatus::Running), TaskStatusCondition::Running)
                    | (Some(TaskStatus::Pending), TaskStatusCondition::Pending)
                    | (Some(TaskStatus::Skipped), TaskStatusCondition::Skipped)
            )
        }
        Condition::StateEquals { key, value } => {
            // Check workflow state
            if let Some(state) = workflow_state {
                state.get_metadata(key) == Some(value)
            } else {
                false
            }
        }
        Condition::StateExists { key } => {
            // Check if state key exists
            if let Some(state) = workflow_state {
                state.get_metadata(key).is_some()
            } else {
                false
            }
        }
        Condition::Always => true,
        Condition::Never => false,
    }
}

/// Check definition of done criteria
///
/// # Arguments
///
/// * `dod` - Definition of done specification
/// * `_task_output` - Output from task execution (for output matching)
///
/// # Returns
///
/// Vector of criterion results
async fn check_definition_of_done(
    dod: &crate::dsl::schema::DefinitionOfDone,
    _task_output: Option<&str>,
    var_context: &crate::dsl::variables::VariableContext,
) -> Vec<crate::dsl::schema::CriterionResult> {
    let mut results = Vec::new();

    for criterion in &dod.criteria {
        let result = check_criterion(criterion, _task_output, var_context).await;
        results.push(result);
    }

    results
}

/// Check a single criterion
async fn check_criterion(
    criterion: &crate::dsl::schema::DoneCriterion,
    _task_output: Option<&str>,
    var_context: &crate::dsl::variables::VariableContext,
) -> crate::dsl::schema::CriterionResult {
    use crate::dsl::schema::{CriterionResult, DoneCriterion, OutputSource};

    match criterion {
        DoneCriterion::FileExists { path, description } => {
            // Interpolate variables in path
            let interpolated_path = var_context
                .interpolate(path)
                .unwrap_or_else(|_| path.clone());
            let exists = std::path::Path::new(&interpolated_path).exists();
            CriterionResult {
                met: exists,
                description: description.clone(),
                details: if exists {
                    format!("File '{}' exists", interpolated_path)
                } else {
                    format!("File '{}' does not exist", interpolated_path)
                },
            }
        }
        DoneCriterion::FileContains {
            path,
            pattern,
            description,
        } => {
            // Interpolate variables in path and pattern
            let interpolated_path = var_context
                .interpolate(path)
                .unwrap_or_else(|_| path.clone());
            let interpolated_pattern = var_context
                .interpolate(pattern)
                .unwrap_or_else(|_| pattern.clone());
            match std::fs::read_to_string(&interpolated_path) {
                Ok(content) => {
                    // Try regex first, fall back to substring matching if regex is invalid
                    let contains = match regex::Regex::new(&interpolated_pattern) {
                        Ok(re) => re.is_match(&content),
                        Err(_) => content.contains(&interpolated_pattern), // Fallback to substring
                    };
                    CriterionResult {
                        met: contains,
                        description: description.clone(),
                        details: if contains {
                            format!(
                                "File '{}' contains pattern '{}'",
                                interpolated_path, interpolated_pattern
                            )
                        } else {
                            format!(
                                "File '{}' does not contain pattern '{}'",
                                interpolated_path, interpolated_pattern
                            )
                        },
                    }
                }
                Err(e) => CriterionResult {
                    met: false,
                    description: description.clone(),
                    details: format!("Failed to read file '{}': {}", interpolated_path, e),
                },
            }
        }
        DoneCriterion::FileNotContains {
            path,
            pattern,
            description,
        } => {
            // Interpolate variables in path and pattern
            let interpolated_path = var_context
                .interpolate(path)
                .unwrap_or_else(|_| path.clone());
            let interpolated_pattern = var_context
                .interpolate(pattern)
                .unwrap_or_else(|_| pattern.clone());
            match std::fs::read_to_string(&interpolated_path) {
                Ok(content) => {
                    // Try regex first, fall back to substring matching if regex is invalid
                    let contains = match regex::Regex::new(&interpolated_pattern) {
                        Ok(re) => re.is_match(&content),
                        Err(_) => content.contains(&interpolated_pattern), // Fallback to substring
                    };
                    let not_contains = !contains;
                    CriterionResult {
                        met: not_contains,
                        description: description.clone(),
                        details: if not_contains {
                            format!(
                                "File '{}' does not contain pattern '{}'",
                                interpolated_path, interpolated_pattern
                            )
                        } else {
                            format!(
                                "File '{}' contains pattern '{}' (should not)",
                                interpolated_path, interpolated_pattern
                            )
                        },
                    }
                }
                Err(e) => CriterionResult {
                    met: false,
                    description: description.clone(),
                    details: format!("Failed to read file '{}': {}", interpolated_path, e),
                },
            }
        }
        DoneCriterion::CommandSucceeds {
            command,
            args,
            description,
            working_dir,
        } => {
            // Interpolate variables in command, args, and working_dir
            let interpolated_command = var_context
                .interpolate(command)
                .unwrap_or_else(|_| command.clone());
            let interpolated_args: Vec<String> = args
                .iter()
                .map(|arg| var_context.interpolate(arg).unwrap_or_else(|_| arg.clone()))
                .collect();
            let interpolated_working_dir = working_dir
                .as_ref()
                .map(|dir| var_context.interpolate(dir).unwrap_or_else(|_| dir.clone()));

            let mut cmd = tokio::process::Command::new(&interpolated_command);
            cmd.args(&interpolated_args);

            if let Some(ref dir) = interpolated_working_dir {
                cmd.current_dir(dir);
            }

            match cmd.output().await {
                Ok(output) => {
                    let success = output.status.success();

                    let details = if success {
                        format!("Command '{}' succeeded", interpolated_command)
                    } else {
                        // Capture stdout and stderr for failed commands so agent can fix errors
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        let mut error_msg = format!(
                            "Command '{}' failed with exit code: {}\n\n",
                            interpolated_command,
                            output.status.code().unwrap_or(-1)
                        );

                        if !stderr.is_empty() {
                            error_msg.push_str("STDERR:\n");
                            error_msg.push_str("```\n");
                            error_msg.push_str(&stderr);
                            error_msg.push_str("```\n\n");
                        }

                        if !stdout.is_empty() {
                            error_msg.push_str("STDOUT:\n");
                            error_msg.push_str("```\n");
                            error_msg.push_str(&stdout);
                            error_msg.push_str("```\n");
                        }

                        error_msg
                    };

                    CriterionResult {
                        met: success,
                        description: description.clone(),
                        details,
                    }
                }
                Err(e) => CriterionResult {
                    met: false,
                    description: description.clone(),
                    details: format!(
                        "Failed to execute command '{}': {}",
                        interpolated_command, e
                    ),
                },
            }
        }
        DoneCriterion::OutputMatches {
            source,
            pattern,
            description,
        } => {
            // Interpolate variables in pattern
            let interpolated_pattern = var_context
                .interpolate(pattern)
                .unwrap_or_else(|_| pattern.clone());

            let content = match source {
                OutputSource::File { path } => {
                    // Interpolate variables in file path
                    let interpolated_path = var_context
                        .interpolate(path)
                        .unwrap_or_else(|_| path.clone());
                    std::fs::read_to_string(&interpolated_path).ok()
                }
                OutputSource::TaskOutput => _task_output.map(|s| s.to_string()),
            };

            match content {
                Some(text) => {
                    let matches = text.contains(&interpolated_pattern);
                    CriterionResult {
                        met: matches,
                        description: description.clone(),
                        details: if matches {
                            format!("Output matches pattern '{}'", interpolated_pattern)
                        } else {
                            format!("Output does not match pattern '{}'", interpolated_pattern)
                        },
                    }
                }
                None => CriterionResult {
                    met: false,
                    description: description.clone(),
                    details: "Failed to read output".to_string(),
                },
            }
        }
        DoneCriterion::DirectoryExists { path, description } => {
            // Interpolate variables in path
            let interpolated_path = var_context
                .interpolate(path)
                .unwrap_or_else(|_| path.clone());
            let exists = std::path::Path::new(&interpolated_path).is_dir();
            CriterionResult {
                met: exists,
                description: description.clone(),
                details: if exists {
                    format!("Directory '{}' exists", interpolated_path)
                } else {
                    format!("Directory '{}' does not exist", interpolated_path)
                },
            }
        }
        DoneCriterion::TestsPassed {
            command,
            args,
            description,
        } => {
            // Interpolate variables in command and args
            let interpolated_command = var_context
                .interpolate(command)
                .unwrap_or_else(|_| command.clone());
            let interpolated_args: Vec<String> = args
                .iter()
                .map(|arg| var_context.interpolate(arg).unwrap_or_else(|_| arg.clone()))
                .collect();

            let mut cmd = tokio::process::Command::new(&interpolated_command);
            cmd.args(&interpolated_args);

            match cmd.output().await {
                Ok(output) => {
                    let success = output.status.success();

                    let details = if success {
                        format!("Tests passed ({})", interpolated_command)
                    } else {
                        // Capture full stdout and stderr for failed tests so agent can fix errors
                        let stdout = String::from_utf8_lossy(&output.stdout);
                        let stderr = String::from_utf8_lossy(&output.stderr);

                        let mut error_msg = format!(
                            "Tests failed ({}) with exit code: {}\n\n",
                            interpolated_command,
                            output.status.code().unwrap_or(-1)
                        );

                        if !stderr.is_empty() {
                            error_msg.push_str("STDERR:\n");
                            error_msg.push_str("```\n");
                            error_msg.push_str(&stderr);
                            error_msg.push_str("```\n\n");
                        }

                        if !stdout.is_empty() {
                            error_msg.push_str("STDOUT:\n");
                            error_msg.push_str("```\n");
                            error_msg.push_str(&stdout);
                            error_msg.push_str("```\n");
                        }

                        error_msg
                    };

                    CriterionResult {
                        met: success,
                        description: description.clone(),
                        details,
                    }
                }
                Err(e) => CriterionResult {
                    met: false,
                    description: description.clone(),
                    details: format!("Failed to run tests '{}': {}", interpolated_command, e),
                },
            }
        }
    }
}

/// Format unmet criteria for agent feedback
fn format_unmet_criteria(results: &[crate::dsl::schema::CriterionResult]) -> String {
    let unmet: Vec<_> = results.iter().filter(|r| !r.met).collect();

    if unmet.is_empty() {
        return String::new();
    }

    let mut feedback = String::from("\n\n=== DEFINITION OF DONE - UNMET CRITERIA ===\n\n");
    feedback.push_str("The following criteria were not met:\n\n");

    for (i, result) in unmet.iter().enumerate() {
        feedback.push_str(&format!(
            "{}. {}\n   Status: ✗ FAILED\n   Details: {}\n\n",
            i + 1,
            result.description,
            result.details
        ));
    }

    feedback.push_str("Please address these issues and retry the task.\n");
    feedback
}

/// Detect if DoD failure is likely due to permission issues
fn detect_permission_issue(output: &str, results: &[crate::dsl::schema::CriterionResult]) -> bool {
    // Check output for permission-related keywords
    let permission_keywords = [
        "permission",
        "permissions",
        "write access",
        "read access",
        "file write",
        "cannot create",
        "cannot write",
        "access denied",
        "forbidden",
    ];

    let output_lower = output.to_lowercase();
    let has_permission_mention = permission_keywords
        .iter()
        .any(|keyword| output_lower.contains(keyword));

    // Check if failures are file-related (likely permission issues)
    let has_file_failures = results.iter().any(|r| {
        !r.met
            && (r.description.to_lowercase().contains("file")
                || r.details.to_lowercase().contains("does not exist")
                || r.details.to_lowercase().contains("not found"))
    });

    has_permission_mention || has_file_failures
}

/// Enhance feedback with permission hints if permission issues detected
fn enhance_feedback_with_permission_hints(
    mut feedback: String,
    output: &str,
    results: &[crate::dsl::schema::CriterionResult],
    auto_elevate: bool,
) -> String {
    if detect_permission_issue(output, results) {
        feedback.push_str("\n⚠️  PERMISSION HINT:\n");
        feedback.push_str("The failure appears to be related to file access or permissions.\n");

        if auto_elevate {
            feedback.push_str(
                "Auto-elevation is enabled - 'bypassPermissions' mode will be granted on retry.\n",
            );
            feedback
                .push_str("All tool operations will be automatically approved for this retry.\n");
        } else {
            feedback.push_str("Consider:\n");
            feedback.push_str("1. Ensuring required files exist before checking\n");
            feedback.push_str("2. Creating necessary files if they don't exist\n");
            feedback.push_str("3. Requesting write permissions if needed\n\n");
            feedback.push_str(
                "TIP: Add 'auto_elevate_permissions: true' to the definition_of_done config\n",
            );
            feedback.push_str("     to automatically grant enhanced permissions on retry.\n");
        }
    }

    feedback
}

/// Extract a value from JSON using a simple path notation
///
/// Supports:
/// - Dot notation: "data.items"
/// - Array indexing: "data[0]"
/// - Combined: "data.items[0].name"
///
/// # Arguments
///
/// * `value` - The JSON value to extract from
/// * `path` - The path to extract (e.g., "data.items")
///
/// # Returns
///
/// The extracted JSON value
fn extract_json_path(value: &serde_json::Value, path: &str) -> Result<serde_json::Value> {
    let mut current = value.clone();

    // Split path by dots and handle array indices
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }

        // Check for array index notation like "items[0]"
        if let Some(bracket_pos) = segment.find('[') {
            let key = &segment[..bracket_pos];
            let index_str = &segment[bracket_pos + 1..segment.len() - 1];
            let index: usize = index_str.parse().map_err(|_| {
                Error::InvalidInput(format!("Invalid array index in JSON path: {}", segment))
            })?;

            // First navigate to the key
            if !key.is_empty() {
                current = current
                    .get(key)
                    .ok_or_else(|| {
                        Error::InvalidInput(format!("JSON path key '{}' not found", key))
                    })?
                    .clone();
            }

            // Then get the array element
            current = current
                .get(index)
                .ok_or_else(|| {
                    Error::InvalidInput(format!("JSON path index {} out of bounds", index))
                })?
                .clone();
        } else {
            // Simple key access
            current = current
                .get(segment)
                .ok_or_else(|| {
                    Error::InvalidInput(format!("JSON path key '{}' not found", segment))
                })?
                .clone();
        }
    }

    Ok(current)
}

/// Resolve a collection source to a vector of JSON values
///
/// # Arguments
///
/// * `collection` - The collection source specification
/// * `state` - Optional workflow state for state-based collections
///
/// # Returns
///
/// Vector of JSON values representing the collection items
async fn resolve_collection(
    collection: &CollectionSource,
    state: Option<&WorkflowState>,
) -> Result<Vec<serde_json::Value>> {
    match collection {
        CollectionSource::State { key } => {
            // Read from workflow state
            let state = state.ok_or_else(|| {
                Error::InvalidInput(
                    "Workflow state not available for state-based collection".to_string(),
                )
            })?;

            let value = state
                .get_metadata(key)
                .ok_or_else(|| Error::InvalidInput(format!("State key '{}' not found", key)))?;

            // Expect an array
            match value {
                serde_json::Value::Array(arr) => Ok(arr.clone()),
                _ => Err(Error::InvalidInput(format!(
                    "State key '{}' is not an array",
                    key
                ))),
            }
        }
        CollectionSource::File { path, format } => {
            // Read from file
            let content = std::fs::read_to_string(path).map_err(|e| {
                Error::InvalidInput(format!("Failed to read collection file '{}': {}", path, e))
            })?;

            match format {
                FileFormat::Json => {
                    let value: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                        Error::InvalidInput(format!("Failed to parse JSON file '{}': {}", path, e))
                    })?;

                    match value {
                        serde_json::Value::Array(arr) => Ok(arr),
                        _ => Err(Error::InvalidInput(format!(
                            "JSON file '{}' does not contain an array",
                            path
                        ))),
                    }
                }
                FileFormat::JsonLines => {
                    let items: Result<Vec<serde_json::Value>> = content
                        .lines()
                        .enumerate()
                        .map(|(i, line)| {
                            serde_json::from_str(line).map_err(|e| {
                                Error::InvalidInput(format!(
                                    "Failed to parse JSON line {} in '{}': {}",
                                    i + 1,
                                    path,
                                    e
                                ))
                            })
                        })
                        .collect();
                    items
                }
                FileFormat::Csv => {
                    // Simple CSV parsing - split by comma, each row becomes array
                    let items: Vec<serde_json::Value> = content
                        .lines()
                        .skip(1) // Skip header
                        .map(|line| {
                            let fields: Vec<&str> = line.split(',').collect();
                            serde_json::Value::Array(
                                fields
                                    .iter()
                                    .map(|f| serde_json::Value::String(f.to_string()))
                                    .collect(),
                            )
                        })
                        .collect();
                    Ok(items)
                }
                FileFormat::Lines => {
                    let items: Vec<serde_json::Value> = content
                        .lines()
                        .map(|line| serde_json::Value::String(line.to_string()))
                        .collect();
                    Ok(items)
                }
            }
        }
        CollectionSource::Range { start, end, step } => {
            // Generate numeric range
            let step_size = step.unwrap_or(1);
            let mut items = Vec::new();

            let mut current = *start;
            while current < *end {
                items.push(serde_json::Value::Number(current.into()));
                current += step_size;
            }

            Ok(items)
        }
        CollectionSource::Inline { items } => {
            // Use provided items directly
            Ok(items.clone())
        }
        CollectionSource::Http {
            url,
            method,
            headers,
            body,
            format,
            json_path,
        } => {
            // Make HTTP request
            let client = reqwest::Client::new();
            let mut request = match method.to_uppercase().as_str() {
                "GET" => client.get(url),
                "POST" => client.post(url),
                "PUT" => client.put(url),
                "DELETE" => client.delete(url),
                "PATCH" => client.patch(url),
                _ => {
                    return Err(Error::InvalidInput(format!(
                        "Unsupported HTTP method: {}",
                        method
                    )))
                }
            };

            // Add headers if provided
            if let Some(headers_map) = headers {
                for (key, value) in headers_map {
                    request = request.header(key, value);
                }
            }

            // Add body if provided
            if let Some(body_content) = body {
                request = request.body(body_content.clone());
            }

            // Execute request
            let response = request.send().await.map_err(|e| {
                Error::InvalidInput(format!("HTTP request to '{}' failed: {}", url, e))
            })?;

            // Check status
            if !response.status().is_success() {
                return Err(Error::InvalidInput(format!(
                    "HTTP request to '{}' returned status: {}",
                    url,
                    response.status()
                )));
            }

            // Get response body
            let response_text = response.text().await.map_err(|e| {
                Error::InvalidInput(format!("Failed to read response from '{}': {}", url, e))
            })?;

            // Parse based on format
            let mut value: serde_json::Value = match format {
                FileFormat::Json => serde_json::from_str(&response_text).map_err(|e| {
                    Error::InvalidInput(format!(
                        "Failed to parse JSON response from '{}': {}",
                        url, e
                    ))
                })?,
                FileFormat::JsonLines => {
                    let items: Result<Vec<serde_json::Value>> = response_text
                        .lines()
                        .enumerate()
                        .map(|(i, line)| {
                            serde_json::from_str(line).map_err(|e| {
                                Error::InvalidInput(format!(
                                    "Failed to parse JSON line {} from '{}': {}",
                                    i + 1,
                                    url,
                                    e
                                ))
                            })
                        })
                        .collect();
                    serde_json::Value::Array(items?)
                }
                FileFormat::Lines => {
                    let items: Vec<serde_json::Value> = response_text
                        .lines()
                        .map(|line| serde_json::Value::String(line.to_string()))
                        .collect();
                    serde_json::Value::Array(items)
                }
                FileFormat::Csv => {
                    let items: Vec<serde_json::Value> = response_text
                        .lines()
                        .skip(1) // Skip header
                        .map(|line| {
                            let fields: Vec<&str> = line.split(',').collect();
                            serde_json::Value::Array(
                                fields
                                    .iter()
                                    .map(|f| serde_json::Value::String(f.to_string()))
                                    .collect(),
                            )
                        })
                        .collect();
                    serde_json::Value::Array(items)
                }
            };

            // Apply JSON path if provided
            if let Some(path) = json_path {
                value = extract_json_path(&value, path)?;
            }

            // Ensure result is an array
            match value {
                serde_json::Value::Array(arr) => Ok(arr),
                _ => Err(Error::InvalidInput(format!(
                    "HTTP response from '{}' does not contain an array",
                    url
                ))),
            }
        }
    }
}

/// Execute a task with loop specification (ForEach or Repeat only in Phase 2)
///
/// # Arguments
///
/// * `task_id` - Task identifier
/// * `spec` - Task specification with loop_spec
/// * `loop_spec` - Loop specification
/// * `agents` - Shared agent map
/// * `task_graph` - Shared task graph
/// * `state` - Shared workflow state
async fn execute_task_with_loop(
    task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    loop_spec: &LoopSpec,
    ctx: &ExecutionContext<'_>,
) -> Result<()> {
    match loop_spec {
        LoopSpec::ForEach {
            collection,
            iterator,
            parallel,
            ..
        } => {
            // Resolve collection
            let items = {
                let state_guard = ctx.state.lock().await;
                resolve_collection(collection, state_guard.as_ref()).await?
            };

            // Initialize loop state
            {
                let mut state_guard = ctx.state.lock().await;
                if let Some(ref mut workflow_state) = *state_guard {
                    workflow_state.init_loop(task_id, Some(items.len()));
                }
            }

            println!(
                "Executing ForEach loop for task '{}': {} items",
                task_id,
                items.len()
            );

            if *parallel {
                // Parallel execution with semaphore limiting
                let max_parallel = loop_spec.max_parallel().unwrap_or(items.len().min(10)); // Default to 10 or item count
                let workflow_inputs_arc = Arc::new(ctx.workflow_inputs.clone());
                execute_foreach_parallel(
                    task_id,
                    spec,
                    &items,
                    iterator,
                    max_parallel,
                    workflow_inputs_arc,
                    ctx,
                )
                .await?;
            } else {
                // Sequential execution
                execute_foreach_sequential(task_id, spec, &items, iterator, ctx).await?;
            }

            println!("ForEach loop completed for task '{}'", task_id);
            Ok(())
        }
        LoopSpec::Repeat {
            count,
            iterator,
            parallel,
            ..
        } => {
            // Initialize loop state
            {
                let mut state_guard = ctx.state.lock().await;
                if let Some(ref mut workflow_state) = *state_guard {
                    workflow_state.init_loop(task_id, Some(*count));
                }
            }

            println!(
                "Executing Repeat loop for task '{}': {} iterations",
                task_id, count
            );

            if *parallel {
                // Parallel execution with semaphore limiting
                let max_parallel = loop_spec.max_parallel().unwrap_or((*count).min(10)); // Default to 10 or count
                let workflow_inputs_arc = Arc::new(ctx.workflow_inputs.clone());
                execute_repeat_parallel(
                    task_id,
                    spec,
                    *count,
                    iterator.as_deref(),
                    max_parallel,
                    workflow_inputs_arc,
                    ctx,
                )
                .await?;
            } else {
                // Sequential execution
                execute_repeat_sequential(task_id, spec, *count, iterator.as_deref(), ctx).await?;
            }

            println!("Repeat loop completed for task '{}'", task_id);
            Ok(())
        }
        LoopSpec::While {
            condition,
            max_iterations,
            iteration_variable,
            delay_between_secs,
        } => {
            // Initialize loop state (unknown total iterations)
            {
                let mut state_guard = ctx.state.lock().await;
                if let Some(ref mut workflow_state) = *state_guard {
                    workflow_state.init_loop(task_id, None);
                }
            }

            println!(
                "Executing While loop for task '{}': max {} iterations",
                task_id, max_iterations
            );

            // Sequential execution
            execute_while_sequential(
                task_id,
                spec,
                condition,
                *max_iterations,
                iteration_variable.as_deref(),
                *delay_between_secs,
                ctx,
            )
            .await?;

            println!("While loop completed for task '{}'", task_id);
            Ok(())
        }
        LoopSpec::RepeatUntil {
            condition,
            min_iterations,
            max_iterations,
            iteration_variable,
            delay_between_secs,
        } => {
            // Initialize loop state (unknown total iterations)
            {
                let mut state_guard = ctx.state.lock().await;
                if let Some(ref mut workflow_state) = *state_guard {
                    workflow_state.init_loop(task_id, None);
                }
            }

            println!(
                "Executing RepeatUntil loop for task '{}': min {}, max {} iterations",
                task_id,
                min_iterations.unwrap_or(1),
                max_iterations
            );

            // Sequential execution
            execute_repeat_until_sequential(
                task_id,
                spec,
                condition,
                *min_iterations,
                *max_iterations,
                iteration_variable.as_deref(),
                *delay_between_secs,
                ctx,
            )
            .await?;

            println!("RepeatUntil loop completed for task '{}'", task_id);
            Ok(())
        }
    }
}

/// Execute subtasks within a loop iteration
///
/// This function executes all subtasks of a loop task for a single iteration.
/// Subtasks are executed sequentially, respecting their dependencies.
///
/// NOTE: The `substituted_parent` should already have loop variables substituted
/// via `substitute_task_variables`, so the subtasks within it will also have
/// variables properly replaced.
#[allow(clippy::too_many_arguments)]
async fn execute_subtasks_in_loop_iteration(
    parent_task_id: &str,
    substituted_parent: &crate::dsl::schema::TaskSpec,
    workflow_inputs: &HashMap<String, serde_json::Value>,
    agents: &Arc<Mutex<HashMap<String, PeriplonSDKClient>>>,
    task_graph: &Arc<Mutex<TaskGraph>>,
    state: &Arc<Mutex<Option<WorkflowState>>>,
    workflow_name: &Arc<String>,
    json_output: bool,
) -> Result<Option<String>> {
    // Execute each subtask in order
    // NOTE: subtasks are already substituted because they're part of substituted_parent
    for subtask_map in &substituted_parent.subtasks {
        for (subtask_name, subtask_spec) in subtask_map {
            let full_subtask_id = format!("{}.{}", parent_task_id, subtask_name);

            println!(
                "    Executing subtask: {} - {}",
                full_subtask_id, subtask_spec.description
            );

            // Check if subtask has dependencies and if they're met
            if !subtask_spec.depends_on.is_empty() {
                // Check dependencies within the same iteration
                let deps_met = {
                    let graph = task_graph.lock().await;
                    subtask_spec.depends_on.iter().all(|dep| {
                        // For local dependencies (sibling subtasks), check if they completed
                        let dep_full_id = if dep.contains('.') {
                            dep.clone()
                        } else {
                            format!("{}.{}", parent_task_id, dep)
                        };

                        // Check in task graph
                        if let Some(dep_task) = graph.get_task(&dep_full_id) {
                            dep_task.status == TaskStatus::Completed
                        } else {
                            // Dependency not in graph, assume it's met
                            true
                        }
                    })
                };

                if !deps_met {
                    println!(
                        "      Skipping subtask {} - dependencies not met",
                        subtask_name
                    );
                    continue;
                }
            }

            // Check condition if present
            if let Some(ref condition) = subtask_spec.condition {
                let condition_met = {
                    let graph = task_graph.lock().await;
                    let workflow_state = state.lock().await;
                    evaluate_condition(condition, &graph, workflow_state.as_ref())
                };

                if !condition_met {
                    println!(
                        "      Skipping subtask {} - condition not met",
                        subtask_name
                    );
                    continue;
                }
            }

            // Execute subtask
            let result = execute_task_attempt(
                &full_subtask_id,
                &subtask_spec.description,
                subtask_spec,
                workflow_inputs,
                agents,
                0,
                state,
                workflow_name,
                json_output,
            )
            .await;

            // Update status in task graph and handle errors
            match result {
                Ok(_) => {
                    let mut graph = task_graph.lock().await;
                    let _ = graph.update_task_status(&full_subtask_id, TaskStatus::Completed);
                    println!("      ✓ Subtask {} completed", subtask_name);
                }
                Err(e) => {
                    let mut graph = task_graph.lock().await;
                    let _ = graph.update_task_status(&full_subtask_id, TaskStatus::Failed);
                    println!("      ✗ Subtask {} failed: {}", subtask_name, e);
                    // If subtask fails and has no error recovery, fail the iteration
                    return Err(e);
                }
            }
        }
    }

    Ok(None)
}

/// Execute ForEach loop sequentially
async fn execute_foreach_sequential(
    task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    items: &[serde_json::Value],
    iterator: &str,
    ctx: &ExecutionContext<'_>,
) -> Result<()> {
    // Wrap loop execution with timeout if specified
    let timeout_duration = spec
        .loop_control
        .as_ref()
        .and_then(|lc| lc.timeout_secs)
        .map(tokio::time::Duration::from_secs);

    let loop_future = async {
        // Get checkpoint interval if configured
        let checkpoint_interval = spec
            .loop_control
            .as_ref()
            .and_then(|lc| lc.checkpoint_interval)
            .filter(|&interval| interval > 0);

        for (iteration, item) in items.iter().enumerate() {
            // Check if this iteration was already completed (resume capability)
            let already_completed = {
                let state_guard = ctx.state.lock().await;
                if let Some(ref workflow_state) = *state_guard {
                    workflow_state.is_iteration_completed(task_id, iteration)
                } else {
                    false
                }
            };

            if already_completed {
                println!(
                    "  Iteration {}: Already completed, skipping (resume)",
                    iteration + 1
                );
                continue;
            }

            // Check continue condition BEFORE executing iteration
            if let Some(ref loop_control) = spec.loop_control {
                if let Some(ref continue_cond) = loop_control.continue_condition {
                    let should_continue = {
                        let task_graph_guard = ctx.task_graph.lock().await;
                        let state_guard = ctx.state.lock().await;
                        evaluate_condition(continue_cond, &task_graph_guard, state_guard.as_ref())
                    };
                    if should_continue {
                        println!(
                            "  Iteration {}: Skipping due to continue condition",
                            iteration + 1
                        );
                        continue;
                    }
                }
            }

            println!(
                "  Iteration {}/{}: Processing item: {:?}",
                iteration + 1,
                items.len(),
                item
            );

            // Create loop context
            let mut context = LoopContext::new(iteration);
            context.set_variable(iterator.to_string(), item.clone());

            // Substitute variables in task
            let substituted_task = substitute_task_variables(spec, &context);

            // Update loop state
            {
                let mut state_guard = ctx.state.lock().await;
                if let Some(ref mut workflow_state) = *state_guard {
                    workflow_state.update_loop_iteration(
                        task_id,
                        iteration,
                        TaskStatus::Running,
                        Some(item.clone()),
                    );
                }
            }

            // Execute task iteration
            // If the task has subtasks, execute them instead of the parent task
            let result = if !substituted_task.subtasks.is_empty() {
                println!(
                    "  Task has {} subtasks - executing within loop iteration",
                    substituted_task.subtasks.len()
                );
                execute_subtasks_in_loop_iteration(
                    task_id,
                    &substituted_task,
                    ctx.workflow_inputs,
                    ctx.agents,
                    ctx.task_graph,
                    ctx.state,
                    ctx.workflow_name,
                    ctx.json_output,
                )
                .await
            } else {
                execute_task_attempt(
                    &format!("{}[{}]", task_id, iteration),
                    &substituted_task.description,
                    &substituted_task,
                    ctx.workflow_inputs,
                    ctx.agents,
                    0,
                    ctx.state,
                    ctx.workflow_name,
                    ctx.json_output,
                )
                .await
            };

            match result {
                Ok(output) => {
                    // Update iteration status
                    {
                        let mut state_guard = ctx.state.lock().await;
                        if let Some(ref mut workflow_state) = *state_guard {
                            workflow_state.update_loop_iteration(
                                task_id,
                                iteration,
                                TaskStatus::Completed,
                                Some(item.clone()),
                            );

                            // Store result if collection is enabled
                            if spec
                                .loop_control
                                .as_ref()
                                .is_some_and(|lc| lc.collect_results)
                            {
                                if let Some(output_str) = output {
                                    workflow_state.store_loop_result(
                                        task_id,
                                        serde_json::Value::String(output_str),
                                    );
                                }
                            }
                        }
                    }
                    println!("  Iteration {} completed successfully", iteration + 1);

                    // Save checkpoint if configured and interval reached
                    if let Some(interval) = checkpoint_interval {
                        if (iteration + 1) % interval == 0 {
                            let state_guard = ctx.state.lock().await;
                            if let Some(ref workflow_state) = *state_guard {
                                if let Err(e) = workflow_state.save_checkpoint() {
                                    println!("  Warning: Failed to save checkpoint: {}", e);
                                } else {
                                    println!("  Checkpoint saved at iteration {}", iteration + 1);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    // Update iteration status
                    {
                        let mut state_guard = ctx.state.lock().await;
                        if let Some(ref mut workflow_state) = *state_guard {
                            workflow_state.update_loop_iteration(
                                task_id,
                                iteration,
                                TaskStatus::Failed,
                                Some(item.clone()),
                            );
                        }
                    }

                    // Check if we should break on error
                    let break_on_error = spec
                        .loop_control
                        .as_ref()
                        .and_then(|lc| lc.break_condition.as_ref())
                        .is_some();

                    if break_on_error {
                        return Err(e);
                    }

                    println!("  Iteration {} failed: {}", iteration + 1, e);
                }
            }

            // Check break condition AFTER executing iteration
            if let Some(ref loop_control) = spec.loop_control {
                if let Some(ref break_cond) = loop_control.break_condition {
                    let should_break = {
                        let task_graph_guard = ctx.task_graph.lock().await;
                        let state_guard = ctx.state.lock().await;
                        evaluate_condition(break_cond, &task_graph_guard, state_guard.as_ref())
                    };
                    if should_break {
                        println!(
                            "  Breaking loop due to break condition at iteration {}",
                            iteration + 1
                        );
                        break;
                    }
                }
            }
        }

        Ok(())
    };

    // Execute with or without timeout
    if let Some(timeout) = timeout_duration {
        match tokio::time::timeout(timeout, loop_future).await {
            Ok(result) => result,
            Err(_) => {
                println!("  Loop timed out after {:?}", timeout);
                Err(Error::InvalidInput(format!(
                    "Loop '{}' timed out after {} seconds",
                    task_id,
                    timeout.as_secs()
                )))
            }
        }
    } else {
        loop_future.await
    }
}

/// Execute Repeat loop sequentially
async fn execute_repeat_sequential(
    task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    count: usize,
    iterator: Option<&str>,
    ctx: &ExecutionContext<'_>,
) -> Result<()> {
    for iteration in 0..count {
        // Check continue condition BEFORE executing iteration
        if let Some(ref loop_control) = spec.loop_control {
            if let Some(ref continue_cond) = loop_control.continue_condition {
                let should_continue = {
                    let task_graph_guard = ctx.task_graph.lock().await;
                    let state_guard = ctx.state.lock().await;
                    evaluate_condition(continue_cond, &task_graph_guard, state_guard.as_ref())
                };
                if should_continue {
                    println!(
                        "  Iteration {}: Skipping due to continue condition",
                        iteration + 1
                    );
                    continue;
                }
            }
        }

        println!("  Iteration {}/{}", iteration + 1, count);

        // Create loop context
        let mut context = LoopContext::new(iteration);
        if let Some(iter_name) = iterator {
            context.set_variable(
                iter_name.to_string(),
                serde_json::Value::Number(iteration.into()),
            );
        }

        // Substitute variables in task
        let substituted_task = substitute_task_variables(spec, &context);

        // Update loop state
        {
            let mut state_guard = ctx.state.lock().await;
            if let Some(ref mut workflow_state) = *state_guard {
                workflow_state.update_loop_iteration(
                    task_id,
                    iteration,
                    TaskStatus::Running,
                    Some(serde_json::Value::Number(iteration.into())),
                );
            }
        }

        // Execute task iteration
        match execute_task_attempt(
            &format!("{}[{}]", task_id, iteration),
            &substituted_task.description,
            &substituted_task,
            ctx.workflow_inputs,
            ctx.agents,
            0,
            ctx.state,
            ctx.workflow_name,
            ctx.json_output,
        )
        .await
        {
            Ok(output) => {
                // Update iteration status
                {
                    let mut state_guard = ctx.state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            task_id,
                            iteration,
                            TaskStatus::Completed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );

                        // Store result if collection is enabled
                        if spec
                            .loop_control
                            .as_ref()
                            .is_some_and(|lc| lc.collect_results)
                        {
                            if let Some(output_str) = output {
                                workflow_state.store_loop_result(
                                    task_id,
                                    serde_json::Value::String(output_str),
                                );
                            }
                        }
                    }
                }
                println!("  Iteration {} completed successfully", iteration + 1);
            }
            Err(e) => {
                // Update iteration status
                {
                    let mut state_guard = ctx.state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            task_id,
                            iteration,
                            TaskStatus::Failed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );
                    }
                }

                // Check if we should break on error
                let break_on_error = spec
                    .loop_control
                    .as_ref()
                    .and_then(|lc| lc.break_condition.as_ref())
                    .is_some();

                if break_on_error {
                    return Err(e);
                }

                println!("  Iteration {} failed: {}", iteration + 1, e);
            }
        }

        // Check break condition AFTER executing iteration
        if let Some(ref loop_control) = spec.loop_control {
            if let Some(ref break_cond) = loop_control.break_condition {
                let should_break = {
                    let task_graph_guard = ctx.task_graph.lock().await;
                    let state_guard = ctx.state.lock().await;
                    evaluate_condition(break_cond, &task_graph_guard, state_guard.as_ref())
                };
                if should_break {
                    println!(
                        "  Breaking loop due to break condition at iteration {}",
                        iteration + 1
                    );
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Execute While loop sequentially
///
/// Evaluates condition BEFORE each iteration. Stops when condition becomes false.
async fn execute_while_sequential(
    task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    condition: &crate::dsl::schema::ConditionSpec,
    max_iterations: usize,
    iteration_variable: Option<&str>,
    delay_between_secs: Option<u64>,
    ctx: &ExecutionContext<'_>,
) -> Result<()> {
    let mut iteration = 0;

    loop {
        // Check max iterations safety limit
        if iteration >= max_iterations {
            println!(
                "  While loop reached max iterations limit: {}",
                max_iterations
            );
            break;
        }

        // Evaluate condition BEFORE executing iteration
        let condition_met = {
            let state_guard = ctx.state.lock().await;
            let task_graph = TaskGraph::new(); // We need access to task_graph for condition evaluation
            evaluate_condition(condition, &task_graph, state_guard.as_ref())
        };

        if !condition_met {
            println!(
                "  While loop condition became false at iteration {}",
                iteration
            );
            break;
        }

        println!(
            "  Iteration {}: Condition is true, executing...",
            iteration + 1
        );

        // Create loop context
        let mut context = LoopContext::new(iteration);
        if let Some(iter_name) = iteration_variable {
            context.set_variable(
                iter_name.to_string(),
                serde_json::Value::Number(iteration.into()),
            );
        }

        // Substitute variables in task
        let substituted_task = substitute_task_variables(spec, &context);

        // Update loop state
        {
            let mut state_guard = ctx.state.lock().await;
            if let Some(ref mut workflow_state) = *state_guard {
                workflow_state.update_loop_iteration(
                    task_id,
                    iteration,
                    TaskStatus::Running,
                    Some(serde_json::Value::Number(iteration.into())),
                );
            }
        }

        // Execute task iteration
        match execute_task_attempt(
            &format!("{}[{}]", task_id, iteration),
            &substituted_task.description,
            &substituted_task,
            ctx.workflow_inputs,
            ctx.agents,
            0,
            ctx.state,
            ctx.workflow_name,
            ctx.json_output,
        )
        .await
        {
            Ok(output) => {
                // Update iteration status
                {
                    let mut state_guard = ctx.state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            task_id,
                            iteration,
                            TaskStatus::Completed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );

                        // Store result if collection is enabled
                        if spec
                            .loop_control
                            .as_ref()
                            .is_some_and(|lc| lc.collect_results)
                        {
                            if let Some(output_str) = output {
                                workflow_state.store_loop_result(
                                    task_id,
                                    serde_json::Value::String(output_str),
                                );
                            }
                        }
                    }
                }
                println!("  Iteration {} completed successfully", iteration + 1);
            }
            Err(e) => {
                // Update iteration status
                {
                    let mut state_guard = ctx.state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            task_id,
                            iteration,
                            TaskStatus::Failed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );
                    }
                }

                // Check if we should break on error
                let break_on_error = spec
                    .loop_control
                    .as_ref()
                    .and_then(|lc| lc.break_condition.as_ref())
                    .is_some();

                if break_on_error {
                    return Err(e);
                }

                println!("  Iteration {} failed: {}", iteration + 1, e);
            }
        }

        iteration += 1;

        // Apply delay if configured
        if let Some(delay_secs) = delay_between_secs {
            if delay_secs > 0 {
                println!("  Waiting {} seconds before next iteration...", delay_secs);
                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
            }
        }
    }

    println!("  While loop completed after {} iterations", iteration);
    Ok(())
}

/// Execute RepeatUntil loop sequentially
///
/// Executes at least once (or min_iterations times), evaluates condition AFTER each iteration.
/// Stops when condition becomes true.
#[allow(clippy::too_many_arguments)]
async fn execute_repeat_until_sequential(
    task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    condition: &crate::dsl::schema::ConditionSpec,
    min_iterations: Option<usize>,
    max_iterations: usize,
    iteration_variable: Option<&str>,
    delay_between_secs: Option<u64>,
    ctx: &ExecutionContext<'_>,
) -> Result<()> {
    let min = min_iterations.unwrap_or(1);
    let mut iteration = 0;

    loop {
        // Check max iterations safety limit
        if iteration >= max_iterations {
            println!(
                "  RepeatUntil loop reached max iterations limit: {}",
                max_iterations
            );
            break;
        }

        println!("  Iteration {}", iteration + 1);

        // Create loop context
        let mut context = LoopContext::new(iteration);
        if let Some(iter_name) = iteration_variable {
            context.set_variable(
                iter_name.to_string(),
                serde_json::Value::Number(iteration.into()),
            );
        }

        // Substitute variables in task
        let substituted_task = substitute_task_variables(spec, &context);

        // Update loop state
        {
            let mut state_guard = ctx.state.lock().await;
            if let Some(ref mut workflow_state) = *state_guard {
                workflow_state.update_loop_iteration(
                    task_id,
                    iteration,
                    TaskStatus::Running,
                    Some(serde_json::Value::Number(iteration.into())),
                );
            }
        }

        // Execute task iteration
        match execute_task_attempt(
            &format!("{}[{}]", task_id, iteration),
            &substituted_task.description,
            &substituted_task,
            ctx.workflow_inputs,
            ctx.agents,
            0,
            ctx.state,
            ctx.workflow_name,
            ctx.json_output,
        )
        .await
        {
            Ok(output) => {
                // Update iteration status
                {
                    let mut state_guard = ctx.state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            task_id,
                            iteration,
                            TaskStatus::Completed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );

                        // Store result if collection is enabled
                        if spec
                            .loop_control
                            .as_ref()
                            .is_some_and(|lc| lc.collect_results)
                        {
                            if let Some(output_str) = output {
                                workflow_state.store_loop_result(
                                    task_id,
                                    serde_json::Value::String(output_str),
                                );
                            }
                        }
                    }
                }
                println!("  Iteration {} completed successfully", iteration + 1);
            }
            Err(e) => {
                // Update iteration status
                {
                    let mut state_guard = ctx.state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            task_id,
                            iteration,
                            TaskStatus::Failed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );
                    }
                }

                // Check if we should break on error
                let break_on_error = spec
                    .loop_control
                    .as_ref()
                    .and_then(|lc| lc.break_condition.as_ref())
                    .is_some();

                if break_on_error {
                    return Err(e);
                }

                println!("  Iteration {} failed: {}", iteration + 1, e);
            }
        }

        iteration += 1;

        // Evaluate condition AFTER executing iteration
        let condition_met = {
            let state_guard = ctx.state.lock().await;
            let task_graph = TaskGraph::new(); // We need access to task_graph for condition evaluation
            evaluate_condition(condition, &task_graph, state_guard.as_ref())
        };

        // Stop if condition is met AND we've done minimum iterations
        if condition_met && iteration >= min {
            println!(
                "  RepeatUntil condition became true at iteration {}",
                iteration
            );
            break;
        }

        // Apply delay if configured
        if let Some(delay_secs) = delay_between_secs {
            if delay_secs > 0 {
                println!("  Waiting {} seconds before next iteration...", delay_secs);
                tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
            }
        }
    }

    println!(
        "  RepeatUntil loop completed after {} iterations",
        iteration
    );
    Ok(())
}

/// Execute ForEach loop in parallel with semaphore-based concurrency limiting
///
/// Spawns tasks for each iteration but limits concurrent execution via Semaphore.
async fn execute_foreach_parallel(
    task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    items: &[serde_json::Value],
    iterator: &str,
    max_parallel: usize,
    workflow_inputs: Arc<HashMap<String, serde_json::Value>>,
    ctx: &ExecutionContext<'_>,
) -> Result<()> {
    use tokio::task::JoinSet;

    println!(
        "  Executing ForEach in parallel: {} items, max {} concurrent",
        items.len(),
        max_parallel
    );

    // Clone items to owned Vec to avoid lifetime issues
    let items_owned: Vec<_> = items.to_vec();
    let total_items = items_owned.len();

    // Create semaphore to limit concurrency
    let semaphore = Arc::new(Semaphore::new(max_parallel));

    // Create join set to track spawned tasks
    let mut join_set = JoinSet::new();

    // Spawn tasks for each iteration
    for (iteration, item) in items_owned.into_iter().enumerate() {
        let task_id = task_id.to_string();
        let spec = spec.clone();
        let iterator = iterator.to_string();
        let agents = ctx.agents.clone();
        let state = ctx.state.clone();
        let semaphore = semaphore.clone();
        let workflow_inputs_clone = workflow_inputs.clone();
        let workflow_name_clone = ctx.workflow_name.clone();
        let json_output = ctx.json_output;

        join_set.spawn(async move {
            // Acquire semaphore permit
            let _permit = semaphore.acquire().await.unwrap();

            println!(
                "  [Parallel] Iteration {}/{}: Processing item: {:?}",
                iteration + 1,
                total_items,
                item
            );

            // Create loop context
            let mut context = LoopContext::new(iteration);
            context.set_variable(iterator, item.clone()); // Clone for state tracking

            // Substitute variables in task
            let substituted_task = substitute_task_variables(&spec, &context);

            // Update loop state to Running
            {
                let mut state_guard = state.lock().await;
                if let Some(ref mut workflow_state) = *state_guard {
                    workflow_state.update_loop_iteration(
                        &task_id,
                        iteration,
                        TaskStatus::Running,
                        Some(item.clone()),
                    );
                }
            }

            // Execute task iteration
            let result = execute_task_attempt(
                &format!("{}[{}]", task_id, iteration),
                &substituted_task.description,
                &substituted_task,
                &workflow_inputs_clone,
                &agents,
                0,
                &state,
                &workflow_name_clone,
                json_output,
            )
            .await;

            // Update state based on result
            match result {
                Ok(output) => {
                    let mut state_guard = state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            &task_id,
                            iteration,
                            TaskStatus::Completed,
                            Some(item.clone()),
                        );

                        // Store result if collection is enabled
                        if spec
                            .loop_control
                            .as_ref()
                            .is_some_and(|lc| lc.collect_results)
                        {
                            if let Some(output_str) = output {
                                workflow_state.store_loop_result(
                                    &task_id,
                                    serde_json::Value::String(output_str),
                                );
                            }
                        }
                    }
                    println!(
                        "  [Parallel] Iteration {} completed successfully",
                        iteration + 1
                    );
                    Ok(())
                }
                Err(e) => {
                    let mut state_guard = state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            &task_id,
                            iteration,
                            TaskStatus::Failed,
                            Some(item),
                        );
                    }
                    println!("  [Parallel] Iteration {} failed: {}", iteration + 1, e);
                    Err(e)
                }
            }
        });
    }

    // Wait for all tasks to complete and collect results
    let mut errors = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => {
                // Task completed successfully
            }
            Ok(Err(e)) => {
                // Task failed
                errors.push(e);
            }
            Err(join_err) => {
                // Join error (task panicked)
                errors.push(Error::InvalidInput(format!("Task panicked: {}", join_err)));
            }
        }
    }

    // Return first error if any occurred
    if let Some(error) = errors.into_iter().next() {
        return Err(error);
    }

    Ok(())
}

/// Execute Repeat loop in parallel with semaphore-based concurrency limiting
///
/// Spawns tasks for each iteration but limits concurrent execution via Semaphore.
async fn execute_repeat_parallel(
    task_id: &str,
    spec: &crate::dsl::schema::TaskSpec,
    count: usize,
    iterator: Option<&str>,
    max_parallel: usize,
    workflow_inputs: Arc<HashMap<String, serde_json::Value>>,
    ctx: &ExecutionContext<'_>,
) -> Result<()> {
    use tokio::task::JoinSet;

    println!(
        "  Executing Repeat in parallel: {} iterations, max {} concurrent",
        count, max_parallel
    );

    // Create semaphore to limit concurrency
    let semaphore = Arc::new(Semaphore::new(max_parallel));

    // Create join set to track spawned tasks
    let mut join_set = JoinSet::new();

    // Spawn tasks for each iteration
    for iteration in 0..count {
        let task_id = task_id.to_string();
        let spec = spec.clone();
        let iterator = iterator.map(|s| s.to_string());
        let agents = ctx.agents.clone();
        let state = ctx.state.clone();
        let semaphore = semaphore.clone();
        let workflow_inputs_clone = workflow_inputs.clone();
        let workflow_name_clone = ctx.workflow_name.clone();
        let json_output = ctx.json_output;

        join_set.spawn(async move {
            // Acquire semaphore permit
            let _permit = semaphore.acquire().await.unwrap();

            println!("  [Parallel] Iteration {}/{}", iteration + 1, count);

            // Create loop context
            let mut context = LoopContext::new(iteration);
            if let Some(iter_name) = iterator {
                context.set_variable(iter_name, serde_json::Value::Number(iteration.into()));
            }

            // Substitute variables in task
            let substituted_task = substitute_task_variables(&spec, &context);

            // Update loop state to Running
            {
                let mut state_guard = state.lock().await;
                if let Some(ref mut workflow_state) = *state_guard {
                    workflow_state.update_loop_iteration(
                        &task_id,
                        iteration,
                        TaskStatus::Running,
                        Some(serde_json::Value::Number(iteration.into())),
                    );
                }
            }

            // Execute task iteration
            let result = execute_task_attempt(
                &format!("{}[{}]", task_id, iteration),
                &substituted_task.description,
                &substituted_task,
                &workflow_inputs_clone,
                &agents,
                0,
                &state,
                &workflow_name_clone,
                json_output,
            )
            .await;

            // Update state based on result
            match result {
                Ok(output) => {
                    let mut state_guard = state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            &task_id,
                            iteration,
                            TaskStatus::Completed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );

                        // Store result if collection is enabled
                        if spec
                            .loop_control
                            .as_ref()
                            .is_some_and(|lc| lc.collect_results)
                        {
                            if let Some(output_str) = output {
                                workflow_state.store_loop_result(
                                    &task_id,
                                    serde_json::Value::String(output_str),
                                );
                            }
                        }
                    }
                    println!(
                        "  [Parallel] Iteration {} completed successfully",
                        iteration + 1
                    );
                    Ok(())
                }
                Err(e) => {
                    let mut state_guard = state.lock().await;
                    if let Some(ref mut workflow_state) = *state_guard {
                        workflow_state.update_loop_iteration(
                            &task_id,
                            iteration,
                            TaskStatus::Failed,
                            Some(serde_json::Value::Number(iteration.into())),
                        );
                    }
                    println!("  [Parallel] Iteration {} failed: {}", iteration + 1, e);
                    Err(e)
                }
            }
        });
    }

    // Wait for all tasks to complete and collect results
    let mut errors = Vec::new();
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => {
                // Task completed successfully
            }
            Ok(Err(e)) => {
                // Task failed
                errors.push(e);
            }
            Err(join_err) => {
                // Join error (task panicked)
                errors.push(Error::InvalidInput(format!("Task panicked: {}", join_err)));
            }
        }
    }

    // Return first error if any occurred
    if let Some(error) = errors.into_iter().next() {
        return Err(error);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Provider;
    use crate::dsl::schema::PermissionsSpec;

    #[test]
    fn test_executor_creation() {
        let workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test Workflow".to_string(),
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

        let executor = DSLExecutor::new(workflow).unwrap();
        let (name, version) = executor.get_workflow_info();
        assert_eq!(name, "Test Workflow");
        assert_eq!(version, "1.0.0");
    }

    #[test]
    fn test_agent_spec_to_options() {
        let workflow = DSLWorkflow {
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
            provider: None,
            description: "Test agent".to_string(),
            model: Some("claude-sonnet-4-5".to_string()),
            system_prompt: None,
            cwd: None,
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec!["Read".to_string(), "Write".to_string()],
            permissions: PermissionsSpec {
                mode: "acceptEdits".to_string(),
                allowed_directories: vec!["./src".to_string(), "/tmp/test".to_string()],
            },
            max_turns: Some(10),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, None, None, &var_context)
            .unwrap();
        assert_eq!(options.allowed_tools.len(), 2);
        assert_eq!(options.model, Some("claude-sonnet-4-5".to_string()));
        assert_eq!(options.max_turns, Some(10));
        assert_eq!(options.permission_mode, Some("acceptEdits".to_string()));
        assert_eq!(options.add_dirs.len(), 2);
        assert_eq!(options.add_dirs[0], std::path::PathBuf::from("./src"));
        assert_eq!(options.add_dirs[1], std::path::PathBuf::from("/tmp/test"));
        assert_eq!(options.cwd, None); // No cwd set
        assert!(!options.create_cwd); // Default is false
    }

    #[test]
    fn test_cwd_workflow_level_only() {
        let workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: Some("/workflow/dir".to_string()),
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
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
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, Some("/workflow/dir"), None, &var_context)
            .unwrap();
        assert_eq!(options.cwd, Some(std::path::PathBuf::from("/workflow/dir")));
    }

    #[test]
    fn test_cwd_agent_level_only() {
        let workflow = DSLWorkflow {
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
            provider: None,
            description: "Test agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: Some("/agent/dir".to_string()),
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec![],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, None, None, &var_context)
            .unwrap();
        assert_eq!(options.cwd, Some(std::path::PathBuf::from("/agent/dir")));
    }

    #[test]
    fn test_cwd_agent_overrides_workflow() {
        let workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: Some("/workflow/dir".to_string()),
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
            provider: None,
            description: "Test agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: Some("/agent/dir".to_string()),
            create_cwd: None,
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec![],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, Some("/workflow/dir"), None, &var_context)
            .unwrap();
        // Agent-level cwd should override workflow-level
        assert_eq!(options.cwd, Some(std::path::PathBuf::from("/agent/dir")));
    }

    #[test]
    fn test_cwd_backwards_compatibility() {
        let workflow = DSLWorkflow {
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
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
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, None, None, &var_context)
            .unwrap();
        // No cwd set - should default to None (current directory)
        assert_eq!(options.cwd, None);
    }

    #[test]
    fn test_create_cwd_workflow_level() {
        let workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: Some("/tmp/test".to_string()),
            create_cwd: Some(true),
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
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
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, Some("/tmp/test"), Some(true), &var_context)
            .unwrap();
        assert!(options.create_cwd);
    }

    #[test]
    fn test_create_cwd_agent_overrides_workflow() {
        let workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: Some("/tmp/test".to_string()),
            create_cwd: Some(true),
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
            provider: None,
            description: "Test agent".to_string(),
            model: None,
            system_prompt: None,
            cwd: Some("/agent/dir".to_string()),
            create_cwd: Some(false), // Agent explicitly disables create_cwd
            inputs: HashMap::new(),
            outputs: HashMap::new(),
            tools: vec![],
            permissions: PermissionsSpec::default(),
            max_turns: None,
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, Some("/tmp/test"), Some(true), &var_context)
            .unwrap();
        // Agent-level create_cwd should override workflow-level
        assert!(!options.create_cwd);
    }

    #[test]
    fn test_create_cwd_defaults_to_false() {
        let workflow = DSLWorkflow {
            provider: Provider::Claude,
            model: None,
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            dsl_version: "1.0.0".to_string(),
            cwd: Some("/tmp/test".to_string()),
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

        let executor = DSLExecutor::new(workflow).unwrap();

        let agent_spec = AgentSpec {
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
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let options = executor
            .agent_spec_to_options(&agent_spec, Some("/tmp/test"), None, &var_context)
            .unwrap();
        // No create_cwd set - should default to false
        assert!(!options.create_cwd);
    }

    #[test]
    fn test_condition_always() {
        use crate::dsl::schema::{Condition, ConditionSpec};

        let task_graph = TaskGraph::new();
        let condition = ConditionSpec::Single(Condition::Always);
        assert!(evaluate_condition(&condition, &task_graph, None));
    }

    #[test]
    fn test_condition_never() {
        use crate::dsl::schema::{Condition, ConditionSpec};

        let task_graph = TaskGraph::new();
        let condition = ConditionSpec::Single(Condition::Never);
        assert!(!evaluate_condition(&condition, &task_graph, None));
    }

    #[test]
    fn test_condition_task_status_completed() {
        use crate::dsl::schema::{Condition, ConditionSpec, TaskSpec, TaskStatusCondition};

        let mut task_graph = TaskGraph::new();
        task_graph.add_task("task1".to_string(), TaskSpec::default());
        task_graph
            .update_task_status("task1", TaskStatus::Completed)
            .unwrap();

        let condition = ConditionSpec::Single(Condition::TaskStatus {
            task: "task1".to_string(),
            status: TaskStatusCondition::Completed,
        });

        assert!(evaluate_condition(&condition, &task_graph, None));
    }

    #[test]
    fn test_condition_task_status_failed() {
        use crate::dsl::schema::{Condition, ConditionSpec, TaskSpec, TaskStatusCondition};

        let mut task_graph = TaskGraph::new();
        task_graph.add_task("task1".to_string(), TaskSpec::default());
        task_graph
            .update_task_status("task1", TaskStatus::Failed)
            .unwrap();

        let condition = ConditionSpec::Single(Condition::TaskStatus {
            task: "task1".to_string(),
            status: TaskStatusCondition::Failed,
        });

        assert!(evaluate_condition(&condition, &task_graph, None));
    }

    #[test]
    fn test_condition_task_status_skipped() {
        use crate::dsl::schema::{Condition, ConditionSpec, TaskSpec, TaskStatusCondition};

        let mut task_graph = TaskGraph::new();
        task_graph.add_task("task1".to_string(), TaskSpec::default());
        task_graph
            .update_task_status("task1", TaskStatus::Skipped)
            .unwrap();

        let condition = ConditionSpec::Single(Condition::TaskStatus {
            task: "task1".to_string(),
            status: TaskStatusCondition::Skipped,
        });

        assert!(evaluate_condition(&condition, &task_graph, None));
    }

    #[test]
    fn test_condition_state_equals() {
        use crate::dsl::schema::{Condition, ConditionSpec};

        let task_graph = TaskGraph::new();
        let mut workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
        workflow_state.add_metadata("environment".to_string(), serde_json::json!("production"));

        let condition = ConditionSpec::Single(Condition::StateEquals {
            key: "environment".to_string(),
            value: serde_json::json!("production"),
        });

        assert!(evaluate_condition(
            &condition,
            &task_graph,
            Some(&workflow_state)
        ));
    }

    #[test]
    fn test_condition_state_exists() {
        use crate::dsl::schema::{Condition, ConditionSpec};

        let task_graph = TaskGraph::new();
        let mut workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
        workflow_state.add_metadata("key1".to_string(), serde_json::json!("value1"));

        let condition = ConditionSpec::Single(Condition::StateExists {
            key: "key1".to_string(),
        });

        assert!(evaluate_condition(
            &condition,
            &task_graph,
            Some(&workflow_state)
        ));

        let condition_not_exists = ConditionSpec::Single(Condition::StateExists {
            key: "key2".to_string(),
        });

        assert!(!evaluate_condition(
            &condition_not_exists,
            &task_graph,
            Some(&workflow_state)
        ));
    }

    #[test]
    fn test_condition_and() {
        use crate::dsl::schema::{Condition, ConditionSpec};

        let task_graph = TaskGraph::new();
        let condition = ConditionSpec::And {
            and: vec![
                ConditionSpec::Single(Condition::Always),
                ConditionSpec::Single(Condition::Always),
            ],
        };

        assert!(evaluate_condition(&condition, &task_graph, None));

        let condition_with_never = ConditionSpec::And {
            and: vec![
                ConditionSpec::Single(Condition::Always),
                ConditionSpec::Single(Condition::Never),
            ],
        };

        assert!(!evaluate_condition(
            &condition_with_never,
            &task_graph,
            None
        ));
    }

    #[test]
    fn test_condition_or() {
        use crate::dsl::schema::{Condition, ConditionSpec};

        let task_graph = TaskGraph::new();
        let condition = ConditionSpec::Or {
            or: vec![
                ConditionSpec::Single(Condition::Always),
                ConditionSpec::Single(Condition::Never),
            ],
        };

        assert!(evaluate_condition(&condition, &task_graph, None));

        let condition_all_never = ConditionSpec::Or {
            or: vec![
                ConditionSpec::Single(Condition::Never),
                ConditionSpec::Single(Condition::Never),
            ],
        };

        assert!(!evaluate_condition(&condition_all_never, &task_graph, None));
    }

    #[test]
    fn test_condition_not() {
        use crate::dsl::schema::{Condition, ConditionSpec};

        let task_graph = TaskGraph::new();
        let condition = ConditionSpec::Not {
            not: Box::new(ConditionSpec::Single(Condition::Never)),
        };

        assert!(evaluate_condition(&condition, &task_graph, None));

        let condition_not_always = ConditionSpec::Not {
            not: Box::new(ConditionSpec::Single(Condition::Always)),
        };

        assert!(!evaluate_condition(
            &condition_not_always,
            &task_graph,
            None
        ));
    }

    #[test]
    fn test_condition_complex() {
        use crate::dsl::schema::{Condition, ConditionSpec, TaskSpec, TaskStatusCondition};

        let mut task_graph = TaskGraph::new();
        task_graph.add_task("task1".to_string(), TaskSpec::default());
        task_graph
            .update_task_status("task1", TaskStatus::Completed)
            .unwrap();

        let mut workflow_state = WorkflowState::new("test".to_string(), "1.0.0".to_string());
        workflow_state.add_metadata("env".to_string(), serde_json::json!("prod"));

        // (task1 completed) AND (env == "prod")
        let condition = ConditionSpec::And {
            and: vec![
                ConditionSpec::Single(Condition::TaskStatus {
                    task: "task1".to_string(),
                    status: TaskStatusCondition::Completed,
                }),
                ConditionSpec::Single(Condition::StateEquals {
                    key: "env".to_string(),
                    value: serde_json::json!("prod"),
                }),
            ],
        };

        assert!(evaluate_condition(
            &condition,
            &task_graph,
            Some(&workflow_state)
        ));
    }

    #[tokio::test]
    async fn test_dod_file_exists_met() {
        use crate::dsl::schema::DoneCriterion;

        // Create a temp file
        let temp_file = std::env::temp_dir().join("test_dod_file_exists.txt");
        std::fs::write(&temp_file, "test content").unwrap();

        let criterion = DoneCriterion::FileExists {
            path: temp_file.to_string_lossy().to_string(),
            description: "Test file must exist".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(result.met);
        assert_eq!(result.description, "Test file must exist");

        // Cleanup
        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_dod_file_exists_not_met() {
        use crate::dsl::schema::DoneCriterion;

        let nonexistent_file = std::env::temp_dir().join("nonexistent_file_xyz.txt");
        let criterion = DoneCriterion::FileExists {
            path: nonexistent_file.to_string_lossy().to_string(),
            description: "Nonexistent file".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(!result.met);
        assert!(result.details.contains("does not exist"));
    }

    #[tokio::test]
    async fn test_dod_file_contains_met() {
        use crate::dsl::schema::DoneCriterion;

        let temp_file = std::env::temp_dir().join("test_dod_file_contains.txt");
        std::fs::write(&temp_file, "Hello World\nTest Content").unwrap();

        let criterion = DoneCriterion::FileContains {
            path: temp_file.to_string_lossy().to_string(),
            pattern: "Hello World".to_string(),
            description: "File must contain greeting".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(result.met);

        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_dod_file_contains_regex_pattern() {
        use crate::dsl::schema::DoneCriterion;

        let temp_file = std::env::temp_dir().join("test_dod_file_contains_regex.txt");
        std::fs::write(
            &temp_file,
            "group_list\ngroup_install\ngroup_update\ngroup_validate",
        )
        .unwrap();

        // Test regex pattern matching with .*
        let criterion = DoneCriterion::FileContains {
            path: temp_file.to_string_lossy().to_string(),
            pattern: "group.*list".to_string(),
            description: "File must contain group_list".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(
            result.met,
            "Regex pattern 'group.*list' should match 'group_list'"
        );

        // Test another regex pattern
        let criterion2 = DoneCriterion::FileContains {
            path: temp_file.to_string_lossy().to_string(),
            pattern: "group.*(update|validate)".to_string(),
            description: "File must contain group_update or group_validate".to_string(),
        };

        let result2 = check_criterion(&criterion2, None, &var_context).await;
        assert!(
            result2.met,
            "Regex pattern 'group.*(update|validate)' should match"
        );

        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_dod_file_not_contains_met() {
        use crate::dsl::schema::DoneCriterion;

        let temp_file = std::env::temp_dir().join("test_dod_file_not_contains.txt");
        std::fs::write(&temp_file, "Clean code").unwrap();

        let criterion = DoneCriterion::FileNotContains {
            path: temp_file.to_string_lossy().to_string(),
            pattern: "TODO".to_string(),
            description: "No TODOs allowed".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(result.met);

        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_dod_file_not_contains_failed() {
        use crate::dsl::schema::DoneCriterion;

        let temp_file = std::env::temp_dir().join("test_dod_file_not_contains_fail.txt");
        std::fs::write(&temp_file, "Code with TODO: fix this").unwrap();

        let criterion = DoneCriterion::FileNotContains {
            path: temp_file.to_string_lossy().to_string(),
            pattern: "TODO".to_string(),
            description: "No TODOs allowed".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(!result.met);
        assert!(result.details.contains("should not"));

        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_dod_directory_exists() {
        use crate::dsl::schema::DoneCriterion;

        let temp_dir = std::env::temp_dir().join("test_dod_directory");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let criterion = DoneCriterion::DirectoryExists {
            path: temp_dir.to_string_lossy().to_string(),
            description: "Output directory must exist".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(result.met);

        std::fs::remove_dir(temp_dir).ok();
    }

    #[tokio::test]
    async fn test_dod_command_succeeds() {
        use crate::dsl::schema::DoneCriterion;

        let criterion = DoneCriterion::CommandSucceeds {
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            description: "Echo command should succeed".to_string(),
            working_dir: None,
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(result.met);
    }

    #[tokio::test]
    async fn test_dod_command_fails() {
        use crate::dsl::schema::DoneCriterion;

        let criterion = DoneCriterion::CommandSucceeds {
            command: "false".to_string(),
            args: vec![],
            description: "False command should fail".to_string(),
            working_dir: None,
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;
        assert!(!result.met);
        assert!(result.details.contains("failed"));
    }

    #[tokio::test]
    async fn test_dod_command_captures_output() {
        use crate::dsl::schema::DoneCriterion;

        // Use a shell command that writes to both stdout and stderr, then fails
        let criterion = DoneCriterion::CommandSucceeds {
            command: "sh".to_string(),
            args: vec![
                "-c".to_string(),
                "echo 'This is stdout output'; echo 'This is stderr output' >&2; exit 1"
                    .to_string(),
            ],
            description: "Command that outputs to stdout/stderr and fails".to_string(),
            working_dir: None,
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, None, &var_context).await;

        // Verify command failed
        assert!(!result.met);

        // Verify stdout and stderr are captured in details
        assert!(result.details.contains("STDOUT"));
        assert!(result.details.contains("This is stdout output"));
        assert!(result.details.contains("STDERR"));
        assert!(result.details.contains("This is stderr output"));
    }

    #[tokio::test]
    async fn test_dod_output_matches() {
        use crate::dsl::schema::{DoneCriterion, OutputSource};

        let task_output = "Build succeeded\nAll tests passed";

        let criterion = DoneCriterion::OutputMatches {
            source: OutputSource::TaskOutput,
            pattern: "tests passed".to_string(),
            description: "Output must indicate tests passed".to_string(),
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let result = check_criterion(&criterion, Some(task_output), &var_context).await;
        assert!(result.met);
    }

    #[test]
    fn test_format_unmet_criteria() {
        use crate::dsl::schema::CriterionResult;

        let results = vec![
            CriterionResult {
                met: true,
                description: "File exists".to_string(),
                details: "File found".to_string(),
            },
            CriterionResult {
                met: false,
                description: "Tests must pass".to_string(),
                details: "Tests failed with 3 errors".to_string(),
            },
            CriterionResult {
                met: false,
                description: "No TODO comments".to_string(),
                details: "Found TODO in line 42".to_string(),
            },
        ];

        let feedback = format_unmet_criteria(&results);
        assert!(feedback.contains("DEFINITION OF DONE"));
        assert!(feedback.contains("Tests must pass"));
        assert!(feedback.contains("No TODO comments"));
        assert!(feedback.contains("✗ FAILED"));
        assert!(!feedback.contains("File exists")); // Met criteria not included
    }

    #[tokio::test]
    async fn test_check_definition_of_done_all_met() {
        use crate::dsl::schema::{DefinitionOfDone, DoneCriterion};

        let temp_file = std::env::temp_dir().join("test_dod_all_met.txt");
        std::fs::write(&temp_file, "Complete").unwrap();

        let temp_file_str = temp_file.to_string_lossy().to_string();
        let dod = DefinitionOfDone {
            criteria: vec![
                DoneCriterion::FileExists {
                    path: temp_file_str.clone(),
                    description: "File must exist".to_string(),
                },
                DoneCriterion::FileContains {
                    path: temp_file_str,
                    pattern: "Complete".to_string(),
                    description: "File must be complete".to_string(),
                },
            ],
            max_retries: 3,
            fail_on_unmet: true,
            auto_elevate_permissions: false,
        };

        let var_context = crate::dsl::variables::VariableContext::new();
        let results = check_definition_of_done(&dod, None, &var_context).await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.met));

        std::fs::remove_file(temp_file).ok();
    }

    #[tokio::test]
    async fn test_check_definition_of_done_some_unmet() {
        use crate::dsl::schema::{DefinitionOfDone, DoneCriterion};

        let existing_file = std::env::temp_dir().join("existing_file.txt");
        let nonexistent_file = std::env::temp_dir().join("nonexistent_xyz_abc.txt");

        let dod = DefinitionOfDone {
            criteria: vec![
                DoneCriterion::FileExists {
                    path: existing_file.to_string_lossy().to_string(),
                    description: "File must exist".to_string(),
                },
                DoneCriterion::FileExists {
                    path: nonexistent_file.to_string_lossy().to_string(),
                    description: "Another file must exist".to_string(),
                },
            ],
            max_retries: 2,
            fail_on_unmet: true,
            auto_elevate_permissions: false,
        };

        // Create one file but not the other
        std::fs::write(&existing_file, "test").unwrap();

        let var_context = crate::dsl::variables::VariableContext::new();
        let results = check_definition_of_done(&dod, None, &var_context).await;
        assert_eq!(results.len(), 2);
        assert!(results[0].met);
        assert!(!results[1].met);

        std::fs::remove_file(existing_file).ok();
    }
}
