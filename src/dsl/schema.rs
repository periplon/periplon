//! DSL Schema Definitions
//!
//! This module defines the type structures for the DSL, including agents, tasks,
//! workflows, tools, and communication protocols.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Root DSL workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DSLWorkflow {
    /// Workflow name
    pub name: String,
    /// Workflow version
    pub version: String,
    /// DSL grammar version (tracks DSL syntax version for compatibility)
    #[serde(
        default = "default_dsl_version",
        skip_serializing_if = "is_default_dsl_version"
    )]
    pub dsl_version: String,
    /// Working directory for all agents (can be overridden per agent)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Create working directory if it doesn't exist (can be overridden per agent)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_cwd: Option<bool>,
    /// Secret definitions for secure credential management
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub secrets: HashMap<String, SecretSpec>,
    /// Input variable definitions for the workflow
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, InputSpec>,
    /// Output variable definitions for the workflow
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, OutputSpec>,
    /// Agent definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub agents: HashMap<String, AgentSpec>,
    /// Task definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tasks: HashMap<String, TaskSpec>,
    /// Workflow orchestration definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub workflows: HashMap<String, WorkflowSpec>,
    /// Tool configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsConfig>,
    /// Communication configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub communication: Option<CommunicationConfig>,
    /// MCP server configuration
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub mcp_servers: HashMap<String, McpServerSpec>,
    /// Subflow definitions (can be inline or referenced from external sources)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub subflows: HashMap<String, SubflowSpec>,
    /// Task group imports (namespace -> group reference string)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub imports: HashMap<String, String>,
    /// Default notification settings for the workflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notifications: Option<NotificationDefaults>,
    /// Stdio and context management limits
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<LimitsConfig>,
}

/// Agent specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    /// Agent description
    pub description: String,
    /// Model to use (e.g., "claude-sonnet-4-5")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// System prompt
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    /// Working directory for this agent (overrides workflow-level cwd)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Create working directory if it doesn't exist (overrides workflow-level setting)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_cwd: Option<bool>,
    /// Input variable definitions for the agent
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, InputSpec>,
    /// Output variable definitions for the agent
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, OutputSpec>,
    /// Allowed tools
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<String>,
    /// Permission settings
    #[serde(default, skip_serializing_if = "is_default_permissions")]
    pub permissions: PermissionsSpec,
    /// Maximum number of turns
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_turns: Option<u32>,
}

/// Permission specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PermissionsSpec {
    /// Permission mode: default, acceptEdits, plan, bypassPermissions
    #[serde(
        default = "default_permission_mode",
        skip_serializing_if = "is_default_permission_mode"
    )]
    pub mode: String,
    /// Allowed directories
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_directories: Vec<String>,
}

impl Default for PermissionsSpec {
    fn default() -> Self {
        Self {
            mode: "default".to_string(),
            allowed_directories: Vec::new(),
        }
    }
}

fn default_permission_mode() -> String {
    "default".to_string()
}

fn default_dsl_version() -> String {
    // Import from template module
    "1.0.0".to_string()
}

fn is_default_dsl_version(version: &str) -> bool {
    version == "1.0.0"
}

fn is_default_permission_mode(mode: &str) -> bool {
    mode == "default"
}

fn is_default_permissions(perms: &PermissionsSpec) -> bool {
    perms == &PermissionsSpec::default()
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

fn is_zero_u32(value: &u32) -> bool {
    *value == 0
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_default_retry_delay(value: &u64) -> bool {
    *value == 1
}

fn is_sequential_mode(mode: &ExecutionMode) -> bool {
    *mode == ExecutionMode::Sequential
}

/// Condition specification for conditional task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionSpec {
    /// Single condition
    Single(Condition),
    /// Combined conditions with logical operators
    And {
        and: Vec<ConditionSpec>,
    },
    Or {
        or: Vec<ConditionSpec>,
    },
    Not {
        not: Box<ConditionSpec>,
    },
}

/// Individual condition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Condition {
    /// Check if a task has a specific status
    TaskStatus {
        task: String,
        status: TaskStatusCondition,
    },
    /// Check if a state variable equals a value
    StateEquals {
        key: String,
        value: serde_json::Value,
    },
    /// Check if a state variable exists
    StateExists { key: String },
    /// Always true (useful for testing)
    Always,
    /// Always false (skip task)
    Never,
}

/// Task status conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatusCondition {
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task is running
    Running,
    /// Task is pending
    Pending,
    /// Task was skipped
    Skipped,
}

/// Task specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskSpec {
    /// Task description
    pub description: String,
    /// Agent to execute this task (mutually exclusive with other execution types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Subflow to execute (mutually exclusive with other execution types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subflow: Option<String>,
    /// Reference to a predefined task (e.g., "google-drive-upload@1.2.0")
    /// Mutually exclusive with other execution types
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<String>,
    /// Embed a predefined task (copy definition instead of referencing)
    /// Mutually exclusive with 'uses' and other execution types
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub embed: Option<String>,
    /// Overrides for embedded tasks (only applies when 'embed' is used)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overrides: Option<serde_yaml::Value>,
    /// Script execution specification (mutually exclusive with other execution types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<ScriptSpec>,
    /// Command execution specification (mutually exclusive with other execution types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<CommandSpec>,
    /// HTTP request specification (mutually exclusive with other execution types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpSpec>,
    /// MCP tool invocation specification (mutually exclusive with other execution types)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_tool: Option<McpToolSpec>,
    /// Reference to a prebuilt workflow from a task group (e.g., "google:upload-files")
    /// Format: "namespace:workflow_name"
    /// Mutually exclusive with other execution types
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses_workflow: Option<String>,
    /// Inputs to pass to the subflow or task (runtime values)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, serde_json::Value>,
    /// Output variable definitions for the task
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, OutputSpec>,
    /// Priority (lower number = higher priority)
    #[serde(default, skip_serializing_if = "is_zero")]
    pub priority: u32,
    /// Nested subtasks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subtasks: Vec<HashMap<String, TaskSpec>>,
    /// Tasks this depends on
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    /// Tasks that can run in parallel with this
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parallel_with: Vec<String>,
    /// Output file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Action to execute on completion
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_complete: Option<ActionSpec>,
    /// Error handling specification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_error: Option<ErrorHandlingSpec>,
    /// Condition for conditional execution (task only runs if condition is met)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<ConditionSpec>,
    /// Definition of done - criteria that must be met for task completion
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub definition_of_done: Option<DefinitionOfDone>,
    /// Loop specification for iterative execution
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "loop")]
    pub loop_spec: Option<LoopSpec>,
    /// Loop control flow settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_control: Option<LoopControl>,
    /// Inject workflow execution context (completed tasks, results, etc.) into agent-based tasks
    #[serde(default, skip_serializing_if = "is_false")]
    pub inject_context: bool,
    /// Task-level limits override (overrides workflow-level limits)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limits: Option<LimitsConfig>,
    /// Context injection control for this specific task
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<ContextConfig>,
}

