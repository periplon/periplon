//! REPL Debugging Example
//!
//! Demonstrates how to use the interactive REPL debugger to inspect and control
//! workflow execution with breakpoints, stepping, and state inspection.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example repl_debug_example
//! ```
use periplon_sdk::dsl::{parse_workflow_file, BreakCondition, DSLExecutor, ReplSession, StepMode};
use periplon_sdk::error::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== REPL Debug Example ===\n");

    // Load workflow
    let workflow_path = PathBuf::from("examples/workflows/debug_example.yaml");
    println!("📄 Loading workflow: {:?}", workflow_path);

    let workflow = parse_workflow_file(&workflow_path)?;
    println!("✅ Workflow loaded: {}\n", workflow.name);

    // Create executor with debugging enabled
    println!("🐛 Creating executor with debug mode enabled...");
    let executor = DSLExecutor::new(workflow)?.with_debugger();
    println!("✅ Debug mode enabled\n");

    // Configure initial debugger state
    if let Some(debugger) = executor.debugger() {
        let mut dbg = debugger.lock().await;

        println!("🔧 Configuring debugger:");

        // Set breakpoint on a specific task
        dbg.breakpoints.add_task_breakpoint("analyze".to_string());
        println!("  ✓ Breakpoint set on task: analyze");

        // Set conditional breakpoint for errors
        dbg.breakpoints.add_conditional_breakpoint(
            BreakCondition::OnError,
            Some("Break on any error".to_string()),
        );
        println!("  ✓ Conditional breakpoint set: OnError");

        // Set step mode - this will pause before each task
        dbg.set_step_mode(StepMode::StepTask);
        println!("  ✓ Step mode: StepTask");

        // Start paused so user can interact
        dbg.pause();
        println!("  ✓ Starting in paused mode");
    }

    println!("\n📝 REPL Commands Available:");
    println!("  - continue (c)     : Continue execution until next breakpoint");
    println!("  - step (s)         : Step to next task");
    println!("  - status           : Show current debugger status");
    println!("  - vars             : Show all variables");
    println!("  - stack            : Show call stack");
    println!("  - timeline         : Show execution timeline");
    println!("  - breaks           : List all breakpoints");
    println!("  - help             : Show all available commands");
    println!("  - quit (q)         : Exit REPL\n");

    println!("⚠️  Note: This is an interactive REPL debugger!");
    println!("    The workflow will pause at breakpoints and wait for your commands.\n");

    // Create and run REPL session
    println!("🚀 Starting REPL session...\n");

    let mut repl = ReplSession::new(executor)?;
    repl.run().await?;

    println!("\n✅ REPL session ended");

    Ok(())
}
