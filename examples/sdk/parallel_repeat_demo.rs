//! Parallel Repeat Loop Demo
//!
//! This example demonstrates parallel Repeat loop execution:
//! - Concurrent count-based iterations
//! - Semaphore-based concurrency limiting
//! - max_parallel configuration
//! - Result collection from parallel tasks
//!
//! Usage:
//! ```bash
//! cargo run --example parallel_repeat_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Parallel Repeat Loop Demo ===\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/parallel_repeat_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    println!("Executing workflow with Parallel Repeat loops...\n");
    println!("This will demonstrate:");
    println!("  1. Parallel execution of repeated tasks");
    println!("  2. Semaphore-based concurrency control");
    println!("  3. Different max_parallel settings (4 and 6 concurrent tasks)");
    println!("  4. Result collection across parallel iterations");
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
                    println!("\nParallel Repeat Loop '{}' Statistics:", task_id);
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
                if let Some(results) = state.get_loop_results("parallel_iterations") {
                    println!(
                        "\nParallel Iterations Results: {} tasks completed",
                        results.len()
                    );
                }

                if let Some(results) = state.get_loop_results("concurrent_tasks") {
                    println!(
                        "Concurrent Tasks Results: {} tasks completed",
                        results.len()
                    );
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
