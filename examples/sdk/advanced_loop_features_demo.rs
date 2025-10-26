//! Advanced Loop Features Demo
//!
//! This example demonstrates advanced loop control features:
//! - Break conditions for early loop termination
//! - Continue conditions for skipping iterations
//! - Loop timeouts for preventing runaway loops
//! - Combined usage of all features
//!
//! Usage:
//! ```bash
//! cargo run --example advanced_loop_features_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Advanced Loop Features Demo ===\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/advanced_loop_features_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    println!("Executing workflow with Advanced Loop Features...\n");
    println!("This will demonstrate:");
    println!("  1. Break condition - early loop termination");
    println!("  2. Continue condition - skipping iterations");
    println!("  3. Timeout - preventing runaway loops");
    println!("  4. Combined features - all controls together");
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
                    println!(
                        "  Iterations executed: {}",
                        loop_state.current_iteration + 1
                    );
                    if let Some(total) = loop_state.total_iterations {
                        println!("  Total planned iterations: {}", total);
                    }
                    println!(
                        "  Completed: {}",
                        loop_state
                            .iteration_statuses
                            .iter()
                            .filter(|s| **s == periplon_sdk::dsl::TaskStatus::Completed)
                            .count()
                    );
                    println!(
                        "  Skipped (continue): {}",
                        loop_state.total_iterations.unwrap_or(0)
                            - loop_state.iteration_statuses.len()
                    );
                }

                // Show loop results if available
                if let Some(results) = state.get_loop_results("process_with_break") {
                    println!(
                        "\nBreak Results: {} items processed before break",
                        results.len()
                    );
                }

                if let Some(results) = state.get_loop_results("process_with_continue") {
                    println!(
                        "Continue Results: {} items processed (others skipped)",
                        results.len()
                    );
                }

                if let Some(results) = state.get_loop_results("combined_results") {
                    println!("Combined Features Results: {} items", results.len());
                }
            }
        }
        Err(e) => {
            println!("\n✗ Workflow execution failed: {}", e);
            // Note: Timeout errors are expected and demonstrate the feature
            if e.to_string().contains("timed out") {
                println!("   (This timeout is expected and demonstrates the timeout feature)");
            }
            return Err(e);
        }
    }

    // Cleanup
    executor.shutdown().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}
