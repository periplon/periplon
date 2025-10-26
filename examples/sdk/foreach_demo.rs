//! ForEach Loop Demo
//!
//! This example demonstrates ForEach loop patterns in the DSL:
//! - Inline collections
//! - Range-based iteration
//! - File-based collections
//!
//! Usage:
//! ```bash
//! cargo run --example foreach_demo
//! ```

use periplon_sdk::dsl::{parse_workflow_file, DSLExecutor};
use periplon_sdk::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== ForEach Loop Demo ===\n");

    // Create sample data file for file-based collection example
    let sample_data = serde_json::json!([
        {"name": "Item 1", "value": 100},
        {"name": "Item 2", "value": 200},
        {"name": "Item 3", "value": 300}
    ]);

    std::fs::write(
        "/tmp/demo_data.json",
        serde_json::to_string_pretty(&sample_data)?,
    )?;

    println!("Created sample data file: /tmp/demo_data.json\n");

    // Load and parse the workflow
    let workflow_path = "examples/workflows/foreach_loop_demo.yaml";
    println!("Loading workflow from: {}\n", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;

    println!("Workflow: {}", workflow.name);
    println!("Version: {}", workflow.version);
    println!("Tasks: {}\n", workflow.tasks.len());

    // Create and initialize executor
    let mut executor = DSLExecutor::new(workflow)?;

    println!("Initializing executor...\n");
    executor.initialize().await?;

    println!("Executing workflow with ForEach loops...\n");
    println!("This will demonstrate:");
    println!("  1. Processing an inline collection of items");
    println!("  2. Iterating over a numeric range");
    println!("  3. Processing data from a JSON file");
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

                // Show loop results if available
                if let Some(results) = state.get_loop_results("process_inline_items") {
                    println!(
                        "\nInline Items Loop Results: {} items processed",
                        results.len()
                    );
                }

                if let Some(results) = state.get_loop_results("process_range") {
                    println!("Range Loop Results: {} numbers processed", results.len());
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
    std::fs::remove_file("/tmp/demo_data.json").ok();

    println!("\n=== Demo Complete ===");
    Ok(())
}
