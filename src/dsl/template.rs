//! DSL Template Generator
//!
//! This module provides functionality to generate DSL templates with comprehensive
//! documentation and examples. It ensures templates are always up-to-date with
//! the current DSL schema.

use std::fmt::Write as FmtWrite;

/// Generate JSON schema for notification system
///
/// Returns a JSON schema that precisely defines the NotificationSpec structure,
/// including all valid channel types and their required fields.
fn generate_notification_json_schema() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "NotificationSpec",
        "description": "Notification specification - untagged enum with two variants",
        "oneOf": [
            {
                "type": "string",
                "description": "Simple string notification (uses default channels)"
            },
            {
                "type": "object",
                "description": "Structured notification with full configuration",
                "required": ["message"],
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Notification message (supports variable interpolation)"
                    },
                    "title": {
                        "type": "string",
                        "description": "Optional notification title"
                    },
                    "priority": {
                        "type": "string",
                        "enum": ["low", "normal", "high", "critical"],
                        "description": "Priority level (NOT 'urgent' - use 'critical' instead)"
                    },
                    "channels": {
                        "type": "array",
                        "description": "Notification channels (inline definitions only)",
                        "items": {
                            "$ref": "#/definitions/NotificationChannel"
                        }
                    },
                    "metadata": {
                        "type": "object",
                        "description": "Additional metadata key-value pairs",
                        "additionalProperties": {
                            "type": "string"
                        }
                    }
                }
            }
        ],
        "definitions": {
            "NotificationChannel": {
                "oneOf": [
                    {
                        "type": "object",
                        "required": ["type"],
                        "properties": {
                            "type": { "const": "console" },
                            "colored": { "type": "boolean", "default": true },
                            "timestamp": { "type": "boolean", "default": true }
                        }
                    },
                    {
                        "type": "object",
                        "required": ["type", "to", "smtp"],
                        "properties": {
                            "type": { "const": "email" },
                            "to": { "type": "array", "items": { "type": "string" } },
                            "subject": { "type": "string" },
                            "smtp": {
                                "type": "object",
                                "required": ["host", "port", "username", "password", "from"],
                                "properties": {
                                    "host": { "type": "string" },
                                    "port": { "type": "integer" },
                                    "username": { "type": "string" },
                                    "password": { "type": "string" },
                                    "from": { "type": "string" },
                                    "use_tls": { "type": "boolean", "default": true }
                                }
                            }
                        }
                    },
                    {
                        "type": "object",
                        "required": ["type", "credential", "channel"],
                        "properties": {
                            "type": { "const": "slack" },
                            "credential": { "type": "string" },
                            "channel": { "type": "string" },
                            "method": { "enum": ["webhook", "bot"], "default": "webhook" }
                        }
                    },
                    {
                        "type": "object",
                        "required": ["type", "webhook_url"],
                        "properties": {
                            "type": { "const": "discord" },
                            "webhook_url": { "type": "string" },
                            "username": { "type": "string" },
                            "avatar_url": { "type": "string" },
                            "tts": { "type": "boolean", "default": false }
                        }
                    }
                ]
            }
        }
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

/// Current DSL grammar version
/// This version tracks the DSL schema itself, not individual workflows
/// Increment when making breaking changes to the DSL syntax
pub const DSL_GRAMMAR_VERSION: &str = "1.0.0";

