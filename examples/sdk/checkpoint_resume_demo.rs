//! Checkpoint and Resume Demo
//!
//! This example demonstrates state persistence and resume capabilities:
//! - Automatic checkpointing during loop execution
//! - Resume capability for interrupted loops
//! - Checkpoint interval configuration
//! - State restoration from .state directory
//!
//! Usage:
//! ```bash
//! # First run (may be interrupted)
//! cargo run --example checkpoint_resume_demo
//!
//! # Resume from checkpoint (will skip completed iterations)
//! cargo run --example checkpoint_resume_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Checkpoint and Resume Demo ===\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/checkpoint_resume_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    // Enable state persistence to .state directory
    println!("Enabling state persistence to .state directory...");
    executor.enable_state_persistence(Some(".state"))?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    // Check if we're resuming from a previous run
    if let Some(state) = executor.get_state() {
        for (task_id, _loop_state) in &state.loop_states {
            if let Some(last_completed) = state.get_last_completed_iteration(task_id) {
                println!(
                    "Found checkpoint for '{}': {} iterations completed, resuming...",
                    task_id,
                    last_completed + 1
                );
            }
        }
    }

    println!("\nExecuting workflow with Checkpoint/Resume...\n");
    println!("This will demonstrate:");
    println!("  1. Automatic checkpointing every N iterations");
    println!("  2. State persistence to .state directory");
    println!("  3. Resume capability - skips completed iterations");
    println!("  4. Combined with break conditions and timeouts");
    println!();
    println!("Checkpoints will be saved:");
    println!("  - Every 10 iterations for process_large_batch");
    println!("  - Every 5 iterations for long_running_computation");
    println!("  - Every 20 iterations for resumable_with_break");
    println!();
    println!("You can interrupt this demo (Ctrl+C) and restart it -");
    println!("it will resume from the last checkpoint!\n");

    // Execute the workflow
    match executor.execute().await {
        Ok(()) => {
            println!("\n✓ Workflow completed successfully!");

            // Show some statistics
            if let Some(state) = executor.get_state() {
                println!("\nWorkflow Statistics:");
                println!("  Total tasks: {}", state.task_statuses.len());
                println!("  Completed: {}", state.get_completed_tasks().len());
                println!("  Failed: {}", state.get_failed_tasks().len());

                // Show loop states
                for (task_id, loop_state) in &state.loop_states {
                    println!("\nLoop '{}' Statistics:", task_id);
                    println!(
                        "  Iterations executed: {}",
                        loop_state.current_iteration + 1
                    );
                    if let Some(total) = loop_state.total_iterations {
                        println!("  Total iterations: {}", total);
                        let completed = loop_state
                            .iteration_statuses
                            .iter()
                            .filter(|s| **s == periplon_sdk::dsl::TaskStatus::Completed)
                            .count();
                        let progress = (completed as f64 / total as f64) * 100.0;
                        println!("  Completed: {} ({:.1}%)", completed, progress);
                    }
                }

                // Show loop results if available
                if let Some(results) = state.get_loop_results("batch_results") {
                    println!("\nBatch Processing: {} items processed", results.len());
                }

                if let Some(results) = state.get_loop_results("resumable_results") {
                    println!("Resumable Task: {} items processed", results.len());
                }
            }

            println!("\nState saved to .state directory");
            println!("Run again to see resume capability in action!");
        }
        Err(e) => {
            println!("\n✗ Workflow execution failed/interrupted: {}", e);
            println!("\nPartial progress has been checkpointed to .state directory");
            println!("Run the demo again to resume from the last checkpoint!");
            return Err(e);
        }
    }

    // Cleanup
    executor.shutdown().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}
