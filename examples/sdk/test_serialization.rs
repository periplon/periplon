//! Test serialization of DSL workflows to verify skip_serializing_if works

use periplon_sdk::dsl::{parse_workflow_file, serialize_workflow};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workflow_path = std::env::args()
        .nth(1)
        .expect("Usage: test_serialization <workflow.yaml>");

    println!("Reading workflow from: {}", workflow_path);
    let workflow = parse_workflow_file(&workflow_path)?;

    println!("\n=== Serialized Output (clean) ===\n");
    let yaml = serialize_workflow(&workflow)?;
    println!("{}", yaml);

    Ok(())
}