/// Generate a fully documented DSL template with all available options
///
/// This template is auto-generated from the schema and includes:
/// - All fields with descriptions
/// - Type information
/// - Default values
/// - Examples
///
/// # Returns
///
/// A YAML string containing the complete DSL template with comments
pub fn generate_template() -> String {
    let mut template = String::new();

    // Header
    writeln!(&mut template, "# Agentic AI DSL Template").unwrap();
    writeln!(
        &mut template,
        "# DSL Grammar Version: {}",
        DSL_GRAMMAR_VERSION
    )
    .unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(
        &mut template,
        "# This template shows all available DSL options with documentation."
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Required fields are marked with (REQUIRED)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Optional fields are marked with (optional)"
    )
    .unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template).unwrap();

    // Root workflow fields
    writeln!(
        &mut template,
        "# (REQUIRED) Workflow name - identifies this workflow"
    )
    .unwrap();
    writeln!(&mut template, "name: \"My Workflow\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "# (REQUIRED) Workflow version - semantic versioning (e.g., 1.0.0)"
    )
    .unwrap();
    writeln!(&mut template, "version: \"1.0.0\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "# (optional) DSL grammar version - tracks DSL syntax version"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Defaults to current version: {}",
        DSL_GRAMMAR_VERSION
    )
    .unwrap();
    writeln!(&mut template, "dsl_version: \"{}\"", DSL_GRAMMAR_VERSION).unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "# (optional) Default working directory for all agents"
    )
    .unwrap();
    writeln!(&mut template, "# Can be overridden per agent").unwrap();
    writeln!(&mut template, "# cwd: \"/path/to/project\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "# (optional) Create working directory if it doesn't exist"
    )
    .unwrap();
    writeln!(&mut template, "# Can be overridden per agent").unwrap();
    writeln!(&mut template, "# create_cwd: true").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "# (optional) Workflow-level limits for output truncation and context management"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Prevents memory exhaustion in long-running workflows"
    )
    .unwrap();
    writeln!(&mut template, "# limits:").unwrap();
    writeln!(&mut template, "#   # Output Truncation").unwrap();
    writeln!(
        &mut template,
        "#   max_stdout_bytes: 1048576       # 1MB per task (default)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#   max_stderr_bytes: 262144        # 256KB per task (default)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#   truncation_strategy: tail        # head|tail|both|summary (default: tail)"
    )
    .unwrap();
    writeln!(&mut template, "#   # Context Injection").unwrap();
    writeln!(
        &mut template,
        "#   max_context_bytes: 102400        # 100KB total context (default)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#   max_context_tasks: 10            # Max tasks to include (default)"
    )
    .unwrap();
    writeln!(&mut template, "#   # Cleanup Strategy").unwrap();
    writeln!(&mut template, "#   cleanup_strategy:").unwrap();
    writeln!(&mut template, "#     type: highest_relevance        # most_recent|highest_relevance|lru|direct_dependencies").unwrap();
    writeln!(
        &mut template,
        "#     keep_count: 20                 # Number of task outputs to retain"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "# (optional) Globally bypass all permissions for all agents and tasks"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# WARNING: This is dangerous and should only be used in trusted environments"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# When true, all permission checks are skipped for all agents and tasks"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# dangerously_skip_permissions: false  # Default: false"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    // Secrets section
    writeln!(&mut template, "# (optional) Secrets management").unwrap();
    writeln!(
        &mut template,
        "# Securely manage credentials and sensitive data"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Reference secrets using ${{secret.secret_name}} syntax"
    )
    .unwrap();
    writeln!(&mut template, "# secrets:").unwrap();
    writeln!(&mut template, "#   api_token:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(
        &mut template,
        "#       type: env                      # env, file, or value"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       var: \"API_TOKEN\"              # For env source: environment variable name"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#     description: \"API authentication token\""
    )
    .unwrap();
    writeln!(&mut template, "#   db_password:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(&mut template, "#       type: file").unwrap();
    writeln!(
        &mut template,
        "#       path: \".secrets/password.txt\" # For file source: path to secret file"
    )
    .unwrap();
    writeln!(&mut template, "#     description: \"Database password\"").unwrap();
    writeln!(&mut template, "#   signing_key:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(&mut template, "#       type: value").unwrap();
    writeln!(&mut template, "#       value: \"hardcoded-key\"        # For value source: direct value (not recommended for production)").unwrap();
    writeln!(
        &mut template,
        "#     description: \"Signing key for tokens\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    // Variables section
    writeln!(&mut template, "# (optional) Workflow input variables").unwrap();
    writeln!(&mut template, "# Variables can be referenced in descriptions, prompts, and inputs using ${{scope.var}} syntax").unwrap();
    writeln!(
        &mut template,
        "# Available scopes: workflow, agent, task, subflow"
    )
    .unwrap();
    writeln!(&mut template, "# inputs:").unwrap();
    writeln!(&mut template, "#   project_name:").unwrap();
    writeln!(
        &mut template,
        "#     type: string                    # string, number, boolean, object, array"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#     required: true                  # Whether this input must be provided"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#     default: \"MyProject\"            # Optional default value"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#     description: \"Name of project\"  # Optional description"
    )
    .unwrap();
    writeln!(&mut template, "#   max_iterations:").unwrap();
    writeln!(&mut template, "#     type: number").unwrap();
    writeln!(&mut template, "#     required: false").unwrap();
    writeln!(&mut template, "#     default: 10").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "# (optional) Workflow output variables").unwrap();
    writeln!(
        &mut template,
        "# Outputs can read from files, state, or task outputs"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# IMPORTANT: source must be an object with 'type' field"
    )
    .unwrap();
    writeln!(&mut template, "# outputs:").unwrap();
    writeln!(&mut template, "#   final_report:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(
        &mut template,
        "#       type: file                  # REQUIRED: file, state, or task_output"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       path: \"./report.md\"         # For type: file"
    )
    .unwrap();
    writeln!(&mut template, "#     description: \"Generated report\"").unwrap();
    writeln!(&mut template, "#   task_result:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(
        &mut template,
        "#       type: state                 # Read from workflow state"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       key: \"result\"               # For type: state"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#     description: \"Task result from state\""
    )
    .unwrap();
    writeln!(&mut template, "#   analysis:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(
        &mut template,
        "#       type: task_output           # Read from task output"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       task: \"analyze\"             # For type: task_output"
    )
    .unwrap();
    writeln!(&mut template, "#     description: \"Analysis task output\"").unwrap();
    writeln!(&mut template).unwrap();

    // Agents section
    writeln!(&mut template, "# Agent definitions").unwrap();
    writeln!(
        &mut template,
        "# Agents are AI assistants that execute tasks"
    )
    .unwrap();
    writeln!(&mut template, "agents:").unwrap();
    writeln!(
        &mut template,
        "  # Agent name (used to reference this agent in tasks)"
    )
    .unwrap();
    writeln!(&mut template, "  example_agent:").unwrap();
    writeln!(
        &mut template,
        "    # (REQUIRED) Description of what this agent does"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    description: \"Performs research and analysis\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Model to use (e.g., claude-sonnet-4-5, claude-opus-4)"
    )
    .unwrap();
    writeln!(&mut template, "    # Defaults to claude-sonnet-4-5").unwrap();
    writeln!(&mut template, "    model: \"claude-sonnet-4-5\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Custom system prompt for this agent"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # system_prompt: \"You are a specialized research assistant...\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Working directory for this agent (overrides workflow cwd)"
    )
    .unwrap();
    writeln!(&mut template, "    # cwd: \"/path/to/agent/workspace\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Create working directory if it doesn't exist"
    )
    .unwrap();
    writeln!(&mut template, "    # create_cwd: true").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "    # (optional) Agent input variables").unwrap();
    writeln!(
        &mut template,
        "    # Can be referenced as ${{agent.var_name}} in descriptions and prompts"
    )
    .unwrap();
    writeln!(&mut template, "    # inputs:").unwrap();
    writeln!(&mut template, "#       api_key:").unwrap();
    writeln!(&mut template, "#         type: string").unwrap();
    writeln!(&mut template, "#         required: true").unwrap();
    writeln!(
        &mut template,
        "#         description: \"API key for external service\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "    # (optional) Agent output variables").unwrap();
    writeln!(
        &mut template,
        "    # IMPORTANT: source must be an object with 'type' field"
    )
    .unwrap();
    writeln!(&mut template, "    # outputs:").unwrap();
    writeln!(&mut template, "#       result:").unwrap();
    writeln!(&mut template, "#         source:").unwrap();
    writeln!(
        &mut template,
        "#           type: state             # REQUIRED: file, state, or task_output"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#           key: \"agent_result\"    # For type: state"
    )
    .unwrap();
    writeln!(&mut template, "#         description: \"Agent result\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "    # (optional) Tools this agent can use").unwrap();
    writeln!(&mut template, "    # Available tools: Read, Write, Edit, Bash, WebSearch, WebFetch, Glob, Grep, Task, etc.").unwrap();
    writeln!(&mut template, "    tools:").unwrap();
    writeln!(&mut template, "      - Read").unwrap();
    writeln!(&mut template, "      - Write").unwrap();
    writeln!(&mut template, "      - Bash").unwrap();
    writeln!(&mut template, "      - WebSearch").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Permission settings for this agent"
    )
    .unwrap();
    writeln!(&mut template, "    permissions:").unwrap();
    writeln!(
        &mut template,
        "      # Permission mode: default, acceptEdits, plan, bypassPermissions"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      # - default: Ask for permission on dangerous operations"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      # - acceptEdits: Auto-approve file edits"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      # - plan: Planning mode without execution"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      # - bypassPermissions: Skip all permission checks (use with caution)"
    )
    .unwrap();
    writeln!(&mut template, "      mode: \"default\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "      # (optional) Directories this agent is allowed to access"
    )
    .unwrap();
    writeln!(&mut template, "      # allowed_directories:").unwrap();
    writeln!(&mut template, "      #   - \"/path/to/allowed/dir\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Maximum number of conversation turns"
    )
    .unwrap();
    writeln!(&mut template, "    # max_turns: 10").unwrap();
    writeln!(&mut template).unwrap();

    // Add processor_agent for the hierarchical tasks example
    writeln!(&mut template, "  # Additional agent for processing tasks").unwrap();
    writeln!(&mut template, "  processor_agent:").unwrap();
    writeln!(
        &mut template,
        "    description: \"Processes and transforms data\""
    )
    .unwrap();
    writeln!(&mut template, "    tools:").unwrap();
    writeln!(&mut template, "      - Read").unwrap();
    writeln!(&mut template, "      - Write").unwrap();
    writeln!(&mut template, "      - Edit").unwrap();
    writeln!(&mut template, "    permissions:").unwrap();
    writeln!(&mut template, "      mode: \"acceptEdits\"").unwrap();
    writeln!(&mut template).unwrap();

    // Tasks section
    writeln!(&mut template, "# Task definitions").unwrap();
    writeln!(
        &mut template,
        "# Tasks are units of work executed by agents"
    )
    .unwrap();
    writeln!(&mut template, "tasks:").unwrap();
    writeln!(&mut template, "  # Task name (unique identifier)").unwrap();
    writeln!(&mut template, "  example_task:").unwrap();
    writeln!(
        &mut template,
        "    # (REQUIRED) Description of what this task should accomplish"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    description: \"Research and summarize the topic\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "    # (optional) Agent to execute this task").unwrap();
    writeln!(&mut template, "    agent: \"example_agent\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "    # (optional) Reference a predefined task (mutually exclusive with agent/script/command/http/mcp_tool)").unwrap();
    writeln!(
        &mut template,
        "    # Format: \"task-name@version\" or \"namespace/task-name@version\""
    )
    .unwrap();
    writeln!(&mut template, "    # uses: \"google-drive-upload@1.2.0\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Embed a predefined task (copy definition instead of referencing)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Mutually exclusive with 'uses' and other execution types"
    )
    .unwrap();
    writeln!(&mut template, "    # embed: \"data-validation@2.0.0\"").unwrap();
    writeln!(
        &mut template,
        "    # overrides:  # Optional overrides for embedded tasks"
    )
    .unwrap();
    writeln!(&mut template, "    #   description: \"Custom description\"").unwrap();
    writeln!(&mut template, "    #   inputs:").unwrap();
    writeln!(&mut template, "    #     custom_input: \"value\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Reference a prebuilt workflow from a task group"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Format: \"namespace:workflow_name\" (mutually exclusive with other execution types)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # uses_workflow: \"google:upload-files\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Task input values (can reference variables)"
    )
    .unwrap();
    writeln!(&mut template, "    # inputs:").unwrap();
    writeln!(
        &mut template,
        "#       config: \"${{workflow.project_name}}\""
    )
    .unwrap();
    writeln!(&mut template, "#       iterations: 5").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "    # (optional) Task output variables").unwrap();
    writeln!(
        &mut template,
        "    # Can be referenced by dependent tasks as ${{task.var_name}}"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # IMPORTANT: source must be an object with 'type' field"
    )
    .unwrap();
    writeln!(&mut template, "    # outputs:").unwrap();
    writeln!(&mut template, "#       analysis_result:").unwrap();
    writeln!(&mut template, "#         source:").unwrap();
    writeln!(
        &mut template,
        "#           type: file              # REQUIRED: file, state, or task_output"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#           path: \"./analysis.json\" # For type: file"
    )
    .unwrap();
    writeln!(&mut template, "#         description: \"Analysis result\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "# DETERMINISTIC TASK EXECUTION").unwrap();
    writeln!(
        &mut template,
        "# Tasks can execute deterministic operations instead of AI agents:"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# - Scripts in multiple languages (Python, JavaScript, Bash, Ruby, Perl)"
    )
    .unwrap();
    writeln!(&mut template, "# - External commands/executables").unwrap();
    writeln!(&mut template, "# - HTTP requests with authentication").unwrap();
    writeln!(&mut template, "# - Direct MCP tool invocation").unwrap();
    writeln!(
        &mut template,
        "# All support variable interpolation using ${{scope.variable}} syntax"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "  # Example: Script execution task").unwrap();
    writeln!(&mut template, "  script_task:").unwrap();
    writeln!(
        &mut template,
        "    description: \"Execute Python/JavaScript/Bash/Ruby/Perl script\""
    )
    .unwrap();
    writeln!(&mut template, "    # VARIABLE INTERPOLATION:").unwrap();
    writeln!(
        &mut template,
        "    # - ${{workflow.var}} and ${{task.var}} are substituted before execution"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # - Use \\${{...}} to escape (becomes literal ${{...}} in script)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # - Useful for mixing DSL variables with shell/language variables"
    )
    .unwrap();
    writeln!(&mut template, "    # (option 1) Inline script content").unwrap();
    writeln!(&mut template, "    script:").unwrap();
    writeln!(
        &mut template,
        "      language: python  # python, javascript, bash, ruby, perl"
    )
    .unwrap();
    writeln!(&mut template, "      content: |").unwrap();
    writeln!(&mut template, "        import json").unwrap();
    writeln!(&mut template, "        import os").unwrap();
    writeln!(
        &mut template,
        "        # DSL variable interpolation: ${{workflow.project_name}} becomes actual value"
    )
    .unwrap();
    writeln!(
        &mut template,
        "        project = \"${{workflow.project_name}}\"  # Replaced with actual value"
    )
    .unwrap();
    writeln!(
        &mut template,
        "        # Shell variable (escaped): \\${{PATH}} becomes literal ${{PATH}}"
    )
    .unwrap();
    writeln!(
        &mut template,
        "        path = os.environ.get('PATH', \\${{PATH}})  # Literal ${{PATH}} in script"
    )
    .unwrap();
    writeln!(&mut template, "        print(f\"Processing: {{project}}\")").unwrap();
    writeln!(
        &mut template,
        "        result = {{\"project\": project, \"status\": \"complete\"}}"
    )
    .unwrap();
    writeln!(&mut template, "        print(json.dumps(result))").unwrap();
    writeln!(
        &mut template,
        "      env:  # Optional environment variables"
    )
    .unwrap();
    writeln!(
        &mut template,
        "        PROJECT_NAME: \"${{workflow.project_name}}\""
    )
    .unwrap();
    writeln!(&mut template, "        API_KEY: \"${{secret.api_token}}\"").unwrap();
    writeln!(
        &mut template,
        "      working_dir: \".\"  # Optional working directory"
    )
    .unwrap();
    writeln!(&mut template, "      timeout_secs: 60  # Optional timeout").unwrap();
    writeln!(&mut template, "    # (option 2) External script file").unwrap();
    writeln!(&mut template, "    # script:").unwrap();
    writeln!(&mut template, "#       language: python").unwrap();
    writeln!(&mut template, "#       file: \"scripts/process.py\"").unwrap();
    writeln!(&mut template, "#       env:").unwrap();
    writeln!(
        &mut template,
        "#         DATA_DIR: \"${{workflow.data_directory}}\""
    )
    .unwrap();
    writeln!(&mut template, "#       timeout_secs: 120").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "  # Example: Command execution task").unwrap();
    writeln!(&mut template, "  command_task:").unwrap();
    writeln!(
        &mut template,
        "    description: \"Execute external command or tool\""
    )
    .unwrap();
    writeln!(&mut template, "    command:").unwrap();
    writeln!(
        &mut template,
        "      executable: \"pandoc\"  # Command to execute"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      args:  # Command arguments (supports variable interpolation)"
    )
    .unwrap();
    writeln!(&mut template, "        - \"--from=markdown\"").unwrap();
    writeln!(&mut template, "        - \"--to=html\"").unwrap();
    writeln!(&mut template, "        - \"-o\"").unwrap();
    writeln!(
        &mut template,
        "        - \"output/${{workflow.project_name}}.html\""
    )
    .unwrap();
    writeln!(&mut template, "        - \"input.md\"").unwrap();
    writeln!(
        &mut template,
        "      env:  # Optional environment variables"
    )
    .unwrap();
    writeln!(&mut template, "        PANDOC_DATA_DIR: \"/custom/path\"").unwrap();
    writeln!(
        &mut template,
        "      working_dir: \".\"  # Optional working directory"
    )
    .unwrap();
    writeln!(&mut template, "      timeout_secs: 60  # Optional timeout").unwrap();
    writeln!(
        &mut template,
        "      capture_stdout: true  # Capture standard output"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      capture_stderr: true  # Capture standard error"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "  # Example: HTTP request task").unwrap();
    writeln!(&mut template, "  http_task:").unwrap();
    writeln!(
        &mut template,
        "    description: \"Make HTTP request with authentication\""
    )
    .unwrap();
    writeln!(&mut template, "    http:").unwrap();
    writeln!(
        &mut template,
        "      method: GET  # GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      url: \"${{workflow.api_endpoint}}/data\"  # Supports variable interpolation"
    )
    .unwrap();
    writeln!(&mut template, "      headers:  # Optional headers").unwrap();
    writeln!(&mut template, "        Content-Type: \"application/json\"").unwrap();
    writeln!(&mut template, "        User-Agent: \"DSL-Workflow/1.0\"").unwrap();
    writeln!(
        &mut template,
        "      body: |  # Optional request body (for POST/PUT/PATCH)"
    )
    .unwrap();
    writeln!(&mut template, "        {{").unwrap();
    writeln!(
        &mut template,
        "          \"project\": \"${{workflow.project_name}}\","
    )
    .unwrap();
    writeln!(&mut template, "          \"action\": \"process\"").unwrap();
    writeln!(&mut template, "        }}").unwrap();
    writeln!(
        &mut template,
        "      # Authentication options (choose one):"
    )
    .unwrap();
    writeln!(&mut template, "      # (1) Bearer token authentication").unwrap();
    writeln!(&mut template, "      auth:").unwrap();
    writeln!(&mut template, "        type: bearer").unwrap();
    writeln!(&mut template, "        token: \"${{secret.api_token}}\"").unwrap();
    writeln!(&mut template, "      # (2) Basic authentication").unwrap();
    writeln!(&mut template, "#       auth:").unwrap();
    writeln!(&mut template, "#         type: basic").unwrap();
    writeln!(&mut template, "#         username: \"api_user\"").unwrap();
    writeln!(
        &mut template,
        "#         password: \"${{secret.api_password}}\""
    )
    .unwrap();
    writeln!(&mut template, "      # (3) API Key authentication").unwrap();
    writeln!(&mut template, "#       auth:").unwrap();
    writeln!(&mut template, "#         type: api_key").unwrap();
    writeln!(&mut template, "#         header: \"X-API-Key\"").unwrap();
    writeln!(&mut template, "#         key: \"${{secret.api_key}}\"").unwrap();
    writeln!(&mut template, "      # (4) Custom headers authentication").unwrap();
    writeln!(&mut template, "#       auth:").unwrap();
    writeln!(&mut template, "#         type: custom").unwrap();
    writeln!(&mut template, "#         headers:").unwrap();
    writeln!(
        &mut template,
        "#           X-Custom-Auth: \"${{secret.custom_token}}\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "#           X-Request-ID: \"${{workflow.request_id}}\""
    )
    .unwrap();
    writeln!(&mut template, "      timeout_secs: 30  # Optional timeout").unwrap();
    writeln!(
        &mut template,
        "      follow_redirects: true  # Follow HTTP redirects"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      verify_tls: true  # Verify TLS certificates"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "  # Example: MCP tool invocation task").unwrap();
    writeln!(&mut template, "  mcp_tool_task:").unwrap();
    writeln!(
        &mut template,
        "    description: \"Invoke MCP tool directly\""
    )
    .unwrap();
    writeln!(&mut template, "    mcp_tool:").unwrap();
    writeln!(
        &mut template,
        "      server: \"data_processor\"  # MCP server name (must be defined in mcp_servers)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      tool: \"transform_data\"  # Tool name from MCP server"
    )
    .unwrap();
    writeln!(
        &mut template,
        "      parameters:  # Tool-specific parameters (supports variable interpolation)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "        input_file: \"data/${{workflow.project_name}}.json\""
    )
    .unwrap();
    writeln!(&mut template, "        output_format: \"csv\"").unwrap();
    writeln!(&mut template, "        options:").unwrap();
    writeln!(&mut template, "          normalize: true").unwrap();
    writeln!(&mut template, "          remove_duplicates: true").unwrap();
    writeln!(&mut template, "      timeout_secs: 60  # Optional timeout").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "# STANDARD TASK FIELDS").unwrap();
    writeln!(&mut template, "# The following fields apply to all task types (AI agent, script, command, http, mcp_tool)").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "  standard_task:").unwrap();
    writeln!(
        &mut template,
        "    description: \"Standard task with common fields\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "    agent: \"example_agent\"  # For AI agent tasks"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Priority (lower number = higher priority, default: 0)"
    )
    .unwrap();
    writeln!(&mut template, "    priority: 1").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Tasks this task depends on (must complete first)"
    )
    .unwrap();
    writeln!(&mut template, "    # depends_on:").unwrap();
    writeln!(&mut template, "    #   - other_task").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Tasks that can run in parallel with this one"
    )
    .unwrap();
    writeln!(&mut template, "    # parallel_with:").unwrap();
    writeln!(&mut template, "    #   - parallel_task").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) File to save task output to"
    )
    .unwrap();
    writeln!(&mut template, "    # output: \"results.txt\"").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Nested subtasks (hierarchical task decomposition)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # IMPORTANT: Parent tasks with subtasks serve ONLY as grouping/template mechanisms"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # - Parent tasks with subtasks DO NOT EXECUTE - only their subtasks execute"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # - Parent tasks DON'T need an 'agent' if they have subtasks"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # - Subtasks inherit attributes (agent, priority, on_error, etc.) from parent"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # - Subtasks can override any inherited attribute"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # - IMPORTANT: If subtask defines ANY execution type (agent, subflow, script, etc.),"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   it will NOT inherit parent's agent - allows mixing execution types!"
    )
    .unwrap();
    writeln!(&mut template, "    # subtasks:").unwrap();
    writeln!(&mut template, "    #   - subtask_1:").unwrap();
    writeln!(&mut template, "    #       description: \"First subtask\"").unwrap();
    writeln!(
        &mut template,
        "    #       # No execution type - inherits agent from parent"
    )
    .unwrap();
    writeln!(&mut template, "    #   - subtask_2:").unwrap();
    writeln!(&mut template, "    #       description: \"Second subtask\"").unwrap();
    writeln!(
        &mut template,
        "    #       agent: \"different_agent\"  # Override with different agent"
    )
    .unwrap();
    writeln!(&mut template, "    #   - subtask_3:").unwrap();
    writeln!(
        &mut template,
        "    #       description: \"Third subtask with script\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #       script:  # Different execution type - won't inherit parent's agent"
    )
    .unwrap();
    writeln!(&mut template, "    #         language: bash").unwrap();
    writeln!(
        &mut template,
        "    #         content: \"echo 'Running script'\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Action to execute on task completion"
    )
    .unwrap();
    writeln!(&mut template, "    # on_complete:").unwrap();
    writeln!(
        &mut template,
        "    #   notify: \"Task completed successfully!\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Error handling configuration"
    )
    .unwrap();
    writeln!(&mut template, "    # on_error:").unwrap();
    writeln!(
        &mut template,
        "    #   retry: 3                    # Number of retries"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   retry_delay_secs: 1         # Delay between retries"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   exponential_backoff: true   # Use exponential backoff"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   fallback_agent: \"backup_agent\"  # Agent to use if this fails"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(&mut template, "    # (optional) Conditional execution").unwrap();
    writeln!(
        &mut template,
        "    # Execute this task only if condition is met"
    )
    .unwrap();
    writeln!(&mut template, "    # Single condition:").unwrap();
    writeln!(&mut template, "    # condition:").unwrap();
    writeln!(
        &mut template,
        "    #   type: state_equals  # or state_exists, task_status, always, never"
    )
    .unwrap();
    writeln!(&mut template, "    #   key: \"environment\"").unwrap();
    writeln!(&mut template, "    #   value: \"production\"").unwrap();
    writeln!(&mut template, "    # Combined conditions (AND):").unwrap();
    writeln!(&mut template, "    # condition:").unwrap();
    writeln!(&mut template, "    #   and:").unwrap();
    writeln!(&mut template, "    #     - type: state_equals").unwrap();
    writeln!(&mut template, "    #       key: \"env\"").unwrap();
    writeln!(&mut template, "    #       value: \"prod\"").unwrap();
    writeln!(&mut template, "    #     - type: task_status").unwrap();
    writeln!(&mut template, "    #       task: \"setup\"").unwrap();
    writeln!(&mut template, "    #       status: completed").unwrap();
    writeln!(&mut template, "    # Combined conditions (OR):").unwrap();
    writeln!(&mut template, "    # condition:").unwrap();
    writeln!(&mut template, "    #   or:").unwrap();
    writeln!(&mut template, "    #     - type: always").unwrap();
    writeln!(&mut template, "    #     - type: state_exists").unwrap();
    writeln!(&mut template, "    #       key: \"override\"").unwrap();
    writeln!(&mut template, "    # Negated condition (NOT):").unwrap();
    writeln!(&mut template, "    # condition:").unwrap();
    writeln!(&mut template, "    #   not:").unwrap();
    writeln!(&mut template, "    #     type: task_status").unwrap();
    writeln!(&mut template, "    #     task: \"build\"").unwrap();
    writeln!(&mut template, "    #     status: failed").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Definition of Done - Quality criteria for task completion"
    )
    .unwrap();
    writeln!(&mut template, "    # definition_of_done:").unwrap();
    writeln!(&mut template, "    #   criteria:").unwrap();
    writeln!(&mut template, "    #     # Check if file exists").unwrap();
    writeln!(&mut template, "    #     - type: file_exists").unwrap();
    writeln!(&mut template, "    #       path: \"output.md\"").unwrap();
    writeln!(
        &mut template,
        "    #       description: \"Output file exists\""
    )
    .unwrap();
    writeln!(&mut template, "    #     # Check if file contains pattern").unwrap();
    writeln!(&mut template, "    #     - type: file_contains").unwrap();
    writeln!(&mut template, "    #       path: \"output.md\"").unwrap();
    writeln!(&mut template, "    #       pattern: \"Summary\"").unwrap();
    writeln!(
        &mut template,
        "    #       description: \"Output contains summary\""
    )
    .unwrap();
    writeln!(&mut template, "    #     # Run tests and check they pass").unwrap();
    writeln!(&mut template, "    #     - type: tests_passed").unwrap();
    writeln!(&mut template, "    #       command: \"cargo\"").unwrap();
    writeln!(&mut template, "    #       args: [\"test\"]").unwrap();
    writeln!(&mut template, "    #       description: \"All tests pass\"").unwrap();
    writeln!(
        &mut template,
        "    #     # Run command and check it succeeds"
    )
    .unwrap();
    writeln!(&mut template, "    #     - type: command_succeeds").unwrap();
    writeln!(&mut template, "    #       command: \"cargo\"").unwrap();
    writeln!(&mut template, "    #       args: [\"clippy\"]").unwrap();
    writeln!(
        &mut template,
        "    #       description: \"No linting errors\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #     # Other types: file_not_contains, output_matches, directory_exists"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   max_retries: 2              # Retry if DoD not met (default: 3)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   fail_on_unmet: true         # Fail task if DoD not met (default: true)"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Task inputs - automatically injected into agent prompts"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Inputs provide configuration values that agents can use"
    )
    .unwrap();
    writeln!(&mut template, "    # inputs:").unwrap();
    writeln!(&mut template, "    #   topic: \"Rust async programming\"").unwrap();
    writeln!(&mut template, "    #   depth: \"detailed\"").unwrap();
    writeln!(&mut template).unwrap();
    writeln!(
        &mut template,
        "    # (optional) Task outputs - automatically injected into agent prompts"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Outputs define where task results should be stored"
    )
    .unwrap();
    writeln!(&mut template, "    # outputs:").unwrap();
    writeln!(&mut template, "    #   report:").unwrap();
    writeln!(&mut template, "    #     source:").unwrap();
    writeln!(&mut template, "    #       type: file").unwrap();
    writeln!(&mut template, "    #       path: \"./output/report.md\"").unwrap();
    writeln!(
        &mut template,
        "    #     description: \"Research findings\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();
    writeln!(
        &mut template,
        "    # (optional) Inject workflow execution context into agent-based tasks"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # When true, agent receives context about completed tasks and their results"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # inject_context: true  # Default: false"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Context configuration for smart context injection"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Controls which task outputs are injected into this task's context"
    )
    .unwrap();
    writeln!(&mut template, "    # context:").unwrap();
    writeln!(
        &mut template,
        "#       mode: automatic              # automatic|manual|none (default: automatic)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       min_relevance: 0.5           # 0.0-1.0 filter threshold (default: 0.5)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       max_bytes: 100000            # Override workflow limit"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       max_tasks: 5                 # Override workflow limit"
    )
    .unwrap();
    writeln!(&mut template, "#       # Manual mode only:").unwrap();
    writeln!(
        &mut template,
        "#       include_tasks: [task1]       # Explicit task list"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       exclude_tasks: [noisy_task]  # Tasks to exclude"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Task-level output limits (overrides workflow limits)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Controls stdout/stderr truncation for this specific task"
    )
    .unwrap();
    writeln!(&mut template, "    # limits:").unwrap();
    writeln!(
        &mut template,
        "#       max_stdout_bytes: 10485760   # 10MB override"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       truncation_strategy: both    # head|tail|both|summary"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Bypass all permissions for this specific task"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # WARNING: This is dangerous and should only be used in trusted environments"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # When true, all permission checks are skipped for this task"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # dangerously_skip_permissions: false  # Default: false"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "    # (optional) Loop/iteration configuration"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Execute this task multiple times (ForEach, While, RepeatUntil, Repeat)"
    )
    .unwrap();
    writeln!(&mut template, "    # loop:").unwrap();
    writeln!(
        &mut template,
        "    #   # ForEach Loop - Iterate over collection"
    )
    .unwrap();
    writeln!(&mut template, "    #   type: for_each").unwrap();
    writeln!(&mut template, "    #   collection:").unwrap();
    writeln!(
        &mut template,
        "    #     source: inline            # inline, state, file, range, http"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #     items: [1, 2, 3]          # For inline source"
    )
    .unwrap();
    writeln!(&mut template, "    #     # source: file").unwrap();
    writeln!(&mut template, "    #     # path: \"data.json\"").unwrap();
    writeln!(
        &mut template,
        "    #     # format: json             # json, json_lines, csv, lines"
    )
    .unwrap();
    writeln!(&mut template, "    #     # source: http").unwrap();
    writeln!(
        &mut template,
        "    #     # url: \"https://api.example.com/items\""
    )
    .unwrap();
    writeln!(&mut template, "    #     # method: \"GET\"").unwrap();
    writeln!(&mut template, "    #     # headers:").unwrap();
    writeln!(
        &mut template,
        "    #     #   Authorization: \"Bearer token\""
    )
    .unwrap();
    writeln!(&mut template, "    #     # format: json").unwrap();
    writeln!(
        &mut template,
        "    #     # json_path: \"data.items\" # Extract nested array"
    )
    .unwrap();
    writeln!(&mut template, "    #     # source: range").unwrap();
    writeln!(&mut template, "    #     # start: 0").unwrap();
    writeln!(&mut template, "    #     # end: 100").unwrap();
    writeln!(&mut template, "    #     # step: 1").unwrap();
    writeln!(&mut template, "    #     # source: state").unwrap();
    writeln!(
        &mut template,
        "    #     # key: \"previous_results\"  # From previous task"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   iterator: \"item\"            # Variable name for current item"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # Loop variables available in scripts/env/output:"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   #   - ${{task.loop_index}}    # 0-based iteration index"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   #   - ${{task.item}}          # Current item value (if iterator=\"item\")"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   parallel: false             # Execute in parallel?"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   max_parallel: 5             # Limit concurrent iterations"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # While Loop - Execute while condition is true"
    )
    .unwrap();
    writeln!(&mut template, "    #   # type: while").unwrap();
    writeln!(&mut template, "    #   # condition:").unwrap();
    writeln!(&mut template, "    #   #   type: state_equals").unwrap();
    writeln!(&mut template, "    #   #   key: \"job_complete\"").unwrap();
    writeln!(&mut template, "    #   #   value: false").unwrap();
    writeln!(
        &mut template,
        "    #   # max_iterations: 100       # Safety limit (required)"
    )
    .unwrap();
    writeln!(&mut template, "    #   # iteration_variable: \"iteration\"").unwrap();
    writeln!(
        &mut template,
        "    #   # delay_between_secs: 5     # Wait between iterations"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # RepeatUntil Loop - Execute until condition is true"
    )
    .unwrap();
    writeln!(&mut template, "    #   # type: repeat_until").unwrap();
    writeln!(&mut template, "    #   # condition:").unwrap();
    writeln!(&mut template, "    #   #   type: state_equals").unwrap();
    writeln!(&mut template, "    #   #   key: \"success\"").unwrap();
    writeln!(&mut template, "    #   #   value: true").unwrap();
    writeln!(
        &mut template,
        "    #   # min_iterations: 1         # Minimum iterations"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # max_iterations: 10        # Maximum iterations (required)"
    )
    .unwrap();
    writeln!(&mut template, "    #   # iteration_variable: \"attempt\"").unwrap();
    writeln!(
        &mut template,
        "    #   # delay_between_secs: 2     # Exponential backoff"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # Repeat Loop - Execute fixed number of times"
    )
    .unwrap();
    writeln!(&mut template, "    #   # type: repeat").unwrap();
    writeln!(
        &mut template,
        "    #   # count: 10                 # Number of iterations"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # iterator: \"iteration\"     # Variable name"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # Loop variables available in scripts/env/output:"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   #   - ${{task.loop_index}}    # 0-based iteration index (always available)"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   #   - ${{task.iteration}}     # If iterator=\"iteration\" specified"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # parallel: false           # Execute in parallel?"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   # max_parallel: 3           # Limit concurrent iterations"
    )
    .unwrap();
    writeln!(&mut template, "    # loop_control:").unwrap();
    writeln!(
        &mut template,
        "    #   break_condition:            # Stop loop early"
    )
    .unwrap();
    writeln!(&mut template, "    #     type: state_equals").unwrap();
    writeln!(&mut template, "    #     key: \"critical_error\"").unwrap();
    writeln!(&mut template, "    #     value: true").unwrap();
    writeln!(
        &mut template,
        "    #   continue_condition:         # Skip iteration"
    )
    .unwrap();
    writeln!(&mut template, "    #     type: state_equals").unwrap();
    writeln!(&mut template, "    #     key: \"skip_item\"").unwrap();
    writeln!(&mut template, "    #     value: true").unwrap();
    writeln!(
        &mut template,
        "    #   timeout_secs: 300           # Overall loop timeout"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   checkpoint_interval: 50     # Save state every N iterations"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   collect_results: true       # Collect iteration outputs"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    #   result_key: \"results\"       # Where to store results"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    // Hierarchical tasks example
    writeln!(&mut template, "# HIERARCHICAL TASKS (Parent/Subtasks)").unwrap();
    writeln!(
        &mut template,
        "# Complete example showing parent tasks as grouping mechanisms"
    )
    .unwrap();
    writeln!(&mut template, "# and flexible execution type mixing").unwrap();
    writeln!(&mut template).unwrap();

    writeln!(
        &mut template,
        "  # Example: Parent task for grouping (NO agent needed)"
    )
    .unwrap();
    writeln!(&mut template, "  file_processing_group:").unwrap();
    writeln!(
        &mut template,
        "    description: \"Process files in stages\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # NOTE: No 'agent' field needed - this parent task won't execute"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    # Subtasks without execution types will inherit these settings:"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    agent: \"example_agent\"  # Inherited by subtasks without execution type"
    )
    .unwrap();
    writeln!(
        &mut template,
        "    priority: 5  # Inherited by all subtasks"
    )
    .unwrap();
    writeln!(&mut template, "    on_error:  # Inherited by all subtasks").unwrap();
    writeln!(&mut template, "      retry: 2").unwrap();
    writeln!(&mut template, "      retry_delay_secs: 5").unwrap();
    writeln!(
        &mut template,
        "    # Subtasks (only these will actually execute):"
    )
    .unwrap();
    writeln!(&mut template, "    subtasks:").unwrap();
    writeln!(&mut template, "      - scan_files:").unwrap();
    writeln!(
        &mut template,
        "          description: \"Scan and catalog files\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "          # No execution type - inherits agent, priority, on_error from parent"
    )
    .unwrap();
    writeln!(&mut template).unwrap();
    writeln!(&mut template, "      - validate_files:").unwrap();
    writeln!(
        &mut template,
        "          description: \"Validate file formats using script\""
    )
    .unwrap();
    writeln!(&mut template, "          depends_on:").unwrap();
    writeln!(
        &mut template,
        "            - scan_files  # Depends on sibling subtask"
    )
    .unwrap();
    writeln!(
        &mut template,
        "          # Uses script execution type - won't inherit parent's agent"
    )
    .unwrap();
    writeln!(&mut template, "          script:").unwrap();
    writeln!(&mut template, "            language: bash").unwrap();
    writeln!(
        &mut template,
        "            content: \"find . -type f -name '*.txt' | xargs file\""
    )
    .unwrap();
    writeln!(&mut template).unwrap();
    writeln!(&mut template, "      - process_files:").unwrap();
    writeln!(
        &mut template,
        "          description: \"Process validated files\""
    )
    .unwrap();
    writeln!(&mut template, "          depends_on:").unwrap();
    writeln!(&mut template, "            - validate_files").unwrap();
    writeln!(
        &mut template,
        "          # Override with different agent (still inherits priority, on_error)"
    )
    .unwrap();
    writeln!(&mut template, "          agent: \"processor_agent\"").unwrap();
    writeln!(
        &mut template,
        "          priority: 1  # Override parent's priority (higher priority)"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    // Subflows section
    writeln!(&mut template, "# (optional) Subflow definitions").unwrap();
    writeln!(
        &mut template,
        "# Subflows are reusable workflow components that can be defined inline,"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# in local files, or fetched from remote sources (git, HTTP)"
    )
    .unwrap();
    writeln!(&mut template, "# subflows:").unwrap();
    writeln!(
        &mut template,
        "#   # Inline subflow - defined directly in this file"
    )
    .unwrap();
    writeln!(&mut template, "#   validation:").unwrap();
    writeln!(
        &mut template,
        "#     description: \"Validate inputs and outputs\""
    )
    .unwrap();
    writeln!(&mut template, "#     agents:").unwrap();
    writeln!(&mut template, "#       validator:").unwrap();
    writeln!(&mut template, "#         description: \"Validation agent\"").unwrap();
    writeln!(&mut template, "#         tools:").unwrap();
    writeln!(&mut template, "#           - Read").unwrap();
    writeln!(&mut template, "#           - Bash").unwrap();
    writeln!(&mut template, "#     tasks:").unwrap();
    writeln!(&mut template, "#       validate_data:").unwrap();
    writeln!(
        &mut template,
        "#         description: \"Run validation checks\""
    )
    .unwrap();
    writeln!(&mut template, "#         agent: \"validator\"").unwrap();
    writeln!(&mut template, "#     inputs:").unwrap();
    writeln!(&mut template, "#       data_file:").unwrap();
    writeln!(&mut template, "#         type: \"string\"").unwrap();
    writeln!(&mut template, "#         required: true").unwrap();
    writeln!(
        &mut template,
        "#         description: \"Path to data file to validate\""
    )
    .unwrap();
    writeln!(&mut template, "#     outputs:").unwrap();
    writeln!(&mut template, "#       validation_result:").unwrap();
    writeln!(&mut template, "#         source:").unwrap();
    writeln!(&mut template, "#           type: file").unwrap();
    writeln!(&mut template, "#           path: \"validation.log\"").unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "#   # External subflow from local file").unwrap();
    writeln!(&mut template, "#   testing:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(&mut template, "#       type: file").unwrap();
    writeln!(&mut template, "#       path: \"./subflows/testing.yaml\"").unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "#   # External subflow from git repository").unwrap();
    writeln!(&mut template, "#   deployment:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(&mut template, "#       type: git").unwrap();
    writeln!(
        &mut template,
        "#       url: \"https://github.com/org/workflows.git\""
    )
    .unwrap();
    writeln!(&mut template, "#       path: \"subflows/deploy.yaml\"").unwrap();
    writeln!(
        &mut template,
        "#       reference: \"v1.0.0\"  # branch, tag, or commit"
    )
    .unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "#   # External subflow from HTTP").unwrap();
    writeln!(&mut template, "#   monitoring:").unwrap();
    writeln!(&mut template, "#     source:").unwrap();
    writeln!(&mut template, "#       type: http").unwrap();
    writeln!(
        &mut template,
        "#       url: \"https://example.com/workflows/monitoring.yaml\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "#       checksum: \"sha256:abc123...\"  # optional integrity check"
    )
    .unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "# Reference subflows in tasks:").unwrap();
    writeln!(&mut template, "# tasks:").unwrap();
    writeln!(&mut template, "#   run_validation:").unwrap();
    writeln!(
        &mut template,
        "#     description: \"Execute validation subflow\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "#     subflow: \"validation\"  # Reference to subflow (mutually exclusive with agent)"
    )
    .unwrap();
    writeln!(&mut template, "#     inputs:").unwrap();
    writeln!(&mut template, "#       data_file: \"./data.json\"").unwrap();
    writeln!(&mut template).unwrap();

    // Imports section
    writeln!(&mut template, "# (optional) Task group imports").unwrap();
    writeln!(
        &mut template,
        "# Import prebuilt workflow collections from external sources"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Format: namespace -> \"group-name@version\""
    )
    .unwrap();
    writeln!(&mut template, "# imports:").unwrap();
    writeln!(&mut template, "#   google: \"google-workspace@1.0.0\"").unwrap();
    writeln!(&mut template, "#   aws: \"aws-services@2.1.0\"").unwrap();
    writeln!(&mut template, "#   slack: \"slack-integration@0.5.0\"").unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(
        &mut template,
        "# Use imported workflows in tasks with uses_workflow:"
    )
    .unwrap();
    writeln!(&mut template, "# tasks:").unwrap();
    writeln!(&mut template, "#   upload_to_drive:").unwrap();
    writeln!(
        &mut template,
        "#     description: \"Upload file to Google Drive\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "#     uses_workflow: \"google:upload-file\"  # namespace:workflow_name"
    )
    .unwrap();
    writeln!(&mut template, "#     inputs:").unwrap();
    writeln!(&mut template, "#       file_path: \"./document.pdf\"").unwrap();
    writeln!(&mut template, "#       folder_id: \"abc123\"").unwrap();
    writeln!(&mut template).unwrap();

    // Workflows section
    writeln!(&mut template, "# (optional) Workflow orchestration").unwrap();
    writeln!(
        &mut template,
        "# Define multi-stage workflows with dependencies"
    )
    .unwrap();
    writeln!(&mut template, "# workflows:").unwrap();
    writeln!(&mut template, "#   main_workflow:").unwrap();
    writeln!(
        &mut template,
        "#     description: \"Main execution workflow\""
    )
    .unwrap();
    writeln!(&mut template, "#     steps:").unwrap();
    writeln!(&mut template, "#       - stage: \"research\"").unwrap();
    writeln!(&mut template, "#         agents:").unwrap();
    writeln!(&mut template, "#           - example_agent").unwrap();
    writeln!(&mut template, "#         tasks:").unwrap();
    writeln!(&mut template, "#           - example_task:").unwrap();
    writeln!(
        &mut template,
        "#               description: \"Research phase\""
    )
    .unwrap();
    writeln!(&mut template, "#               agent: \"example_agent\"").unwrap();
    writeln!(&mut template, "#         mode: sequential  # or parallel").unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "#       - stage: \"implementation\"").unwrap();
    writeln!(&mut template, "#         depends_on:").unwrap();
    writeln!(&mut template, "#           - research").unwrap();
    writeln!(&mut template, "#         agents:").unwrap();
    writeln!(&mut template, "#           - coder_agent").unwrap();
    writeln!(&mut template, "#         tasks:").unwrap();
    writeln!(&mut template, "#           - coding_task:").unwrap();
    writeln!(
        &mut template,
        "#               description: \"Implement features\""
    )
    .unwrap();
    writeln!(&mut template, "#               agent: \"coder_agent\"").unwrap();
    writeln!(&mut template, "#         mode: sequential").unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "#     # (optional) Workflow hooks").unwrap();
    writeln!(&mut template, "#     hooks:").unwrap();
    writeln!(&mut template, "#       pre_workflow:").unwrap();
    writeln!(&mut template, "#         - \"echo 'Starting workflow'\"").unwrap();
    writeln!(&mut template, "#       post_workflow:").unwrap();
    writeln!(&mut template, "#         - \"echo 'Workflow complete'\"").unwrap();
    writeln!(&mut template, "#       on_stage_complete:").unwrap();
    writeln!(&mut template, "#         - \"echo 'Stage finished'\"").unwrap();
    writeln!(&mut template, "#       on_error:").unwrap();
    writeln!(&mut template, "#         - \"echo 'Error occurred'\"").unwrap();
    writeln!(&mut template).unwrap();

    // Tools section
    writeln!(&mut template, "# (optional) Tool configuration").unwrap();
    writeln!(&mut template, "# Global tool settings and constraints").unwrap();
    writeln!(&mut template, "# tools:").unwrap();
    writeln!(&mut template, "#   allowed:").unwrap();
    writeln!(&mut template, "#     - Read").unwrap();
    writeln!(&mut template, "#     - Write").unwrap();
    writeln!(&mut template, "#   disallowed:").unwrap();
    writeln!(&mut template, "#     - Bash  # Disable dangerous tools").unwrap();
    writeln!(&mut template, "#   constraints:").unwrap();
    writeln!(&mut template, "#     Bash:").unwrap();
    writeln!(&mut template, "#       timeout: 30000  # milliseconds").unwrap();
    writeln!(&mut template, "#       allowed_commands:").unwrap();
    writeln!(&mut template, "#         - \"ls\"").unwrap();
    writeln!(&mut template, "#         - \"cat\"").unwrap();
    writeln!(&mut template, "#     Write:").unwrap();
    writeln!(
        &mut template,
        "#       max_file_size: 1048576  # 1MB in bytes"
    )
    .unwrap();
    writeln!(&mut template, "#       allowed_extensions:").unwrap();
    writeln!(&mut template, "#         - \".txt\"").unwrap();
    writeln!(&mut template, "#         - \".md\"").unwrap();
    writeln!(&mut template, "#     WebSearch:").unwrap();
    writeln!(
        &mut template,
        "#       rate_limit: 10  # requests per minute"
    )
    .unwrap();
    writeln!(&mut template).unwrap();

    // Communication section
    writeln!(&mut template, "# (optional) Inter-agent communication").unwrap();
    writeln!(
        &mut template,
        "# Define channels and message types for agent collaboration"
    )
    .unwrap();
    writeln!(&mut template, "# communication:").unwrap();
    writeln!(&mut template, "#   channels:").unwrap();
    writeln!(&mut template, "#     research_channel:").unwrap();
    writeln!(
        &mut template,
        "#       description: \"Research findings channel\""
    )
    .unwrap();
    writeln!(&mut template, "#       participants:").unwrap();
    writeln!(&mut template, "#         - researcher").unwrap();
    writeln!(&mut template, "#         - writer").unwrap();
    writeln!(&mut template, "#       message_format: \"markdown\"").unwrap();
    writeln!(&mut template, "#   message_types:").unwrap();
    writeln!(&mut template, "#     research_result:").unwrap();
    writeln!(&mut template, "#       schema:").unwrap();
    writeln!(&mut template, "#         type: object").unwrap();
    writeln!(&mut template, "#         properties:").unwrap();
    writeln!(&mut template, "#           findings: {{ type: string }}").unwrap();
    writeln!(&mut template, "#           confidence: {{ type: number }}").unwrap();
    writeln!(&mut template).unwrap();

    // Notifications section
    writeln!(
        &mut template,
        "# (optional) Notification system configuration"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Define default notification channels and settings"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# IMPORTANT: Channels must be defined inline (not as named references)"
    )
    .unwrap();
    writeln!(&mut template, "# notifications:").unwrap();
    writeln!(
        &mut template,
        "#   default_channels:  # Default channels for notifications"
    )
    .unwrap();
    writeln!(&mut template, "#     - type: console").unwrap();
    writeln!(&mut template, "#       colored: true").unwrap();
    writeln!(&mut template, "#       timestamp: true").unwrap();
    writeln!(&mut template, "#     - type: ntfy").unwrap();
    writeln!(&mut template, "#       server: \"https://ntfy.sh\"").unwrap();
    writeln!(&mut template, "#       topic: \"my-workflow\"").unwrap();
    writeln!(&mut template, "#       priority: 3").unwrap();
    writeln!(&mut template, "#   notify_on_start: true").unwrap();
    writeln!(&mut template, "#   notify_on_completion: true").unwrap();
    writeln!(&mut template, "#   notify_on_failure: true").unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "# Notification channels (inline in tasks):").unwrap();
    writeln!(&mut template, "# tasks:").unwrap();
    writeln!(&mut template, "#   example_task:").unwrap();
    writeln!(
        &mut template,
        "#     description: \"Task with notification\""
    )
    .unwrap();
    writeln!(&mut template, "#     agent: \"example_agent\"").unwrap();
    writeln!(&mut template, "#     on_complete:").unwrap();
    writeln!(
        &mut template,
        "#       notify:  # Simple string notification"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#         message: \"Task completed successfully\""
    )
    .unwrap();
    writeln!(
        &mut template,
        "#         channels:  # MUST be inline, not references"
    )
    .unwrap();
    writeln!(&mut template, "#           - type: console").unwrap();
    writeln!(&mut template, "#             colored: true").unwrap();
    writeln!(&mut template, "#             timestamp: true").unwrap();
    writeln!(
        &mut template,
        "#           - type: slack  # Example: Slack notification"
    )
    .unwrap();
    writeln!(
        &mut template,
        "#             credential: \"${{secret.slack_webhook}}\""
    )
    .unwrap();
    writeln!(&mut template, "#             channel: \"#notifications\"").unwrap();
    writeln!(&mut template, "#             method: webhook").unwrap();
    writeln!(
        &mut template,
        "#         # OR simple string: notify: \"Task completed\""
    )
    .unwrap();
    writeln!(&mut template, "#").unwrap();
    writeln!(&mut template, "# Supported notification channels:").unwrap();
    writeln!(
        &mut template,
        "#   - console: Console/stdout with colors and timestamps"
    )
    .unwrap();
    writeln!(&mut template, "#   - email: SMTP email notifications").unwrap();
    writeln!(&mut template, "#   - slack: Slack webhook or bot").unwrap();
    writeln!(&mut template, "#   - discord: Discord webhook").unwrap();
    writeln!(&mut template, "#   - teams: Microsoft Teams webhook").unwrap();
    writeln!(&mut template, "#   - telegram: Telegram bot API").unwrap();
    writeln!(
        &mut template,
        "#   - pagerduty: PagerDuty incident notifications"
    )
    .unwrap();
    writeln!(&mut template, "#   - webhook: Generic HTTP webhook").unwrap();
    writeln!(&mut template, "#   - file: Write to log file").unwrap();
    writeln!(&mut template, "#   - ntfy: ntfy.sh push notifications").unwrap();
    writeln!(&mut template).unwrap();

    // MCP Servers section
    writeln!(
        &mut template,
        "# (optional) MCP (Model Context Protocol) server configuration"
    )
    .unwrap();
    writeln!(
        &mut template,
        "# Connect to external tools and data sources"
    )
    .unwrap();
    writeln!(&mut template, "# mcp_servers:").unwrap();
    writeln!(&mut template, "#   filesystem:").unwrap();
    writeln!(&mut template, "#     type: \"stdio\"").unwrap();
    writeln!(&mut template, "#     command: \"npx\"").unwrap();
    writeln!(&mut template, "#     args:").unwrap();
    writeln!(&mut template, "#       - \"-y\"").unwrap();
    writeln!(
        &mut template,
        "#       - \"@modelcontextprotocol/server-filesystem\""
    )
    .unwrap();
    writeln!(&mut template, "#     env:").unwrap();
    writeln!(&mut template, "#       NODE_ENV: \"production\"").unwrap();
    writeln!(&mut template, "#   api_server:").unwrap();
    writeln!(&mut template, "#     type: \"http\"").unwrap();
    writeln!(&mut template, "#     url: \"https://api.example.com/mcp\"").unwrap();
    writeln!(&mut template, "#     headers:").unwrap();
    writeln!(&mut template, "#       Authorization: \"Bearer token123\"").unwrap();

    template
}

