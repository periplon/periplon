use futures::StreamExt;
use periplon_sdk::{query, AgentOptions, ContentBlock, Message};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting simple query example...");

    // Try to use the CLI from the standard location
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let cli_path = PathBuf::from(format!("{}/.claude/local/claude", home));

    let options = if cli_path.exists() {
        Some(AgentOptions {
            cli_path: Some(cli_path),
            ..Default::default()
        })
    } else {
        None // Let the SDK try to find it automatically
    };

    let mut stream = query("What is 2 + 2?", options).await?;

    println!("\nReceiving messages:");
    while let Some(msg) = stream.next().await {
        match msg {
            Message::User(user_msg) => {
                println!("User: {:?}", user_msg.message.content);
            }
            Message::Assistant(assistant_msg) => {
                println!("\nAssistant (model: {}):", assistant_msg.message.model);
                for block in assistant_msg.message.content {
                    match block {
                        ContentBlock::Text { text } => {
                            println!("  Text: {}", text);
                        }
                        ContentBlock::Thinking { thinking, .. } => {
                            println!("  Thinking: {}", thinking);
                        }
                        ContentBlock::ToolUse { name, input, .. } => {
                            println!("  Tool use: {} with input: {:?}", name, input);
                        }
                        ContentBlock::ToolResult {
                            content, is_error, ..
                        } => {
                            println!(
                                "  Tool result (error: {}): {:?}",
                                is_error.unwrap_or(false),
                                content
                            );
                        }
                    }
                }
            }
            Message::System(system_msg) => {
                println!("System ({}): {:?}", system_msg.subtype, system_msg.data);
            }
            Message::StreamEvent(event_msg) => {
                println!("Stream event: {}", event_msg.uuid);
            }
            Message::Result(result_msg) => {
                println!("\n=== Result ===");
                println!(
                    "Duration: {}ms (API: {}ms)",
                    result_msg.duration_ms, result_msg.duration_api_ms
                );
                println!("Turns: {}", result_msg.num_turns);
                println!("Error: {}", result_msg.is_error);
                if let Some(cost) = result_msg.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
                if let Some(result) = result_msg.result {
                    println!("Result: {}", result);
                }
            }
        }
    }

    println!("\nQuery complete!");
    Ok(())
}
