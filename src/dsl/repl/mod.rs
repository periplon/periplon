//! REPL (Read-Eval-Print Loop) for Interactive Debugging
//!
//! Provides an interactive command-line interface for debugging DSL workflows.
//!
//! # Usage
//!
//! ```no_run
//! use periplon_sdk::dsl::repl::ReplSession;
//! use periplon_sdk::dsl::{DSLExecutor, parse_workflow};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # let yaml = r#"
//! # name: "Test"
//! # version: "1.0.0"
//! # agents:
//! #   test: { description: "Test", tools: [] }
//! # tasks:
//! #   test: { description: "Test", agent: "test" }
//! # "#;
//! # let workflow = parse_workflow(yaml)?;
//! let executor = DSLExecutor::new(workflow)?.with_debugger();
//! let mut repl = ReplSession::new(executor)?;
//! repl.run().await?;
//! # Ok(())
//! # }
//! ```
pub mod commands;
pub mod completer;
pub mod parser;

pub use commands::{BreakTarget, CommandCategory, InspectTarget, ReplCommand};
pub use completer::{CompletionContext, ReplHelper};
pub use parser::parse_command;

use crate::dsl::debug_ai::DebugAiAssistant;
use crate::dsl::debugger::{DebugMode, DebuggerState, StepMode};
use crate::dsl::DSLExecutor;
use crate::error::Result;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use std::io::{self, Write};
use std::sync::Arc;
use strsim::jaro_winkler;
use tokio::sync::Mutex;

/// REPL session state
pub struct ReplSession {
    executor: Arc<Mutex<DSLExecutor>>,
    debugger: Arc<Mutex<DebuggerState>>,
    ai_assistant: Option<DebugAiAssistant>,
    running: bool,
}

impl ReplSession {
    /// Create a new REPL session
    pub fn new(executor: DSLExecutor) -> Result<Self> {
        // Ensure debugger is enabled
        let debugger = executor
            .debugger()
            .ok_or_else(|| {
                crate::error::Error::InvalidInput(
                    "Debugger not enabled. Call .with_debugger() first".to_string(),
                )
            })?
            .clone();

        // Initialize AI assistant with default config (Ollama)
        let ai_assistant = match DebugAiAssistant::new() {
            Ok(assistant) => Some(assistant),
            Err(_) => {
                eprintln!(
                    "⚠️  AI assistant initialization failed. AI commands will be unavailable."
                );
                None
            }
        };

        Ok(Self {
            executor: Arc::new(Mutex::new(executor)),
            debugger,
            ai_assistant,
            running: true,
        })
    }

    /// Run the REPL loop
    pub async fn run(&mut self) -> Result<()> {
        println!("{}", "🐛 Periplon Debug REPL".bright_cyan().bold());
        println!(
            "Type {} for available commands, {} to exit",
            "help".green(),
            "quit".yellow()
        );
        println!(
            "Use {} for command completion, {} for history\n",
            "TAB".bright_blue(),
            "↑/↓".bright_blue()
        );

        // Configure rustyline editor
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(rustyline::CompletionType::List)
            .edit_mode(rustyline::EditMode::Emacs)
            .build();

        let helper = ReplHelper::new();
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(helper));

        // Load history from file if it exists
        let history_path = dirs::home_dir()
            .map(|mut p| {
                p.push(".periplon_repl_history");
                p
            })
            .unwrap_or_else(|| std::path::PathBuf::from(".periplon_repl_history"));

        let _ = rl.load_history(&history_path);

        // Show initial status
        self.show_status().await?;

