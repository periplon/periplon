//! Domain-Specific Language (DSL) for creating agentic AI systems
//!
//! This module provides a YAML-based DSL for defining complex multi-agent workflows
//! with hierarchical task decomposition, tool usage, and inter-agent collaboration.
//!
//! # Core Components
//!
//! - **Schema**: DSL type definitions and structure
//! - **Parser**: YAML parsing and deserialization
//! - **Validator**: Semantic validation of DSL workflows
//! - **Executor**: DSL execution engine
//! - **Task Graph**: Hierarchical task management and dependency resolution
//! - **Communication**: Inter-agent messaging protocols
//!
//! # Example
//!
//! ```yaml
//! name: "Simple Workflow"
//! version: "1.0.0"
//!
//! agents:
//!   researcher:
//!     description: "Research and gather information"
//!     model: "claude-sonnet-4-5"
//!     tools:
//!       - Read
//!       - WebSearch
//!     permissions:
//!       mode: "default"
//!
//! tasks:
//!   gather_info:
//!     description: "Research the topic"
//!     agent: "researcher"
//! ```

pub mod context_injection;
pub mod debug_ai;
#[cfg(feature = "tui")]
pub mod debug_tui;
pub mod debugger;
pub mod executor;
pub mod fetcher;
pub mod hooks;
pub mod loop_context;
pub mod message_bus;
pub mod message_formatter;
pub mod nl_generator;
pub mod notifications;
pub mod parser;
pub mod predefined_tasks;
pub mod repl;
pub mod schema;
pub mod state;
pub mod task_graph;
pub mod template;
pub mod truncation;
pub mod validator;
pub mod variables;

pub use executor::DSLExecutor;
pub use fetcher::{fetch_subflow, SubflowCache};
pub use loop_context::{substitute_task_variables, LoopContext};
pub use message_bus::{AgentMessage, Channel, MessageBus};
pub use nl_generator::{generate_and_save, generate_from_nl};
pub use notifications::{
    ConsoleSender, DiscordSender, ElevenLabsSender, EmailSender, FileSender, NotificationContext,
    NotificationError, NotificationManager, NotificationResult, NotificationSender, NtfySender,
    SlackSender, SmsSender,
};
pub use parser::{
    merge_subflow_inline, parse_workflow, parse_workflow_file, parse_workflow_with_subflows,
    serialize_workflow, write_workflow_file,
};
pub use schema::{
    AgentSpec, CleanupStrategy, CollectionSource, CommandSpec, Condition, ConditionSpec,
    ContextConfig, ContextMode, CriterionResult, DSLWorkflow, DefinitionOfDone, DiscordEmbed,
    DiscordField, DoneCriterion, ExecutionMode, FileFormat, FileNotificationFormat, HttpAuth,
    HttpMethod, HttpSpec, InputSpec, LimitsConfig, LoopControl, LoopSpec, McpToolSpec,
    NotificationChannel, NotificationDefaults, NotificationPriority, NotificationSpec,
    OutputDataSource, OutputSource, OutputSpec, PagerDutyAction, PagerDutySeverity,
    PermissionsSpec, RetryConfig, ScriptLanguage, ScriptSpec, SecretSource, SecretSpec,
    SlackAttachment, SlackField, SlackMethod, SmtpConfig, StageSpec, SubflowSource, SubflowSpec,
    TaskSpec, TaskStatusCondition, TeamsFact, TelegramParseMode, ToolsConfig, TruncationStrategy,
    WorkflowSpec,
};
pub use state::{
    ContextMetrics, LoopState, OutputType, StatePersistence, TaskOutput, WorkflowState,
    WorkflowStatus,
};
pub use task_graph::{TaskGraph, TaskStatus};
pub use template::{generate_nl_to_dsl_prompt, generate_template, DSL_GRAMMAR_VERSION};
pub use validator::validate_workflow;
pub use variables::{extract_variable_references, Scope, VariableContext};

// Stdio and Context Management
pub use context_injection::{build_smart_context, calculate_relevance};
pub use truncation::{create_task_output, truncate_output};

// Debugger
pub use debugger::{
    BreakCondition, BreakpointInfo, BreakpointManager, BreakpointType, CompensationStrategy,
    DebugMode, DebuggerState, DebuggerStatus, DirectoryTree, ExecutionFrame, ExecutionHistory,
    ExecutionMode as DebugExecutionMode, ExecutionPointer, ExecutionSnapshot, Inspector,
    SideEffect, SideEffectJournal, SideEffectType, StepMode, TaskInspection,
    VariableScope as DebugVariableScope, VariableSnapshot, WatchCondition,
};

// REPL
pub use repl::{
    parse_command, BreakTarget, CommandCategory, InspectTarget, ReplCommand, ReplSession,
};

// Debug TUI
#[cfg(feature = "tui")]
pub use debug_tui::{DebugTUI, Event as TuiEvent, EventHandler as TuiEventHandler};

// Debug AI
pub use debug_ai::{
    analyze_error, create_default_config, generate_task, generate_workflow_block, suggest_fix,
    AiConfig, AiProvider, AiProviderType, AiResponse, DebugAiAssistant,
};