impl TaskSpec {
    /// Parse a workflow reference into (namespace, workflow_name)
    /// Expected format: "namespace:workflow_name"
    pub fn parse_workflow_reference(reference: &str) -> Option<(&str, &str)> {
        let parts: Vec<&str> = reference.split(':').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            Some((parts[0], parts[1]))
        } else {
            None
        }
    }

    /// Check if this task uses any execution type
    pub fn has_execution_type(&self) -> bool {
        self.agent.is_some()
            || self.subflow.is_some()
            || self.uses.is_some()
            || self.embed.is_some()
            || self.script.is_some()
            || self.command.is_some()
            || self.http.is_some()
            || self.mcp_tool.is_some()
            || self.uses_workflow.is_some()
    }

    /// Count how many execution types are defined (should be 0 or 1)
    pub fn execution_type_count(&self) -> usize {
        let mut count = 0;
        if self.agent.is_some() {
            count += 1;
        }
        if self.subflow.is_some() {
            count += 1;
        }
        if self.uses.is_some() {
            count += 1;
        }
        if self.embed.is_some() {
            count += 1;
        }
        if self.script.is_some() {
            count += 1;
        }
        if self.command.is_some() {
            count += 1;
        }
        if self.http.is_some() {
            count += 1;
        }
        if self.mcp_tool.is_some() {
            count += 1;
        }
        if self.uses_workflow.is_some() {
            count += 1;
        }
        count
    }

    /// Inherit attributes from a parent task that make sense to cascade down.
    /// Only inherits if the attribute is not already set in the subtask.
    ///
    /// Attributes that are inherited:
    /// - `agent`: Default to parent's agent if not specified
    /// - `inject_context`: Inherit context injection setting
    /// - `on_error`: Inherit error handling strategy
    /// - `priority`: Inherit priority (if subtask has default priority 0)
    ///
    /// Attributes that are NOT inherited (task-specific):
    /// - `description`, `output`, `depends_on`, `parallel_with`
    /// - `subtasks`, `definition_of_done`, `condition`
    /// - `on_complete`, execution types, `loop_spec`, `inputs`, `outputs`
    pub fn inherit_from_parent(&mut self, parent: &TaskSpec) {
        // Inherit agent if not set AND subtask doesn't have its own execution type
        // (Don't inherit agent if subtask has script, command, subflow, etc.)
        if self.agent.is_none() && parent.agent.is_some() && !self.has_execution_type() {
            self.agent = parent.agent.clone();
        }

        // Inherit inject_context if parent has it enabled and subtask hasn't explicitly set it
        // Note: We can't distinguish between "explicitly set to false" and "default false"
        // So we only inherit if parent is true and child is false
        if parent.inject_context && !self.inject_context {
            self.inject_context = true;
        }

        // Inherit error handling if not set
        if self.on_error.is_none() && parent.on_error.is_some() {
            self.on_error = parent.on_error.clone();
        }

        // Inherit priority if subtask has default priority (0)
        if self.priority == 0 && parent.priority != 0 {
            self.priority = parent.priority;
        }

        // Inherit loop control if not set
        if self.loop_control.is_none() && parent.loop_control.is_some() {
            self.loop_control = parent.loop_control.clone();
        }
    }
}

/// Definition of done specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionOfDone {
    /// Criteria that must be met for task completion
    pub criteria: Vec<DoneCriterion>,
    /// Maximum number of retries if criteria not met (default: 3)
    #[serde(
        default = "default_dod_retries",
        skip_serializing_if = "is_default_dod_retries"
    )]
    pub max_retries: u32,
    /// Whether to fail the task if all retries are exhausted (default: true)
    #[serde(default = "default_fail_on_unmet", skip_serializing_if = "is_true")]
    pub fail_on_unmet: bool,
    /// Automatically grant enhanced permissions on DoD retry (default: false)
    #[serde(default, skip_serializing_if = "is_false")]
    pub auto_elevate_permissions: bool,
}

fn default_dod_retries() -> u32 {
    3
}

fn is_default_dod_retries(value: &u32) -> bool {
    *value == 3
}

fn default_fail_on_unmet() -> bool {
    true
}

fn is_true(value: &bool) -> bool {
    *value
}

/// Individual criterion for definition of done
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DoneCriterion {
    /// Check if a file exists
    FileExists { path: String, description: String },
    /// Check if a file contains a pattern
    FileContains {
        path: String,
        pattern: String,
        description: String,
    },
    /// Check if a file does not contain a pattern
    FileNotContains {
        path: String,
        pattern: String,
        description: String,
    },
    /// Run a command and check exit code
    CommandSucceeds {
        command: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        description: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        working_dir: Option<String>,
    },
    /// Check if output file/stdout matches a pattern
    OutputMatches {
        source: OutputSource,
        pattern: String,
        description: String,
    },
    /// Check if a directory exists
    DirectoryExists { path: String, description: String },
    /// Check if tests pass (runs test command)
    TestsPassed {
        command: String,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        description: String,
    },
}

/// Source for output matching
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputSource {
    /// Check a file's content
    File { path: String },
    /// Check task's output (stored during execution)
    TaskOutput,
}

/// Result of checking a criterion
#[derive(Debug, Clone)]
pub struct CriterionResult {
    pub met: bool,
    pub description: String,
    pub details: String,
}

