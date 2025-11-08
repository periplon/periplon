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
pub use completer::ReplHelper;
pub use parser::parse_command;

use crate::dsl::debug_ai::DebugAiAssistant;
use crate::dsl::debugger::{DebugMode, DebuggerState, StepMode};
use crate::dsl::DSLExecutor;
use crate::error::Result;
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use std::io::{self, Write};
use std::sync::Arc;
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
        println!("🐛 Periplon Debug REPL");
        println!("Type 'help' for available commands, 'quit' to exit");
        println!("Use TAB for command completion, ↑/↓ for history\n");

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
                        eprintln!("❌ Error: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("❌ Parse error: {}", e);
                }
            }
        }

        // Save history to file on exit
        if let Err(e) = rl.save_history(&history_path) {
            eprintln!("⚠️  Failed to save command history: {}", e);
        }

        println!("👋 Goodbye!");
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

            // AI Commands
            ReplCommand::AiGenerate { description } => self.cmd_ai_generate(&description).await,
            ReplCommand::AiFix { error } => self.cmd_ai_fix(&error).await,
            ReplCommand::AiAnalyze { workflow } => self.cmd_ai_analyze(workflow.as_deref()).await,
            ReplCommand::AiExplain { workflow } => self.cmd_ai_explain(workflow.as_deref()).await,
            ReplCommand::AiProvider { provider, model } => {
                self.cmd_ai_provider(&provider, model.as_deref()).await
            }
            ReplCommand::AiConfig => self.cmd_ai_config(),
        }
    }

    /// Continue execution
    async fn cmd_continue(&self) -> Result<()> {
        let mut dbg = self.debugger.lock().await;
        dbg.resume();
        println!("▶️  Execution resumed");
        Ok(())
    }

    /// Step execution
    async fn cmd_step(&self, mode: StepMode) -> Result<()> {
        let mut dbg = self.debugger.lock().await;
        dbg.set_step_mode(mode);
        dbg.resume();
        println!("👣 Stepping ({:?})...", mode);
        Ok(())
    }

    /// Pause execution
    async fn cmd_pause(&self) -> Result<()> {
        let mut dbg = self.debugger.lock().await;
        dbg.pause();
        println!("⏸️  Execution paused");
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

        println!("✓ Breakpoint set: {}", id);
        Ok(())
    }

    /// List breakpoints
    async fn cmd_list_breaks(&self) -> Result<()> {
        let dbg = self.debugger.lock().await;
        let breakpoints = dbg.breakpoints.list_all();

        if breakpoints.is_empty() {
            println!("No breakpoints set");
        } else {
            println!("Breakpoints:");
            for bp in breakpoints {
                let status = if bp.enabled { "enabled" } else { "disabled" };
                println!(
                    "  {} [{}] {} (hits: {})",
                    bp.id, status, bp.description, bp.hit_count
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
                println!("✓ Breakpoint deleted: {}", id);
            } else {
                println!("❌ Breakpoint not found: {}", id);
            }
        } else if dbg.breakpoints.remove_conditional_breakpoint(id) {
            println!("✓ Breakpoint deleted: {}", id);
        } else {
            println!("❌ Breakpoint not found: {}", id);
        }

        Ok(())
    }

    /// Show variables
    async fn cmd_vars(&self, _scope: Option<crate::dsl::debugger::VariableScope>) -> Result<()> {
        let executor = self.executor.lock().await;
        if let Some(inspector) = executor.inspector() {
            let vars = inspector.inspect_variables(None).await;

            println!("Variables:");
            if !vars.workflow_vars.is_empty() {
                println!("  Workflow:");
                for (name, value) in &vars.workflow_vars {
                    println!("    {} = {:?}", name, value);
                }
            }

            if !vars.task_vars.is_empty() {
                println!("  Tasks:");
                for (task_id, task_vars) in &vars.task_vars {
                    println!("    {}:", task_id);
                    for (name, value) in task_vars {
                        println!("      {} = {:?}", name, value);
                    }
                }
            }

            if vars.total_count() == 0 {
                println!("  (no variables)");
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
            println!("Help for '{}':", cmd);
            println!("  (specific command help not yet implemented)");
        } else {
            // Show general help
            println!("Available Commands:\n");

            for category in commands::CommandCategory::all_categories() {
                println!("{}:", category.name());
                for cmd in category.commands() {
                    println!("  {}", cmd);
                }
                println!();
            }

            println!("Type 'help <command>' for more information on a specific command.");
        }
        Ok(())
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
                        println!("❌ Task not found: {}", task_id);
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
            AiProviderType::Ollama => "llama3.3",
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
        // TODO: Implement workflow serialization
        // For now, return a placeholder
        "(current workflow YAML not available - specify a file path)".to_string()
    }
}
