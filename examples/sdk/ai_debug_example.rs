//! AI-Powered Debugging Example
//!
//! Demonstrates using AI features in the REPL debugger:
//! - Generate workflow blocks from natural language
//! - Get AI suggestions for fixing errors
//! - Analyze and explain workflows
//! - Change AI providers on-the-fly
use periplon_sdk::dsl::debug_ai::{AiProviderType, DebugAiAssistant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AI-Powered Debugging Example\n");

    // Example 1: Generate workflow block from description
    println!("=== Example 1: Generate Workflow Block ===");
    let assistant = DebugAiAssistant::new()?;

    let description = "Create a workflow that researches a topic and writes a summary";
    println!("Description: {}", description);
    println!("\nGenerating workflow block...\n");

    match assistant.generate_block(description).await {
        Ok(yaml) => {
            println!("Generated YAML:\n{}\n", yaml);
        }
        Err(e) => {
            eprintln!("❌ Generation failed: {}", e);
            eprintln!("Note: Make sure Ollama is running: `ollama serve`");
            eprintln!("And pull the default model: `ollama pull llama3.3`\n");
        }
    }

    // Example 2: Get fix suggestion for an error
    println!("=== Example 2: AI Fix Suggestion ===");

    let error_message = "Task 'analyze' failed: agent 'researcher' not found";
    let context = "Current task: analyze\nPrevious task: research (completed)";

    println!("Error: {}", error_message);
    println!("Context: {}\n", context);
    println!("Getting AI suggestion...\n");

    match assistant.suggest_fix(error_message, context).await {
        Ok(suggestion) => {
            println!("AI Suggestion:\n{}\n", suggestion);
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);
        }
    }

    // Example 3: Analyze workflow
    println!("=== Example 3: Analyze Workflow ===");

    let workflow_yaml = r#"
name: "Research Workflow"
version: "1.0.0"

agents:
  researcher:
    description: "Research agent"
    tools: [Read, WebSearch]

tasks:
  research:
    description: "Research the topic"
    agent: "researcher"

  analyze:
    description: "Analyze findings"
    agent: "researcher"
    depends_on: [research]
"#;

    println!("Analyzing workflow...\n");

    match assistant.analyze_workflow(workflow_yaml).await {
        Ok(analysis) => {
            println!("Analysis:\n{}\n", analysis);
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);
        }
    }

    // Example 4: Explain workflow
    println!("=== Example 4: Explain Workflow ===");

    println!("Explaining workflow...\n");

    match assistant.explain_workflow(workflow_yaml).await {
        Ok(explanation) => {
            println!("Explanation:\n{}\n", explanation);
        }
        Err(e) => {
            eprintln!("❌ Explanation failed: {}", e);
        }
    }

    // Example 5: Change AI provider
    println!("=== Example 5: Change AI Provider ===");

    let mut assistant = assistant;

    println!(
        "Current config: {:?} ({})",
        assistant.config().provider,
        assistant.config().model
    );

    // Try to switch to OpenAI (will fail if OPENAI_API_KEY not set)
    println!("\nAttempting to switch to OpenAI...");
    match assistant.set_provider(AiProviderType::OpenAi, "gpt-4o".to_string()) {
        Ok(()) => {
            println!("✓ Switched to OpenAI (gpt-4o)");
            println!(
                "New config: {:?} ({})",
                assistant.config().provider,
                assistant.config().model
            );
        }
        Err(e) => {
            println!("❌ Failed to switch: {}", e);
            println!("Note: Set OPENAI_API_KEY environment variable to use OpenAI");
        }
    }

    // Switch back to Ollama
    println!("\nSwitching back to Ollama...");
    match assistant.set_provider(AiProviderType::Ollama, "llama3.3".to_string()) {
        Ok(()) => {
            println!("✓ Switched to Ollama (llama3.3)");
        }
        Err(e) => {
            println!("❌ Failed to switch: {}", e);
        }
    }

    // Example 6: Show all configuration
    println!("\n=== Example 6: Full Configuration ===");
    let config = assistant.config();
    println!("Provider: {:?}", config.provider);
    println!("Model: {}", config.model);
    if let Some(ref endpoint) = config.endpoint {
        println!("Endpoint: {}", endpoint);
    }
    println!("Temperature: {}", config.temperature);
    println!("Max Tokens: {}", config.max_tokens);

    println!("\n=== REPL Usage ===");
    println!("\nTo use these features interactively in the REPL:");
    println!("  1. cargo run --example repl_debug_example");
    println!("  2. Use these commands:");
    println!("     - ai-generate <description>     Generate workflow block");
    println!("     - ai-fix <error_message>        Get fix suggestion");
    println!("     - ai-analyze [workflow.yaml]    Analyze workflow");
    println!("     - ai-explain [workflow.yaml]    Explain workflow");
    println!("     - ai-provider <provider> [model] Change AI provider");
    println!("     - ai-config                     Show AI configuration");
    println!("\nAvailable providers: ollama, openai, anthropic, google");
    println!("\nExample session:");
    println!("  debug> ai-generate Create a task that writes to a file");
    println!("  debug> ai-provider openai gpt-4o");
    println!("  debug> ai-analyze examples/workflows/simple.yaml");
    println!("  debug> ai-fix Task failed: permission denied");

    Ok(())
}
