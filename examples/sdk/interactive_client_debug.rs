use periplon_sdk::{AgentOptions, PeriplonSDKClient, ContentBlock, Message};
use futures::StreamExt;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting interactive client example with debug output...");

    // Try to use the CLI from the standard location
    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let cli_path = PathBuf::from(format!("{}/.claude/local/claude", home));

    println!("Checking CLI path: {:?}", cli_path);
    println!("CLI exists: {}", cli_path.exists());

    let options = AgentOptions {
        cli_path: if cli_path.exists() {
            println!("Using CLI path: {:?}", cli_path);
            Some(cli_path)
        } else {
            println!("CLI path not found, using auto-discovery");
            None
        },
        allowed_tools: vec!["Read".to_string(), "Bash".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        max_turns: Some(5),
        ..Default::default()
    };

    let mut client = PeriplonSDKClient::new(options);

    println!("Connecting to CLI...");
    match client.connect(None).await {
        Ok(_) => println!("Connected successfully!"),
        Err(e) => {
            eprintln!("Connection failed: {:?}", e);
            return Err(e.into());
        }
    }

    // First query
    println!("\n=== Query 1: List files ===");
    match client.query("List files in current directory").await {
        Ok(_) => println!("Query sent successfully"),
        Err(e) => {
            eprintln!("Query failed: {:?}", e);
            return Err(e.into());
        }
    }

    println!("Waiting for response...");
    {
        let stream = client.receive_response()?;
        futures::pin_mut!(stream);
        let mut msg_count = 0;
        while let Some(msg) = stream.next().await {
            msg_count += 1;
            println!(
                "Received message #{}: {:?}",
                msg_count,
                std::mem::discriminant(&msg)
            );
            print_message(&msg);
        }
        println!("Stream ended after {} messages", msg_count);
    }

    // Follow-up query
    println!("\n=== Query 2: Create README ===");
    match client
        .query("Create a README.md file with a brief description")
        .await
    {
        Ok(_) => println!("Query sent successfully"),
        Err(e) => {
            eprintln!("Query failed: {:?}", e);
            return Err(e.into());
        }
    }

    println!("Waiting for response...");
    {
        let stream = client.receive_response()?;
        futures::pin_mut!(stream);
        let mut msg_count = 0;
        while let Some(msg) = stream.next().await {
            msg_count += 1;
            println!(
                "Received message #{}: {:?}",
                msg_count,
                std::mem::discriminant(&msg)
            );
            print_message(&msg);
        }
        println!("Stream ended after {} messages", msg_count);
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
                        println!("  Assistant: {}", text);
                    }
                    ContentBlock::ToolUse { name, input, .. } => {
                        println!("  Using tool: {} with {:?}", name, input);
                    }
                    _ => {
                        println!("  Other content block: {:?}", std::mem::discriminant(block));
                    }
                }
            }
        }
        Message::Result(result_msg) => {
            println!(
                "  Result: {} turns, {}ms, cost: ${:.4}",
                result_msg.num_turns,
                result_msg.duration_ms,
                result_msg.total_cost_usd.unwrap_or(0.0)
            );
        }
        Message::User(user_msg) => {
            println!("  User: {:?}", user_msg.message.content);
        }
        Message::System(system_msg) => {
            println!("  System ({}): {:?}", system_msg.subtype, system_msg.data);
        }
        Message::StreamEvent(event_msg) => {
            println!("  Stream event: {}", event_msg.uuid);
        }
    }
}
