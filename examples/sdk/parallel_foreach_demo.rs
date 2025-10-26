//! Parallel ForEach Loop Demo
//!
//! This example demonstrates parallel ForEach loop execution:
//! - Concurrent iteration over collections
//! - Semaphore-based concurrency limiting
//! - max_parallel configuration
//! - Error aggregation across parallel tasks
//!
//! Usage:
//! ```bash
//! cargo run --example parallel_foreach_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Parallel ForEach Loop Demo ===\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/parallel_foreach_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    println!("Executing workflow with Parallel ForEach loops...\n");
    println!("This will demonstrate:");
    println!("  1. Parallel iteration over inline collections");
    println!("  2. Parallel iteration over numeric ranges");
    println!("  3. Semaphore-based concurrency limiting");
    println!("  4. max_parallel configuration (3 and 5 concurrent tasks)");
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
                    println!("\nParallel Loop '{}' Statistics:", task_id);
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
                if let Some(results) = state.get_loop_results("process_items_parallel") {
                    println!("\nProcessing Results: {} items processed", results.len());
                }

                if let Some(results) = state.get_loop_results("batch_process_range") {
                    println!("Batch Processing Results: {} batches", results.len());
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