/// Action specification for task lifecycle events
///
/// Actions define operations to perform at specific points in a task's lifecycle,
/// such as sending notifications on completion or error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionSpec {
    /// Notification specification (supports both simple string and full NotificationSpec)
    ///
    /// # Examples
    ///
    /// Simple notification:
    /// ```yaml
    /// on_complete:
    ///   notify: "Task completed"
    /// ```
    ///
    /// Structured notification:
    /// ```yaml
    /// on_complete:
    ///   notify:
    ///     message: "Task completed successfully"
    ///     title: "Build Status"
    ///     priority: high
    ///     channels:
    ///       - type: console
    ///         colored: true
    /// ```
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notify: Option<NotificationSpec>,
}

/// Error handling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorHandlingSpec {
    /// Number of retries
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub retry: u32,
    /// Fallback agent name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_agent: Option<String>,
    /// Delay between retries in seconds (default: 1)
    #[serde(
        default = "default_retry_delay",
        skip_serializing_if = "is_default_retry_delay"
    )]
    pub retry_delay_secs: u64,
    /// Use exponential backoff for retries
    #[serde(default, skip_serializing_if = "is_false")]
    pub exponential_backoff: bool,
}

fn default_retry_delay() -> u64 {
    1
}

/// Workflow specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    /// Workflow description
    pub description: String,
    /// Workflow stages
    pub steps: Vec<StageSpec>,
    /// Workflow hooks
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hooks: Option<HooksSpec>,
}

/// Task group import specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowImport {
    /// Namespace identifier for the imported group
    pub namespace: String,
    /// Task group reference (e.g., "google-workspace@1.0.0")
    pub group_reference: String,
}

impl WorkflowImport {
    /// Create a new workflow import
    pub fn new(namespace: String, group_reference: String) -> Self {
        Self {
            namespace,
            group_reference,
        }
    }

    /// Parse from the imports map entry
    pub fn from_entry(namespace: &str, group_reference: &str) -> Self {
        Self {
            namespace: namespace.to_string(),
            group_reference: group_reference.to_string(),
        }
    }

    /// Validate namespace format (alphanumeric, dash, underscore, no leading digit)
    pub fn validate_namespace(namespace: &str) -> bool {
        !namespace.is_empty()
            && !namespace.starts_with(|c: char| c.is_numeric())
            && namespace
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// Parse group reference into (name, version)
    pub fn parse_group_reference(reference: &str) -> Option<(String, String)> {
        let parts: Vec<&str> = reference.split('@').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }
}

/// Stage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSpec {
    /// Stage name
    pub stage: String,
    /// Agents involved in this stage
    pub agents: Vec<String>,
    /// Tasks in this stage
    pub tasks: Vec<HashMap<String, TaskSpec>>,
    /// Stages this depends on
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    /// Execution mode (sequential or parallel)
    #[serde(default, skip_serializing_if = "is_sequential_mode")]
    pub mode: ExecutionMode,
}

/// Execution mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ExecutionMode {
    #[default]
    Sequential,
    Parallel,
}


/// Hooks specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HooksSpec {
    /// Commands to run before workflow
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_workflow: Vec<HookCommand>,
    /// Commands to run after workflow
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub post_workflow: Vec<HookCommand>,
    /// Commands to run on stage completion
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_stage_complete: Vec<HookCommand>,
    /// Commands to run on error
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_error: Vec<HookCommand>,
}

/// Hook command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HookCommand {
    /// Simple command string
    Command(String),
    /// Command with metadata
    CommandSpec {
        command: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

/// Tools configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    /// Allowed tools
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed: Vec<String>,
    /// Disallowed tools
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disallowed: Vec<String>,
    /// Tool-specific constraints
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub constraints: HashMap<String, ToolConstraints>,
}

/// Tool constraints
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConstraints {
    /// Timeout in milliseconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Allowed commands (for Bash)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_commands: Vec<String>,
    /// Maximum file size (for Write)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_file_size: Option<u64>,
    /// Allowed file extensions (for Write)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_extensions: Vec<String>,
    /// Rate limit (for WebSearch)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<u32>,
}

/// Communication configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommunicationConfig {
    /// Communication channels
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub channels: HashMap<String, ChannelSpec>,
    /// Message type definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub message_types: HashMap<String, MessageTypeSpec>,
}

/// Channel specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSpec {
    /// Channel description
    pub description: String,
    /// Participating agents
    pub participants: Vec<String>,
    /// Message format (e.g., "markdown", "json")
    pub message_format: String,
}

/// Message type specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageTypeSpec {
    /// JSON schema for message validation
    pub schema: serde_json::Value,
}

/// MCP server specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerSpec {
    /// Server type (e.g., "stdio", "http")
    #[serde(rename = "type")]
    pub server_type: String,
    /// Command to run (for stdio)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Command arguments
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Environment variables
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// URL (for http)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Headers (for http)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

/// Loop specification - defines iterative task execution patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LoopSpec {
    /// Iterate over a collection of items
    ForEach {
        /// Collection to iterate over
        collection: CollectionSource,
        /// Variable name for current item
        iterator: String,
        /// Execute iterations in parallel
        #[serde(default, skip_serializing_if = "is_false")]
        parallel: bool,
        /// Maximum parallel iterations (None = unlimited)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_parallel: Option<usize>,
    },
    /// Execute while condition is true
    While {
        /// Condition to evaluate
        condition: Box<ConditionSpec>,
        /// Maximum iterations (safety limit)
        max_iterations: usize,
        /// Variable name for iteration count
        #[serde(default, skip_serializing_if = "Option::is_none")]
        iteration_variable: Option<String>,
        /// Delay between iterations in seconds
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delay_between_secs: Option<u64>,
    },
    /// Execute until condition becomes true (do-while pattern)
    RepeatUntil {
        /// Exit condition
        condition: Box<ConditionSpec>,
        /// Minimum iterations (default: 1)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        min_iterations: Option<usize>,
        /// Maximum iterations (safety limit)
        max_iterations: usize,
        /// Variable name for iteration count
        #[serde(default, skip_serializing_if = "Option::is_none")]
        iteration_variable: Option<String>,
        /// Delay between iterations in seconds
        #[serde(default, skip_serializing_if = "Option::is_none")]
        delay_between_secs: Option<u64>,
    },
    /// Execute N times
    Repeat {
        /// Number of iterations
        count: usize,
        /// Variable name for index (0-based)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        iterator: Option<String>,
        /// Execute iterations in parallel
        #[serde(default, skip_serializing_if = "is_false")]
        parallel: bool,
        /// Maximum parallel iterations (None = unlimited)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        max_parallel: Option<usize>,
    },
}

