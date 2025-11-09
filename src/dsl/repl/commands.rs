//! REPL Command Definitions
//!
//! Defines all available commands in the debugging REPL interface.
use crate::dsl::debugger::{BreakCondition, VariableScope};
use serde::{Deserialize, Serialize};

/// REPL command
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    // ========================================================================
    // Execution Control
    // ========================================================================
    /// Continue execution until next breakpoint or completion
    Continue,

    /// Step to next task
    Step,

    /// Step into subtasks
    StepInto,

    /// Step over subtasks (complete them without pausing)
    StepOver,

    /// Step out of current subtask context
    StepOut,

    /// Step one loop iteration
    StepIteration,

    /// Step backward (undo last step)
    StepBack { steps: usize },

    /// Step forward (redo)
    StepForward { steps: usize },

    /// Restart workflow from beginning
    Restart,

    /// Pause execution
    Pause,

    /// Resume execution (alias for Continue)
    Resume,

    // ========================================================================
    // Breakpoints
    // ========================================================================
    /// Set breakpoint
    Break { target: BreakTarget },

    /// Delete breakpoint
    Delete { id: String },

    /// List all breakpoints
    ListBreaks,

    /// Enable breakpoint
    Enable { id: String },

    /// Disable breakpoint
    Disable { id: String },

    /// Clear all breakpoints
    ClearBreaks,

    // ========================================================================
    // Inspection
    // ========================================================================
    /// Inspect task, variable, or state
    Inspect { target: InspectTarget },

    /// Print expression or variable
    Print { expression: String },

    /// Show variables in scope
    Vars { scope: Option<VariableScope> },

    /// Show call stack
    Stack,

    /// Show execution timeline
    Timeline { limit: Option<usize> },

    /// List snapshots
    Snapshots,

    /// Show debugger status
    Status,

    // ========================================================================
    // Navigation
    // ========================================================================
    /// Jump to specific snapshot
    Goto { snapshot_id: usize },

    /// Go back N snapshots
    Back { snapshots: usize },

    /// Go forward N snapshots
    Forward { snapshots: usize },

    // ========================================================================
    // Modification
    // ========================================================================
    /// Set variable value
    Set {
        scope: VariableScope,
        name: String,
        value: String,
    },

    // ========================================================================
    // Utility
    // ========================================================================
    /// Show help
    Help { command: Option<String> },

    /// Quit REPL
    Quit,

    /// Show current working directory
    Pwd,

    /// List files
    Ls { path: Option<String> },

    /// Echo text
    Echo { text: String },

    /// Clear screen
    Clear,

    /// Show command history
    History,

    /// Print workflow structure
    PrintWorkflow,

    /// Save workflow to file
    SaveWorkflow { path: String },

    /// Save REPL configuration
    SaveConfig { path: Option<String> },

    // ========================================================================
    // AI Commands
    // ========================================================================
    /// Generate workflow block from description
    AiGenerate { description: String },

    /// Get AI suggestion for fixing an error
    AiFix { error: String },

    /// Analyze workflow with AI
    AiAnalyze { workflow: Option<String> },

    /// Explain workflow with AI
    AiExplain { workflow: Option<String> },

    /// Change AI provider
    AiProvider {
        provider: String,
        model: Option<String>,
    },

    /// Show current AI configuration
    AiConfig,
}

/// Breakpoint target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BreakTarget {
    /// Break on task
    Task(String),

    /// Break on loop iteration
    Iteration { task: String, iteration: usize },

    /// Break on condition
    Condition(BreakCondition),

    /// Watch variable
    Watch { scope: VariableScope, name: String },
}

/// Inspection target
#[derive(Debug, Clone, PartialEq)]
pub enum InspectTarget {
    /// Inspect specific task
    Task(String),

    /// Inspect variable
    Variable(String),

    /// Inspect workflow state
    State,

    /// Inspect side effects
    SideEffects,

    /// Inspect current execution position
    Position,
}

