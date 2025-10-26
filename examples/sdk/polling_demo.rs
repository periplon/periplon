//! Polling Demo (RepeatUntil)
//!
//! This example demonstrates RepeatUntil loop patterns in the DSL:
//! - Execute-then-check pattern (like do-while)
//! - Minimum iterations guarantee
//! - Polling with backoff delays
//! - Early exit on success condition
//!
//! Usage:
//! ```bash
//! cargo run --example polling_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Polling Demo (RepeatUntil Loop) ===\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/polling_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    println!("Executing workflow with RepeatUntil loops...\n");
    println!("This will demonstrate:");
    println!("  1. Polling pattern - execute then check condition");
    println!("  2. Minimum iterations guarantee");
    println!("  3. Max iterations safety limit");
    println!("  4. Delays between polling attempts (backoff)");
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
                    println!("\nRepeatUntil Loop '{}' Statistics:", task_id);
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
                if let Some(results) = state.get_loop_results("poll_for_completion") {
                    println!("\nPolling Results: {} attempts", results.len());
                }

                if let Some(results) = state.get_loop_results("retry_with_backoff") {
                    println!("Retry Results: {} attempts", results.len());
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
