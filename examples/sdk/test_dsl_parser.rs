//! Test DSL Parser
//!
//! This example tests parsing and validating DSL workflow files.

use periplon_sdk::{parse_workflow_file, validate_workflow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing DSL Parser\n");

    // Test each example workflow
    let workflows = vec![
        "examples/dsl/simple_file_organizer.yaml",
        "examples/dsl/research_pipeline.yaml",
        "examples/dsl/data_pipeline.yaml",
    ];

    for workflow_path in workflows {
        println!("Testing: {}", workflow_path);

        // Parse workflow
        let workflow = parse_workflow_file(workflow_path)?;
        println!("  ✓ Parsed successfully");
        println!("    Name: {} v{}", workflow.name, workflow.version);
        println!("    Agents: {}", workflow.agents.len());
        println!("    Tasks: {}", workflow.tasks.len());

        // Validate workflow
        validate_workflow(&workflow)?;
        println!("  ✓ Validation passed\n");
    }

    println!("All workflows parsed and validated successfully!");

    Ok(())
}
