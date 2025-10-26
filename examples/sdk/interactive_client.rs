use periplon_sdk::{AgentOptions, PeriplonSDKClient, ContentBlock, Message};
use futures::StreamExt;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting interactive client example...");

    // Try to use the CLI from the standard location
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let cli_path = PathBuf::from(format!("{}/.claude/local/claude", home));

    let options = AgentOptions {
        cli_path: if cli_path.exists() {
            Some(cli_path)
        } else {
            None
        },
        allowed_tools: vec!["Read".to_string(), "Bash".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        max_turns: Some(5),
        ..Default::default()
    };

    let mut client = PeriplonSDKClient::new(options);

    println!("Connecting to CLI...");
    client.connect(None).await?;
    println!("Connected!");

    // First query
    println!("\n=== Query 1: List files ===");
    client.query("List files in current directory").await?;

    {
        let stream = client.receive_response()?;
        futures::pin_mut!(stream);
        while let Some(msg) = stream.next().await {
            print_message(&msg);
        }
    }

    // Follow-up query
    println!("\n=== Query 2: Create README ===");
    client
        .query("Create a README.md file with a brief description")
        .await?;

    {
        let stream = client.receive_response()?;
        futures::pin_mut!(stream);
        while let Some(msg) = stream.next().await {
            print_message(&msg);
        }
    }

    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("Done!");

    Ok(())
}

fn print_message(msg: &Message) {
    match msg {
        Message::Assistant(assistant_msg) => {
            for block in &assistant_msg.message.content {
                match block {
                    ContentBlock::Text { text } => {
                        println!("Assistant: {}", text);
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        println!("Using tool: {} with {:?}", name, input);
                    }
                    _ => {}
                }
            }
        }
        Message::Result(result_msg) => {
            println!(
                "Result: {} turns, {}ms, cost: ${:.4}",
                result_msg.num_turns,
                result_msg.duration_ms,
                result_msg.total_cost_usd.unwrap_or(0.0)
            );
        }
        _ => {}
    }
}
