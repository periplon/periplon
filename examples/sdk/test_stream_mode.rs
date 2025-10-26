use periplon_sdk::{AgentOptions, PeriplonSDKClient};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing stream mode with detailed debugging...");

    let home = std::env::var("HOME").expect("HOME environment variable not set");
    let cli_path = PathBuf::from(format!("{}/.claude/local/claude", home));

    let options = AgentOptions {
        cli_path: Some(cli_path),
        stderr: Some(Arc::new(|line| {
            eprintln!("[CLI STDERR] {}", line);
        })),
        ..Default::default()
    };

    let mut client = PeriplonSDKClient::new(options);

    println!("Connecting...");
    client.connect(None).await?;
    println!("Connected!");

    // Try to read the init message first
    println!("\nChecking for init message...");
    {
        use futures::StreamExt;
        let stream = client.receive_messages()?;
        futures::pin_mut!(stream);

        match tokio::time::timeout(tokio::time::Duration::from_secs(2), stream.next()).await {
            Ok(Some(msg)) => println!("Init message received: {:?}", msg),
            Ok(None) => println!("Stream ended"),
            Err(_) => println!("Timeout waiting for init message"),
        }
    }

    println!("\nSending simple query...");
    client.query("Hello, what is 2+2?").await?;
    println!("Query sent!");

    println!("\nWaiting for response messages...");
    {
        use futures::StreamExt;
        let stream = client.receive_messages()?;
        futures::pin_mut!(stream);

        let mut count = 0;
        loop {
            match tokio::time::timeout(tokio::time::Duration::from_secs(3), stream.next()).await {
                Ok(Some(msg)) => {
                    count += 1;
                    println!("Message {}: {:?}", count, msg);
                }
                Ok(None) => {
                    println!("Stream ended");
                    break;
                }
                Err(_) => {
                    println!("Timeout after {} messages", count);
                    break;
                }
            }
        }
    }

    println!("\nDisconnecting...");
    client.disconnect().await?;
    println!("Done!");

    Ok(())
}
