//! DSL Executor Example
//!
//! This example demonstrates how to load and execute a DSL workflow.

use periplon_sdk::{parse_workflow_file, validate_workflow, DSLExecutor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse workflow from YAML file
    let workflow_path = "examples/dsl/simple_file_organizer.yaml";
    println!("Loading workflow from: {}", workflow_path);

    let workflow = parse_workflow_file(workflow_path)?;
    println!("Loaded workflow: {} v{}", workflow.name, workflow.version);

    // Validate workflow
    println!("Validating workflow...");
    validate_workflow(&workflow)?;
    println!("Workflow validation passed");

    // Create executor
    println!("Creating executor...");
    let mut executor = DSLExecutor::new(workflow)?;

    // Initialize executor (connect agents, build task graph)
    println!("Initializing executor...");
    executor.initialize().await?;

    println!(
        "Executor initialized with {} tasks",
        executor.get_task_count()
    );

    // Execute workflow
    println!("\n=== Starting workflow execution ===\n");
    executor.execute().await?;

    println!("\n=== Workflow execution complete ===\n");

    // Shutdown executor
    executor.shutdown().await?;

    Ok(())
}