impl LoopSpec {
    /// Get the max_parallel value for parallel loop types
    pub fn max_parallel(&self) -> Option<usize> {
        match self {
            LoopSpec::ForEach { max_parallel, .. } => *max_parallel,
            LoopSpec::Repeat { max_parallel, .. } => *max_parallel,
            _ => None,
        }
    }
}

/// Collection source for iteration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum CollectionSource {
    /// Read from workflow state
    State {
        /// State key containing the array
        key: String,
    },
    /// Read from a file
    File {
        /// File path
        path: String,
        /// File format
        format: FileFormat,
    },
    /// Generate a numeric range
    Range {
        /// Start value (inclusive)
        start: i64,
        /// End value (exclusive)
        end: i64,
        /// Step size (default: 1)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<i64>,
    },
    /// Inline array of items
    Inline {
        /// Array of items
        items: Vec<serde_json::Value>,
    },
    /// Fetch from HTTP/HTTPS endpoint
    Http {
        /// URL to fetch from
        url: String,
        /// HTTP method (GET, POST, etc.)
        #[serde(default = "default_http_method")]
        method: String,
        /// Optional request headers
        #[serde(default, skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        /// Optional request body (for POST, PUT, etc.)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        body: Option<String>,
        /// Response format (how to parse the response)
        #[serde(default = "default_response_format")]
        format: FileFormat,
        /// JSON path to extract array from response (e.g., "data.items")
        #[serde(default, skip_serializing_if = "Option::is_none")]
        json_path: Option<String>,
    },
}

fn default_http_method() -> String {
    "GET".to_string()
}

fn default_response_format() -> FileFormat {
    FileFormat::Json
}

/// File format for collection sources
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    /// JSON array
    Json,
    /// JSON Lines (one JSON object per line)
    JsonLines,
    /// CSV file (each row is an item)
    Csv,
    /// Plain text lines (each line is a string)
    Lines,
}

/// Loop control flow specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopControl {
    /// Condition to break out of loop early
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub break_condition: Option<ConditionSpec>,
    /// Condition to skip current iteration and continue to next
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continue_condition: Option<ConditionSpec>,
    /// Collect results from iterations
    #[serde(default, skip_serializing_if = "is_false")]
    pub collect_results: bool,
    /// State key to store collected results
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_key: Option<String>,
    /// Overall loop timeout in seconds (applies to entire loop, not per iteration)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// Checkpoint interval - save state every N iterations (0 = no checkpointing)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_interval: Option<usize>,
}

/// Subflow specification - can be inline or reference an external workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubflowSpec {
    /// Description of the subflow
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Source of the subflow (None = inline definition in this file)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SubflowSource>,
    /// Inline agents (only used for inline subflows, ignored for external references)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub agents: HashMap<String, AgentSpec>,
    /// Inline tasks (only used for inline subflows, ignored for external references)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tasks: HashMap<String, TaskSpec>,
    /// Input parameter definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, InputSpec>,
    /// Output definitions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub outputs: HashMap<String, OutputSpec>,
}

/// Subflow source - where to fetch the subflow from
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SubflowSource {
    /// Local file path
    File {
        /// Path to the workflow file (relative or absolute)
        path: String,
    },
    /// Git repository
    Git {
        /// Git repository URL
        url: String,
        /// Path within the repository to the workflow file
        path: String,
        /// Git reference (branch, tag, or commit hash)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reference: Option<String>,
    },
    /// HTTP/HTTPS URL
    Http {
        /// URL to fetch the workflow from
        url: String,
        /// Optional checksum for integrity verification (format: "sha256:hash")
        #[serde(default, skip_serializing_if = "Option::is_none")]
        checksum: Option<String>,
    },
}

/// Input parameter specification for subflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSpec {
    /// Parameter type (string, number, boolean, object, array)
    #[serde(rename = "type")]
    pub param_type: String,
    /// Whether this input is required
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    /// Default value if not provided
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// Description of the input parameter
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Output specification for subflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSpec {
    /// Source of the output data
    pub source: OutputDataSource,
    /// Description of the output
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Output data source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputDataSource {
    /// Read output from a file
    File {
        /// File path to read
        path: String,
    },
    /// Read output from workflow state
    State {
        /// State key to read
        key: String,
    },
    /// Read output from a task's result
    TaskOutput {
        /// Task ID to get output from
        task: String,
    },
}

/// Secret specification for secure credential management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretSpec {
    /// Source of the secret value
    pub source: SecretSource,
    /// Optional description of the secret
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Secret source types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SecretSource {
    /// Read from environment variable
    Env {
        /// Environment variable name
        var: String,
    },
    /// Read from file
    File {
        /// File path containing the secret
        path: String,
    },
    /// Direct value (not recommended for production)
    Value {
        /// Secret value
        value: String,
    },
}

/// Script execution specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSpec {
    /// Programming language/runtime
    pub language: ScriptLanguage,
    /// Script content (inline)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Script file path (alternative to inline content)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Working directory for script execution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    /// Environment variables to pass to the script
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// Timeout in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Supported script languages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScriptLanguage {
    /// Python script
    Python,
    /// JavaScript/Node.js script
    JavaScript,
    /// Bash shell script
    Bash,
    /// Ruby script
    Ruby,
    /// Perl script
    Perl,
}

/// Command execution specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSpec {
    /// Executable name or path
    pub executable: String,
    /// Command arguments (supports variable interpolation)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    /// Working directory for command execution
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    /// Environment variables to pass to the command
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,
    /// Timeout in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// Capture stdout
    #[serde(default = "default_true")]
    pub capture_stdout: bool,
    /// Capture stderr
    #[serde(default = "default_true")]
    pub capture_stderr: bool,
}

fn default_true() -> bool {
    true
}

