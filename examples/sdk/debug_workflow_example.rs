//! Example: Debug Mode Workflow Execution
//!
//! This example demonstrates how to use the debugging capabilities of the DSL executor:
//! - Enable debugging with `.with_debugger()`
//! - Set breakpoints on specific tasks
//! - Set conditional breakpoints
//! - Inspect execution state
//! - View execution timeline and snapshots
//!
//! Run this example with:
//! ```bash
//! cargo run --example debug_workflow_example
//! ```
use periplon_sdk::dsl::{parse_workflow_file, BreakCondition, DSLExecutor, StepMode};
use periplon_sdk::error::Result;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Debug Mode Workflow Example ===\n");

    // Parse the workflow
    let workflow_path = PathBuf::from("examples/workflows/debug_example.yaml");
    println!("📄 Loading workflow: {:?}", workflow_path);

    let workflow = parse_workflow_file(&workflow_path)?;
    println!("✅ Workflow loaded: {}\n", workflow.name);

    // Create executor with debugging enabled
    println!("🐛 Creating executor with debug mode enabled...");
    let mut executor = DSLExecutor::new(workflow)?.with_debugger(); // ✨ Enable debugging!

    println!("✅ Debug mode enabled\n");

    // Configure debugger before execution
    if let Some(debugger) = executor.debugger() {
        let mut dbg = debugger.lock().await;

        println!("🔧 Configuring debugger:");

        // Set a breakpoint on the "analyze" task
        dbg.breakpoints.add_task_breakpoint("analyze".to_string());
        println!("  ✓ Breakpoint set on task: analyze");

        // Set a conditional breakpoint for errors
        dbg.breakpoints.add_conditional_breakpoint(
            BreakCondition::OnError,
            Some("Break on any task error".to_string()),
        );
        println!("  ✓ Conditional breakpoint set: OnError");

        // Set step mode to step through each task
        dbg.set_step_mode(StepMode::StepTask);
        println!("  ✓ Step mode: StepTask");

        // Note: In a real REPL, you would pause execution here
        // For this example, we'll automatically resume after a brief delay
        println!("\n⚠️  Note: In interactive mode, execution would pause at breakpoints.");
        println!("    For this example, we'll auto-resume after showing the status.\n");

        // Immediately resume to avoid blocking (for demo purposes)
        dbg.resume();
    }

    // Execute the workflow
    println!("▶️  Starting workflow execution...\n");
    let execution_result = executor.execute().await;

    println!("\n=== Execution Complete ===\n");

    // Check execution result
    match execution_result {
        Ok(()) => println!("✅ Workflow completed successfully"),
        Err(e) => println!("❌ Workflow failed: {}", e),
    }

    // Inspect execution after completion
    if let Some(inspector) = executor.inspector() {
        println!("\n=== Debug Inspection ===\n");

        // Get debugger status
        let status = inspector.status().await;
        println!("📊 {}", status);

        // Get execution timeline
        let timeline = inspector.timeline().await;
        println!("\n📅 Timeline: {} events", timeline.len());
        for (i, event) in timeline.events.iter().enumerate().take(10) {
            println!("  {}. {:?}", i + 1, event.event_type);
        }
        if timeline.len() > 10 {
            println!("  ... ({} more events)", timeline.len() - 10);
        }

        // Get snapshots
        let snapshots = inspector.snapshots().await;
        println!("\n📸 Snapshots: {}", snapshots.len());
        for snapshot in &snapshots {
            println!(
                "  #{}: {} ({:.2}s)",
                snapshot.id,
                snapshot.description,
                snapshot.elapsed.as_secs_f64()
            );
        }

        // Get variables
        let variables = inspector.inspect_variables(None).await;
        println!("\n📦 Variables: {} total", variables.total_count());
        if !variables.workflow_vars.is_empty() {
            println!("  Workflow variables:");
            for (key, value) in &variables.workflow_vars {
                println!("    {} = {:?}", key, value);
            }
        }

        // Get side effects summary
        let side_effects_summary = inspector.side_effect_summary().await;
        if !side_effects_summary.is_empty() {
            println!("\n📝 Side Effects:");
            for (effect_type, count) in side_effects_summary {
                println!("  {} x {}", count, effect_type);
            }
        }
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