impl ReplCommand {
    /// Get command name for help text
    pub fn name(&self) -> &str {
        match self {
            ReplCommand::Continue => "continue",
            ReplCommand::Step => "step",
            ReplCommand::StepInto => "stepi",
            ReplCommand::StepOver => "next",
            ReplCommand::StepOut => "finish",
            ReplCommand::StepIteration => "stepit",
            ReplCommand::StepBack { .. } => "stepback",
            ReplCommand::StepForward { .. } => "stepforward",
            ReplCommand::Restart => "restart",
            ReplCommand::Pause => "pause",
            ReplCommand::Resume => "resume",
            ReplCommand::Break { .. } => "break",
            ReplCommand::Delete { .. } => "delete",
            ReplCommand::ListBreaks => "breaks",
            ReplCommand::Enable { .. } => "enable",
            ReplCommand::Disable { .. } => "disable",
            ReplCommand::ClearBreaks => "clearbreaks",
            ReplCommand::Inspect { .. } => "inspect",
            ReplCommand::Print { .. } => "print",
            ReplCommand::Vars { .. } => "vars",
            ReplCommand::Stack => "stack",
            ReplCommand::Timeline { .. } => "timeline",
            ReplCommand::Snapshots => "snapshots",
            ReplCommand::Status => "status",
            ReplCommand::Goto { .. } => "goto",
            ReplCommand::Back { .. } => "back",
            ReplCommand::Forward { .. } => "forward",
            ReplCommand::Set { .. } => "set",
            ReplCommand::Help { .. } => "help",
            ReplCommand::Quit => "quit",
            ReplCommand::Pwd => "pwd",
            ReplCommand::Ls { .. } => "ls",
            ReplCommand::Echo { .. } => "echo",
            ReplCommand::Clear => "clear",
            ReplCommand::History => "history",
            ReplCommand::PrintWorkflow => "workflow",
            ReplCommand::SaveWorkflow { .. } => "save",
            ReplCommand::SaveConfig { .. } => "saveconfig",
            ReplCommand::AiGenerate { .. } => "ai-generate",
            ReplCommand::AiFix { .. } => "ai-fix",
            ReplCommand::AiAnalyze { .. } => "ai-analyze",
            ReplCommand::AiExplain { .. } => "ai-explain",
            ReplCommand::AiProvider { .. } => "ai-provider",
            ReplCommand::AiConfig => "ai-config",
        }
    }

    /// Get keyboard shortcut for command (if any)
    pub fn shortcut(&self) -> Option<&str> {
        match self {
            ReplCommand::Continue => Some("c"),
            ReplCommand::Step => Some("s"),
            ReplCommand::StepInto => Some("si"),
            ReplCommand::StepOver => Some("n"),
            ReplCommand::StepOut => Some("fin"),
            ReplCommand::Break { .. } => Some("b"),
            ReplCommand::Delete { .. } => Some("d"),
            ReplCommand::Inspect { .. } => Some("i"),
            ReplCommand::Print { .. } => Some("p"),
            ReplCommand::Stack => Some("bt"),
            ReplCommand::Help { .. } => Some("h, ?"),
            ReplCommand::Quit => Some("q"),
            ReplCommand::Clear => Some("cls"),
            _ => None,
        }
    }