/// HTTP request specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpSpec {
    /// HTTP method
    pub method: HttpMethod,
    /// URL (supports variable interpolation)
    pub url: String,
    /// Request headers
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// Request body (supports variable interpolation)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Authentication specification
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<HttpAuth>,
    /// Timeout in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    /// Follow redirects
    #[serde(default = "default_true")]
    pub follow_redirects: bool,
    /// Validate TLS certificates
    #[serde(default = "default_true")]
    pub verify_tls: bool,
}

/// HTTP methods
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

/// HTTP authentication types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HttpAuth {
    /// Bearer token authentication
    Bearer {
        /// Token (can reference secrets via ${secret.name})
        token: String,
    },
    /// Basic authentication
    Basic {
        /// Username
        username: String,
        /// Password (can reference secrets via ${secret.name})
        password: String,
    },
    /// API key in header
    ApiKey {
        /// Header name
        header: String,
        /// API key value (can reference secrets via ${secret.name})
        key: String,
    },
    /// Custom header-based authentication
    Custom {
        /// Custom headers
        headers: HashMap<String, String>,
    },
}

/// MCP tool invocation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolSpec {
    /// MCP server name
    pub server: String,
    /// Tool name to invoke
    pub tool: String,
    /// Tool parameters (supports variable interpolation in values)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub parameters: HashMap<String, serde_json::Value>,
    /// Timeout in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

// ============================================================================
// Notification System
// ============================================================================

/// Notification specification - supports both simple string and structured formats
///
/// This type acts semantically like a pub struct NotificationSpec while providing
/// enum-based polymorphism for flexible notification handling. The untagged serde
/// representation allows it to deserialize from both simple strings and structured objects.
///
/// # Examples
///
/// Simple string notification:
/// ```yaml
/// notify: "Task completed"
/// ```
///
/// Structured notification:
/// ```yaml
/// notify:
///   message: "Deployment finished"
///   title: "Production Update"
///   priority: high
///   channels:
///     - type: console
///       colored: true
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NotificationSpec {
    /// Simple string notification message (uses default channel from workflow settings)
    Simple(String),
    /// Structured notification with full configuration
    Structured {
        /// Notification message (supports variable interpolation)
        message: String,
        /// Notification channel(s) to use
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        channels: Vec<NotificationChannel>,
        /// Notification title (optional, some channels support titles)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Notification priority level
        #[serde(default, skip_serializing_if = "Option::is_none")]
        priority: Option<NotificationPriority>,
        /// Additional metadata for the notification
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        metadata: HashMap<String, String>,
    },
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationChannel {
    /// Console/stdout notification
    Console {
        /// Use colored output
        #[serde(default = "default_true")]
        colored: bool,
        /// Include timestamp
        #[serde(default = "default_true")]
        timestamp: bool,
    },
    /// Email notification via SMTP
    Email {
        /// Recipient email addresses
        to: Vec<String>,
        /// CC recipients
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        cc: Vec<String>,
        /// BCC recipients
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        bcc: Vec<String>,
        /// Email subject
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subject: Option<String>,
        /// SMTP server configuration (references workflow secrets or config)
        smtp: SmtpConfig,
    },
    /// Slack notification via webhook or bot
    Slack {
        /// Slack webhook URL or bot token (reference via ${secret.name})
        credential: String,
        /// Channel, user, or conversation ID
        channel: String,
        /// Notification type (webhook or bot)
        #[serde(default = "default_slack_method")]
        method: SlackMethod,
        /// Include attachments
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attachments: Vec<SlackAttachment>,
    },
    /// Discord notification via webhook
    Discord {
        /// Discord webhook URL (reference via ${secret.name})
        webhook_url: String,
        /// Username override
        #[serde(default, skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        /// Avatar URL override
        #[serde(default, skip_serializing_if = "Option::is_none")]
        avatar_url: Option<String>,
        /// Enable text-to-speech
        #[serde(default, skip_serializing_if = "is_false")]
        tts: bool,
        /// Discord embed
        #[serde(default, skip_serializing_if = "Option::is_none")]
        embed: Option<DiscordEmbed>,
    },
    /// Microsoft Teams notification via webhook
    Teams {
        /// Teams webhook URL (reference via ${secret.name})
        webhook_url: String,
        /// Card theme color (hex format)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        theme_color: Option<String>,
        /// Include facts section
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        facts: Vec<TeamsFact>,
    },
    /// Telegram notification via bot API
    Telegram {
        /// Telegram bot token (reference via ${secret.name})
        bot_token: String,
        /// Chat ID (can be user ID, group ID, or channel username)
        chat_id: String,
        /// Message parse mode (Markdown, HTML, or None)
        #[serde(default = "default_parse_mode")]
        parse_mode: TelegramParseMode,
        /// Disable link previews
        #[serde(default, skip_serializing_if = "is_false")]
        disable_preview: bool,
        /// Disable notification sound
        #[serde(default, skip_serializing_if = "is_false")]
        silent: bool,
    },
    /// PagerDuty incident notification
    PagerDuty {
        /// PagerDuty integration key (reference via ${secret.name})
        integration_key: String,
        /// Event action (trigger, acknowledge, resolve)
        action: PagerDutyAction,
        /// Severity level
        #[serde(default = "default_pagerduty_severity")]
        severity: PagerDutySeverity,
        /// Deduplication key (optional)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        dedup_key: Option<String>,
        /// Custom details
        #[serde(default, skip_serializing_if = "Option::is_none")]
        custom_details: Option<serde_json::Value>,
    },
    /// Generic webhook notification
    Webhook {
        /// Webhook URL (supports variable interpolation and secret references)
        url: String,
        /// HTTP method
        #[serde(default = "default_webhook_method")]
        method: HttpMethod,
        /// Request headers
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        headers: HashMap<String, String>,
        /// Authentication
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auth: Option<HttpAuth>,
        /// Request body template (supports variable interpolation)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        body_template: Option<String>,
        /// Timeout in seconds
        #[serde(default, skip_serializing_if = "Option::is_none")]
        timeout_secs: Option<u64>,
        /// Retry configuration
        #[serde(default, skip_serializing_if = "Option::is_none")]
        retry: Option<RetryConfig>,
    },
    /// File-based notification (write to log file)
    File {
        /// File path (supports variable interpolation)
        path: String,
        /// Append or overwrite
        #[serde(default = "default_true")]
        append: bool,
        /// Include timestamp
        #[serde(default = "default_true")]
        timestamp: bool,
        /// Message format
        #[serde(default = "default_file_format")]
        format: FileNotificationFormat,
    },
    /// ntfy.sh notification
    Ntfy {
        /// ntfy.sh server URL (default: https://ntfy.sh)
        #[serde(default = "default_ntfy_server")]
        server: String,
        /// Topic name
        topic: String,
        /// Message title
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Priority (1=min, 3=default, 5=max)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        priority: Option<u8>,
        /// Tags/emojis
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        tags: Vec<String>,
        /// Click action URL
        #[serde(default, skip_serializing_if = "Option::is_none")]
        click_url: Option<String>,
        /// Attachment URL
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attach_url: Option<String>,
        /// Markdown support
        #[serde(default, skip_serializing_if = "is_false")]
        markdown: bool,
        /// Authentication token (reference via ${secret.name})
        #[serde(default, skip_serializing_if = "Option::is_none")]
        auth_token: Option<String>,
    },
}

/// Default notification settings for workflow
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationDefaults {
    /// Default notification channels to use when not specified
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_channels: Vec<NotificationChannel>,
    /// Whether to notify on task completion by default
    #[serde(default, skip_serializing_if = "is_false")]
    pub notify_on_completion: bool,
    /// Whether to notify on task failure by default
    #[serde(default = "default_true")]
    pub notify_on_failure: bool,
    /// Whether to notify on workflow start
    #[serde(default, skip_serializing_if = "is_false")]
    pub notify_on_start: bool,
    /// Whether to notify on workflow completion
    #[serde(default = "default_true")]
    pub notify_on_workflow_completion: bool,
}

/// Notification priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPriority {
    /// Low priority
    Low,
    /// Normal priority
    Normal,
    /// High priority
    High,
    /// Critical/urgent priority
    Critical,
}

