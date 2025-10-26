# Quick Start Guide

## Simple Query

The easiest way to get started is with a one-shot query:

```rust
use periplon_sdk::{query, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = query("What is 2 + 2?", None).await?;

    while let Some(msg) = stream.next().await {
        match msg {
            Message::Assistant(assistant_msg) => {
                for block in assistant_msg.message.content {
                    if let ContentBlock::Text { text } = block {
                        println!("Assistant: {}", text);
                    }
                }
            }
            Message::Result(result_msg) => {
                println!("Cost: ${:.4}", result_msg.total_cost_usd.unwrap_or(0.0));
            }
            _ => {}
        }
    }

    Ok(())
}
```

## Interactive Client

For multi-turn conversations, use the `PeriplonSDKClient`:

```rust
use periplon_sdk::{PeriplonSDKClient, AgentOptions};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = AgentOptions {
        allowed_tools: vec!["Read".to_string(), "Bash".to_string()],
        permission_mode: Some("acceptEdits".to_string()),
        ..Default::default()
    };

    let mut client = PeriplonSDKClient::new(options);
    client.connect(None).await?;

    // First query
    client.query("List files in current directory").await?;
    let mut stream = client.receive_response()?;
    while let Some(msg) = stream.next().await {
        println!("{:?}", msg);
    }

    // Follow-up query
    client.query("Create a README.md file").await?;
    let mut stream = client.receive_response()?;
    while let Some(msg) = stream.next().await {
        println!("{:?}", msg);
    }

    client.disconnect().await?;

    Ok(())
}
```

## Running Examples

The repository includes several examples to help you get started:

```bash
# Simple query example
cargo run --example simple_query

# Interactive client example
cargo run --example interactive_client

# DSL executor example
cargo run --example dsl_executor_example
```

## Next Steps

- Learn about [Configuration](configuration.md) options
- Explore [Message Types](../api/message-types.md)
- Understand [Error Handling](error-handling.md)
- Try the [DSL System](dsl-overview.md)
