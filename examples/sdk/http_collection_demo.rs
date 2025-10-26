//! HTTP Collection Demo
//!
//! This example demonstrates fetching data from HTTP/HTTPS APIs as loop collections:
//! - Fetching JSON arrays from REST APIs
//! - Using public JSONPlaceholder API for testing
//! - Processing API responses in loops
//! - Limiting iterations with max_iterations
//!
//! Usage:
//! ```bash
//! cargo run --example http_collection_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== HTTP Collection Demo ===\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/http_collection_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    println!("Executing workflow with HTTP collections...\n");
    println!("This will demonstrate:");
    println!("  1. Fetching JSON arrays from REST APIs");
    println!("  2. JSONPlaceholder API for test data");
    println!("  3. Processing API responses in loops");
    println!("  4. Limiting iterations with max_iterations\n");

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

                // Show loop results
                if let Some(users) = state.get_loop_results("processed_users") {
                    println!("\nUsers processed: {}", users.len());
                }

                if let Some(posts) = state.get_loop_results("processed_posts") {
                    println!("Posts processed: {}", posts.len());
                }

                if let Some(todos) = state.get_loop_results("processed_todos") {
                    println!("Todos processed: {}", todos.len());
                }

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