/// SMTP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    /// SMTP server host
    pub host: String,
    /// SMTP server port
    pub port: u16,
    /// Username (reference via ${secret.name})
    pub username: String,
    /// Password (reference via ${secret.name})
    pub password: String,
    /// From address
    pub from: String,
    /// Use TLS
    #[serde(default = "default_true")]
    pub use_tls: bool,
}

/// Slack notification method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SlackMethod {
    /// Use webhook
    Webhook,
    /// Use bot API
    Bot,
}

fn default_slack_method() -> SlackMethod {
    SlackMethod::Webhook
}

/// Slack message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackAttachment {
    /// Attachment text
    pub text: String,
    /// Attachment color (hex or good/warning/danger)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Attachment fields
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<SlackField>,
}

/// Slack message field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackField {
    /// Field title
    pub title: String,
    /// Field value
    pub value: String,
    /// Display as short field
    #[serde(default, skip_serializing_if = "is_false")]
    pub short: bool,
}

/// Discord embed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordEmbed {
    /// Embed title
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Embed description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Embed color (decimal format)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<u32>,
    /// Embed fields
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fields: Vec<DiscordField>,
    /// Footer text
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub footer: Option<String>,
    /// Timestamp (ISO 8601)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Discord embed field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordField {
    /// Field name
    pub name: String,
    /// Field value
    pub value: String,
    /// Display inline
    #[serde(default, skip_serializing_if = "is_false")]
    pub inline: bool,
}

/// Microsoft Teams fact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsFact {
    /// Fact name
    pub name: String,
    /// Fact value
    pub value: String,
}

/// Telegram parse mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum TelegramParseMode {
    /// Markdown formatting
    Markdown,
    /// HTML formatting
    Html,
    /// No formatting
    None,
}

fn default_parse_mode() -> TelegramParseMode {
    TelegramParseMode::Markdown
}

/// PagerDuty event action
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PagerDutyAction {
    /// Trigger a new incident
    Trigger,
    /// Acknowledge an incident
    Acknowledge,
    /// Resolve an incident
    Resolve,
}

/// PagerDuty severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PagerDutySeverity {
    /// Critical severity
    Critical,
    /// Error severity
    Error,
    /// Warning severity
    Warning,
    /// Info severity
    Info,
}

fn default_pagerduty_severity() -> PagerDutySeverity {
    PagerDutySeverity::Error
}

fn default_webhook_method() -> HttpMethod {
    HttpMethod::Post
}

/// Retry configuration for webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    #[serde(default = "default_retry_attempts")]
    pub max_attempts: u32,
    /// Delay between retries in seconds
    #[serde(default = "default_retry_delay")]
    pub delay_secs: u64,
    /// Use exponential backoff
    #[serde(default, skip_serializing_if = "is_false")]
    pub exponential_backoff: bool,
}

fn default_retry_attempts() -> u32 {
    3
}

/// File notification format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FileNotificationFormat {
    /// Plain text
    Text,
    /// JSON format
    Json,
    /// JSON Lines format
    JsonLines,
}

fn default_file_format() -> FileNotificationFormat {
    FileNotificationFormat::Text
}

fn default_ntfy_server() -> String {
    "https://ntfy.sh".to_string()
}

// ============================================================================
// Stdio and Context Management Configuration
// ============================================================================

/// Configuration for stdout/stderr capture limits and context management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum stdout bytes per task (default: 1MB)
    #[serde(default = "default_max_stdout_bytes")]
    pub max_stdout_bytes: usize,

    /// Maximum stderr bytes per task (default: 256KB)
    #[serde(default = "default_max_stderr_bytes")]
    pub max_stderr_bytes: usize,

    /// Maximum combined stdout+stderr (default: 1.5MB)
    #[serde(default = "default_max_combined_bytes")]
    pub max_combined_bytes: usize,

    /// Truncation strategy: head, tail, both, summary
    #[serde(default = "default_truncation_strategy")]
    pub truncation_strategy: TruncationStrategy,

    /// Maximum context size for injection (default: 100KB)
    #[serde(default = "default_max_context_bytes")]
    pub max_context_bytes: usize,

    /// Maximum tasks in context (default: 10)
    #[serde(default = "default_max_context_tasks")]
    pub max_context_tasks: usize,

    /// Threshold for storing outputs externally (default: 5MB, None = never store externally)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_storage_threshold: Option<usize>,

    /// Directory for external storage (default: .workflow_state/task_outputs)
    #[serde(default = "default_external_storage_dir")]
    pub external_storage_dir: String,

    /// Compress externally stored outputs
    #[serde(default = "default_true")]
    pub compress_external: bool,

    /// Cleanup strategy for context pruning
    #[serde(default = "default_cleanup_strategy")]
    pub cleanup_strategy: CleanupStrategy,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_stdout_bytes: 1_048_576,   // 1MB
            max_stderr_bytes: 262_144,     // 256KB
            max_combined_bytes: 1_572_864, // 1.5MB
            truncation_strategy: TruncationStrategy::Tail,
            max_context_bytes: 102_400, // 100KB
            max_context_tasks: 10,
            external_storage_threshold: Some(5_242_880), // 5MB
            external_storage_dir: ".workflow_state/task_outputs".to_string(),
            compress_external: true,
            cleanup_strategy: CleanupStrategy::MostRecent { keep_count: 20 },
        }
    }
}