    /// Get command description
    pub fn description(&self) -> &str {
        match self {
            ReplCommand::Continue => "Continue execution until next breakpoint",
            ReplCommand::Step => "Step to next task",
            ReplCommand::StepInto => "Step into subtasks",
            ReplCommand::StepOver => "Step over subtasks",
            ReplCommand::StepOut => "Step out of current context",
            ReplCommand::StepIteration => "Step one loop iteration",
            ReplCommand::StepBack { .. } => "Step backward in execution history",
            ReplCommand::StepForward { .. } => "Step forward in execution history",
            ReplCommand::Restart => "Restart workflow from beginning",
            ReplCommand::Pause => "Pause execution",
            ReplCommand::Resume => "Resume execution",
            ReplCommand::Break { .. } => "Set breakpoint",
            ReplCommand::Delete { .. } => "Delete breakpoint",
            ReplCommand::ListBreaks => "List all breakpoints",
            ReplCommand::Enable { .. } => "Enable breakpoint",
            ReplCommand::Disable { .. } => "Disable breakpoint",
            ReplCommand::ClearBreaks => "Clear all breakpoints",
            ReplCommand::Inspect { .. } => "Inspect task, variable, or state",
            ReplCommand::Print { .. } => "Print expression or variable",
            ReplCommand::Vars { .. } => "Show variables",
            ReplCommand::Stack => "Show call stack",
            ReplCommand::Timeline { .. } => "Show execution timeline",
            ReplCommand::Snapshots => "List execution snapshots",
            ReplCommand::Status => "Show debugger status",
            ReplCommand::Goto { .. } => "Jump to snapshot",
            ReplCommand::Back { .. } => "Go back N snapshots",
            ReplCommand::Forward { .. } => "Go forward N snapshots",
            ReplCommand::Set { .. } => "Set variable value",
            ReplCommand::Help { .. } => "Show help",
            ReplCommand::Quit => "Quit REPL",
            ReplCommand::Pwd => "Print working directory",
            ReplCommand::Ls { .. } => "List files",
            ReplCommand::Echo { .. } => "Echo text",
            ReplCommand::Clear => "Clear screen",
            ReplCommand::History => "Show command history",
            ReplCommand::PrintWorkflow => "Print hierarchical workflow structure",
            ReplCommand::SaveWorkflow { .. } => "Save workflow to YAML file",
            ReplCommand::SaveConfig { .. } => "Save REPL configuration",
            ReplCommand::AiGenerate { .. } => "Generate workflow block from description",
            ReplCommand::AiFix { .. } => "Get AI suggestion for fixing an error",
            ReplCommand::AiAnalyze { .. } => "Analyze workflow with AI",
            ReplCommand::AiExplain { .. } => "Explain workflow with AI",
            ReplCommand::AiProvider { .. } => "Change AI provider and model",
            ReplCommand::AiConfig => "Show current AI configuration",
        }
    }

    /// Get command usage
    pub fn usage(&self) -> &str {
        match self {
            ReplCommand::Continue => "continue | c",
            ReplCommand::Step => "step | s",
            ReplCommand::StepInto => "stepi | si",
            ReplCommand::StepOver => "next | n",
            ReplCommand::StepOut => "finish | fin",
            ReplCommand::StepIteration => "stepit",
            ReplCommand::StepBack { .. } => "stepback [N] | sb [N]",
            ReplCommand::StepForward { .. } => "stepforward [N] | sf [N]",
            ReplCommand::Restart => "restart",
            ReplCommand::Pause => "pause",
            ReplCommand::Resume => "resume",
            ReplCommand::Break { .. } => {
                "break <task_id> | b <task_id>\nbreak condition <expr>\nbreak watch <var>"
            }
            ReplCommand::Delete { .. } => "delete <id> | d <id>",
            ReplCommand::ListBreaks => "breaks | info breaks",
            ReplCommand::Enable { .. } => "enable <id>",
            ReplCommand::Disable { .. } => "disable <id>",
            ReplCommand::ClearBreaks => "clearbreaks | bclear",
            ReplCommand::Inspect { .. } => "inspect <task|var|state> | i <target>",
            ReplCommand::Print { .. } => "print <expr> | p <expr>",
            ReplCommand::Vars { .. } => "vars [scope]",
            ReplCommand::Stack => "stack | bt | backtrace",
            ReplCommand::Timeline { .. } => "timeline [limit] | tl [limit]",
            ReplCommand::Snapshots => "snapshots | snaps",
            ReplCommand::Status => "status | info",
            ReplCommand::Goto { .. } => "goto <snapshot_id>",
            ReplCommand::Back { .. } => "back [N]",
            ReplCommand::Forward { .. } => "forward [N]",
            ReplCommand::Set { .. } => "set <scope>.<var> = <value>",
            ReplCommand::Help { .. } => "help [command] | h [command] | ?",
            ReplCommand::Quit => "quit | q | exit",
            ReplCommand::Pwd => "pwd",
            ReplCommand::Ls { .. } => "ls [path]",
            ReplCommand::Echo { .. } => "echo <text>",
            ReplCommand::Clear => "clear | cls",
            ReplCommand::History => "history",
            ReplCommand::PrintWorkflow => "workflow | wf | tree",
            ReplCommand::SaveWorkflow { .. } => "save <file.yaml> | w <file.yaml>",
            ReplCommand::SaveConfig { .. } => "saveconfig [file.toml]",
            ReplCommand::AiGenerate { .. } => "ai-generate <description> | aigen <description>",
            ReplCommand::AiFix { .. } => "ai-fix <error_message> | aifix <error>",
            ReplCommand::AiAnalyze { .. } => "ai-analyze [workflow_file] | aianalyze",
            ReplCommand::AiExplain { .. } => "ai-explain [workflow_file] | aiexplain",
            ReplCommand::AiProvider { .. } => "ai-provider <provider> [model] | aiprovider",
            ReplCommand::AiConfig => "ai-config | aiconfig",
        }
    }
}

