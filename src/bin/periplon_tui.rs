//! DSL TUI - Interactive Terminal User Interface for DSL Workflow Management
//!
//! A terminal-based REPL for creating, editing, executing, and monitoring DSL workflows.
//!
//! # Features
//!
//! - **Workflow Management**: Browse, create, edit, and delete workflows
//! - **Interactive Editor**: YAML editor with syntax highlighting and real-time validation
//! - **Execution Monitor**: Live task progress, agent output, and execution metrics
//! - **State Persistence**: Browse and resume paused workflows
//! - **AI Generation**: Generate workflows from natural language descriptions
//! - **Help System**: Context-sensitive help and keyboard shortcuts
//!
//! # Usage
//!
//! ```bash
//! # Launch TUI with default workflow directory
//! dsl-tui
//!
//! # Launch with custom workflow directory
//! dsl-tui --workflow-dir ./my-workflows
//!
//! # Launch with specific workflow
//! dsl-tui --workflow ./workflow.yaml
//!
//! # Launch in readonly mode
//! dsl-tui --readonly
//!
//! # Launch with custom theme
//! dsl-tui --theme dark
//! ```

use periplon_sdk::tui::{AppConfig, TuiApp};
use clap::Parser;
use std::path::PathBuf;
use std::process;

/// DSL TUI - Interactive Terminal User Interface for DSL Workflows
#[derive(Parser, Debug)]
#[command(
    name = "dsl-tui",
    version,
    about = "Interactive TUI for DSL workflow management",
    long_about = None
)]
struct Cli {
    /// Workflow directory to browse and manage
    #[arg(short = 'd', long, value_name = "DIR", default_value = ".")]
    workflow_dir: PathBuf,

    /// Specific workflow file to open
    #[arg(short = 'w', long, value_name = "FILE")]
    workflow: Option<PathBuf>,

    /// Launch in readonly mode (no edits or execution)
    #[arg(short = 'r', long)]
    readonly: bool,

    /// Color theme (dark, light, monokai, solarized)
    #[arg(short = 't', long, value_name = "THEME", default_value = "dark")]
    theme: String,

    /// State directory for workflow persistence
    #[arg(short = 's', long, value_name = "DIR")]
    state_dir: Option<PathBuf>,

    /// Enable debug logging
    #[arg(long)]
    debug: bool,

    /// Tick rate in milliseconds (lower = more responsive, higher = less CPU)
    #[arg(long, value_name = "MS", default_value = "250")]
    tick_rate: u64,
}

#[tokio::main]
async fn main() {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Setup logging if debug mode enabled
    if cli.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .init();
        log::info!("Debug logging enabled");
    }

    // Validate workflow directory exists
    if !cli.workflow_dir.exists() {
        eprintln!(
            "Error: Workflow directory does not exist: {}",
            cli.workflow_dir.display()
        );
        process::exit(1);
    }

    // If specific workflow provided, validate it exists
    if let Some(ref workflow) = cli.workflow {
        if !workflow.exists() {
            eprintln!(
                "Error: Workflow file does not exist: {}",
                workflow.display()
            );
            process::exit(1);
        }
    }

    // Create app config from CLI arguments
    let config = AppConfig {
        workflow_dir: cli.workflow_dir.clone(),
        workflow: cli.workflow.clone(),
        readonly: cli.readonly,
        theme: cli.theme.clone(),
        state_dir: cli.state_dir.clone(),
        debug: cli.debug,
        tick_rate: cli.tick_rate,
    };

    log::info!("Starting DSL TUI");
    log::info!("Workflow directory: {}", config.workflow_dir.display());

    if config.readonly {
        log::info!("Running in readonly mode");
    }

    // Create and run TUI application with config
    match TuiApp::with_config(config) {
        Ok(mut app) => {
            // Run the application
            if let Err(e) = app.run().await {
                // Cleanup terminal before showing error
                cleanup_terminal();
                eprintln!("Application error: {}", e);
                process::exit(1);
            }

            // Normal cleanup
            cleanup_terminal();
        }
        Err(e) => {
            eprintln!("Failed to initialize TUI: {}", e);
            process::exit(1);
        }
    }
}

/// Cleanup terminal state before exit
fn cleanup_terminal() {
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
    use crossterm::ExecutableCommand;
    use std::io;

    let _ = io::stdout().execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();
}