fn default_max_stdout_bytes() -> usize {
    1_048_576 // 1MB
}

fn default_max_stderr_bytes() -> usize {
    262_144 // 256KB
}

fn default_max_combined_bytes() -> usize {
    1_572_864 // 1.5MB
}

fn default_max_context_bytes() -> usize {
    102_400 // 100KB
}

fn default_max_context_tasks() -> usize {
    10
}

fn default_external_storage_dir() -> String {
    ".workflow_state/task_outputs".to_string()
}

fn default_truncation_strategy() -> TruncationStrategy {
    TruncationStrategy::Tail
}

fn default_cleanup_strategy() -> CleanupStrategy {
    CleanupStrategy::MostRecent { keep_count: 20 }
}

/// Truncation strategy for output capture
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TruncationStrategy {
    /// Keep first N bytes
    Head,
    /// Keep last N bytes
    Tail,
    /// Keep first N/2 and last N/2 bytes
    Both,
    /// Generate AI summary (requires agent call)
    Summary,
}

/// Cleanup strategy for context pruning
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CleanupStrategy {
    /// Keep most recent N tasks
    MostRecent { keep_count: usize },
    /// Keep highest relevance scores
    HighestRelevance { keep_count: usize },
    /// LRU (Least Recently Used)
    Lru { keep_count: usize },
    /// Keep only direct dependencies
    DirectDependencies,
}

/// Context injection configuration for tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Context injection mode
    #[serde(default = "default_context_mode")]
    pub mode: ContextMode,

    /// Include only these specific tasks (for manual mode)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_tasks: Vec<String>,

    /// Exclude these specific tasks
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_tasks: Vec<String>,

    /// Minimum relevance threshold (0.0-1.0)
    #[serde(default = "default_min_relevance")]
    pub min_relevance: f64,

    /// Maximum bytes of context to inject (overrides workflow limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bytes: Option<usize>,

    /// Maximum number of tasks in context (overrides workflow limit)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tasks: Option<usize>,
}

fn default_context_mode() -> ContextMode {
    ContextMode::Automatic
}

fn default_min_relevance() -> f64 {
    0.5
}

