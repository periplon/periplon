use periplon_sdk::{query, AgentOptions, ContentBlock, Message};
use futures::StreamExt;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting simple query example with custom CLI path...");

    // Specify the CLI path explicitly
    let options = AgentOptions {
        cli_path: Some(PathBuf::from("/path/to/your/claude")), // Update this path
        ..Default::default()
    };

    let mut stream = query("What is 2 + 2?", Some(options)).await?;

    println!("\nReceiving messages:");
    while let Some(msg) = stream.next().await {
        match msg {
            Message::Assistant(assistant_msg) => {
                println!("\nAssistant (model: {}):", assistant_msg.message.model);
                for block in assistant_msg.message.content {
                    match block {
                        ContentBlock::Text { text } => {
                            println!("  Text: {}", text);
                        }
                        _ => {}
                    }
                }
            }
            Message::Result(result_msg) => {
                println!("\n=== Result ===");
                println!("Duration: {}ms", result_msg.duration_ms);
                if let Some(cost) = result_msg.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
            }
            _ => {}
        }
    }

    println!("\nQuery complete!");
    Ok(())
}