        while self.running {
            // Update completion context before readline
            if let Some(helper) = rl.helper_mut() {
                let ctx = self.build_completion_context().await;
                helper.update_context(ctx);
            }

            // Read command with readline
            let readline = rl.readline("debug> ");

            let input = match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str())?;
                    line
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl-C
                    println!("^C");
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl-D
                    println!("quit");
                    self.running = false;
                    break;
                }
                Err(err) => {
                    eprintln!("Error reading input: {}", err);
                    break;
                }
            };

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            // Parse and execute command
            match parse_command(input) {
                Ok(command) => {
                    if let Err(e) = self.execute_command(command).await {
                        eprintln!(
                            "{} {}",
                            "❌ Error:".red().bold(),
                            e.to_string().bright_red()
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        "❌ Parse error:".red().bold(),
                        e.to_string().bright_red()
                    );
                }
            }
        }

        // Save history to file on exit
        if let Err(e) = rl.save_history(&history_path) {
            eprintln!("{} {}", "⚠️  Failed to save command history:".yellow(), e);
        }

        println!("{}", "👋 Goodbye!".bright_cyan().bold());
        Ok(())
    }

    /// Execute a parsed command
    async fn execute_command(&mut self, command: ReplCommand) -> Result<()> {
        match command {
            // Execution Control
            ReplCommand::Continue => self.cmd_continue().await,
            ReplCommand::Step => self.cmd_step(StepMode::StepTask).await,
            ReplCommand::StepInto => self.cmd_step(StepMode::StepInto).await,
            ReplCommand::StepOver => self.cmd_step(StepMode::StepOver).await,
            ReplCommand::StepOut => self.cmd_step(StepMode::StepOut).await,
            ReplCommand::StepIteration => self.cmd_step(StepMode::StepIteration).await,
            ReplCommand::Pause => self.cmd_pause().await,
            ReplCommand::Resume => self.cmd_continue().await,

            // Breakpoints
            ReplCommand::Break { target } => self.cmd_break(target).await,
            ReplCommand::ListBreaks => self.cmd_list_breaks().await,
            ReplCommand::Delete { id } => self.cmd_delete_break(&id).await,

            // Inspection
            ReplCommand::Status => self.show_status().await,
            ReplCommand::Vars { scope } => self.cmd_vars(scope).await,
            ReplCommand::Stack => self.cmd_stack().await,
            ReplCommand::Timeline { limit } => self.cmd_timeline(limit).await,
            ReplCommand::Snapshots => self.cmd_snapshots().await,

            // Utility
            ReplCommand::Help { command } => self.cmd_help(command.as_deref()),
            ReplCommand::Quit => {
                self.running = false;
                Ok(())
            }
            ReplCommand::Clear => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush().unwrap();
                Ok(())
            }
            ReplCommand::History => self.cmd_history(),

            // Time-travel navigation
            ReplCommand::StepBack { steps } => self.cmd_step_back(steps).await,
            ReplCommand::StepForward { steps } => self.cmd_step_forward(steps).await,
            ReplCommand::Goto { snapshot_id } => self.cmd_goto(snapshot_id).await,
            ReplCommand::Back { snapshots } => self.cmd_back(snapshots).await,
            ReplCommand::Forward { snapshots } => self.cmd_forward(snapshots).await,
            ReplCommand::Restart => self.cmd_restart().await,

            // Detailed inspection
            ReplCommand::Inspect { target } => self.cmd_inspect(target).await,
            ReplCommand::Print { expression } => self.cmd_print(&expression).await,

            // Modification
            ReplCommand::Set { scope, name, value } => self.cmd_set(scope, name, value).await,

            // Breakpoint management
            ReplCommand::Enable { id } => self.cmd_enable(&id).await,
            ReplCommand::Disable { id } => self.cmd_disable(&id).await,
            ReplCommand::ClearBreaks => self.cmd_clear_breaks().await,

            // Utility already implemented
            ReplCommand::Pwd => self.cmd_pwd(),
            ReplCommand::Ls { path } => self.cmd_ls(path.as_deref()),
            ReplCommand::Echo { text } => self.cmd_echo(&text),
            ReplCommand::PrintWorkflow => self.cmd_print_workflow().await,
            ReplCommand::SaveWorkflow { path } => self.cmd_save_workflow(&path).await,
            ReplCommand::SaveConfig { path } => self.cmd_save_config(path.as_deref()).await,

            // AI Commands
            ReplCommand::AiGenerate { description } => self.cmd_ai_generate(&description).await,
            ReplCommand::AiFix { error } => self.cmd_ai_fix(&error).await,
            ReplCommand::AiAnalyze { workflow } => self.cmd_ai_analyze(workflow.as_deref()).await,
            ReplCommand::AiExplain { workflow } => self.cmd_ai_explain(workflow.as_deref()).await,
            ReplCommand::AiProvider { provider, model } => {
                self.cmd_ai_provider(&provider, model.as_deref()).await
            }
            ReplCommand::AiConfig => self.cmd_ai_config(),
            ReplCommand::AiSetTemperature { temperature } => {
                self.cmd_ai_set_temperature(temperature).await
            }
            ReplCommand::AiSetMaxTokens { max_tokens } => {
                self.cmd_ai_set_max_tokens(max_tokens).await
            }
            ReplCommand::AiSetEndpoint { endpoint } => {
                self.cmd_ai_set_endpoint(endpoint.as_deref()).await
            }
            ReplCommand::AiSetApiKey { api_key } => {
                self.cmd_ai_set_api_key(api_key.as_deref()).await
            }
        }
    }

    /// Continue execution
    async fn cmd_continue(&self) -> Result<()> {
        let mut dbg = self.debugger.lock().await;
        dbg.resume();
        println!("{}", "▶️  Execution resumed".green());
        Ok(())
    }

    /// Step execution
    async fn cmd_step(&self, mode: StepMode) -> Result<()> {
        let mut dbg = self.debugger.lock().await;
        dbg.set_step_mode(mode);
        dbg.resume();
        println!("{} ({:?})...", "👣 Stepping".cyan(), mode);
        Ok(())
    }

    /// Pause execution
    async fn cmd_pause(&self) -> Result<()> {
        let mut dbg = self.debugger.lock().await;
        dbg.pause();
        println!("{}", "⏸️  Execution paused".yellow());
        Ok(())
    }

    /// Set breakpoint
    async fn cmd_break(&self, target: BreakTarget) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        let id = match target {
            BreakTarget::Task(task_id) => {
                dbg.breakpoints.add_task_breakpoint(task_id.clone());
                format!("task:{}", task_id)
            }
            BreakTarget::Condition(condition) => dbg
                .breakpoints
                .add_conditional_breakpoint(condition.clone(), None),
            BreakTarget::Iteration { task, iteration } => {
                dbg.breakpoints.add_loop_breakpoint(task.clone(), iteration);
                format!("loop:{}:{}", task, iteration)
            }
            BreakTarget::Watch { scope, name } => {
                use crate::dsl::debugger::WatchCondition;
                dbg.breakpoints
                    .add_watch(scope.clone(), name.clone(), WatchCondition::AnyChange)
            }
        };

        println!("{} {}", "✓ Breakpoint set:".green().bold(), id.cyan());
        Ok(())
    }

    /// List breakpoints
    async fn cmd_list_breaks(&self) -> Result<()> {
        let dbg = self.debugger.lock().await;
        let breakpoints = dbg.breakpoints.list_all();

        if breakpoints.is_empty() {
            println!("{}", "No breakpoints set".yellow());
        } else {
            println!("{}:", "Breakpoints".bright_yellow().bold());
            for bp in breakpoints {
                let status = if bp.enabled {
                    "enabled".green()
                } else {
                    "disabled".bright_black()
                };
                println!(
                    "  {} [{}] {} (hits: {})",
                    bp.id.cyan(),
                    status,
                    bp.description,
                    bp.hit_count.to_string().bright_white()
                );
            }
        }
        Ok(())
    }

    /// Delete breakpoint
    async fn cmd_delete_break(&self, id: &str) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        // Try different breakpoint types
        if id.starts_with("task:") {
            let task_id = id.strip_prefix("task:").unwrap();
            if dbg.breakpoints.remove_task_breakpoint(task_id) {
                println!("{} {}", "✓ Breakpoint deleted:".green().bold(), id.cyan());
            } else {
                println!("{} {}", "❌ Breakpoint not found:".red(), id.yellow());
            }
        } else if dbg.breakpoints.remove_conditional_breakpoint(id) {
            println!("{} {}", "✓ Breakpoint deleted:".green().bold(), id.cyan());
        } else {
            println!("{} {}", "❌ Breakpoint not found:".red(), id.yellow());
        }

        Ok(())
    }

    /// Show variables
    async fn cmd_vars(&self, _scope: Option<crate::dsl::debugger::VariableScope>) -> Result<()> {
        let executor = self.executor.lock().await;
        if let Some(inspector) = executor.inspector() {
            let vars = inspector.inspect_variables(None).await;

            println!("{}:", "Variables".bright_yellow().bold());
            if !vars.workflow_vars.is_empty() {
                println!("  {}:", "Workflow".cyan());
                for (name, value) in &vars.workflow_vars {
                    println!("    {} {} {:?}", name.green(), "=".bright_black(), value);
                }
            }

            if !vars.task_vars.is_empty() {
                println!("  {}:", "Tasks".cyan());
                for (task_id, task_vars) in &vars.task_vars {
                    println!("    {}:", task_id.magenta());
                    for (name, value) in task_vars {
                        println!("      {} {} {:?}", name.green(), "=".bright_black(), value);
                    }
                }
            }

            if vars.total_count() == 0 {
                println!("  {}", "(no variables)".bright_black());
            }
        }
        Ok(())
    }

    /// Show call stack
    async fn cmd_stack(&self) -> Result<()> {
        let executor = self.executor.lock().await;
        if let Some(inspector) = executor.inspector() {
            let stack_string = inspector.call_stack_string().await;
            if stack_string.is_empty() {
                println!("Call stack: (empty)");
            } else {
                println!("Call stack:\n{}", stack_string);
            }
        }
        Ok(())
    }

    /// Show timeline
    async fn cmd_timeline(&self, limit: Option<usize>) -> Result<()> {
        let executor = self.executor.lock().await;
        if let Some(inspector) = executor.inspector() {
            let timeline = inspector.timeline().await;
            let events = if let Some(lim) = limit {
                &timeline.events[..timeline.events.len().min(lim)]
            } else {
                &timeline.events
            };

            println!("Timeline ({} events):", timeline.events.len());
            for (i, event) in events.iter().enumerate() {
                println!("  {}. {:?}", i + 1, event.event_type);
            }

            if timeline.events.len() > events.len() {
                println!("  ... ({} more)", timeline.events.len() - events.len());
            }
        }
        Ok(())
    }

    /// Show snapshots
    async fn cmd_snapshots(&self) -> Result<()> {
        let executor = self.executor.lock().await;
        if let Some(inspector) = executor.inspector() {
            let snapshots = inspector.snapshots().await;

            if snapshots.is_empty() {
                println!("No snapshots");
            } else {
                println!("Snapshots:");
                for snap in snapshots {
                    println!(
                        "  #{}: {} ({:.2}s)",
                        snap.id,
                        snap.description,
                        snap.elapsed.as_secs_f64()
                    );
                }
            }
        }
        Ok(())
    }

    /// Show status
    async fn show_status(&self) -> Result<()> {
        let dbg = self.debugger.lock().await;
        let status = dbg.status_summary();
        println!("{}", status);
        Ok(())
    }

    /// Show help
    fn cmd_help(&self, command: Option<&str>) -> Result<()> {
        if let Some(cmd) = command {
            // Show help for specific command
            self.show_command_help(cmd)?;
        } else {
            // Show general help with sections and columns
            self.show_general_help()?;
        }
        Ok(())
    }

    /// Show general help with categorized commands in columns
    fn show_general_help(&self) -> Result<()> {
        println!(
            "{}\n",
            "🐛 Periplon Debug REPL - Available Commands"
                .bright_cyan()
                .bold()
        );

        for category in commands::CommandCategory::all_categories() {
            println!("{}", category.name().bright_yellow().bold());
            println!("{}", "─".repeat(50).bright_black());

            let cmds = category.commands();
            self.print_commands_in_columns(&cmds, 3);
            println!();
        }

        println!("{}", "💡 Tips:".bright_green().bold());
        println!(
            "  • Type {} for detailed information",
            "help <command>".cyan()
        );
        println!("  • Use {} for command completion", "TAB".bright_blue());
        println!("  • Use {} for history navigation", "↑/↓".bright_blue());
        println!("  • Type {} or {} to quit", "q".yellow(), "Ctrl+D".yellow());
        Ok(())
    }

    /// Print commands in columns
    fn print_commands_in_columns(&self, commands: &[&str], columns: usize) {
        let max_width = commands.iter().map(|c| c.len()).max().unwrap_or(0) + 2;

        for (i, cmd) in commands.iter().enumerate() {
            print!("  {:width$}", cmd.green(), width = max_width + 9); // +9 for ANSI color codes

            // New line after every N columns or at the end
            if (i + 1) % columns == 0 || i == commands.len() - 1 {
                println!();
            }
        }
    }

    /// Show detailed help for a specific command
    fn show_command_help(&self, cmd_name: &str) -> Result<()> {
        // Normalize command name (handle aliases)
        let normalized = self.normalize_command_name(cmd_name);

        // Find command info
        if let Some((cmd, category)) = self.find_command_info(&normalized) {
            println!(
                "{} {}",
                "Command:".bright_cyan().bold(),
                cmd.name().green().bold()
            );

            // Show shortcut if available
            if let Some(shortcut) = cmd.shortcut() {
                println!("{} {}", "Shortcut:".bright_blue(), shortcut.yellow());
            }

            println!("{}", "═".repeat(50).bright_black());
            println!("\n{}:", "Description".bright_yellow());
            println!("  {}", cmd.description());
            println!("\n{}:", "Usage".bright_yellow());
            for line in cmd.usage().lines() {
                println!("  {}", line.cyan());
            }

            // Show examples if available
            if let Some(examples) = self.get_command_examples(&normalized) {
                println!("\n{}:", "Examples".bright_yellow());
                for example in examples {
                    println!("  {}", example.bright_white());
                }
            }

            println!("\n{} {}", "Category:".bright_magenta(), category.name());
        } else {
            println!("{} Unknown command: '{}'", "❌".red(), cmd_name.yellow());
            println!("\nType {} to see all available commands.", "help".cyan());
        }

        Ok(())
    }

    /// Normalize command name (convert aliases to canonical names)
    fn normalize_command_name(&self, name: &str) -> String {
        match name {
            // Execution control aliases
            "c" => "continue".to_string(),
            "s" => "step".to_string(),
            "si" => "stepi".to_string(),
            "n" => "next".to_string(),
            "fin" => "finish".to_string(),
            "sb" => "stepback".to_string(),
            "sf" => "stepforward".to_string(),

            // Breakpoint aliases
            "b" => "break".to_string(),
            "d" => "delete".to_string(),
            "bclear" => "clearbreaks".to_string(),

            // Inspection aliases
            "i" => "inspect".to_string(),
            "p" => "print".to_string(),
            "bt" | "backtrace" => "stack".to_string(),
            "tl" => "timeline".to_string(),
            "snaps" => "snapshots".to_string(),
            "info" => "status".to_string(),

            // Utility aliases
            "h" | "?" => "help".to_string(),
            "q" | "exit" => "quit".to_string(),
            "cls" => "clear".to_string(),
            "wf" | "tree" => "workflow".to_string(),
            "w" => "save".to_string(),

            // AI aliases
            "aigen" => "ai-generate".to_string(),
            "aifix" => "ai-fix".to_string(),
            "aianalyze" => "ai-analyze".to_string(),
            "aiexplain" => "ai-explain".to_string(),
            "aiprovider" => "ai-provider".to_string(),
            "aiconfig" => "ai-config".to_string(),

            // Default: return as-is
            _ => name.to_string(),
        }
    }

    /// Find command info by name
    fn find_command_info(&self, name: &str) -> Option<(ReplCommand, CommandCategory)> {
        for category in commands::CommandCategory::all_categories() {
            for cmd_name in category.commands() {
                if cmd_name == name {
                    // Create a dummy command instance for metadata
                    let cmd = self.create_command_instance(name);
                    return Some((cmd, category));
                }
            }
        }
        None
    }

    /// Create command instance for help display
    fn create_command_instance(&self, name: &str) -> ReplCommand {
        match name {
            "continue" => ReplCommand::Continue,
            "step" => ReplCommand::Step,
            "stepi" => ReplCommand::StepInto,
            "next" => ReplCommand::StepOver,
            "finish" => ReplCommand::StepOut,
            "stepit" => ReplCommand::StepIteration,
            "stepback" => ReplCommand::StepBack { steps: 1 },
            "stepforward" => ReplCommand::StepForward { steps: 1 },
            "restart" => ReplCommand::Restart,
            "pause" => ReplCommand::Pause,
            "resume" => ReplCommand::Resume,
            "break" => ReplCommand::Break {
                target: BreakTarget::Task("example".to_string()),
            },
            "delete" => ReplCommand::Delete {
                id: "id".to_string(),
            },
            "breaks" => ReplCommand::ListBreaks,
            "enable" => ReplCommand::Enable {
                id: "id".to_string(),
            },
            "disable" => ReplCommand::Disable {
                id: "id".to_string(),
            },
            "clearbreaks" => ReplCommand::ClearBreaks,
            "inspect" => ReplCommand::Inspect {
                target: InspectTarget::State,
            },
            "print" => ReplCommand::Print {
                expression: "var".to_string(),
            },
            "vars" => ReplCommand::Vars { scope: None },
            "stack" => ReplCommand::Stack,
            "timeline" => ReplCommand::Timeline { limit: None },
            "snapshots" => ReplCommand::Snapshots,
            "status" => ReplCommand::Status,
            "goto" => ReplCommand::Goto { snapshot_id: 0 },
            "back" => ReplCommand::Back { snapshots: 1 },
            "forward" => ReplCommand::Forward { snapshots: 1 },
            "set" => ReplCommand::Set {
                scope: crate::dsl::debugger::VariableScope::Workflow,
                name: "var".to_string(),
                value: "value".to_string(),
            },
            "help" => ReplCommand::Help { command: None },
            "quit" => ReplCommand::Quit,
            "pwd" => ReplCommand::Pwd,
            "ls" => ReplCommand::Ls { path: None },
            "echo" => ReplCommand::Echo {
                text: "text".to_string(),
            },
            "clear" => ReplCommand::Clear,
            "history" => ReplCommand::History,
            "workflow" => ReplCommand::PrintWorkflow,
            "save" => ReplCommand::SaveWorkflow {
                path: "workflow.yaml".to_string(),
            },
            "saveconfig" => ReplCommand::SaveConfig { path: None },
            "ai-generate" => ReplCommand::AiGenerate {
                description: "desc".to_string(),
            },
            "ai-fix" => ReplCommand::AiFix {
                error: "error".to_string(),
            },
            "ai-analyze" => ReplCommand::AiAnalyze { workflow: None },
            "ai-explain" => ReplCommand::AiExplain { workflow: None },
            "ai-provider" => ReplCommand::AiProvider {
                provider: "ollama".to_string(),
                model: None,
            },
            "ai-config" => ReplCommand::AiConfig,
            _ => ReplCommand::Help { command: None },
        }
    }

    /// Get examples for a command
    fn get_command_examples(&self, name: &str) -> Option<Vec<String>> {
        let examples = match name {
            "break" => vec![
                "break task_id          # Break on task".to_string(),
                "break condition state.failed  # Break on condition".to_string(),
                "break watch workflow.status   # Watch variable".to_string(),
            ],
            "step" | "stepi" | "next" | "finish" => vec![
                "step                   # Step to next task".to_string(),
                "s                      # Same as step (alias)".to_string(),
            ],
            "stepback" | "stepforward" => vec![
                format!("{} 5                  # Move 5 steps", name),
                format!("{}                    # Move 1 step (default)", name),
            ],
            "print" => vec![
                "print workflow.status  # Print workflow variable".to_string(),
                "p task:t1.output       # Print task variable (alias)".to_string(),
            ],
            "inspect" => vec![
                "inspect task_id        # Inspect task details".to_string(),
                "inspect state          # Inspect workflow state".to_string(),
                "inspect effects        # Inspect side effects".to_string(),
            ],
            "goto" | "back" | "forward" => {
                vec![format!("{} 3                   # Jump to position 3", name)]
            }
            "ai-generate" => vec![
                "ai-generate Create a task that downloads a file".to_string(),
                "aigen Add error handling to workflow  # Using alias".to_string(),
            ],
            "ai-fix" => vec![
                "ai-fix Task failed: connection timeout".to_string(),
                "aifix Invalid YAML syntax              # Using alias".to_string(),
            ],
            "ai-provider" => vec![
                "ai-provider ollama olmo2:13b".to_string(),
                "ai-provider openai gpt-4o".to_string(),
                "ai-provider anthropic".to_string(),
            ],
            "save" => vec![
                "save workflow.yaml     # Save current workflow".to_string(),
                "w my_workflow.yaml     # Using shortcut 'w'".to_string(),
                "save output.yaml       # Export to custom file".to_string(),
            ],
            "saveconfig" => vec![
                "saveconfig             # Save to default location".to_string(),
                "saveconfig my.toml     # Save to custom file".to_string(),
            ],
            "workflow" => vec![
                "workflow               # Display workflow structure".to_string(),
                "wf                     # Using shortcut 'wf'".to_string(),
                "tree                   # Using alias 'tree'".to_string(),
            ],
            _ => return None,
        };

        Some(examples)
    }

    /// Show command history
    fn cmd_history(&self) -> Result<()> {
        println!("Command History:");
        println!("  History is managed by the line editor.");
        println!("  Use ↑/↓ arrow keys to navigate through command history.");
        println!("  History persists across sessions in: ~/.periplon_repl_history");
        println!();
        println!("  Pro tip: Use Ctrl+R for reverse history search");
        Ok(())
    }

    /// Step backward in execution history
    async fn cmd_step_back(&self, steps: usize) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        for i in 0..steps {
            if dbg.history.back(1).is_none() {
                println!(
                    "⚠️  Already at beginning of history (stepped back {} steps)",
                    i
                );
                break;
            }
        }

        dbg.mode = DebugMode::TimeTraveling;
        println!("⏮️  Stepped back {} step(s)", steps);
        println!(
            "📍 Position: {}",
            dbg.pointer.current_task.as_deref().unwrap_or("(none)")
        );
        Ok(())
    }

    /// Step forward in execution history
    async fn cmd_step_forward(&self, steps: usize) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        for i in 0..steps {
            if dbg.history.forward(1).is_none() {
                println!(
                    "⚠️  Already at end of history (stepped forward {} steps)",
                    i
                );
                break;
            }
        }

        dbg.mode = DebugMode::TimeTraveling;
        println!("⏭️  Stepped forward {} step(s)", steps);
        println!(
            "📍 Position: {}",
            dbg.pointer.current_task.as_deref().unwrap_or("(none)")
        );
        Ok(())
    }

    /// Go to specific snapshot
    async fn cmd_goto(&self, snapshot_id: usize) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        if dbg.history.goto(snapshot_id).is_some() {
            dbg.mode = DebugMode::TimeTraveling;
            println!("🎯 Jumped to snapshot #{}", snapshot_id);
            println!(
                "📍 Position: {}",
                dbg.pointer.current_task.as_deref().unwrap_or("(none)")
            );
        } else {
            println!("❌ Invalid snapshot ID: {}", snapshot_id);
        }
        Ok(())
    }

    /// Go back N snapshots
    async fn cmd_back(&self, snapshots: usize) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        for i in 0..snapshots {
            if dbg.history.back(1).is_none() {
                println!(
                    "⚠️  Already at beginning of history (moved back {} snapshots)",
                    i
                );
                break;
            }
        }

        dbg.mode = DebugMode::TimeTraveling;
        println!("⏮️  Moved back {} snapshot(s)", snapshots);
        Ok(())
    }

    /// Go forward N snapshots
    async fn cmd_forward(&self, snapshots: usize) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        for i in 0..snapshots {
            if dbg.history.forward(1).is_none() {
                println!(
                    "⚠️  Already at end of history (moved forward {} snapshots)",
                    i
                );
                break;
            }
        }

        dbg.mode = DebugMode::TimeTraveling;
        println!("⏭️  Moved forward {} snapshot(s)", snapshots);
        Ok(())
    }

    /// Restart workflow from beginning
    async fn cmd_restart(&self) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        // Reset debugger state
        dbg.pointer = crate::dsl::debugger::ExecutionPointer::new();
        dbg.side_effects.clear();
        dbg.step_count = 0;
        dbg.last_breakpoint = None;
        dbg.mode = DebugMode::Paused;

        println!("🔄 Workflow restarted");
        println!("⚠️  Note: You'll need to re-execute the workflow to see changes");
        Ok(())
    }

    /// Suggest similar task names using fuzzy matching
    fn suggest_similar_task(&self, task_name: &str, executor: &DSLExecutor) -> Option<String> {
        let workflow = executor.workflow();
        let task_ids: Vec<&String> = workflow.tasks.keys().collect();

        if task_ids.is_empty() {
            return None;
        }

        // Calculate similarity scores for all task IDs
        let mut similarities: Vec<(&String, f64)> = task_ids
            .iter()
            .map(|id| (*id, jaro_winkler(task_name, id)))
            .collect();

        // Sort by similarity (highest first)
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return the most similar if above threshold
        let threshold = 0.7;
        if let Some((best_match, score)) = similarities.first() {
            if *score >= threshold {
                return Some(best_match.to_string());
            }
        }

        None
    }

    /// Inspect task, variable, or state
    async fn cmd_inspect(&self, target: InspectTarget) -> Result<()> {
        let executor = self.executor.lock().await;

        match target {
            InspectTarget::Task(task_id) => {
                if let Some(inspector) = executor.inspector() {
                    if let Some(task_info) = inspector.inspect_task(&task_id).await {
                        println!("Task: {}", task_id);
                        println!("  Status: {:?}", task_info.status);
                        println!("  Duration: {:?}", task_info.duration);
                        println!("  Attempts: {}", task_info.attempts);

                        if !task_info.inputs.is_empty() {
                            println!("  Inputs:");
                            for (k, v) in &task_info.inputs {
                                println!("    {} = {:?}", k, v);
                            }
                        }

                        if let Some(ref output) = task_info.outputs {
                            println!("  Output:");
                            println!("    Type: {:?}", output.output_type);
                            println!("    Size: {} bytes", output.original_size);
                            if output.truncated {
                                println!("    Truncated: yes");
                            }
                            if let Some(ref path) = output.file_path {
                                println!("    File: {}", path.display());
                            }
                        }

                        if let Some(ref error) = task_info.error {
                            println!("  Error: {}", error);
                        }
                    } else {
                        // Check if task exists in workflow definition but not in state
                        if executor.workflow().tasks.contains_key(&task_id) {
                            println!("❌ Task '{}' exists in workflow but hasn't been executed yet", task_id);
                            println!("   Use 'workflow' command to see the workflow definition");
                        } else {
                            println!("❌ Task not found: {}", task_id);
                            if let Some(suggestion) = self.suggest_similar_task(&task_id, &executor) {
                                println!("   Did you mean '{}'?", suggestion.bright_yellow());
                            }
                        }
                    }
                }
            }

            InspectTarget::Variable(name) => {
                if let Some(inspector) = executor.inspector() {
                    let vars = inspector.inspect_variables(None).await;

                    // Search in all scopes
                    let mut found = false;

                    if let Some(value) = vars.workflow_vars.get(&name) {
                        println!("Variable: workflow.{}", name);
                        println!("  Value: {:?}", value);
                        found = true;
                    }

                    for (task_id, task_vars) in &vars.task_vars {
                        if let Some(value) = task_vars.get(&name) {
                            println!("Variable: task:{}.{}", task_id, name);
                            println!("  Value: {:?}", value);
                            found = true;
                        }
                    }

                    if !found {
                        println!("❌ Variable not found: {}", name);
                    }
                }
            }

            InspectTarget::State => {
                if let Some(inspector) = executor.inspector() {
                    let status = inspector.status().await;
                    println!("{}", status);
                }
            }

            InspectTarget::SideEffects => {
                if let Some(inspector) = executor.inspector() {
                    let effects = inspector.side_effects(None).await;

                    if effects.is_empty() {
                        println!("No side effects recorded");
                    } else {
                        println!("Side Effects ({}):", effects.len());
                        for (i, effect) in effects.iter().enumerate() {
                            println!(
                                "  {}. {:?} in task: {}",
                                i + 1,
                                effect.effect_type,
                                effect.task_id
                            );
                        }
                    }
                }
            }

            InspectTarget::Position => {
                if let Some(inspector) = executor.inspector() {
                    let pos = inspector.current_position().await;
                    println!("Current Position:");
                    println!(
                        "  Task: {}",
                        pos.current_task.as_deref().unwrap_or("(none)")
                    );
                    if let Some(ref loop_pos) = pos.loop_position {
                        let total = loop_pos
                            .total_iterations
                            .map(|t| t.to_string())
                            .unwrap_or_else(|| "?".to_string());
                        println!(
                            "  Loop: {} (iteration {}/{})",
                            loop_pos.task_id, loop_pos.iteration, total
                        );
                    }
                    println!("  Call Stack Depth: {}", pos.call_stack.len());
                    println!("  Steps: {}", pos.step_count);
                }
            }
        }

        Ok(())
    }

    /// Print expression or variable
    async fn cmd_print(&self, expression: &str) -> Result<()> {
        let executor = self.executor.lock().await;

        if let Some(inspector) = executor.inspector() {
            let vars = inspector.inspect_variables(None).await;

            // Try to find the variable by parsing scope.name format
            if let Some((scope_str, name)) = expression.split_once('.') {
                match scope_str {
                    "workflow" | "wf" => {
                        if let Some(value) = vars.workflow_vars.get(name) {
                            println!("{:?}", value);
                        } else {
                            println!("❌ Variable not found: {}", expression);
                        }
                    }
                    scope if scope.starts_with("task:") || scope.starts_with("t:") => {
                        let task_name = scope
                            .strip_prefix("task:")
                            .or_else(|| scope.strip_prefix("t:"))
                            .unwrap();

                        if let Some(task_vars) = vars.task_vars.get(task_name) {
                            if let Some(value) = task_vars.get(name) {
                                println!("{:?}", value);
                            } else {
                                println!("❌ Variable not found: {}", expression);
                            }
                        } else {
                            println!("❌ Task not found: {}", task_name);
                            if let Some(suggestion) =
                                self.suggest_similar_task(task_name, &executor)
                            {
                                println!("   Did you mean '{}'?", suggestion.bright_yellow());
                            }
                        }
                    }
                    _ => {
                        println!("❌ Invalid scope: {}", scope_str);
                    }
                }
            } else {
                // Simple variable name - search all scopes
                if let Some(value) = vars.workflow_vars.get(expression) {
                    println!("{:?}", value);
                } else {
                    println!("❌ Variable not found: {}", expression);
                }
            }
        }

        Ok(())
    }

    /// Set variable value
    async fn cmd_set(
        &self,
        scope: crate::dsl::debugger::VariableScope,
        name: String,
        value: String,
    ) -> Result<()> {
        println!("⚠️  Variable modification not yet fully implemented");
        println!("Would set: {:?}.{} = {}", scope, name, value);
        println!("Note: This requires deep integration with workflow state");
        Ok(())
    }

    /// Enable breakpoint
    async fn cmd_enable(&self, id: &str) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        if dbg.breakpoints.enable_conditional(id) {
            println!("✓ Breakpoint enabled: {}", id);
        } else {
            println!("❌ Breakpoint not found: {}", id);
        }
        Ok(())
    }

    /// Disable breakpoint
    async fn cmd_disable(&self, id: &str) -> Result<()> {
        let mut dbg = self.debugger.lock().await;

        if dbg.breakpoints.disable_conditional(id) {
            println!("✓ Breakpoint disabled: {}", id);
        } else {
            println!("❌ Breakpoint not found: {}", id);
        }
        Ok(())
    }

    /// Clear all breakpoints
    async fn cmd_clear_breaks(&self) -> Result<()> {
        let mut dbg = self.debugger.lock().await;
        dbg.breakpoints.clear_all();
        println!("✓ All breakpoints cleared");
        Ok(())
    }

    /// Show current working directory
    fn cmd_pwd(&self) -> Result<()> {
        if let Ok(cwd) = std::env::current_dir() {
            println!("{}", cwd.display());
        } else {
            println!("❌ Failed to get current directory");
        }
        Ok(())
    }

    /// List files in directory
    fn cmd_ls(&self, path: Option<&str>) -> Result<()> {
        let target = path.unwrap_or(".");

        match std::fs::read_dir(target) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    let prefix = if is_dir { "📁" } else { "📄" };
                    println!("{} {}", prefix, name.to_string_lossy());
                }
            }
            Err(e) => {
                println!("❌ Failed to list directory: {}", e);
            }
        }
        Ok(())
    }

    /// Echo text
    fn cmd_echo(&self, text: &str) -> Result<()> {
        println!("{}", text);
        Ok(())
    }

    /// Print workflow structure
    async fn cmd_print_workflow(&self) -> Result<()> {
        let executor = self.executor.lock().await;
        let workflow = executor.workflow();

        // Print header
        println!("\n{}", "═".repeat(70).bright_black());
        println!(
            "{} {}",
            "Workflow:".bright_cyan().bold(),
            workflow.name.green().bold()
        );
        println!(
            "{} {}",
            "Version:".bright_black(),
            workflow.version.yellow()
        );
        println!("{}", "═".repeat(70).bright_black());

        // Get task graph to understand dependencies
        if let Some(inspector) = executor.inspector() {
            let status = inspector.status().await;
            println!("{} {}", "Status:".bright_magenta(), status);
        }

        // Print agents
        if !workflow.agents.is_empty() {
            println!("\n{}", "Agents:".bright_yellow().bold());
            for (agent_id, agent) in &workflow.agents {
                let provider_str = if let Some(ref p) = agent.provider {
                    format!("{:?}", p)
                } else {
                    format!("{:?}", workflow.provider)
                };

                let model = agent
                    .model
                    .as_deref()
                    .or(workflow.model.as_deref())
                    .unwrap_or("default");

                println!(
                    "  {} {} {} {} {}",
                    "●".green(),
                    agent_id.cyan().bold(),
                    format!("[{}]", provider_str).bright_black(),
                    format!("({})", model).bright_black(),
                    agent.description.bright_white()
                );

                if !agent.tools.is_empty() {
                    println!(
                        "    {} {}",
                        "Tools:".bright_black(),
                        agent.tools.join(", ").blue()
                    );
                }
            }
        }

        // Build task hierarchy
        println!("\n{}", "Tasks:".bright_yellow().bold());

        let root_tasks = self.find_root_tasks(workflow);
        let mut visited = std::collections::HashSet::new();

        for (i, task_id) in root_tasks.iter().enumerate() {
            let is_last = i == root_tasks.len() - 1;
            self.print_task_tree(workflow, task_id, "", is_last, &mut visited, &executor)
                .await;
        }

        // Print any orphaned tasks (tasks not in the hierarchy)
        let all_task_ids: Vec<_> = workflow.tasks.keys().cloned().collect();
        let orphaned: Vec<_> = all_task_ids
            .iter()
            .filter(|id| !visited.contains(*id))
            .collect();

        if !orphaned.is_empty() {
            println!("\n{}", "Orphaned Tasks:".yellow().bold());
            for task_id in orphaned {
                self.print_task_tree(workflow, task_id, "", true, &mut visited, &executor)
                    .await;
            }
        }

        println!("{}", "═".repeat(70).bright_black());
        println!();

        Ok(())
    }

    /// Find root tasks (tasks with no dependencies)
    fn find_root_tasks(&self, workflow: &crate::dsl::schema::DSLWorkflow) -> Vec<String> {
        let mut roots = Vec::new();

        for (task_id, task) in &workflow.tasks {
            let has_deps = !task.depends_on.is_empty();
            let is_subtask = workflow.tasks.values().any(|t| {
                t.subtasks
                    .iter()
                    .any(|subtask_map| subtask_map.contains_key(task_id))
            });

            if !has_deps && !is_subtask {
                roots.push(task_id.clone());
            }
        }

        // Sort for consistent output
        roots.sort();
        roots
    }

    /// Print task tree recursively
    async fn print_task_tree(
        &self,
        workflow: &crate::dsl::schema::DSLWorkflow,
        task_id: &str,
        prefix: &str,
        is_last: bool,
        visited: &mut std::collections::HashSet<String>,
        executor: &tokio::sync::MutexGuard<'_, crate::dsl::DSLExecutor>,
    ) {
        if visited.contains(task_id) {
            return; // Avoid cycles
        }
        visited.insert(task_id.to_string());

        let task = match workflow.tasks.get(task_id) {
            Some(t) => t,
            None => return,
        };

        // Determine tree characters
        let connector = if is_last { "└──" } else { "├──" };
        let extension = if is_last { "   " } else { "│  " };

        // Get task status if available
        let status_indicator = if let Some(inspector) = executor.inspector() {
            if let Some(task_info) = inspector.inspect_task(task_id).await {
                match task_info.status {
                    crate::dsl::task_graph::TaskStatus::Pending => "○".bright_black(),
                    crate::dsl::task_graph::TaskStatus::Ready => "◯".cyan(),
                    crate::dsl::task_graph::TaskStatus::Running => "◐".yellow(),
                    crate::dsl::task_graph::TaskStatus::Completed => "●".green(),
                    crate::dsl::task_graph::TaskStatus::Failed => "✗".red(),
                    crate::dsl::task_graph::TaskStatus::Skipped => "⊘".bright_black(),
                }
            } else {
                "○".bright_black()
            }
        } else {
            "○".bright_black()
        };

        // Print task line
        print!(
            "{}{} {} {}",
            prefix.bright_black(),
            connector.bright_black(),
            status_indicator,
            task_id.green().bold()
        );

        // Print agent
        if let Some(ref agent) = task.agent {
            print!(" {} {}", "→".bright_black(), agent.cyan());
        }

        // Print description if available
        if !task.description.is_empty() {
            print!(" {}", format!("({})", task.description).bright_white());
        }

        println!();

        // Print dependencies
        if !task.depends_on.is_empty() {
            print!(
                "{}{}    {} ",
                prefix,
                extension.bright_black(),
                "Depends on:".bright_black()
            );
            for (i, dep) in task.depends_on.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                print!("{}", dep.magenta());
            }
            println!();
        }

        // Print subtasks recursively
        if !task.subtasks.is_empty() {
            let new_prefix = format!("{}{}", prefix, extension);
            for (i, subtask_map) in task.subtasks.iter().enumerate() {
                let is_last_subtask = i == task.subtasks.len() - 1;
                // Each subtask is a HashMap with one entry
                for subtask_id in subtask_map.keys() {
                    Box::pin(self.print_task_tree(
                        workflow,
                        subtask_id,
                        &new_prefix,
                        is_last_subtask,
                        visited,
                        executor,
                    ))
                    .await;
                }
            }
        }

        // Find and print dependent tasks (tasks that depend on this one)
        let dependents: Vec<_> = workflow
            .tasks
            .iter()
            .filter(|(_, t)| t.depends_on.contains(&task_id.to_string()))
            .map(|(id, _)| id.clone())
            .collect();

        if !dependents.is_empty() && task.subtasks.is_empty() {
            let new_prefix = format!("{}{}", prefix, extension);
            for (i, dep_id) in dependents.iter().enumerate() {
                if visited.contains(dep_id) {
                    continue;
                }
                let is_last_dep = i == dependents.len() - 1;
                Box::pin(self.print_task_tree(
                    workflow,
                    dep_id,
                    &new_prefix,
                    is_last_dep,
                    visited,
                    executor,
                ))
                .await;
            }
        }
    }

    /// Save workflow to YAML file
    async fn cmd_save_workflow(&self, path: &str) -> Result<()> {
        let executor = self.executor.lock().await;

        // Get workflow from executor
        let workflow = executor.workflow();

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&workflow).map_err(|e| {
            crate::error::Error::InvalidInput(format!("Failed to serialize workflow: {}", e))
        })?;

        // Write to file
        std::fs::write(path, yaml).map_err(|e| {
            crate::error::Error::InvalidInput(format!("Failed to write file: {}", e))
        })?;

        println!("{} {}", "✓ Workflow saved to:".green().bold(), path.cyan());
        Ok(())
    }

    /// Save REPL configuration
    async fn cmd_save_config(&self, path: Option<&str>) -> Result<()> {
        use serde::Serialize;

        #[derive(Serialize)]
        struct ReplConfig {
            ai_provider: Option<String>,
            ai_model: Option<String>,
            history_path: String,
        }

        let config = ReplConfig {
            ai_provider: self
                .ai_assistant
                .as_ref()
                .map(|ai| format!("{:?}", ai.config().provider)),
            ai_model: self
                .ai_assistant
                .as_ref()
                .map(|ai| ai.config().model.clone()),
            history_path: dirs::home_dir()
                .map(|mut p| {
                    p.push(".periplon_repl_history");
                    p.display().to_string()
                })
                .unwrap_or_else(|| ".periplon_repl_history".to_string()),
        };

        let config_path = path.unwrap_or(".periplon_repl_config.toml");

        // Serialize to TOML
        let toml = toml::to_string(&config).map_err(|e| {
            crate::error::Error::InvalidInput(format!("Failed to serialize config: {}", e))
        })?;

        // Write to file
        std::fs::write(config_path, toml).map_err(|e| {
            crate::error::Error::InvalidInput(format!("Failed to write config: {}", e))
        })?;

        println!(
            "{} {}",
            "✓ Configuration saved to:".green().bold(),
            config_path.cyan()
        );
        Ok(())
    }

    // ========================================================================
    // AI Commands
    // ========================================================================

    /// Generate workflow block from description
    async fn cmd_ai_generate(&self, description: &str) -> Result<()> {
        let ai = self.ai_assistant.as_ref().ok_or_else(|| {
            crate::error::Error::InvalidInput("AI assistant not available".to_string())
        })?;

        println!("🤖 Generating workflow block...");

        match ai.generate_block(description).await {
            Ok(yaml) => {
                println!("\n✓ Generated YAML:\n");
                println!("{}", yaml);
            }
            Err(e) => {
                println!("❌ Generation failed: {}", e);
            }
        }

        Ok(())
    }

    /// Get AI suggestion for fixing an error
    async fn cmd_ai_fix(&self, error: &str) -> Result<()> {
        let ai = self.ai_assistant.as_ref().ok_or_else(|| {
            crate::error::Error::InvalidInput("AI assistant not available".to_string())
        })?;

        println!("🤖 Analyzing error...");

        // Get current workflow context if available
        let context = self.get_workflow_context().await;

        match ai.suggest_fix(error, &context).await {
            Ok(suggestion) => {
                println!("\n✓ AI Suggestion:\n");
                println!("{}", suggestion);
            }
            Err(e) => {
                println!("❌ Analysis failed: {}", e);
            }
        }

        Ok(())
    }

    /// Analyze workflow with AI
    async fn cmd_ai_analyze(&self, workflow_path: Option<&str>) -> Result<()> {
        let ai = self.ai_assistant.as_ref().ok_or_else(|| {
            crate::error::Error::InvalidInput("AI assistant not available".to_string())
        })?;

        println!("🤖 Analyzing workflow...");

        // Load workflow YAML
        let yaml = if let Some(path) = workflow_path {
            match std::fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => {
                    println!("❌ Failed to read workflow file: {}", e);
                    return Ok(());
                }
            }
        } else {
            // Get current workflow from executor
            self.get_workflow_yaml().await
        };

        match ai.analyze_workflow(&yaml).await {
            Ok(analysis) => {
                println!("\n✓ Analysis:\n");
                println!("{}", analysis);
            }
            Err(e) => {
                println!("❌ Analysis failed: {}", e);
            }
        }

        Ok(())
    }

    /// Explain workflow with AI
    async fn cmd_ai_explain(&self, workflow_path: Option<&str>) -> Result<()> {
        let ai = self.ai_assistant.as_ref().ok_or_else(|| {
            crate::error::Error::InvalidInput("AI assistant not available".to_string())
        })?;

        println!("🤖 Explaining workflow...");

        // Load workflow YAML
        let yaml = if let Some(path) = workflow_path {
            match std::fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => {
                    println!("❌ Failed to read workflow file: {}", e);
                    return Ok(());
                }
            }
        } else {
            // Get current workflow from executor
            self.get_workflow_yaml().await
        };

        match ai.explain_workflow(&yaml).await {
            Ok(explanation) => {
                println!("\n✓ Explanation:\n");
                println!("{}", explanation);
            }
            Err(e) => {
                println!("❌ Explanation failed: {}", e);
            }
        }

        Ok(())
    }

    /// Change AI provider and model
    async fn cmd_ai_provider(&mut self, provider: &str, model: Option<&str>) -> Result<()> {
        use crate::dsl::debug_ai::AiProviderType;

        let provider_type = match provider.to_lowercase().as_str() {
            "ollama" => AiProviderType::Ollama,
            "openai" => AiProviderType::OpenAi,
            "anthropic" => AiProviderType::Anthropic,
            "google" => AiProviderType::Google,
            _ => {
                println!("❌ Unknown provider: {}", provider);
                println!("Available providers: ollama, openai, anthropic, google");
                return Ok(());
            }
        };

        let model_name = model.unwrap_or(match provider_type {
            AiProviderType::Ollama => "olmo2:13b",
            AiProviderType::OpenAi => "gpt-4o",
            AiProviderType::Anthropic => "claude-3-5-sonnet-20241022",
            AiProviderType::Google => "gemini-2.0-flash-exp",
        });

        if let Some(ref mut ai) = self.ai_assistant {
            match ai.set_provider(provider_type, model_name.to_string()) {
                Ok(()) => {
                    println!(
                        "✓ AI provider changed to {} (model: {})",
                        provider, model_name
                    );
                }
                Err(e) => {
                    println!("❌ Failed to change provider: {}", e);
                }
            }
        } else {
            println!("❌ AI assistant not available");
        }

        Ok(())
    }

    /// Show current AI configuration
    fn cmd_ai_config(&self) -> Result<()> {
        if let Some(ref ai) = self.ai_assistant {
            let config = ai.config();
            println!("AI Configuration:");
            println!("  Provider: {:?}", config.provider);
            println!("  Model: {}", config.model);
            if let Some(ref endpoint) = config.endpoint {
                println!("  Endpoint: {}", endpoint);
            } else {
                println!("  Endpoint: <default>");
            }
            if config.api_key.is_some() {
                println!("  API Key: <set>");
            } else {
                println!("  API Key: <from environment>");
            }
            println!("  Temperature: {}", config.temperature);
            println!("  Max Tokens: {}", config.max_tokens);

            if !config.extra_params.is_empty() {
                println!("  Extra Parameters:");
                for (key, value) in &config.extra_params {
                    println!("    {}: {}", key, value);
                }
            }
        } else {
            println!("❌ AI assistant not available");
        }
        Ok(())
    }

    /// Set AI temperature
    async fn cmd_ai_set_temperature(&mut self, temperature: f32) -> Result<()> {
        if let Some(ref mut ai) = self.ai_assistant {
            ai.set_temperature(temperature);
            println!("✓ AI temperature set to {}", temperature);
        } else {
            println!("❌ AI assistant not available");
        }
        Ok(())
    }

    /// Set AI max tokens
    async fn cmd_ai_set_max_tokens(&mut self, max_tokens: u32) -> Result<()> {
        if let Some(ref mut ai) = self.ai_assistant {
            ai.set_max_tokens(max_tokens);
            println!("✓ AI max tokens set to {}", max_tokens);
        } else {
            println!("❌ AI assistant not available");
        }
        Ok(())
    }

    /// Set AI endpoint
    async fn cmd_ai_set_endpoint(&mut self, endpoint: Option<&str>) -> Result<()> {
        if let Some(ref mut ai) = self.ai_assistant {
            match ai.set_endpoint(endpoint.map(|s| s.to_string())) {
                Ok(()) => {
                    if let Some(ep) = endpoint {
                        println!("✓ AI endpoint set to {}", ep);
                    } else {
                        println!("✓ AI endpoint cleared (using default)");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to set endpoint: {}", e);
                }
            }
        } else {
            println!("❌ AI assistant not available");
        }
        Ok(())
    }

    /// Set AI API key
    async fn cmd_ai_set_api_key(&mut self, api_key: Option<&str>) -> Result<()> {
        if let Some(ref mut ai) = self.ai_assistant {
            match ai.set_api_key(api_key.map(|s| s.to_string())) {
                Ok(()) => {
                    if api_key.is_some() {
                        println!("✓ AI API key set");
                    } else {
                        println!("✓ AI API key cleared (using environment variable)");
                    }
                }
                Err(e) => {
                    println!("❌ Failed to set API key: {}", e);
                }
            }
        } else {
            println!("❌ AI assistant not available");
        }
        Ok(())
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Get current workflow context as string
    async fn get_workflow_context(&self) -> String {
        let executor = self.executor.lock().await;
        if let Some(inspector) = executor.inspector() {
            format!(
                "Current Task: {:?}\nCall Stack: {}",
                inspector.current_position().await.current_task,
                inspector.call_stack_string().await
            )
        } else {
            "(no context available)".to_string()
        }
    }

    /// Get workflow YAML representation
    async fn get_workflow_yaml(&self) -> String {
        let executor = self.executor.lock().await;
        let workflow = executor.workflow();

        match serde_yaml::to_string(workflow) {
            Ok(yaml) => yaml,
            Err(e) => {
                eprintln!("Warning: Failed to serialize workflow: {}", e);
                "(failed to serialize current workflow)".to_string()
            }
        }
    }

    /// Build completion context from current executor state
    async fn build_completion_context(&self) -> CompletionContext {
        let executor = self.executor.lock().await;
        let debugger = self.debugger.lock().await;

        let mut ctx = CompletionContext::default();

        // Get task names from workflow
        ctx.task_names = executor.workflow().tasks.keys().cloned().collect();
        ctx.task_names.sort();

        // Get agent names from workflow
        ctx.agent_names = executor.workflow().agents.keys().cloned().collect();
        ctx.agent_names.sort();

        // Get variable names from inspector if available
        if let Some(inspector) = executor.inspector() {
            let vars = inspector.inspect_variables(None).await;

            // Collect all variable names from all scopes
            ctx.variable_names.extend(vars.workflow_vars.keys().cloned());
            ctx.variable_names.extend(vars.agent_vars.keys().cloned());
            ctx.variable_names.extend(vars.task_vars.keys().cloned());
            ctx.variable_names.extend(vars.loop_vars.keys().cloned());
            ctx.variable_names.sort();
            ctx.variable_names.dedup();
        }

        // Get breakpoint IDs
        ctx.breakpoint_ids = debugger
            .breakpoints
            .list_all()
            .iter()
            .map(|bp| bp.id.clone())
            .collect();
        ctx.breakpoint_ids.sort();

        ctx
    }
}