/// Context injection mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContextMode {
    /// Automatic (dependency-based)
    Automatic,
    /// Manual (use include_tasks/exclude_tasks)
    Manual,
    /// No context injection
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_mode_default() {
        let mode: ExecutionMode = Default::default();
        assert_eq!(mode, ExecutionMode::Sequential);
    }

    #[test]
    fn test_permissions_spec_default() {
        let perms: PermissionsSpec = Default::default();
        assert_eq!(perms.mode, "default");
        assert!(perms.allowed_directories.is_empty());
    }

    #[test]
    fn test_task_spec_default() {
        let task: TaskSpec = Default::default();
        assert!(task.description.is_empty());
        assert_eq!(task.priority, 0);
        assert!(task.subtasks.is_empty());
    }

    #[test]
    fn test_inherit_from_parent_agent() {
        let mut parent = TaskSpec::default();
        parent.agent = Some("parent_agent".to_string());
        parent.description = "Parent task".to_string();

        let mut child = TaskSpec::default();
        child.description = "Child task".to_string();

        child.inherit_from_parent(&parent);

        // Child should inherit agent from parent
        assert_eq!(child.agent, Some("parent_agent".to_string()));
        // Child's own description should remain
        assert_eq!(child.description, "Child task");
    }

    #[test]
    fn test_inherit_from_parent_agent_not_overridden() {
        let mut parent = TaskSpec::default();
        parent.agent = Some("parent_agent".to_string());

        let mut child = TaskSpec::default();
        child.agent = Some("child_agent".to_string());

        child.inherit_from_parent(&parent);

        // Child's explicit agent should not be overridden
        assert_eq!(child.agent, Some("child_agent".to_string()));
    }

    #[test]
    fn test_inherit_from_parent_inject_context() {
        let mut parent = TaskSpec::default();
        parent.inject_context = true;

        let mut child = TaskSpec::default();
        child.inject_context = false;

        child.inherit_from_parent(&parent);

        // Child should inherit inject_context=true from parent
        assert!(child.inject_context);
    }

    #[test]
    fn test_inherit_from_parent_priority() {
        let mut parent = TaskSpec::default();
        parent.priority = 5;

        let mut child = TaskSpec::default();
        child.priority = 0; // Default priority

        child.inherit_from_parent(&parent);

        // Child should inherit priority from parent
        assert_eq!(child.priority, 5);
    }

    #[test]
    fn test_inherit_from_parent_priority_not_overridden() {
        let mut parent = TaskSpec::default();
        parent.priority = 5;

        let mut child = TaskSpec::default();
        child.priority = 10; // Explicit non-zero priority

        child.inherit_from_parent(&parent);

        // Child's explicit priority should not be overridden
        assert_eq!(child.priority, 10);
    }

    #[test]
    fn test_inherit_from_parent_error_handling() {
        let mut parent = TaskSpec::default();
        parent.on_error = Some(ErrorHandlingSpec {
            retry: 3,
            retry_delay_secs: 5,
            exponential_backoff: true,
            fallback_agent: Some("fallback".to_string()),
        });

        let mut child = TaskSpec::default();

        child.inherit_from_parent(&parent);

        // Child should inherit error handling from parent
        assert!(child.on_error.is_some());
        let error_handling = child.on_error.as_ref().unwrap();
        assert_eq!(error_handling.retry, 3);
        assert_eq!(error_handling.retry_delay_secs, 5);
        assert!(error_handling.exponential_backoff);
    }

    #[test]
    fn test_inherit_from_parent_multiple_attributes() {
        let mut parent = TaskSpec::default();
        parent.agent = Some("shared_agent".to_string());
        parent.inject_context = true;
        parent.priority = 5;
        parent.on_error = Some(ErrorHandlingSpec {
            retry: 2,
            retry_delay_secs: 3,
            exponential_backoff: false,
            fallback_agent: None,
        });

        let mut child = TaskSpec::default();
        child.description = "Child task".to_string();

        child.inherit_from_parent(&parent);

        // Verify all inheritable attributes are inherited
        assert_eq!(child.agent, Some("shared_agent".to_string()));
        assert!(child.inject_context);
        assert_eq!(child.priority, 5);
        assert!(child.on_error.is_some());
        assert_eq!(child.on_error.as_ref().unwrap().retry, 2);
        // Verify child's own attributes remain
        assert_eq!(child.description, "Child task");
    }

    #[test]
    fn test_inherit_from_parent_loop_control() {
        let mut parent = TaskSpec::default();
        parent.loop_control = Some(LoopControl {
            break_condition: None,
            continue_condition: None,
            collect_results: true,
            result_key: Some("results".to_string()),
            timeout_secs: Some(300),
            checkpoint_interval: None,
        });

        let mut child = TaskSpec::default();

        child.inherit_from_parent(&parent);

        // Child should inherit loop control from parent
        assert!(child.loop_control.is_some());
        let control = child.loop_control.as_ref().unwrap();
        assert!(control.collect_results);
        assert_eq!(control.result_key, Some("results".to_string()));
        assert_eq!(control.timeout_secs, Some(300));
    }

    #[test]
    fn test_notification_spec_simple() {
        let yaml = r#""Task completed successfully""#;
        let spec: NotificationSpec = serde_yaml::from_str(yaml).unwrap();
        match spec {
            NotificationSpec::Simple(msg) => assert_eq!(msg, "Task completed successfully"),
            _ => panic!("Expected Simple variant"),
        }
    }

    #[test]
    fn test_notification_spec_structured() {
        let yaml = r#"
message: "Deployment finished"
title: "Production Update"
priority: high
channels:
  - type: console
    colored: true
    timestamp: true
"#;
        let spec: NotificationSpec = serde_yaml::from_str(yaml).unwrap();
        match spec {
            NotificationSpec::Structured {
                message,
                title,
                priority,
                channels,
                ..
            } => {
                assert_eq!(message, "Deployment finished");
                assert_eq!(title, Some("Production Update".to_string()));
                assert_eq!(priority, Some(NotificationPriority::High));
                assert_eq!(channels.len(), 1);
            }
            _ => panic!("Expected Structured variant"),
        }
    }

    #[test]
    fn test_notification_channel_console() {
        let yaml = r#"
type: console
colored: true
timestamp: false
"#;
        let channel: NotificationChannel = serde_yaml::from_str(yaml).unwrap();
        match channel {
            NotificationChannel::Console { colored, timestamp } => {
                assert!(colored);
                assert!(!timestamp);
            }
            _ => panic!("Expected Console variant"),
        }
    }

    #[test]
    fn test_notification_channel_slack() {
        let yaml = r##"
type: slack
credential: "${secret.slack_webhook}"
channel: "#general"
method: webhook
"##;
        let channel: NotificationChannel = serde_yaml::from_str(yaml).unwrap();
        match channel {
            NotificationChannel::Slack {
                credential,
                channel,
                method,
                ..
            } => {
                assert_eq!(credential, "${secret.slack_webhook}");
                assert_eq!(channel, "#general");
                assert_eq!(method, SlackMethod::Webhook);
            }
            _ => panic!("Expected Slack variant"),
        }
    }

    #[test]
    fn test_notification_channel_ntfy() {
        let yaml = r#"
type: ntfy
server: "https://ntfy.sh"
topic: "workflow-updates"
priority: 4
tags: ["tada", "rocket"]
markdown: true
"#;
        let channel: NotificationChannel = serde_yaml::from_str(yaml).unwrap();
        match channel {
            NotificationChannel::Ntfy {
                server,
                topic,
                priority,
                tags,
                markdown,
                ..
            } => {
                assert_eq!(server, "https://ntfy.sh");
                assert_eq!(topic, "workflow-updates");
                assert_eq!(priority, Some(4));
                assert_eq!(tags.len(), 2);
                assert!(markdown);
            }
            _ => panic!("Expected Ntfy variant"),
        }
    }

    #[test]
    fn test_notification_defaults() {
        let defaults = NotificationDefaults {
            notify_on_completion: true,
            notify_on_failure: true,
            notify_on_start: false,
            notify_on_workflow_completion: true,
            default_channels: vec![],
        };
        assert!(defaults.notify_on_completion);
        assert!(defaults.notify_on_failure);
        assert!(!defaults.notify_on_start);
        assert!(defaults.notify_on_workflow_completion);
    }

    #[test]
    fn test_action_spec_with_notification() {
        let yaml = r#"
notify: "Build completed"
"#;
        let action: ActionSpec = serde_yaml::from_str(yaml).unwrap();
        assert!(action.notify.is_some());
        match action.notify.unwrap() {
            NotificationSpec::Simple(msg) => assert_eq!(msg, "Build completed"),
            _ => panic!("Expected Simple notification"),
        }
    }

    #[test]
    fn test_workflow_with_notification_defaults() {
        let yaml = r#"
name: "Test Workflow"
version: "1.0.0"
notifications:
  notify_on_completion: true
  notify_on_failure: true
  default_channels:
    - type: console
      colored: true
      timestamp: true
"#;
        let workflow: DSLWorkflow = serde_yaml::from_str(yaml).unwrap();
        assert!(workflow.notifications.is_some());
        let notif = workflow.notifications.unwrap();
        assert!(notif.notify_on_completion);
        assert!(notif.notify_on_failure);
        assert_eq!(notif.default_channels.len(), 1);
    }
}
