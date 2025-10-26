//! Repeat Loop Demo
//!
//! This example demonstrates Repeat loop patterns in the DSL:
//! - Simple count-based iteration
//! - Repeat with iterator variable
//! - Repeat with definition of done
//!
//! Usage:
//! ```bash
//! cargo run --example repeat_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Repeat Loop Demo ===\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/repeat_loop_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    println!("Executing workflow with Repeat loops...\n");
    println!("This will demonstrate:");
    println!("  1. Simple repeat loop (3 iterations)");
    println!("  2. Repeat loop with iterator variable (5 iterations)");
    println!("  3. Repeat loop with definition of done");
    println!();

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
                    println!("  Iterations: {}", loop_state.current_iteration + 1);
                    if let Some(total) = loop_state.total_iterations {
                        println!("  Total iterations: {}", total);
                    }
                    println!(
                        "  Completed: {}",
                        loop_state
                            .iteration_statuses
                            .iter()
                            .filter(|s| **s == periplon_sdk::dsl::TaskStatus::Completed)
                            .count()
                    );
                }

                // Show loop results if available
                if let Some(results) = state.get_loop_results("generate_reports") {
                    println!("\nGenerate Reports Results: {} reports", results.len());
                }

                if let Some(results) = state.get_loop_results("create_files") {
                    println!("Create Files Results: {} files", results.len());
                }
            }
        }
        Err(e) => {
            println!("\n✗ Workflow execution failed: {}", e);
            return Err(e);
        }
    }

    // Cleanup
    executor.shutdown().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}