/// Command categories for help display
pub enum CommandCategory {
    ExecutionControl,
    Breakpoints,
    Inspection,
    Navigation,
    Modification,
    Utility,
    Ai,
}

impl CommandCategory {
    pub fn name(&self) -> &str {
        match self {
            CommandCategory::ExecutionControl => "Execution Control",
            CommandCategory::Breakpoints => "Breakpoints",
            CommandCategory::Inspection => "Inspection",
            CommandCategory::Navigation => "Navigation",
            CommandCategory::Modification => "Modification",
            CommandCategory::Utility => "Utility",
            CommandCategory::Ai => "AI Assistant",
        }
    }

    pub fn commands(&self) -> Vec<&str> {
        match self {
            CommandCategory::ExecutionControl => vec![
                "continue",
                "step",
                "stepi",
                "next",
                "finish",
                "stepit",
                "stepback",
                "stepforward",
                "restart",
                "pause",
                "resume",
            ],
            CommandCategory::Breakpoints => {
                vec![
                    "break",
                    "delete",
                    "breaks",
                    "enable",
                    "disable",
                    "clearbreaks",
                ]
            }
            CommandCategory::Inspection => vec![
                "inspect",
                "print",
                "vars",
                "stack",
                "timeline",
                "snapshots",
                "status",
            ],
            CommandCategory::Navigation => vec!["goto", "back", "forward"],
            CommandCategory::Modification => vec!["set"],
            CommandCategory::Utility => {
                vec![
                    "help",
                    "quit",
                    "pwd",
                    "ls",
                    "echo",
                    "clear",
                    "history",
                    "workflow",
                    "save",
                    "saveconfig",
                ]
            }
            CommandCategory::Ai => {
                vec![
                    "ai-generate",
                    "ai-fix",
                    "ai-analyze",
                    "ai-explain",
                    "ai-provider",
                    "ai-config",
                ]
            }
        }
    }

    pub fn all_categories() -> Vec<CommandCategory> {
        vec![
            CommandCategory::ExecutionControl,
            CommandCategory::Breakpoints,
            CommandCategory::Inspection,
            CommandCategory::Navigation,
            CommandCategory::Modification,
            CommandCategory::Utility,
            CommandCategory::Ai,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_names() {
        assert_eq!(ReplCommand::Continue.name(), "continue");
        assert_eq!(ReplCommand::Step.name(), "step");
        assert_eq!(ReplCommand::Quit.name(), "quit");
    }

    #[test]
    fn test_command_descriptions() {
        assert!(!ReplCommand::Continue.description().is_empty());
        assert!(!ReplCommand::Step.description().is_empty());
    }

    #[test]
    fn test_all_categories() {
        let categories = CommandCategory::all_categories();
        assert_eq!(categories.len(), 7);
    }
}
