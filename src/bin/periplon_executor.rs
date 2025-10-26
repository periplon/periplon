//! DSL Workflow Executor CLI
//!
//! Command-line tool for executing DSL workflows with support for:
//! - Workflow validation
//! - State persistence and resume
//! - Progress tracking
//! - Verbose output
//! - JSON output with syntax coloring

use clap::{Parser, Subcommand};
use colored::*;
use periplon_sdk::dsl::{
    generate_and_save, generate_template, parse_workflow_file, validate_workflow, DSLExecutor,
    StatePersistence, DSL_GRAMMAR_VERSION,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Instant;

/// DSL Workflow Executor
#[derive(Parser)]
#[command(name = "dsl-executor")]
#[command(about = "Execute multi-agent DSL workflows", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

// JSON output structures
#[derive(Debug, Serialize, Deserialize)]
struct RunOutput {
    success: bool,
    workflow_name: String,
    workflow_version: String,
    duration_secs: f64,
    progress: f64,
    completed_tasks: usize,
    total_tasks: usize,
    failed_tasks: Vec<String>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ValidateOutput {
    valid: bool,
    workflow_name: String,
    workflow_version: String,
    agents_count: usize,
    tasks_count: usize,
    agents: Option<Vec<AgentInfo>>,
    tasks: Option<Vec<TaskInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentInfo {
    name: String,
    description: String,
    tools: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskInfo {
    name: String,
    description: String,
    depends_on: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ListOutput {
    state_dir: String,
    workflows: Vec<WorkflowInfo>,
    total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowInfo {
    name: String,
    version: String,
    status: String,
    progress: f64,
    completed_tasks: usize,
    total_tasks: usize,
    finished: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct StatusOutput {
    name: String,
    version: String,
    status: String,
    progress: f64,
    total_tasks: usize,
    completed_tasks: usize,
    failed_tasks: Vec<FailedTask>,
    started_at: String,
    ended_at: Option<String>,
    duration_secs: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FailedTask {
    name: String,
    error: Option<String>,
}

/// Helper function to print colored JSON
fn print_json<T: Serialize>(value: &T) -> Result<(), Box<dyn std::error::Error>> {
    let json_value = serde_json::to_value(value)?;
    let colored = colored_json::to_colored_json_auto(&json_value)?;
    println!("{}", colored);
    Ok(())
}

#[derive(Subcommand)]
enum Commands {
    /// Manage task groups
    Group {
        #[command(subcommand)]
        group_command: GroupCommands,
    },

    /// Execute a workflow from a YAML file
    Run {
        /// Path to the workflow YAML file
        #[arg(value_name = "WORKFLOW_FILE")]
        workflow_file: PathBuf,

        /// Directory to store workflow state (default: .workflow_states)
        #[arg(short, long, value_name = "DIR")]
        state_dir: Option<PathBuf>,

        /// Resume from saved state if available
        #[arg(short, long)]
        resume: bool,

        /// Clean state before execution (delete existing state)
        #[arg(short, long)]
        clean: bool,

        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Validate workflow without executing
        #[arg(long)]
        dry_run: bool,

        /// Output results in JSON format with syntax coloring
        #[arg(short, long)]
        json: bool,

        /// Input variables as key=value pairs (can be specified multiple times)
        /// Example: -i name=John -i age=30 -i config='{"key":"value"}'
        #[arg(short = 'i', long = "input", value_name = "KEY=VALUE")]
        inputs: Vec<String>,
    },

    /// Validate a workflow file without executing
    Validate {
        /// Path to the workflow YAML file
        #[arg(value_name = "WORKFLOW_FILE")]
        workflow_file: PathBuf,

        /// Show detailed validation information
        #[arg(short, long)]
        verbose: bool,

        /// Output results in JSON format with syntax coloring
        #[arg(short, long)]
        json: bool,
    },

    /// List saved workflow states
    List {
        /// Directory containing workflow states (default: .workflow_states)
        #[arg(short, long, value_name = "DIR")]
        state_dir: Option<PathBuf>,

        /// Output results in JSON format with syntax coloring
        #[arg(short, long)]
        json: bool,
    },

    /// Clean saved workflow states
    Clean {
        /// Workflow name to clean (omit to clean all)
        #[arg(value_name = "WORKFLOW_NAME")]
        workflow_name: Option<String>,

        /// Directory containing workflow states (default: .workflow_states)
        #[arg(short, long, value_name = "DIR")]
        state_dir: Option<PathBuf>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Show workflow status and progress
    Status {
        /// Workflow name
        #[arg(value_name = "WORKFLOW_NAME")]
        workflow_name: String,

        /// Directory containing workflow states (default: .workflow_states)
        #[arg(short, long, value_name = "DIR")]
        state_dir: Option<PathBuf>,

        /// Output results in JSON format with syntax coloring
        #[arg(short, long)]
        json: bool,
    },

    /// Start the HTTP/WebSocket server
    #[cfg(feature = "server")]
    Server {
        /// Server port
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Configuration file path
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Also start workers in same process
        #[arg(long)]
        workers: bool,

        /// Number of worker threads when --workers is enabled
        #[arg(long, default_value = "3")]
        worker_concurrency: usize,
    },

    /// Start a background worker
    #[cfg(feature = "server")]
    Worker {
        /// Number of concurrent jobs to process
        #[arg(short, long, default_value = "3")]
        concurrency: usize,

        /// Configuration file path
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Worker ID (auto-generated if not specified)
        #[arg(long)]
        worker_id: Option<String>,
    },

    /// Run database migrations
    #[cfg(feature = "server")]
    Migrate {
        /// Configuration file path
        #[arg(short, long, value_name = "FILE")]
        config: Option<PathBuf>,

        /// Migration action: up (default), down, status
        #[arg(long, default_value = "up")]
        action: String,
    },

    /// Generate a DSL template with documentation
    Template {
        /// Output file path (prints to stdout if not specified)
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Generate DSL workflow from natural language description
    ///
    /// Tip: Give tasks enough context so they can be executed in new conversations
    /// and are aware of previous achievements and the whole workflow.
    Generate {
        /// Natural language description of the workflow or modifications to make
        /// (can be omitted if --file is provided)
        #[arg(value_name = "DESCRIPTION")]
        description: Option<String>,

        /// File containing natural language description
        #[arg(short, long, value_name = "FILE")]
        file: Option<PathBuf>,

        /// Output file path (required)
        #[arg(short, long, value_name = "FILE")]
        output: PathBuf,

        /// Existing workflow file to modify (optional - omit to create new workflow)
        #[arg(short = 'w', long = "workflow", value_name = "FILE")]
        workflow: Option<PathBuf>,

        /// Enable verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },

    /// Show DSL grammar version
    Version {},
}

#[derive(Subcommand)]
enum GroupCommands {
    /// List all available task groups
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,

        /// Output results in JSON format with syntax coloring
        #[arg(short, long)]
        json: bool,

        /// Search directory (default: standard search paths)
        #[arg(short, long, value_name = "DIR")]
        path: Option<PathBuf>,
    },

    /// Install (validate) a task group
    Install {
        /// Task group reference (e.g., "my-group@1.0.0")
        #[arg(value_name = "GROUP_REF")]
        group_ref: String,

        /// Show detailed installation steps
        #[arg(short, long)]
        verbose: bool,

        /// Output results in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Update task group cache and information
    Update {
        /// Specific group to update (omit to update all)
        #[arg(value_name = "GROUP_REF")]
        group_ref: Option<String>,

        /// Force cache refresh
        #[arg(short, long)]
        force: bool,

        /// Show detailed update information
        #[arg(short, long)]
        verbose: bool,

        /// Output results in JSON format
        #[arg(short, long)]
        json: bool,
    },

    /// Validate a task group file
    Validate {
        /// Path to the task group file
        #[arg(value_name = "GROUP_FILE")]
        group_file: PathBuf,

        /// Show detailed validation information
        #[arg(short, long)]
        verbose: bool,

        /// Output results in JSON format
        #[arg(short, long)]
        json: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Group { group_command } => match group_command {
            GroupCommands::List {
                verbose,
                json,
                path,
            } => group_list(verbose, json, path).await,
            GroupCommands::Install {
                group_ref,
                verbose,
                json,
            } => group_install(&group_ref, verbose, json).await,
            GroupCommands::Update {
                group_ref,
                force,
                verbose,
                json,
            } => group_update(group_ref.as_deref(), force, verbose, json).await,
            GroupCommands::Validate {
                group_file,
                verbose,
                json,
            } => group_validate(group_file, verbose, json).await,
        },
        Commands::Run {
            workflow_file,
            state_dir,
            resume,
            clean,
            verbose,
            dry_run,
            json,
            inputs,
        } => {
            run_workflow(
                workflow_file,
                state_dir,
                resume,
                clean,
                verbose,
                dry_run,
                json,
                inputs,
            )
            .await
        }
        Commands::Validate {
            workflow_file,
            verbose,
            json,
        } => validate_workflow_cmd(workflow_file, verbose, json).await,
        Commands::List { state_dir, json } => list_states(state_dir, json).await,
        Commands::Clean {
            workflow_name,
            state_dir,
            yes,
        } => clean_states(workflow_name, state_dir, yes).await,
        Commands::Status {
            workflow_name,
            state_dir,
            json,
        } => show_status(workflow_name, state_dir, json).await,
        #[cfg(feature = "server")]
        Commands::Server {
            port,
            config,
            workers,
            worker_concurrency,
        } => start_server(port, config, workers, worker_concurrency).await,
        #[cfg(feature = "server")]
        Commands::Worker {
            concurrency,
            config,
            worker_id,
        } => start_worker(concurrency, config, worker_id).await,
        #[cfg(feature = "server")]
        Commands::Migrate { config, action } => run_migrations(config, action).await,
        Commands::Template { output } => generate_template_cmd(output).await,
        Commands::Generate {
            description,
            file,
            output,
            workflow,
            verbose,
        } => generate_from_nl_cmd(description, file, output, workflow, verbose).await,
        Commands::Version {} => show_version().await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

/// Run a workflow
/// Parse input variables from key=value strings
fn parse_input_variables(
    inputs: Vec<String>,
) -> Result<std::collections::HashMap<String, serde_json::Value>, Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    let mut result = HashMap::new();

    for input in inputs {
        let parts: Vec<&str> = input.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid input format: '{}'. Expected KEY=VALUE", input).into());
        }

        let key = parts[0].to_string();
        let value_str = parts[1];

        // Try to parse as JSON first, fall back to string
        let value = if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(value_str) {
            json_val
        } else {
            // If it's not valid JSON, treat as string
            serde_json::Value::String(value_str.to_string())
        };

        result.insert(key, value);
    }

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
async fn run_workflow(
    workflow_file: PathBuf,
    state_dir: Option<PathBuf>,
    resume: bool,
    clean: bool,
    verbose: bool,
    dry_run: bool,
    json: bool,
    cli_inputs: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();

    // Print header (skip if JSON mode)
    if !json {
        println!("{}", "=".repeat(60).cyan());
        println!("{}", "DSL Workflow Executor".cyan().bold());
        println!("{}", "=".repeat(60).cyan());
        println!();
    }

    // Parse workflow
    if !json {
        print!("{}  ", "Parsing workflow...".bold());
    }
    let mut workflow = parse_workflow_file(&workflow_file)?;
    if !json {
        println!("{}", "✓".green().bold());
    }

    // Parse and merge CLI inputs
    if !cli_inputs.is_empty() {
        let parsed_inputs = parse_input_variables(cli_inputs)?;

        if verbose && !json {
            println!(
                "  {} Setting {} CLI input(s)",
                "Inputs:".dimmed(),
                parsed_inputs.len()
            );
        }

        // Merge CLI inputs with workflow inputs (set as default values)
        for (key, value) in parsed_inputs {
            if let Some(input_spec) = workflow.inputs.get_mut(&key) {
                // Override the default value
                input_spec.default = Some(value.clone());
                if verbose && !json {
                    println!("    {} = {:?}", key.bright_white(), value);
                }
            } else if verbose && !json {
                // Warn if input not defined in workflow
                println!(
                    "    {} {} = {:?} (not defined in workflow)",
                    "⚠".yellow(),
                    key.yellow(),
                    value
                );
            }
        }
    }

    if verbose && !json {
        println!(
            "  {} {}",
            "Workflow:".dimmed(),
            workflow.name.bright_white()
        );
        println!(
            "  {} {}",
            "Version:".dimmed(),
            workflow.version.bright_white()
        );
        println!("  {} {}", "Agents:".dimmed(), workflow.agents.len());
        println!("  {} {}", "Tasks:".dimmed(), workflow.tasks.len());
        println!();
    }

    // Validate workflow
    if !json {
        print!("{}  ", "Validating workflow...".bold());
    }
    validate_workflow(&workflow)?;
    if !json {
        println!("{}", "✓".green().bold());
    }

    if dry_run {
        if json {
            let output = ValidateOutput {
                valid: true,
                workflow_name: workflow.name.clone(),
                workflow_version: workflow.version.clone(),
                agents_count: workflow.agents.len(),
                tasks_count: workflow.tasks.len(),
                agents: None,
                tasks: None,
            };
            print_json(&output)?;
        } else {
            println!();
            println!("{} Workflow is valid (dry-run mode)", "✓".green().bold());
        }
        return Ok(());
    }

    if !json {
        println!();
    }

    // Create executor
    let mut executor = DSLExecutor::new(workflow.clone())?;

    // Set output mode
    executor.set_json_output(json);

    // Enable state persistence if requested
    let state_dir_str = state_dir.as_ref().map(|p| p.to_string_lossy().to_string());
    let enable_state = resume || clean || state_dir.is_some();

    if enable_state {
        executor.enable_state_persistence(state_dir_str.as_deref())?;

        // Clean state if requested
        if clean {
            let persistence =
                StatePersistence::new(state_dir_str.as_deref().unwrap_or(".workflow_states"))?;
            if persistence.has_state(&workflow.name) {
                persistence.delete_state(&workflow.name)?;
                if !json {
                    println!("{} Cleaned existing state", "✓".green().bold());
                    println!();
                }
            }
        }

        // Try to resume
        if resume && !clean {
            let resumed = executor.try_resume()?;
            if resumed && !json {
                if let Some(state) = executor.get_state() {
                    println!("{} Resuming from checkpoint", "→".yellow().bold());
                    println!("  Progress: {:.1}%", state.get_progress() * 100.0);
                    println!(
                        "  Completed: {}/{}",
                        state.get_completed_task_count(),
                        state.get_total_task_count()
                    );
                    println!();
                }
            }
        }
    }

    // Initialize
    if !json {
        print!("{}  ", "Initializing workflow...".bold());
    }
    executor.initialize().await?;
    if !json {
        println!("{}", "✓".green().bold());
        println!();
    }

    // Execute
    if !json {
        println!("{}", "Executing workflow...".bold());
        println!("{}", "-".repeat(60).dimmed());
        println!();
    }

    let exec_result = executor.execute().await;

    if !json {
        println!();
        println!("{}", "-".repeat(60).dimmed());
    }

    // Shutdown
    if !json {
        print!("{}  ", "Shutting down...".bold());
    }
    executor.shutdown().await?;
    if !json {
        println!("{}", "✓".green().bold());
    }

    // Handle execution result
    match exec_result {
        Ok(()) => {
            let duration = start_time.elapsed();

            if json {
                // JSON output
                let state = executor.get_state();
                let output = RunOutput {
                    success: true,
                    workflow_name: workflow.name.clone(),
                    workflow_version: workflow.version.clone(),
                    duration_secs: duration.as_secs_f64(),
                    progress: state.as_ref().map(|s| s.get_progress()).unwrap_or(1.0),
                    completed_tasks: state
                        .as_ref()
                        .map(|s| s.get_completed_tasks().len())
                        .unwrap_or(0),
                    total_tasks: state.as_ref().map(|s| s.task_statuses.len()).unwrap_or(0),
                    failed_tasks: vec![],
                    error: None,
                };
                print_json(&output)?;
            } else {
                // Regular output
                println!();
                println!("{} Workflow completed successfully!", "✓".green().bold());
                println!("  Duration: {:.2}s", duration.as_secs_f64());

                // Show final progress if state is available
                if let Some(state) = executor.get_state() {
                    println!(
                        "  Completed: {}/{}",
                        state.get_completed_task_count(),
                        state.get_total_task_count()
                    );
                }
            }

            Ok(())
        }
        Err(e) => {
            let duration = start_time.elapsed();

            if json {
                // JSON output
                let state = executor.get_state();
                let output = RunOutput {
                    success: false,
                    workflow_name: workflow.name.clone(),
                    workflow_version: workflow.version.clone(),
                    duration_secs: duration.as_secs_f64(),
                    progress: state.as_ref().map(|s| s.get_progress()).unwrap_or(0.0),
                    completed_tasks: state
                        .as_ref()
                        .map(|s| s.get_completed_tasks().len())
                        .unwrap_or(0),
                    total_tasks: state.as_ref().map(|s| s.task_statuses.len()).unwrap_or(0),
                    failed_tasks: state
                        .as_ref()
                        .map(|s| s.get_failed_tasks())
                        .unwrap_or_default(),
                    error: Some(e.to_string()),
                };
                print_json(&output)?;
            } else {
                // Regular output
                println!();
                println!("{} Workflow failed!", "✗".red().bold());
                println!("  Error: {}", e);

                // Show progress even on failure
                if let Some(state) = executor.get_state() {
                    println!("  Progress: {:.1}%", state.get_progress() * 100.0);
                    println!(
                        "  Completed: {}/{}",
                        state.get_completed_task_count(),
                        state.get_total_task_count()
                    );
                    if !state.get_failed_tasks().is_empty() {
                        println!("  Failed: {}", state.get_failed_tasks().join(", "));
                    }
                }
            }

            Err(e.into())
        }
    }
}

/// Validate a workflow file
async fn validate_workflow_cmd(
    workflow_file: PathBuf,
    verbose: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !json {
        println!("{}", "Validating workflow...".bold());
        println!();
    }

    // Parse workflow
    if !json {
        print!("  Parsing YAML...  ");
    }
    let workflow = parse_workflow_file(&workflow_file)?;
    if !json {
        println!("{}", "✓".green().bold());
    }

    if verbose && !json {
        println!("    {} {}", "Workflow:".dimmed(), workflow.name);
        println!("    {} {}", "Version:".dimmed(), workflow.version);
    }

    // Validate workflow
    if !json {
        print!("  Checking semantics...  ");
    }
    validate_workflow(&workflow)?;
    if !json {
        println!("{}", "✓".green().bold());
    }

    if json {
        // JSON output
        let agents = if verbose {
            Some(
                workflow
                    .agents
                    .iter()
                    .map(|(name, spec)| AgentInfo {
                        name: name.clone(),
                        description: spec.description.clone(),
                        tools: spec.tools.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        let tasks = if verbose {
            Some(
                workflow
                    .tasks
                    .iter()
                    .map(|(name, spec)| TaskInfo {
                        name: name.clone(),
                        description: spec.description.clone(),
                        depends_on: spec.depends_on.clone(),
                    })
                    .collect(),
            )
        } else {
            None
        };

        let output = ValidateOutput {
            valid: true,
            workflow_name: workflow.name.clone(),
            workflow_version: workflow.version.clone(),
            agents_count: workflow.agents.len(),
            tasks_count: workflow.tasks.len(),
            agents,
            tasks,
        };
        print_json(&output)?;
    } else {
        if verbose {
            println!("    {} {}", "Agents:".dimmed(), workflow.agents.len());
            println!("    {} {}", "Tasks:".dimmed(), workflow.tasks.len());

            // Show agents
            if !workflow.agents.is_empty() {
                println!();
                println!("  {}:", "Agents".bold());
                for (name, spec) in &workflow.agents {
                    println!("    • {} - {}", name.bright_white(), spec.description);
                    println!("      Tools: {}", spec.tools.join(", ").dimmed());
                }
            }

            // Show tasks
            if !workflow.tasks.is_empty() {
                println!();
                println!("  {}:", "Tasks".bold());
                for (name, spec) in &workflow.tasks {
                    println!("    • {} - {}", name.bright_white(), spec.description);
                    if !spec.depends_on.is_empty() {
                        println!("      Depends on: {}", spec.depends_on.join(", ").dimmed());
                    }
                }
            }
        }

        println!();
        println!("{} Workflow is valid", "✓".green().bold());
    }

    Ok(())
}

/// List saved workflow states
async fn list_states(
    state_dir: Option<PathBuf>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = state_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".workflow_states".to_string());

    let persistence = StatePersistence::new(&dir)?;
    let workflows = persistence.list_states()?;

    if json {
        // JSON output
        let mut workflow_infos = Vec::new();
        for workflow_name in &workflows {
            if let Ok(state) = persistence.load_state(workflow_name) {
                workflow_infos.push(WorkflowInfo {
                    name: state.workflow_name.clone(),
                    version: state.workflow_version.clone(),
                    status: format!("{:?}", state.status),
                    progress: state.get_progress(),
                    completed_tasks: state.get_completed_tasks().len(),
                    total_tasks: state.task_statuses.len(),
                    finished: state.ended_at.is_some(),
                });
            }
        }

        let output = ListOutput {
            state_dir: dir.clone(),
            workflows: workflow_infos,
            total: workflows.len(),
        };
        print_json(&output)?;
    } else {
        // Regular output
        if workflows.is_empty() {
            println!("{}", "No saved workflow states found".dimmed());
            println!("  Directory: {}", dir.dimmed());
            return Ok(());
        }

        println!("{}", "Saved Workflow States:".bold());
        println!("  Directory: {}", dir.dimmed());
        println!();

        for workflow_name in &workflows {
            // Try to load state to show details
            match persistence.load_state(workflow_name) {
                Ok(state) => {
                    println!("  {} {}", "•".cyan(), workflow_name.bright_white());
                    println!("    Status: {}", format!("{:?}", state.status).yellow());
                    println!("    Progress: {:.1}%", state.get_progress() * 100.0);
                    println!(
                        "    Tasks: {}/{}",
                        state.get_completed_tasks().len(),
                        state.task_statuses.len()
                    );
                    if state.ended_at.is_some() {
                        println!("    {}", "Finished".green());
                    } else {
                        println!("    {}", "In Progress".yellow());
                    }
                    println!();
                }
                Err(_) => {
                    println!("  {} {}", "•".cyan(), workflow_name);
                    println!("    {}", "(Unable to load state)".red());
                    println!();
                }
            }
        }

        println!("Total: {} workflows", workflows.len());
    }

    Ok(())
}

/// Clean saved workflow states
async fn clean_states(
    workflow_name: Option<String>,
    state_dir: Option<PathBuf>,
    yes: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = state_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".workflow_states".to_string());

    let persistence = StatePersistence::new(&dir)?;

    if let Some(name) = workflow_name {
        // Clean specific workflow
        if !persistence.has_state(&name) {
            println!(
                "{} No state found for workflow '{}'",
                "!".yellow().bold(),
                name
            );
            return Ok(());
        }

        if !yes {
            print!(
                "Delete state for workflow '{}'? [y/N] ",
                name.bright_white()
            );
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled");
                return Ok(());
            }
        }

        persistence.delete_state(&name)?;
        println!("{} Deleted state for '{}'", "✓".green().bold(), name);
    } else {
        // Clean all workflows
        let workflows = persistence.list_states()?;

        if workflows.is_empty() {
            println!("{}", "No saved states to clean".dimmed());
            return Ok(());
        }

        if !yes {
            println!("Delete {} saved workflow states? [y/N] ", workflows.len());
            use std::io::{self, Write};
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled");
                return Ok(());
            }
        }

        for name in &workflows {
            persistence.delete_state(name)?;
        }

        println!(
            "{} Deleted {} workflow states",
            "✓".green().bold(),
            workflows.len()
        );
    }

    Ok(())
}

/// Show workflow status
async fn show_status(
    workflow_name: String,
    state_dir: Option<PathBuf>,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = state_dir
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".workflow_states".to_string());

    let persistence = StatePersistence::new(&dir)?;

    if !persistence.has_state(&workflow_name) {
        if json {
            let error_output = serde_json::json!({
                "error": format!("No state found for workflow '{}'", workflow_name)
            });
            print_json(&error_output)?;
        } else {
            println!(
                "{} No state found for workflow '{}'",
                "!".yellow().bold(),
                workflow_name
            );
        }
        return Ok(());
    }

    let state = persistence.load_state(&workflow_name)?;

    if json {
        // JSON output
        let failed_tasks: Vec<FailedTask> = state
            .get_failed_tasks()
            .into_iter()
            .map(|task| FailedTask {
                name: task.clone(),
                error: state.task_errors.get(&task).cloned(),
            })
            .collect();

        let duration_secs = state.ended_at.and_then(|ended| {
            ended
                .duration_since(state.started_at)
                .ok()
                .map(|d| d.as_secs_f64())
        });

        let output = StatusOutput {
            name: state.workflow_name.clone(),
            version: state.workflow_version.clone(),
            status: format!("{:?}", state.status),
            progress: state.get_progress(),
            total_tasks: state.task_statuses.len(),
            completed_tasks: state.get_completed_tasks().len(),
            failed_tasks,
            started_at: format!("{:?}", state.started_at),
            ended_at: state.ended_at.map(|t| format!("{:?}", t)),
            duration_secs,
        };
        print_json(&output)?;
    } else {
        // Regular output
        println!("{}", "Workflow Status".bold());
        println!("{}", "=".repeat(60).dimmed());
        println!();

        println!("  {} {}", "Name:".bold(), state.workflow_name);
        println!("  {} {}", "Version:".bold(), state.workflow_version);
        println!(
            "  {} {}",
            "Status:".bold(),
            format!("{:?}", state.status).yellow()
        );
        println!();

        println!(
            "  {} {:.1}%",
            "Progress:".bold(),
            state.get_progress() * 100.0
        );
        println!(
            "  {} {}",
            "Total Tasks:".bold(),
            state.get_total_task_count()
        );
        println!(
            "  {} {}",
            "Completed:".bold(),
            state.get_completed_task_count().to_string().green()
        );
        println!(
            "  {} {}",
            "Failed:".bold(),
            if state.get_failed_tasks().is_empty() {
                "0".green()
            } else {
                state.get_failed_tasks().len().to_string().red()
            }
        );
        println!();

        // Show failed tasks if any
        if !state.get_failed_tasks().is_empty() {
            println!("  {}:", "Failed Tasks".red().bold());
            for task in state.get_failed_tasks() {
                println!("    • {}", task);
                if let Some(error) = state.task_errors.get(&task) {
                    println!("      Error: {}", error.dimmed());
                }
            }
            println!();
        }

        // Show timing
        println!("  {} {:?}", "Started:".bold(), state.started_at);
        if let Some(ended) = state.ended_at {
            println!("  {} {:?}", "Ended:".bold(), ended);
            if let Ok(duration) = ended.duration_since(state.started_at) {
                println!("  {} {:.2}s", "Duration:".bold(), duration.as_secs_f64());
            }
        } else {
            println!("  {} {}", "Ended:".bold(), "In Progress".yellow());
        }
    }

    Ok(())
}

/// Generate a DSL template with documentation
async fn generate_template_cmd(output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let template = generate_template();

    if let Some(path) = output {
        std::fs::write(&path, &template)?;
        println!(
            "{} Template written to: {}",
            "✓".green().bold(),
            path.display()
        );
        println!("  DSL Grammar Version: {}", DSL_GRAMMAR_VERSION);
    } else {
        // Print to stdout
        println!("{}", template);
    }

    Ok(())
}

/// Generate DSL workflow from natural language
async fn generate_from_nl_cmd(
    description: Option<String>,
    file: Option<PathBuf>,
    output: PathBuf,
    existing_workflow: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate that at least one input method is provided
    if description.is_none() && file.is_none() {
        return Err("Either DESCRIPTION or --file must be provided".into());
    }

    // If both are provided, prefer the file
    let description_text = if let Some(file_path) = file {
        if verbose {
            println!(
                "  Reading description from file: {}",
                file_path.display().to_string().dimmed()
            );
        }
        std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Failed to read file '{}': {}", file_path.display(), e))?
    } else {
        description.unwrap()
    };

    let is_modification = existing_workflow.is_some();

    if verbose {
        if is_modification {
            println!("{}", "Modifying existing DSL workflow...".bold());
            println!(
                "  Input: {}",
                existing_workflow
                    .as_ref()
                    .unwrap()
                    .display()
                    .to_string()
                    .dimmed()
            );
        } else {
            println!("{}", "Generating DSL from natural language...".bold());
        }
        println!("  Description: {}", description_text.dimmed());
        println!();
    }

    // Generate or modify the workflow
    if verbose {
        if is_modification {
            print!("  Loading existing workflow...  ");
        } else {
            print!("  Analyzing description...  ");
        }
    }

    let workflow = if let Some(ref workflow_path) = existing_workflow {
        // Read existing workflow
        let existing_content = std::fs::read_to_string(workflow_path)?;

        if verbose {
            println!("{}", "✓".green().bold());
            print!("  Applying modifications...  ");
        }

        // Generate modified workflow using NL generator
        use periplon_sdk::dsl::nl_generator::modify_workflow_from_nl;
        let modified = modify_workflow_from_nl(&description_text, &existing_content, None).await?;

        // Save to output file
        use periplon_sdk::dsl::parser::write_workflow_file;
        write_workflow_file(&modified, output.to_str().unwrap())?;

        modified
    } else {
        // Generate new workflow
        generate_and_save(&description_text, output.to_str().unwrap(), None).await?
    };

    if verbose {
        println!("{}", "✓".green().bold());
        print!("  Validating workflow...  ");
    }

    // Validate the generated/modified workflow
    validate_workflow(&workflow)?;

    if verbose {
        println!("{}", "✓".green().bold());
    }

    println!();
    if is_modification {
        println!("{} Workflow modified successfully!", "✓".green().bold());
    } else {
        println!("{} Workflow generated successfully!", "✓".green().bold());
    }
    println!("  Output: {}", output.display());
    println!("  Workflow: {} v{}", workflow.name, workflow.version);
    println!("  DSL Version: {}", workflow.dsl_version);
    println!("  Agents: {}", workflow.agents.len());
    println!("  Tasks: {}", workflow.tasks.len());

    if verbose && !workflow.agents.is_empty() {
        println!();
        println!("  {}:", "Agents".bold());
        for (name, spec) in &workflow.agents {
            println!("    • {} - {}", name.bright_white(), spec.description);
        }
    }

    if verbose && !workflow.tasks.is_empty() {
        println!();
        println!("  {}:", "Tasks".bold());
        for (name, spec) in &workflow.tasks {
            println!("    • {} - {}", name.bright_white(), spec.description);
        }
    }

    Ok(())
}

/// Show DSL grammar version
async fn show_version() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "DSL Grammar Information".bold());
    println!("{}", "=".repeat(60).dimmed());
    println!();
    println!("  {} {}", "Version:".bold(), DSL_GRAMMAR_VERSION);
    println!(
        "  {} Compatible with workflows using DSL v{}",
        "Compatibility:".bold(),
        DSL_GRAMMAR_VERSION
    );
    println!();
    println!(
        "  {} Use 'dsl-executor template' to see all available options",
        "Tip:".yellow().bold()
    );
    println!(
        "  {} Use 'dsl-executor generate' to create workflows from natural language",
        "Tip:".yellow().bold()
    );

    Ok(())
}

// ============================================================================
// Task Group Management Commands
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct GroupListOutput {
    search_paths: Vec<String>,
    groups: Vec<GroupListItem>,
    total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroupListItem {
    name: String,
    version: String,
    reference: String,
    description: Option<String>,
    tasks_count: usize,
    source_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroupInstallOutput {
    success: bool,
    group_reference: String,
    name: String,
    version: String,
    tasks_count: usize,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroupUpdateOutput {
    success: bool,
    updated_count: usize,
    groups: Vec<String>,
    message: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct GroupValidateOutput {
    valid: bool,
    group_name: String,
    group_version: String,
    tasks_count: usize,
    tasks: Option<Vec<TaskValidationInfo>>,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskValidationInfo {
    name: String,
    version: String,
    required: bool,
    found: bool,
}

/// List available task groups
async fn group_list(
    verbose: bool,
    json: bool,
    path: Option<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    use periplon_sdk::dsl::predefined_tasks::groups::TaskGroupLoader;

    let loader = if let Some(custom_path) = path {
        TaskGroupLoader::with_paths(vec![custom_path])
    } else {
        TaskGroupLoader::new()
    };

    if !json {
        println!("{}", "Task Groups".bold());
        println!("{}", "=".repeat(60).dimmed());
        println!();
    }

    // Discover all groups
    let discovered = loader.discover_all()?;

    if discovered.is_empty() {
        if json {
            let output = GroupListOutput {
                search_paths: loader
                    .search_paths
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect(),
                groups: vec![],
                total: 0,
            };
            print_json(&output)?;
        } else {
            println!("{}", "No task groups found".dimmed());
            println!();
            println!("  {}:", "Search Paths".bold());
            for path in &loader.search_paths {
                println!("    • {}", path.display().to_string().dimmed());
            }
        }
        return Ok(());
    }

    if json {
        // JSON output
        let mut groups_list = Vec::new();
        let mut loader_mut = loader;

        for (group_ref, group_path) in &discovered {
            // Load the group to get full details
            match loader_mut.load_from_file(group_path) {
                Ok(resolved) => {
                    groups_list.push(GroupListItem {
                        name: resolved.group.metadata.name.clone(),
                        version: resolved.group.metadata.version.clone(),
                        reference: group_ref.clone(),
                        description: resolved.group.metadata.description.clone(),
                        tasks_count: resolved.tasks.len(),
                        source_path: group_path.display().to_string(),
                        error: None,
                    });
                }
                Err(e) => {
                    // Parse the group reference to extract name and version
                    let parts: Vec<&str> = group_ref.split('@').collect();
                    let (name, version) = if parts.len() == 2 {
                        (parts[0].to_string(), parts[1].to_string())
                    } else {
                        (group_ref.clone(), "unknown".to_string())
                    };

                    groups_list.push(GroupListItem {
                        name,
                        version,
                        reference: group_ref.clone(),
                        description: None,
                        tasks_count: 0,
                        source_path: group_path.display().to_string(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        let output = GroupListOutput {
            search_paths: loader_mut
                .search_paths
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
            groups: groups_list,
            total: discovered.len(),
        };
        print_json(&output)?;
    } else {
        // Regular output
        println!("  {}:", "Search Paths".bold());
        for path in &loader.search_paths {
            println!("    • {}", path.display().to_string().dimmed());
        }
        println!();

        println!("  {} Available Groups:", "Found".green().bold());
        println!();

        let mut loader_mut = loader;
        for (group_ref, group_path) in &discovered {
            // Load the group to get full details
            match loader_mut.load_from_file(group_path) {
                Ok(resolved) => {
                    println!("  {} {}", "•".cyan(), group_ref.bright_white());

                    if verbose {
                        if let Some(desc) = &resolved.group.metadata.description {
                            println!("    {}", desc.dimmed());
                        }
                        println!("    {} {}", "Tasks:".dimmed(), resolved.tasks.len());
                        println!(
                            "    {} {}",
                            "Path:".dimmed(),
                            group_path.display().to_string().dimmed()
                        );

                        if verbose {
                            println!("    {}:", "Task List".dimmed());
                            for task_name in resolved.task_names() {
                                if let Some(task) = resolved.get_task(&task_name) {
                                    println!("      • {} v{}", task_name, task.metadata.version);
                                }
                            }
                        }
                    } else {
                        println!(
                            "    {} tasks | {}",
                            resolved.tasks.len(),
                            group_path.display().to_string().dimmed()
                        );
                    }
                    println!();
                }
                Err(e) => {
                    // Show group even if tasks couldn't be resolved
                    println!(
                        "  {} {} {}",
                        "•".cyan(),
                        group_ref.bright_white(),
                        "(tasks not available)".dimmed()
                    );
                    if verbose {
                        println!("    {} {}", "Error:".red(), e.to_string().dimmed());
                        println!(
                            "    {} {}",
                            "Path:".dimmed(),
                            group_path.display().to_string().dimmed()
                        );
                    }
                    println!();
                }
            }
        }

        println!("Total: {} groups", discovered.len());
    }

    Ok(())
}

/// Install (validate) a task group
async fn group_install(
    group_ref: &str,
    verbose: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use periplon_sdk::dsl::predefined_tasks::groups::{TaskGroupLoader, TaskGroupReference};

    if !json {
        println!("{}", "Installing Task Group".bold());
        println!("{}", "=".repeat(60).dimmed());
        println!();
    }

    // Parse group reference
    let parsed_ref = TaskGroupReference::parse(group_ref)
        .map_err(|e| format!("Invalid group reference '{}': {}", group_ref, e))?;

    if verbose && !json {
        println!("  {} {}", "Group:".bold(), parsed_ref.name);
        println!("  {} {}", "Version:".bold(), parsed_ref.version);
        println!();
    }

    // Load the group
    if !json && verbose {
        print!("  Resolving task group...  ");
    }

    let mut loader = TaskGroupLoader::new();
    let resolved = loader.load(&parsed_ref)?;

    if !json && verbose {
        println!("{}", "✓".green().bold());
    }

    // Validate all tasks are available
    if !json && verbose {
        print!("  Validating tasks...  ");
    }

    for (task_name, task) in &resolved.tasks {
        if verbose && !json {
            println!();
            println!(
                "    {} {} v{}",
                "✓".green(),
                task_name,
                task.metadata.version
            );
        }
    }

    if !json && verbose {
        println!("{}", "✓".green().bold());
    }

    if json {
        let output = GroupInstallOutput {
            success: true,
            group_reference: group_ref.to_string(),
            name: resolved.group.metadata.name.clone(),
            version: resolved.group.metadata.version.clone(),
            tasks_count: resolved.tasks.len(),
            message: format!("Task group '{}' is valid and ready to use", group_ref),
        };
        print_json(&output)?;
    } else {
        println!();
        println!("{} Task group installed successfully!", "✓".green().bold());
        println!("  {} {}", "Group:".bold(), resolved.group.metadata.name);
        println!(
            "  {} {}",
            "Version:".bold(),
            resolved.group.metadata.version
        );
        println!("  {} {}", "Tasks:".bold(), resolved.tasks.len());

        if let Some(desc) = &resolved.group.metadata.description {
            println!("  {} {}", "Description:".bold(), desc);
        }

        println!();
        println!(
            "  {} {}",
            "Source:".dimmed(),
            resolved.source_path.display()
        );
    }

    Ok(())
}

/// Update task group cache and information
async fn group_update(
    group_ref: Option<&str>,
    force: bool,
    verbose: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use periplon_sdk::dsl::predefined_tasks::groups::TaskGroupLoader;

    if !json {
        println!("{}", "Updating Task Groups".bold());
        println!("{}", "=".repeat(60).dimmed());
        println!();
    }

    let mut loader = TaskGroupLoader::new();

    if let Some(specific_ref) = group_ref {
        // Update specific group
        use periplon_sdk::dsl::predefined_tasks::groups::TaskGroupReference;
        let parsed_ref = TaskGroupReference::parse(specific_ref)
            .map_err(|e| format!("Invalid group reference '{}': {}", specific_ref, e))?;

        if force {
            // Clear cache for this group
            loader.clear_cache();
        }

        if verbose && !json {
            println!("  Updating group: {}", specific_ref.bright_white());
        }

        let resolved = loader.load(&parsed_ref)?;

        if json {
            let output = GroupUpdateOutput {
                success: true,
                updated_count: 1,
                groups: vec![specific_ref.to_string()],
                message: format!("Successfully updated task group '{}'", specific_ref),
            };
            print_json(&output)?;
        } else {
            println!();
            println!("{} Group updated successfully!", "✓".green().bold());
            println!("  {} {}", "Group:".bold(), resolved.group.metadata.name);
            println!(
                "  {} {}",
                "Version:".bold(),
                resolved.group.metadata.version
            );
            println!("  {} {}", "Tasks:".bold(), resolved.tasks.len());
        }
    } else {
        // Update all groups
        if force {
            loader.clear_cache();
            if verbose && !json {
                println!("  {} Cache cleared", "✓".green());
            }
        }

        let discovered = loader.discover_all()?;

        if verbose && !json {
            println!("  Found {} groups", discovered.len());
            println!();
        }

        let mut updated = Vec::new();
        for group_ref in discovered.keys() {
            if verbose && !json {
                println!("  Updating: {}", group_ref.dimmed());
            }
            updated.push(group_ref.clone());
        }

        if json {
            let output = GroupUpdateOutput {
                success: true,
                updated_count: updated.len(),
                groups: updated,
                message: format!("Successfully refreshed {} task groups", discovered.len()),
            };
            print_json(&output)?;
        } else {
            println!();
            println!("{} Updated {} groups", "✓".green().bold(), discovered.len());
        }
    }

    Ok(())
}

/// Validate a task group file
async fn group_validate(
    group_file: PathBuf,
    verbose: bool,
    json: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    use periplon_sdk::dsl::predefined_tasks::groups::{load_task_group, TaskGroupLoader};

    if !json {
        println!("{}", "Validating Task Group".bold());
        println!("{}", "=".repeat(60).dimmed());
        println!();
    }

    // Load the group file
    if verbose && !json {
        print!("  Loading group file...  ");
    }

    let group = load_task_group(&group_file)?;

    if verbose && !json {
        println!("{}", "✓".green().bold());
        println!("    {} {}", "Name:".dimmed(), group.metadata.name);
        println!("    {} {}", "Version:".dimmed(), group.metadata.version);
        println!("    {} {}", "Tasks:".dimmed(), group.spec.tasks.len());
    }

    // Resolve all tasks
    if verbose && !json {
        println!();
        print!("  Resolving tasks...  ");
    }

    let mut loader = TaskGroupLoader::new();
    let mut errors = Vec::new();
    let mut task_validations = Vec::new();

    for task_ref in &group.spec.tasks {
        use periplon_sdk::dsl::predefined_tasks::schema::TaskReference;
        let task_reference = TaskReference {
            name: task_ref.name.clone(),
            version: task_ref.version.clone(),
        };

        let found = loader.task_loader_mut().load(&task_reference).is_ok();

        task_validations.push(TaskValidationInfo {
            name: task_ref.name.clone(),
            version: task_ref.version.clone(),
            required: task_ref.required,
            found,
        });

        if !found {
            let error_msg = format!("Task '{}' v{} not found", task_ref.name, task_ref.version);
            errors.push(error_msg.clone());

            if verbose && !json {
                println!();
                println!("    {} {}", "✗".red(), error_msg);
            }
        } else if verbose && !json {
            println!();
            println!(
                "    {} {} v{}",
                "✓".green(),
                task_ref.name,
                task_ref.version
            );
        }
    }

    if verbose && !json && errors.is_empty() {
        println!("{}", "✓".green().bold());
    }

    let is_valid = errors.is_empty();

    if json {
        let output = GroupValidateOutput {
            valid: is_valid,
            group_name: group.metadata.name.clone(),
            group_version: group.metadata.version.clone(),
            tasks_count: group.spec.tasks.len(),
            tasks: if verbose {
                Some(task_validations)
            } else {
                None
            },
            errors,
        };
        print_json(&output)?;
    } else {
        println!();
        if is_valid {
            println!("{} Task group is valid!", "✓".green().bold());
            println!("  {} {}", "Group:".bold(), group.metadata.name);
            println!("  {} {}", "Version:".bold(), group.metadata.version);
            println!("  {} {}", "Tasks:".bold(), group.spec.tasks.len());

            if verbose && !group.spec.tasks.is_empty() {
                println!();
                println!("  {}:", "Tasks".bold());
                for task_ref in &group.spec.tasks {
                    println!(
                        "    • {} v{}{}",
                        task_ref.name,
                        task_ref.version,
                        if task_ref.required {
                            " (required)".dimmed().to_string()
                        } else {
                            String::new()
                        }
                    );
                }
            }
        } else {
            println!("{} Task group validation failed!", "✗".red().bold());
            println!();
            println!("  {}:", "Errors".red().bold());
            for error in &errors {
                println!("    • {}", error);
            }
        }
    }

    if !is_valid {
        return Err("Task group validation failed".into());
    }

    Ok(())
}

// ============================================================================
// Server Mode Functions (only compiled with "server" feature)
// ============================================================================

#[cfg(feature = "server")]
async fn start_server(
    port: u16,
    config_path: Option<PathBuf>,
    workers: bool,
    worker_concurrency: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    use periplon_sdk::server::{
        api::routes, auth::JwtManager, queue::filesystem::FilesystemQueue,
        storage::filesystem::FilesystemStorage, storage::UserStorage, worker::Worker, Config,
    };
    use std::sync::Arc;

    println!("{}", "Starting DSL Executor Server".bold().green());
    println!("{}", "=".repeat(60).dimmed());

    // Load configuration
    let mut config = Config::load(config_path)?;
    println!("  {} Configuration loaded", "✓".green());

    // Override with CLI arguments
    config.server.port = port;
    config.server.workers = workers;
    config.server.worker_concurrency = worker_concurrency;

    // Ensure JWT secret is set (generates one for development if not provided)
    config.ensure_jwt_secret();

    let server_port = config.server.port;
    println!(
        "  {} Server will listen on port {}",
        "✓".green(),
        server_port
    );

    // Initialize storage backend
    let storage: Arc<dyn periplon_sdk::server::Storage> = match &config.storage.backend {
        periplon_sdk::server::config::StorageBackend::Filesystem(fs_config) => {
            let storage = FilesystemStorage::new(
                fs_config.base_path.clone(),
                fs_config.workflows_dir.clone(),
                fs_config.executions_dir.clone(),
                fs_config.checkpoints_dir.clone(),
                fs_config.logs_dir.clone(),
            )
            .await?;
            println!("  {} Storage backend: filesystem", "✓".green());
            Arc::new(storage)
        }
        periplon_sdk::server::config::StorageBackend::Postgres(pg_config) => {
            use periplon_sdk::server::storage::postgres::PostgresStorage;
            let storage = PostgresStorage::new(&pg_config.url).await?;
            println!("  {} Storage backend: PostgreSQL", "✓".green());
            Arc::new(storage)
        }
        periplon_sdk::server::config::StorageBackend::S3(s3_config) => {
            use periplon_sdk::server::storage::s3::S3Storage;
            let storage = S3Storage::new(
                s3_config.endpoint.clone(),
                s3_config.region.clone(),
                s3_config.bucket.clone(),
                s3_config.access_key_id.clone(),
                s3_config.secret_access_key.clone(),
                Some(s3_config.path_prefix.clone()),
            )
            .await?;
            println!("  {} Storage backend: S3", "✓".green());
            Arc::new(storage)
        }
    };

    // Initialize queue backend
    let queue: Arc<dyn periplon_sdk::server::WorkQueue> = match &config.queue.backend {
        periplon_sdk::server::config::QueueBackend::Filesystem(fs_config) => {
            let queue = FilesystemQueue::new(
                fs_config.queue_dir.clone(),
                fs_config.poll_interval_ms,
                fs_config.lock_timeout_secs,
            )
            .await?;
            println!("  {} Queue backend: filesystem", "✓".green());
            Arc::new(queue)
        }
        periplon_sdk::server::config::QueueBackend::Postgres(pg_config) => {
            use periplon_sdk::server::queue::postgres::PostgresQueue;
            let queue = PostgresQueue::new(
                &pg_config.url,
                pg_config.poll_interval_ms,
                pg_config.max_retries,
            )
            .await?;
            println!("  {} Queue backend: PostgreSQL", "✓".green());
            Arc::new(queue)
        }
        periplon_sdk::server::config::QueueBackend::Redis(redis_config) => {
            use periplon_sdk::server::queue::redis::RedisQueue;
            let queue = RedisQueue::new(&redis_config.url, None).await?;
            println!("  {} Queue backend: Redis", "✓".green());
            Arc::new(queue)
        }
        _ => {
            return Err("S3 queue is not yet implemented".into());
        }
    };

    // Initialize user storage backend
    let user_storage: Arc<dyn UserStorage> = match &config.user_storage.backend {
        periplon_sdk::server::config::UserStorageBackend::Filesystem(fs_config) => {
            use periplon_sdk::server::storage::user_filesystem::FilesystemUserStorage;
            let storage = FilesystemUserStorage::new(fs_config.base_path.clone()).await?;
            println!("  {} User storage backend: filesystem", "✓".green());
            Arc::new(storage)
        }
        periplon_sdk::server::config::UserStorageBackend::Postgres(pg_config) => {
            use periplon_sdk::server::storage::user_postgres::PostgresUserStorage;
            let storage = PostgresUserStorage::new(&pg_config.url).await?;
            println!("  {} User storage backend: PostgreSQL", "✓".green());
            Arc::new(storage)
        }
        periplon_sdk::server::config::UserStorageBackend::S3(s3_config) => {
            use periplon_sdk::server::storage::user_s3::S3UserStorage;
            let storage =
                S3UserStorage::new(s3_config.bucket.clone(), s3_config.prefix.clone()).await?;
            println!("  {} User storage backend: S3", "✓".green());
            Arc::new(storage)
        }
    };

    // Initialize JWT manager
    let jwt_manager = Arc::new(JwtManager::new(
        &config.auth.jwt_secret,
        (config.auth.jwt_expiration_secs / 3600) as i64, // Convert seconds to hours
    ));
    println!("  {} JWT manager initialized", "✓".green());

    // Start workers if requested
    if workers {
        println!(
            "  {} Starting {} background workers",
            "✓".green(),
            worker_concurrency
        );
        for i in 0..worker_concurrency {
            let worker_id = format!("server-worker-{}", i);
            let worker = Worker::new(
                worker_id.clone(),
                Arc::clone(&queue),
                Arc::clone(&storage),
                1,
            );

            tokio::spawn(async move {
                worker.run().await;
            });
        }
    }

    // Create API router with user storage, JWT manager, storage, queue, CORS config, and rate limiting
    let app = routes::create_router(
        user_storage,
        jwt_manager,
        Arc::clone(&storage),
        Arc::clone(&queue),
        config.server.cors.clone(),
        config.rate_limit.clone(),
    );

    println!();
    println!("{}", "Server Status".bold());
    println!("{}", "=".repeat(60).dimmed());
    println!(
        "  {} Server running at http://0.0.0.0:{}",
        "●".green().bold(),
        server_port
    );
    println!(
        "  {} Health check: http://0.0.0.0:{}/health",
        "ⓘ".blue(),
        server_port
    );
    println!(
        "  {} API docs: http://0.0.0.0:{}/api/v1",
        "ⓘ".blue(),
        server_port
    );
    println!();
    println!("Press Ctrl+C to stop the server");

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", server_port)).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "server")]
async fn start_worker(
    concurrency: usize,
    config_path: Option<PathBuf>,
    worker_id: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    use periplon_sdk::server::{
        queue::filesystem::FilesystemQueue, storage::filesystem::FilesystemStorage, worker::Worker,
        Config,
    };
    use std::sync::Arc;

    println!("{}", "Starting DSL Executor Worker".bold().green());
    println!("{}", "=".repeat(60).dimmed());

    // Load configuration
    let config = Config::load(config_path)?;
    println!("  {} Configuration loaded", "✓".green());

    // Generate worker ID if not provided
    let worker_id = worker_id.unwrap_or_else(|| format!("worker-{}", uuid::Uuid::new_v4()));
    println!("  {} Worker ID: {}", "✓".green(), worker_id);
    println!("  {} Concurrency: {}", "✓".green(), concurrency);

    // Initialize storage backend
    let storage: Arc<dyn periplon_sdk::server::Storage> = match &config.storage.backend {
        periplon_sdk::server::config::StorageBackend::Filesystem(fs_config) => {
            let storage = FilesystemStorage::new(
                fs_config.base_path.clone(),
                fs_config.workflows_dir.clone(),
                fs_config.executions_dir.clone(),
                fs_config.checkpoints_dir.clone(),
                fs_config.logs_dir.clone(),
            )
            .await?;
            println!("  {} Storage backend: filesystem", "✓".green());
            Arc::new(storage)
        }
        periplon_sdk::server::config::StorageBackend::Postgres(pg_config) => {
            use periplon_sdk::server::storage::postgres::PostgresStorage;
            let storage = PostgresStorage::new(&pg_config.url).await?;
            println!("  {} Storage backend: PostgreSQL", "✓".green());
            Arc::new(storage)
        }
        periplon_sdk::server::config::StorageBackend::S3(s3_config) => {
            use periplon_sdk::server::storage::s3::S3Storage;
            let storage = S3Storage::new(
                s3_config.endpoint.clone(),
                s3_config.region.clone(),
                s3_config.bucket.clone(),
                s3_config.access_key_id.clone(),
                s3_config.secret_access_key.clone(),
                Some(s3_config.path_prefix.clone()),
            )
            .await?;
            println!("  {} Storage backend: S3", "✓".green());
            Arc::new(storage)
        }
    };

    // Initialize queue backend
    let queue: Arc<dyn periplon_sdk::server::WorkQueue> = match &config.queue.backend {
        periplon_sdk::server::config::QueueBackend::Filesystem(fs_config) => {
            let queue = FilesystemQueue::new(
                fs_config.queue_dir.clone(),
                fs_config.poll_interval_ms,
                fs_config.lock_timeout_secs,
            )
            .await?;
            println!("  {} Queue backend: filesystem", "✓".green());
            Arc::new(queue)
        }
        periplon_sdk::server::config::QueueBackend::Postgres(pg_config) => {
            use periplon_sdk::server::queue::postgres::PostgresQueue;
            let queue = PostgresQueue::new(
                &pg_config.url,
                pg_config.poll_interval_ms,
                pg_config.max_retries,
            )
            .await?;
            println!("  {} Queue backend: PostgreSQL", "✓".green());
            Arc::new(queue)
        }
        periplon_sdk::server::config::QueueBackend::Redis(redis_config) => {
            use periplon_sdk::server::queue::redis::RedisQueue;
            let queue = RedisQueue::new(&redis_config.url, None).await?;
            println!("  {} Queue backend: Redis", "✓".green());
            Arc::new(queue)
        }
        _ => {
            return Err("S3 queue is not yet implemented".into());
        }
    };

    // Create and run worker
    let worker = Worker::new(worker_id.clone(), queue, storage, concurrency);

    println!();
    println!("{}", "Worker Status".bold());
    println!("{}", "=".repeat(60).dimmed());
    println!("  {} Worker running", "●".green().bold());
    println!();
    println!("Press Ctrl+C to stop the worker");

    worker.run().await;

    Ok(())
}

#[cfg(feature = "server")]
async fn run_migrations(
    _config_path: Option<PathBuf>,
    action: String,
) -> Result<(), Box<dyn std::error::Error>> {
    use periplon_sdk::server::db::migrations::MigrationRunner;

    println!("{}", "Database Migrations".bold().green());
    println!("{}", "=".repeat(60).dimmed());
    println!();

    // Get database URL from environment
    let database_url =
        std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL environment variable not set")?;

    println!("  {} Connecting to database...", "ⓘ".blue());
    let runner = MigrationRunner::new(&database_url).await?;

    // Initialize migrations table
    runner.init().await?;

    // Get migrations directory (at project root)
    let migrations_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");

    if !migrations_dir.exists() {
        return Err(format!("Migrations directory not found: {:?}. Please ensure migrations/ directory exists at project root.", migrations_dir).into());
    }

    println!(
        "  {} Loading migrations from {:?}...",
        "ⓘ".blue(),
        migrations_dir
    );
    let migrations = runner.load_migrations(&migrations_dir).await?;

    println!("  {} Found {} migration(s)", "ℹ".blue(), migrations.len());
    println!();

    match action.as_str() {
        "up" => {
            println!("  {} Running pending migrations...", "⚙".yellow());
            println!();
            runner.migrate_up(migrations).await?;
            println!();
            println!("  {} All migrations applied successfully!", "✓".green());
        }
        "down" => {
            println!("  {} Rolling back last migration...", "⚙".yellow());
            println!();
            runner.migrate_down(migrations).await?;
            println!();
            println!("  {} Rollback complete!", "✓".green());
        }
        "status" => {
            runner.status(migrations).await?;
        }
        _ => {
            return Err(
                format!("Unknown action: {}. Use 'up', 'down', or 'status'", action).into(),
            );
        }
    }

    runner.close().await;
    Ok(())
}