/// Generate a system prompt for natural language to DSL conversion
///
/// This prompt instructs an AI agent on how to convert natural language
/// descriptions into valid DSL workflows. It includes the complete grammar
/// specification and examples.
///
/// # Returns
///
/// A comprehensive system prompt string
pub fn generate_nl_to_dsl_prompt() -> String {
    let mut prompt = String::new();

    writeln!(&mut prompt, "You are an expert at converting natural language descriptions into Agentic AI DSL (Domain-Specific Language) workflows.").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "# DSL Grammar Version: {}",
        DSL_GRAMMAR_VERSION
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "# Your Task").unwrap();
    writeln!(&mut prompt, "Convert natural language descriptions of multi-agent AI workflows into valid YAML-based DSL syntax.").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "# ⚠️ CRITICAL: ALWAYS START WITH REQUIRED FIELDS ⚠️"
    )
    .unwrap();
    writeln!(&mut prompt, "EVERY workflow MUST begin with:").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "name: \"Your Workflow Name\"").unwrap();
    writeln!(&mut prompt, "version: \"1.0.0\"").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(
        &mut prompt,
        "These two fields are MANDATORY and must appear at the top of every workflow."
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "Workflows without `name` and `version` fields will fail validation."
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "# CRITICAL VALIDATION RULES").unwrap();
    writeln!(
        &mut prompt,
        "When using loop specifications, you MUST include ALL required fields:"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `repeat` type REQUIRES `count` field (number of iterations)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `for_each` type REQUIRES `collection` and `iterator` fields"
    )
    .unwrap();
    writeln!(&mut prompt, "- `while` type REQUIRES `condition` field").unwrap();
    writeln!(
        &mut prompt,
        "- `repeat_until` type REQUIRES `condition` field"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "DO NOT use loop types without their required fields or validation will fail."
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "# DSL Structure").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "## Required Fields").unwrap();
    writeln!(&mut prompt, "- `name`: Workflow name (string)").unwrap();
    writeln!(
        &mut prompt,
        "- `version`: Workflow version (semantic version string, e.g., \"1.0.0\")"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "## Optional Root Fields").unwrap();
    writeln!(
        &mut prompt,
        "- `dsl_version`: DSL grammar version (default: \"{}\")\"",
        DSL_GRAMMAR_VERSION
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `cwd`: Default working directory for all agents"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `create_cwd`: Create working directory if missing (boolean)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `secrets`: Map of secret definitions for secure credential management"
    )
    .unwrap();
    writeln!(&mut prompt, "- `inputs`: Map of workflow input variables").unwrap();
    writeln!(&mut prompt, "- `outputs`: Map of workflow output variables (IMPORTANT: source must be object with 'type' field)").unwrap();
    writeln!(&mut prompt, "- `notifications`: Notification defaults (IMPORTANT: channels must be inline, not named references)").unwrap();
    writeln!(
        &mut prompt,
        "- `limits`: Workflow-level limits for output truncation and context management (see Stdio & Context Management)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `dangerously_skip_permissions`: Globally bypass all permissions (boolean, default: false) - WARNING: Use only in trusted environments"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `agents`: Map of agent definitions (see Agent Schema)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `tasks`: Map of task definitions (see Task Schema)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `workflows`: Map of workflow orchestrations (see Workflow Schema)"
    )
    .unwrap();
    writeln!(&mut prompt, "- `tools`: Global tool configuration").unwrap();
    writeln!(
        &mut prompt,
        "- `communication`: Inter-agent communication setup"
    )
    .unwrap();
    writeln!(&mut prompt, "- `mcp_servers`: MCP server configurations").unwrap();
    writeln!(
        &mut prompt,
        "- `subflows`: Map of reusable subflow definitions (can be inline or external)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `imports`: Map of task group imports (namespace -> \"group@version\")"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## Stdio & Context Management").unwrap();
    writeln!(&mut prompt, "Prevent memory exhaustion and improve context quality with bounded output limits and smart context injection.").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Workflow-Level Limits").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "limits:").unwrap();
    writeln!(&mut prompt, "  # Output Truncation").unwrap();
    writeln!(
        &mut prompt,
        "  max_stdout_bytes: 1048576       # 1MB per task (default)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "  max_stderr_bytes: 262144        # 256KB per task (default)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "  truncation_strategy: tail        # head|tail|both|summary (default: tail)"
    )
    .unwrap();
    writeln!(&mut prompt, "  # Context Injection").unwrap();
    writeln!(
        &mut prompt,
        "  max_context_bytes: 102400        # 100KB total context (default)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "  max_context_tasks: 10            # Max tasks to include (default)"
    )
    .unwrap();
    writeln!(&mut prompt, "  # Cleanup Strategy").unwrap();
    writeln!(&mut prompt, "  cleanup_strategy:").unwrap();
    writeln!(&mut prompt, "    type: highest_relevance        # most_recent|highest_relevance|lru|direct_dependencies").unwrap();
    writeln!(
        &mut prompt,
        "    keep_count: 20                 # Number of task outputs to retain"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Truncation Strategies").unwrap();
    writeln!(
        &mut prompt,
        "- `tail`: Keep last N bytes (most recent output) - DEFAULT"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- `head`: Keep first N bytes (startup messages)"
    )
    .unwrap();
    writeln!(&mut prompt, "- `both`: Keep first N/2 and last N/2 bytes").unwrap();
    writeln!(&mut prompt, "- `summary`: AI-generated summary (future)").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Task-Level Context Configuration").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "tasks:").unwrap();
    writeln!(&mut prompt, "  my_task:").unwrap();
    writeln!(&mut prompt, "    description: \"Task with smart context\"").unwrap();
    writeln!(&mut prompt, "    agent: \"worker\"").unwrap();
    writeln!(
        &mut prompt,
        "    inject_context: true           # Enable context injection"
    )
    .unwrap();
    writeln!(&mut prompt, "    context:").unwrap();
    writeln!(
        &mut prompt,
        "      mode: automatic              # automatic|manual|none (default: automatic)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      min_relevance: 0.5           # 0.0-1.0 filter threshold (default: 0.5)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      max_bytes: 100000            # Override workflow limit"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      max_tasks: 5                 # Override workflow limit"
    )
    .unwrap();
    writeln!(&mut prompt, "      # Manual mode only:").unwrap();
    writeln!(
        &mut prompt,
        "      include_tasks: [task1, task2] # Explicit task list"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      exclude_tasks: [noisy_task]   # Tasks to exclude"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Context Modes").unwrap();
    writeln!(
        &mut prompt,
        "- `automatic`: Dependency-based relevance scoring"
    )
    .unwrap();
    writeln!(&mut prompt, "  - Direct dependency: relevance = 1.0").unwrap();
    writeln!(
        &mut prompt,
        "  - Transitive dependency: relevance = 0.8 / depth"
    )
    .unwrap();
    writeln!(&mut prompt, "  - Same agent: relevance = 0.5").unwrap();
    writeln!(&mut prompt, "- `manual`: Explicit include/exclude lists").unwrap();
    writeln!(&mut prompt, "- `none`: Disable context injection").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Task-Level Limit Overrides").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "tasks:").unwrap();
    writeln!(&mut prompt, "  verbose_task:").unwrap();
    writeln!(&mut prompt, "    description: \"Task with custom limits\"").unwrap();
    writeln!(&mut prompt, "    agent: \"worker\"").unwrap();
    writeln!(&mut prompt, "    limits:").unwrap();
    writeln!(
        &mut prompt,
        "      max_stdout_bytes: 10485760   # 10MB override"
    )
    .unwrap();
    writeln!(&mut prompt, "      truncation_strategy: both").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Use Cases").unwrap();
    writeln!(
        &mut prompt,
        "- **Long-running workflows**: Prevent memory exhaustion with cleanup strategies"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- **Data pipelines**: Smart context injection for processing chains"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- **Resource-constrained environments**: Minimal memory footprint"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## Secrets Management").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "secrets:").unwrap();
    writeln!(&mut prompt, "  api_token:").unwrap();
    writeln!(&mut prompt, "    source:").unwrap();
    writeln!(&mut prompt, "      type: env  # env, file, or value").unwrap();
    writeln!(&mut prompt, "      var: \"API_TOKEN\"  # For env source").unwrap();
    writeln!(&mut prompt, "    description: \"API token\"").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(
        &mut prompt,
        "Reference secrets using: ${{secret.api_token}}"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## Variable System").unwrap();
    writeln!(
        &mut prompt,
        "- Define inputs/outputs at workflow, agent, task, or subflow level"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- Reference using ${{scope.variable}} syntax (e.g., ${{workflow.project_name}})"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- Available scopes: workflow, agent, task, subflow, secret"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- Variables work in ALL field types: strings, numbers, integers, booleans"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "Examples:").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "# String variables:").unwrap();
    writeln!(
        &mut prompt,
        "description: \"Deploy ${{workflow.project_name}} to production\""
    )
    .unwrap();
    writeln!(&mut prompt, "path: \"${{workflow.output_dir}}/result.txt\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "# Numeric variables (interpolated at runtime):"
    )
    .unwrap();
    writeln!(&mut prompt, "inputs:").unwrap();
    writeln!(&mut prompt, "  retry_count:").unwrap();
    writeln!(&mut prompt, "    type: integer").unwrap();
    writeln!(&mut prompt, "    default: 3").unwrap();
    writeln!(&mut prompt, "  iterations:").unwrap();
    writeln!(&mut prompt, "    type: integer").unwrap();
    writeln!(&mut prompt, "    default: 10").unwrap();
    writeln!(&mut prompt, "  timeout:").unwrap();
    writeln!(&mut prompt, "    type: integer").unwrap();
    writeln!(&mut prompt, "    default: 30").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "tasks:").unwrap();
    writeln!(&mut prompt, "  example_retry:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Task with configurable retry\""
    )
    .unwrap();
    writeln!(&mut prompt, "    agent: \"worker\"").unwrap();
    writeln!(&mut prompt, "    on_error:").unwrap();
    writeln!(
        &mut prompt,
        "      retry: ${{workflow.retry_count}}  # Numeric variable"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      retry_delay_secs: ${{workflow.timeout}}  # Numeric variable"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "  example_loop:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Task with configurable loop count\""
    )
    .unwrap();
    writeln!(&mut prompt, "    agent: \"worker\"").unwrap();
    writeln!(&mut prompt, "    loop:").unwrap();
    writeln!(&mut prompt, "      type: repeat").unwrap();
    writeln!(
        &mut prompt,
        "      count: ${{workflow.iterations}}  # Numeric variable for loop count"
    )
    .unwrap();
    writeln!(&mut prompt, "      iterator: \"i\"").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(
        &mut prompt,
        "IMPORTANT: Numeric/boolean variables must reference inputs with matching types."
    )
    .unwrap();
    writeln!(&mut prompt, "Variables can be used in: on_error.retry, on_error.retry_delay_secs, loop.count, and other numeric fields.").unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## Input Schema (CRITICAL)").unwrap();
    writeln!(
        &mut prompt,
        "Input variables MUST include a 'type' field. Structure:"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "inputs:").unwrap();
    writeln!(&mut prompt, "  input_name:").unwrap();
    writeln!(
        &mut prompt,
        "    type: string  # REQUIRED: string, number, integer, boolean, object, array"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    required: true  # Optional: whether input must be provided (default: false)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    default: \"value\"  # Optional: default value if not provided"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Input description\"  # Optional: what this input is for"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "Examples:").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "inputs:").unwrap();
    writeln!(&mut prompt, "  project_name:").unwrap();
    writeln!(&mut prompt, "    type: string").unwrap();
    writeln!(&mut prompt, "    required: true").unwrap();
    writeln!(&mut prompt, "    default: \"my-project\"").unwrap();
    writeln!(&mut prompt, "    description: \"Name of the project\"").unwrap();
    writeln!(&mut prompt, "  port:").unwrap();
    writeln!(&mut prompt, "    type: integer").unwrap();
    writeln!(&mut prompt, "    default: 8080").unwrap();
    writeln!(&mut prompt, "  enable_debug:").unwrap();
    writeln!(&mut prompt, "    type: boolean").unwrap();
    writeln!(&mut prompt, "    default: false").unwrap();
    writeln!(&mut prompt, "  config:").unwrap();
    writeln!(&mut prompt, "    type: object").unwrap();
    writeln!(&mut prompt, "    required: false").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: 'type' field is REQUIRED for all inputs. Do NOT omit it."
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## Output Schema (IMPORTANT)").unwrap();
    writeln!(
        &mut prompt,
        "Output variables MUST use this exact structure:"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "outputs:").unwrap();
    writeln!(&mut prompt, "  output_name:").unwrap();
    writeln!(&mut prompt, "    source:").unwrap();
    writeln!(
        &mut prompt,
        "      type: file              # REQUIRED: file, state, or task_output"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      path: \"./result.txt\"    # For type: file"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      # key: \"result\"         # For type: state"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      # task: \"task_id\"      # For type: task_output"
    )
    .unwrap();
    writeln!(&mut prompt, "    description: \"Description\"  # Optional").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: 'source' is an OBJECT with a 'type' field, NOT a flat string."
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "## Agent Schema").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "agents:").unwrap();
    writeln!(&mut prompt, "  agent_name:  # Unique identifier").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"What this agent does\" # REQUIRED"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    model: \"claude-sonnet-4-5\"  # optional, defaults to claude-sonnet-4-5"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    system_prompt: \"Custom instructions\"  # optional"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    cwd: \"/workspace\"  # optional, overrides workflow cwd"
    )
    .unwrap();
    writeln!(&mut prompt, "    create_cwd: true  # optional").unwrap();
    writeln!(&mut prompt, "    tools:  # optional list").unwrap();
    writeln!(&mut prompt, "      - Read").unwrap();
    writeln!(&mut prompt, "      - Write").unwrap();
    writeln!(&mut prompt, "      - Bash").unwrap();
    writeln!(&mut prompt, "      - WebSearch").unwrap();
    writeln!(&mut prompt, "    permissions:  # optional").unwrap();
    writeln!(
        &mut prompt,
        "      mode: \"default\"  # default, acceptEdits, plan, bypassPermissions"
    )
    .unwrap();
    writeln!(&mut prompt, "      allowed_directories:  # optional list").unwrap();
    writeln!(&mut prompt, "        - \"/allowed/path\"").unwrap();
    writeln!(&mut prompt, "    max_turns: 10  # optional").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "## Task Schema").unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: Tasks MUST use EXACTLY ONE execution type. Execution types are MUTUALLY EXCLUSIVE."
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: Tasks must specify EXACTLY ONE execution type."
    )
    .unwrap();
    writeln!(&mut prompt, "CRITICAL: Valid execution types: agent, subflow, uses, embed, script, command, http, mcp_tool").unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: DO NOT combine multiple types (e.g., agent + script, script + command, etc.)"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "Tasks can be executed by AI agents OR as deterministic operations:"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "tasks:").unwrap();
    writeln!(&mut prompt, "  task_name:  # Unique identifier").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"What to accomplish\" # REQUIRED"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # ===== CHOOSE EXACTLY ONE EXECUTION TYPE ====="
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # Option 1: AI agent execution (mutually exclusive with options 2-8)"
    )
    .unwrap();
    writeln!(&mut prompt, "    agent: \"agent_name\"").unwrap();
    writeln!(&mut prompt, "    # Option 2: Predefined task reference").unwrap();
    writeln!(
        &mut prompt,
        "    # uses: \"task-name@version\"  # e.g., \"google-drive-upload@1.2.0\""
    )
    .unwrap();
    writeln!(&mut prompt, "    # Option 3: Embed predefined task").unwrap();
    writeln!(
        &mut prompt,
        "    # embed: \"task-name@version\"  # Copy definition with optional overrides"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # overrides:  # Optional overrides for embedded tasks"
    )
    .unwrap();
    writeln!(&mut prompt, "    #   description: \"Custom description\"").unwrap();
    writeln!(&mut prompt, "    # Option 4: Reference prebuilt workflow").unwrap();
    writeln!(
        &mut prompt,
        "    # uses_workflow: \"namespace:workflow_name\"  # e.g., \"google:upload-files\""
    )
    .unwrap();
    writeln!(&mut prompt, "    # Option 5: Script execution").unwrap();
    writeln!(&mut prompt, "    # Variable interpolation in scripts:").unwrap();
    writeln!(
        &mut prompt,
        "    #   - ${{workflow.var}} and ${{task.var}} are substituted"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    #   - Use \\${{...}} to escape (becomes literal ${{...}})"
    )
    .unwrap();
    writeln!(&mut prompt, "    # script:").unwrap();
    writeln!(
        &mut prompt,
        "#       language: python  # python, javascript, bash, ruby, perl"
    )
    .unwrap();
    writeln!(&mut prompt, "#       content: |  # Inline script").unwrap();
    writeln!(&mut prompt, "#         import os").unwrap();
    writeln!(
        &mut prompt,
        "#         project = \"${{workflow.project_name}}\"  # DSL var"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "#         path = os.environ.get('PATH', \\${{PATH}})  # Escaped"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "#       # OR file: \"scripts/process.py\"  # External script"
    )
    .unwrap();
    writeln!(&mut prompt, "#       env:").unwrap();
    writeln!(&mut prompt, "#         VAR: \"${{workflow.value}}\"").unwrap();
    writeln!(&mut prompt, "#       timeout_secs: 60").unwrap();
    writeln!(&mut prompt, "    # Option 6: Command execution").unwrap();
    writeln!(&mut prompt, "    # command:").unwrap();
    writeln!(&mut prompt, "#       executable: \"pandoc\"").unwrap();
    writeln!(
        &mut prompt,
        "#       args: [\"--from=markdown\", \"input.md\"]"
    )
    .unwrap();
    writeln!(&mut prompt, "#       env:").unwrap();
    writeln!(&mut prompt, "#         PATH: \"/usr/local/bin\"").unwrap();
    writeln!(&mut prompt, "#       timeout_secs: 60").unwrap();
    writeln!(&mut prompt, "#       capture_stdout: true").unwrap();
    writeln!(&mut prompt, "    # Option 7: HTTP request").unwrap();
    writeln!(&mut prompt, "    # http:").unwrap();
    writeln!(
        &mut prompt,
        "#       method: GET  # GET, POST, PUT, DELETE, etc."
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "#       url: \"${{workflow.api_endpoint}}/data\""
    )
    .unwrap();
    writeln!(&mut prompt, "#       headers:").unwrap();
    writeln!(&mut prompt, "#         Content-Type: \"application/json\"").unwrap();
    writeln!(&mut prompt, "#       auth:").unwrap();
    writeln!(
        &mut prompt,
        "#         type: bearer  # bearer, basic, api_key, custom"
    )
    .unwrap();
    writeln!(&mut prompt, "#         token: \"${{secret.api_token}}\"").unwrap();
    writeln!(&mut prompt, "#       timeout_secs: 30").unwrap();
    writeln!(&mut prompt, "    # Option 8: MCP tool invocation").unwrap();
    writeln!(&mut prompt, "    # mcp_tool:").unwrap();
    writeln!(&mut prompt, "#       server: \"data_processor\"").unwrap();
    writeln!(&mut prompt, "#       tool: \"transform_data\"").unwrap();
    writeln!(&mut prompt, "#       parameters:").unwrap();
    writeln!(&mut prompt, "#         input: \"${{workflow.data_file}}\"").unwrap();
    writeln!(&mut prompt, "#       timeout_secs: 60").unwrap();
    writeln!(&mut prompt, "    # Common task fields (all types):").unwrap();
    writeln!(
        &mut prompt,
        "    priority: 1  # optional, default 0 (lower = higher priority)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    depends_on:  # optional list of task names"
    )
    .unwrap();
    writeln!(&mut prompt, "      - other_task").unwrap();
    writeln!(
        &mut prompt,
        "    parallel_with:  # optional list of task names"
    )
    .unwrap();
    writeln!(&mut prompt, "      - concurrent_task").unwrap();
    writeln!(
        &mut prompt,
        "    output: \"result.txt\"  # optional output file"
    )
    .unwrap();
    writeln!(&mut prompt, "    # Hierarchical task decomposition:").unwrap();
    writeln!(
        &mut prompt,
        "    # IMPORTANT: Parent tasks with subtasks DO NOT EXECUTE"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # - Parent tasks serve only as grouping/template mechanisms"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # - Parent tasks DON'T need an 'agent' if they have subtasks"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # - Subtasks inherit parent attributes (agent, priority, on_error, etc.)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # - Subtasks can override any inherited attribute"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # - If subtask has ANY execution type, it won't inherit parent's agent"
    )
    .unwrap();
    writeln!(&mut prompt, "    subtasks:  # optional nested tasks").unwrap();
    writeln!(&mut prompt, "      - subtask_1:").unwrap();
    writeln!(
        &mut prompt,
        "          description: \"Subtask description\""
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "          # No execution type - inherits parent's agent/priority/etc"
    )
    .unwrap();
    writeln!(&mut prompt, "      - subtask_2:").unwrap();
    writeln!(&mut prompt, "          description: \"Another subtask\"").unwrap();
    writeln!(
        &mut prompt,
        "          agent: \"different_agent\"  # Override parent's agent"
    )
    .unwrap();
    writeln!(&mut prompt, "      - subtask_3:").unwrap();
    writeln!(
        &mut prompt,
        "          description: \"Script-based subtask\""
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "          script:  # Different execution type - won't inherit parent's agent"
    )
    .unwrap();
    writeln!(&mut prompt, "            language: bash").unwrap();
    writeln!(&mut prompt, "            content: \"echo 'test'\"").unwrap();
    writeln!(&mut prompt, "    on_complete:  # optional").unwrap();
    writeln!(&mut prompt, "      notify: \"Success message\"").unwrap();
    writeln!(&mut prompt, "    on_error:  # optional").unwrap();
    writeln!(&mut prompt, "      retry: 3").unwrap();
    writeln!(&mut prompt, "      retry_delay_secs: 1").unwrap();
    writeln!(&mut prompt, "      exponential_backoff: true").unwrap();
    writeln!(&mut prompt, "      fallback_agent: \"backup_agent\"").unwrap();
    writeln!(
        &mut prompt,
        "    condition:  # optional - conditional execution"
    )
    .unwrap();
    writeln!(&mut prompt, "      # Single condition (choose type: state_equals, state_exists, task_status, always, never):").unwrap();
    writeln!(&mut prompt, "      type: state_equals").unwrap();
    writeln!(&mut prompt, "      key: \"environment\"").unwrap();
    writeln!(&mut prompt, "      value: \"production\"").unwrap();
    writeln!(&mut prompt, "      # OR use combined conditions:").unwrap();
    writeln!(
        &mut prompt,
        "      # and: [condition1, condition2]  # All must be true"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      # or: [condition1, condition2]   # Any can be true"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      # not: condition                 # Negate condition"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    definition_of_done:  # optional - quality criteria"
    )
    .unwrap();
    writeln!(&mut prompt, "      criteria:").unwrap();
    writeln!(&mut prompt, "        - type: file_exists").unwrap();
    writeln!(&mut prompt, "          path: \"output.md\"").unwrap();
    writeln!(&mut prompt, "          description: \"Output file exists\"").unwrap();
    writeln!(&mut prompt, "        - type: tests_passed").unwrap();
    writeln!(&mut prompt, "          command: \"cargo\"").unwrap();
    writeln!(&mut prompt, "          args: [\"test\"]").unwrap();
    writeln!(&mut prompt, "          description: \"All tests pass\"").unwrap();
    writeln!(&mut prompt, "      # Types: file_exists, file_contains, file_not_contains, command_succeeds, output_matches, directory_exists, tests_passed").unwrap();
    writeln!(&mut prompt, "      max_retries: 2").unwrap();
    writeln!(&mut prompt, "      fail_on_unmet: true").unwrap();
    writeln!(&mut prompt, "    loop:  # optional - iteration").unwrap();
    writeln!(
        &mut prompt,
        "      # LOOP TYPE: for_each (REQUIRES collection + iterator)"
    )
    .unwrap();
    writeln!(&mut prompt, "      type: for_each").unwrap();
    writeln!(&mut prompt, "      collection:").unwrap();
    writeln!(
        &mut prompt,
        "        source: inline  # inline, state, file, range, http"
    )
    .unwrap();
    writeln!(&mut prompt, "        items: [1, 2, 3]").unwrap();
    writeln!(&mut prompt, "      iterator: \"item\"").unwrap();
    writeln!(&mut prompt, "      parallel: false").unwrap();
    writeln!(&mut prompt, "      max_parallel: 5").unwrap();
    writeln!(&mut prompt, "      # LOOP TYPE: repeat (REQUIRES count)").unwrap();
    writeln!(&mut prompt, "      # type: repeat").unwrap();
    writeln!(
        &mut prompt,
        "      # count: 10  # REQUIRED - can be number or variable like ${{workflow.iterations}}"
    )
    .unwrap();
    writeln!(&mut prompt, "      # iterator: \"i\"  # optional").unwrap();
    writeln!(&mut prompt, "      # LOOP TYPE: while (REQUIRES condition)").unwrap();
    writeln!(&mut prompt, "      # type: while").unwrap();
    writeln!(
        &mut prompt,
        "      # condition: {{ type: state_equals, key: \"done\", value: false }}"
    )
    .unwrap();
    writeln!(&mut prompt, "      # max_iterations: 100  # optional").unwrap();
    writeln!(
        &mut prompt,
        "      # LOOP TYPE: repeat_until (REQUIRES condition)"
    )
    .unwrap();
    writeln!(&mut prompt, "      # type: repeat_until").unwrap();
    writeln!(
        &mut prompt,
        "      # condition: {{ type: state_equals, key: \"done\", value: true }}"
    )
    .unwrap();
    writeln!(&mut prompt, "    loop_control:  # optional - loop control").unwrap();
    writeln!(&mut prompt, "      break_condition:").unwrap();
    writeln!(&mut prompt, "        type: state_equals").unwrap();
    writeln!(&mut prompt, "        key: \"error_found\"").unwrap();
    writeln!(&mut prompt, "        value: true").unwrap();
    writeln!(&mut prompt, "      timeout_secs: 300").unwrap();
    writeln!(&mut prompt, "      checkpoint_interval: 50").unwrap();
    writeln!(&mut prompt, "      collect_results: true").unwrap();
    writeln!(&mut prompt, "      result_key: \"results\"").unwrap();
    writeln!(
        &mut prompt,
        "    inputs:  # optional - task configuration (auto-injected into agent prompts)"
    )
    .unwrap();
    writeln!(&mut prompt, "      config_file: \"config.yaml\"").unwrap();
    writeln!(&mut prompt, "      verbose: true").unwrap();
    writeln!(
        &mut prompt,
        "    outputs:  # optional - define where task results are stored (auto-injected)"
    )
    .unwrap();
    writeln!(&mut prompt, "      result:").unwrap();
    writeln!(&mut prompt, "        source:").unwrap();
    writeln!(&mut prompt, "          type: file").unwrap();
    writeln!(&mut prompt, "          path: \"./output/result.json\"").unwrap();
    writeln!(
        &mut prompt,
        "        description: \"Task execution result\""
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    inject_context: true  # optional - inject workflow execution context into agent"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # When true, agent receives context about completed tasks and their results"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    context:  # optional - configure context injection (see Stdio & Context Management)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      mode: automatic  # automatic|manual|none"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      min_relevance: 0.5  # Filter threshold (0.0-1.0)"
    )
    .unwrap();
    writeln!(&mut prompt, "      max_bytes: 100000  # Context size limit").unwrap();
    writeln!(
        &mut prompt,
        "      max_tasks: 5  # Max number of tasks to include"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    limits:  # optional - task-level output limits (see Stdio & Context Management)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      max_stdout_bytes: 1048576  # Override workflow limit"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "      truncation_strategy: tail  # head|tail|both|summary"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    dangerously_skip_permissions: false  # optional - bypass all permissions for this task (WARNING: dangerous)"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(
        &mut prompt,
        "## Workflow Schema (CRITICAL - READ CAREFULLY)"
    )
    .unwrap();
    writeln!(&mut prompt, "⚠️ CRITICAL: DO NOT DUPLICATE TASKS ⚠️").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "Tasks MUST be defined in the top-level 'tasks:' section ONLY."
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "The 'workflows:' section is OPTIONAL and used ONLY for lifecycle hooks."
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "WRONG - Duplicates task definitions (DO NOT DO THIS):"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "name: \"Bad Example\"").unwrap();
    writeln!(&mut prompt, "version: \"1.0.0\"").unwrap();
    writeln!(&mut prompt, "tasks:  # Tasks defined here").unwrap();
    writeln!(&mut prompt, "  process_data:").unwrap();
    writeln!(&mut prompt, "    description: \"Process data\"").unwrap();
    writeln!(&mut prompt, "    agent: \"processor\"").unwrap();
    writeln!(&mut prompt, "workflows:  # DON'T DUPLICATE TASKS HERE").unwrap();
    writeln!(&mut prompt, "  main:").unwrap();
    writeln!(&mut prompt, "    steps:").unwrap();
    writeln!(&mut prompt, "      - stage: \"processing\"").unwrap();
    writeln!(
        &mut prompt,
        "        tasks:  # WRONG - This duplicates the tasks above!"
    )
    .unwrap();
    writeln!(&mut prompt, "          - process_data:").unwrap();
    writeln!(
        &mut prompt,
        "              description: \"Process data\"  # DUPLICATE - Don't do this!"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "CORRECT - Tasks defined once, workflows section only for hooks (RECOMMENDED):"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "name: \"Good Example\"").unwrap();
    writeln!(&mut prompt, "version: \"1.0.0\"").unwrap();
    writeln!(&mut prompt, "agents:").unwrap();
    writeln!(&mut prompt, "  processor:").unwrap();
    writeln!(&mut prompt, "    description: \"Data processor\"").unwrap();
    writeln!(&mut prompt, "    tools: [Read, Write]").unwrap();
    writeln!(&mut prompt, "tasks:  # Define ALL tasks here ONLY").unwrap();
    writeln!(&mut prompt, "  fetch_data:").unwrap();
    writeln!(&mut prompt, "    description: \"Fetch raw data\"").unwrap();
    writeln!(&mut prompt, "    agent: \"processor\"").unwrap();
    writeln!(&mut prompt, "  process_data:").unwrap();
    writeln!(&mut prompt, "    description: \"Process data\"").unwrap();
    writeln!(&mut prompt, "    agent: \"processor\"").unwrap();
    writeln!(
        &mut prompt,
        "    depends_on: [fetch_data]  # Use depends_on for ordering"
    )
    .unwrap();
    writeln!(&mut prompt, "  analyze_results:").unwrap();
    writeln!(&mut prompt, "    description: \"Analyze processed data\"").unwrap();
    writeln!(&mut prompt, "    agent: \"processor\"").unwrap();
    writeln!(&mut prompt, "    depends_on: [process_data]").unwrap();
    writeln!(
        &mut prompt,
        "# workflows: section is OPTIONAL - only use for hooks"
    )
    .unwrap();
    writeln!(&mut prompt, "workflows:").unwrap();
    writeln!(&mut prompt, "  main:").unwrap();
    writeln!(&mut prompt, "    description: \"Main workflow\"").unwrap();
    writeln!(
        &mut prompt,
        "    hooks:  # Only use workflows for lifecycle hooks"
    )
    .unwrap();
    writeln!(&mut prompt, "      pre_workflow:").unwrap();
    writeln!(&mut prompt, "        - \"echo 'Starting workflow'\"").unwrap();
    writeln!(&mut prompt, "      post_workflow:").unwrap();
    writeln!(&mut prompt, "        - \"echo 'Workflow complete'\"").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "REMEMBER:").unwrap();
    writeln!(
        &mut prompt,
        "- ✓ Define tasks ONCE in top-level 'tasks:' section"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- ✓ Use 'depends_on' in tasks for execution order"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- ✓ Use 'workflows:' ONLY for lifecycle hooks (pre_workflow, post_workflow, on_error)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "- ✗ DO NOT define tasks in both 'tasks:' and 'workflows.steps.tasks'"
    )
    .unwrap();
    writeln!(&mut prompt, "- ✗ DO NOT duplicate task definitions").unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## CRITICAL: Top-Level Tasks Format").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "Top-level 'tasks' field is a MAP (object), NOT an array."
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "WRONG (will cause 'expected a map' error):").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "tasks:  # Top-level in DSLWorkflow").unwrap();
    writeln!(
        &mut prompt,
        "  - task_1:  # WRONG - this is an array (note the dash)"
    )
    .unwrap();
    writeln!(&mut prompt, "      description: \"Do something\"").unwrap();
    writeln!(&mut prompt, "  - task_2:  # WRONG - array format").unwrap();
    writeln!(&mut prompt, "      description: \"Do another thing\"").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "CORRECT (map/object format):").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "tasks:  # Top-level in DSLWorkflow").unwrap();
    writeln!(
        &mut prompt,
        "  task_1:  # Task ID as direct key (no dash before task name)"
    )
    .unwrap();
    writeln!(&mut prompt, "    description: \"Do something\"").unwrap();
    writeln!(&mut prompt, "    agent: \"agent_id\"").unwrap();
    writeln!(
        &mut prompt,
        "  task_2:  # Another task in the map (no dash before task name)"
    )
    .unwrap();
    writeln!(&mut prompt, "    description: \"Do another thing\"").unwrap();
    writeln!(&mut prompt, "    agent: \"agent_id\"").unwrap();
    writeln!(
        &mut prompt,
        "    depends_on: [task_1]  # Execute after task_1"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## Condition Schema (IMPORTANT)").unwrap();
    writeln!(&mut prompt, "Conditions control when tasks execute. ConditionSpec is an untagged enum with THREE formats:").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### 1. Single Condition").unwrap();
    writeln!(
        &mut prompt,
        "A condition object with a 'type' field and type-specific fields:"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "condition:").unwrap();
    writeln!(&mut prompt, "  type: state_equals  # REQUIRED: state_equals, state_exists, task_status, always, or never").unwrap();
    writeln!(
        &mut prompt,
        "  key: \"environment\"  # For state_equals/state_exists"
    )
    .unwrap();
    writeln!(&mut prompt, "  value: \"production\" # For state_equals").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "condition:").unwrap();
    writeln!(&mut prompt, "  type: task_status").unwrap();
    writeln!(&mut prompt, "  task: \"setup\"       # Task ID to check").unwrap();
    writeln!(
        &mut prompt,
        "  status: completed    # completed, failed, running, pending, skipped"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "condition:").unwrap();
    writeln!(
        &mut prompt,
        "  type: always         # Always execute (no other fields)"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "condition:").unwrap();
    writeln!(
        &mut prompt,
        "  type: never          # Never execute (no other fields)"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### 2. Combined Conditions (AND)").unwrap();
    writeln!(
        &mut prompt,
        "All conditions must be true. Use 'and' field with array of ConditionSpecs:"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "condition:").unwrap();
    writeln!(&mut prompt, "  and:").unwrap();
    writeln!(&mut prompt, "    - type: state_equals").unwrap();
    writeln!(&mut prompt, "      key: \"env\"").unwrap();
    writeln!(&mut prompt, "      value: \"prod\"").unwrap();
    writeln!(&mut prompt, "    - type: task_status").unwrap();
    writeln!(&mut prompt, "      task: \"validation\"").unwrap();
    writeln!(&mut prompt, "      status: completed").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### 3. Combined Conditions (OR)").unwrap();
    writeln!(
        &mut prompt,
        "Any condition can be true. Use 'or' field with array of ConditionSpecs:"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "condition:").unwrap();
    writeln!(&mut prompt, "  or:").unwrap();
    writeln!(&mut prompt, "    - type: state_exists").unwrap();
    writeln!(&mut prompt, "      key: \"override\"").unwrap();
    writeln!(&mut prompt, "    - type: always").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### 4. Negated Condition (NOT)").unwrap();
    writeln!(
        &mut prompt,
        "Invert a condition. Use 'not' field with a single ConditionSpec:"
    )
    .unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "condition:").unwrap();
    writeln!(&mut prompt, "  not:").unwrap();
    writeln!(&mut prompt, "    type: task_status").unwrap();
    writeln!(&mut prompt, "    task: \"build\"").unwrap();
    writeln!(&mut prompt, "    status: failed").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: Do NOT use non-existent types like 'state_contains', 'all_of', or 'any_of'."
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: Use 'and', 'or', 'not' fields for combining conditions, NOT as type values."
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(
        &mut prompt,
        "## Notification Schema (CRITICAL - UNTAGGED ENUM)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "NotificationSpec is an UNTAGGED ENUM with TWO formats. Choose ONE:"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### JSON Schema (Precise Definition)").unwrap();
    writeln!(
        &mut prompt,
        "The following JSON Schema defines the EXACT structure required:"
    )
    .unwrap();
    writeln!(&mut prompt, "```json").unwrap();
    writeln!(&mut prompt, "{}", generate_notification_json_schema()).unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Format 1: Simple String").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "on_complete:").unwrap();
    writeln!(
        &mut prompt,
        "  notify: \"Task completed successfully\"  # Just a string"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Format 2: Structured Object").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "on_complete:").unwrap();
    writeln!(&mut prompt, "  notify:  # Object with 'message' field").unwrap();
    writeln!(
        &mut prompt,
        "    message: \"Task completed\"  # REQUIRED for structured format"
    )
    .unwrap();
    writeln!(&mut prompt, "    title: \"Success\"  # Optional").unwrap();
    writeln!(
        &mut prompt,
        "    priority: high  # Optional: low, normal, high, critical (NOT urgent)"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    channels:  # Optional - inline channel configs only"
    )
    .unwrap();
    writeln!(&mut prompt, "      - type: console").unwrap();
    writeln!(&mut prompt, "        colored: true").unwrap();
    writeln!(&mut prompt, "        timestamp: true").unwrap();
    writeln!(&mut prompt, "      - type: slack").unwrap();
    writeln!(
        &mut prompt,
        "        credential: \"${{secret.slack_webhook}}\""
    )
    .unwrap();
    writeln!(&mut prompt, "        channel: \"#notifications\"").unwrap();
    writeln!(
        &mut prompt,
        "      - type: email  # Email channel with nested smtp config"
    )
    .unwrap();
    writeln!(&mut prompt, "        to:").unwrap();
    writeln!(&mut prompt, "          - \"user@example.com\"").unwrap();
    writeln!(&mut prompt, "        subject: \"Notification Subject\"").unwrap();
    writeln!(
        &mut prompt,
        "        smtp:  # CRITICAL: SMTP fields must be nested under 'smtp:'"
    )
    .unwrap();
    writeln!(&mut prompt, "          host: \"smtp.gmail.com\"").unwrap();
    writeln!(&mut prompt, "          port: 587").unwrap();
    writeln!(&mut prompt, "          username: \"${{secret.smtp_user}}\"").unwrap();
    writeln!(&mut prompt, "          password: \"${{secret.smtp_pass}}\"").unwrap();
    writeln!(&mut prompt, "          from: \"bot@example.com\"").unwrap();
    writeln!(&mut prompt, "          use_tls: true").unwrap();
    writeln!(&mut prompt, "    metadata:  # Optional key-value pairs").unwrap();
    writeln!(&mut prompt, "      key: \"value\"").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "### Workflow-level Notification Settings").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "notifications:").unwrap();
    writeln!(
        &mut prompt,
        "  default_channels:  # Inline channel configs (NOT references)"
    )
    .unwrap();
    writeln!(&mut prompt, "    - type: console").unwrap();
    writeln!(&mut prompt, "      colored: true").unwrap();
    writeln!(&mut prompt, "      timestamp: true").unwrap();
    writeln!(&mut prompt, "  notify_on_start: true").unwrap();
    writeln!(&mut prompt, "  notify_on_completion: true").unwrap();
    writeln!(&mut prompt, "  notify_on_failure: true").unwrap();
    writeln!(&mut prompt, "  notify_on_workflow_completion: true").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: You CANNOT mix formats. Either use a string OR an object with 'message'."
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "CRITICAL: Channels must be INLINE definitions, NOT named references."
    )
    .unwrap();
    writeln!(&mut prompt, "CRITICAL: Priority must be one of: low, normal, high, critical. Do NOT use 'urgent' - use 'critical' instead.").unwrap();
    writeln!(&mut prompt, "CRITICAL: Email channels REQUIRE nested 'smtp:' object. Do NOT put host/port/username/password directly in channel.").unwrap();
    writeln!(&mut prompt, "Available channel types: console, email, slack, discord, teams, telegram, pagerduty, webhook, file, ntfy").unwrap();
    writeln!(&mut prompt).unwrap();

    writeln!(&mut prompt, "## Available Tools").unwrap();
    writeln!(&mut prompt, "- Read: Read files").unwrap();
    writeln!(&mut prompt, "- Write: Write files").unwrap();
    writeln!(&mut prompt, "- Edit: Edit files").unwrap();
    writeln!(&mut prompt, "- Bash: Execute shell commands").unwrap();
    writeln!(&mut prompt, "- WebSearch: Search the web").unwrap();
    writeln!(&mut prompt, "- WebFetch: Fetch web pages").unwrap();
    writeln!(&mut prompt, "- Glob: Find files by pattern").unwrap();
    writeln!(&mut prompt, "- Grep: Search file contents").unwrap();
    writeln!(&mut prompt, "- Task: Spawn sub-agents").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "## Permission Modes").unwrap();
    writeln!(
        &mut prompt,
        "- `default`: Ask for permission on dangerous operations"
    )
    .unwrap();
    writeln!(&mut prompt, "- `acceptEdits`: Auto-approve file edits").unwrap();
    writeln!(&mut prompt, "- `plan`: Planning mode without execution").unwrap();
    writeln!(
        &mut prompt,
        "- `bypassPermissions`: Skip all permission checks (dangerous)"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "# Instructions").unwrap();
    writeln!(
        &mut prompt,
        "1. Analyze the natural language description carefully"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "2. Identify the key components: agents, tasks, dependencies, tools needed"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "3. Generate valid YAML following the schema above"
    )
    .unwrap();
    writeln!(&mut prompt, "4. Use meaningful names for agents and tasks").unwrap();
    writeln!(
        &mut prompt,
        "5. Set appropriate permissions based on task requirements"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "6. Add dependencies between tasks when order matters"
    )
    .unwrap();
    writeln!(&mut prompt, "7. Include error handling for critical tasks").unwrap();
    writeln!(&mut prompt, "8. Output ONLY valid YAML, no additional text").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "# Example Conversion").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "Input: \"Create a workflow to research a topic and write a report\""
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "Output:").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "name: \"Research and Report\"").unwrap();
    writeln!(&mut prompt, "version: \"1.0.0\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "agents:").unwrap();
    writeln!(&mut prompt, "  researcher:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Research information on the topic\""
    )
    .unwrap();
    writeln!(&mut prompt, "    tools:").unwrap();
    writeln!(&mut prompt, "      - WebSearch").unwrap();
    writeln!(&mut prompt, "      - WebFetch").unwrap();
    writeln!(&mut prompt, "      - Read").unwrap();
    writeln!(&mut prompt, "    permissions:").unwrap();
    writeln!(&mut prompt, "      mode: \"default\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "  writer:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Write and format the report\""
    )
    .unwrap();
    writeln!(&mut prompt, "    tools:").unwrap();
    writeln!(&mut prompt, "      - Read").unwrap();
    writeln!(&mut prompt, "      - Write").unwrap();
    writeln!(&mut prompt, "    permissions:").unwrap();
    writeln!(&mut prompt, "      mode: \"acceptEdits\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "tasks:").unwrap();
    writeln!(&mut prompt, "  research:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Research the topic and gather information\""
    )
    .unwrap();
    writeln!(&mut prompt, "    agent: \"researcher\"").unwrap();
    writeln!(&mut prompt, "    output: \"research_findings.md\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "  write_report:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Write a comprehensive report based on research\""
    )
    .unwrap();
    writeln!(&mut prompt, "    agent: \"writer\"").unwrap();
    writeln!(&mut prompt, "    depends_on:").unwrap();
    writeln!(&mut prompt, "      - research").unwrap();
    writeln!(&mut prompt, "    output: \"final_report.md\"").unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "# Example with Hierarchical Tasks").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "Input: \"Create a workflow to organize files in stages: scan, validate, then process\""
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "Output:").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "name: \"File Organization\"").unwrap();
    writeln!(&mut prompt, "version: \"1.0.0\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "agents:").unwrap();
    writeln!(&mut prompt, "  file_worker:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Process and organize files\""
    )
    .unwrap();
    writeln!(&mut prompt, "    tools:").unwrap();
    writeln!(&mut prompt, "      - Read").unwrap();
    writeln!(&mut prompt, "      - Write").unwrap();
    writeln!(&mut prompt, "      - Bash").unwrap();
    writeln!(&mut prompt, "      - Glob").unwrap();
    writeln!(&mut prompt, "    permissions:").unwrap();
    writeln!(&mut prompt, "      mode: \"acceptEdits\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "tasks:").unwrap();
    writeln!(
        &mut prompt,
        "  # Parent task for grouping - NO agent field needed!"
    )
    .unwrap();
    writeln!(&mut prompt, "  file_processing:").unwrap();
    writeln!(
        &mut prompt,
        "    description: \"Process files in organized stages\""
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # Note: No 'agent' specified here - parent task won't execute"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    # Subtasks without execution types will inherit:"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "    agent: \"file_worker\"  # Inherited by subtasks without execution type"
    )
    .unwrap();
    writeln!(&mut prompt, "    on_error:").unwrap();
    writeln!(&mut prompt, "      retry: 2").unwrap();
    writeln!(&mut prompt, "    subtasks:").unwrap();
    writeln!(&mut prompt, "      - scan:").unwrap();
    writeln!(
        &mut prompt,
        "          description: \"Scan and catalog all files\""
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "          # No execution type - inherits agent and on_error from parent"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "      - validate:").unwrap();
    writeln!(
        &mut prompt,
        "          description: \"Validate file formats using script\""
    )
    .unwrap();
    writeln!(&mut prompt, "          depends_on:").unwrap();
    writeln!(&mut prompt, "            - scan  # Sibling dependency").unwrap();
    writeln!(
        &mut prompt,
        "          # Uses script - won't inherit parent's agent"
    )
    .unwrap();
    writeln!(&mut prompt, "          script:").unwrap();
    writeln!(&mut prompt, "            language: bash").unwrap();
    writeln!(&mut prompt, "            content: \"file *.txt\"").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "      - process:").unwrap();
    writeln!(
        &mut prompt,
        "          description: \"Process validated files\""
    )
    .unwrap();
    writeln!(&mut prompt, "          depends_on:").unwrap();
    writeln!(&mut prompt, "            - validate").unwrap();
    writeln!(
        &mut prompt,
        "          # No execution type - inherits agent and on_error"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "          # Inherits agent and on_error from parent"
    )
    .unwrap();
    writeln!(&mut prompt, "```").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "## CRITICAL: Output Format Requirements").unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(
        &mut prompt,
        "1. **ALWAYS** start your response with valid YAML"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "2. **ALWAYS** include `name` and `version` as the first two fields"
    )
    .unwrap();
    writeln!(&mut prompt, "3. Wrap your YAML in ```yaml code blocks").unwrap();
    writeln!(
        &mut prompt,
        "4. Do NOT add explanations before the YAML - provide the workflow first"
    )
    .unwrap();
    writeln!(
        &mut prompt,
        "5. Ensure all YAML is properly indented (2 spaces per level)"
    )
    .unwrap();
    writeln!(&mut prompt).unwrap();
    writeln!(&mut prompt, "Example response format:").unwrap();
    writeln!(&mut prompt, "```yaml").unwrap();
    writeln!(&mut prompt, "name: \"Workflow Name\"").unwrap();
    writeln!(&mut prompt, "version: \"1.0.0\"").unwrap();
    writeln!(&mut prompt, "# ... rest of workflow").unwrap();
    writeln!(&mut prompt, "```").unwrap();

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template() {
        let template = generate_template();

        // Check header
        assert!(template.contains("# Agentic AI DSL Template"));
        assert!(template.contains(&format!("# DSL Grammar Version: {}", DSL_GRAMMAR_VERSION)));

        // Check main sections
        assert!(template.contains("name:"));
        assert!(template.contains("version:"));
        assert!(template.contains("agents:"));
        assert!(template.contains("tasks:"));

        // Check documentation
        assert!(template.contains("(REQUIRED)"));
        assert!(template.contains("(optional)"));

        // Check examples
        assert!(template.contains("example_agent"));
        assert!(template.contains("example_task"));
    }

    #[test]
    fn test_generate_nl_to_dsl_prompt() {
        let prompt = generate_nl_to_dsl_prompt();

        // Check structure
        assert!(prompt.contains("DSL Grammar Version"));
        assert!(prompt.contains("# Your Task"));
        assert!(prompt.contains("# DSL Structure"));
        assert!(prompt.contains("## Agent Schema"));
        assert!(prompt.contains("## Task Schema"));

        // Check it includes examples
        assert!(prompt.contains("# Example Conversion"));
        assert!(prompt.contains("Research and Report"));

        // Check it includes tool list
        assert!(prompt.contains("## Available Tools"));
        assert!(prompt.contains("Read:"));
        assert!(prompt.contains("Write:"));

        // Check it includes instructions
        assert!(prompt.contains("# Instructions"));
        assert!(prompt.contains("Output ONLY valid YAML"));
    }

    #[test]
    fn test_dsl_grammar_version() {
        // DSL_GRAMMAR_VERSION is a const, so we check it's a valid semantic version
        assert!(DSL_GRAMMAR_VERSION.len() > 0);
        // Should be semantic version
        assert!(DSL_GRAMMAR_VERSION.contains('.'));
    }
}
