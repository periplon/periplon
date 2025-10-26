# Error Handling Guide

## Overview

The SDK uses `thiserror` for comprehensive, structured error handling. All errors implement the standard `std::error::Error` trait.

## Error Types

```rust
use periplon_sdk::Error;

pub enum Error {
    CliNotFound,
    NotConnected,
    InvalidMessage(String),
    TransportError(String),
    ValidationError(String),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
}
```

## Error Variants

### CliNotFound

The CLI binary could not be found on the system.

```rust
match query("test", None).await {
    Err(Error::CliNotFound) => {
        eprintln!("CLI not installed. Please install it first.");
        eprintln!("See: https://docs.example.com/install");
    }
    Ok(stream) => { /* process */ }
    Err(e) => eprintln!("Error: {}", e),
}
```

### NotConnected

Attempted to use a client that hasn't been connected.

```rust
let mut client = PeriplonSDKClient::new(AgentOptions::default());

match client.query("test").await {
    Err(Error::NotConnected) => {
        eprintln!("Client not connected. Call connect() first.");
        client.connect(None).await?;
    }
    Ok(_) => { /* continue */ }
    Err(e) => eprintln!("Error: {}", e),
}
```

### InvalidMessage

Received a malformed or unexpected message format.

```rust
while let Some(msg) = stream.next().await {
    match msg {
        Err(Error::InvalidMessage(details)) => {
            eprintln!("Invalid message: {}", details);
            continue; // Skip and continue processing
        }
        Ok(message) => { /* process */ }
        Err(e) => return Err(e),
    }
}
```

### TransportError

Communication failure with the CLI process.

```rust
match client.connect(None).await {
    Err(Error::TransportError(details)) => {
        eprintln!("Communication failed: {}", details);
        eprintln!("Check if CLI process is running");
    }
    Ok(_) => { /* connected */ }
    Err(e) => eprintln!("Error: {}", e),
}
```

### ValidationError

DSL workflow validation failure.

```rust
use periplon_sdk::dsl::{parse_workflow, validate_workflow};

let workflow = parse_workflow("workflow.yaml")?;

match validate_workflow(&workflow) {
    Err(Error::ValidationError(details)) => {
        eprintln!("Workflow validation failed:");
        eprintln!("{}", details);
        // Display validation errors to user
    }
    Ok(_) => {
        println!("Workflow is valid");
        // Proceed with execution
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### IoError

File system or I/O operation failure.

```rust
use periplon_sdk::Error;

match std::fs::read_to_string("config.yaml") {
    Err(e) => {
        let error = Error::IoError(e);
        eprintln!("Failed to read config: {}", error);
    }
    Ok(content) => { /* process */ }
}
```

### SerdeError

JSON serialization/deserialization failure.

```rust
use periplon_sdk::Message;

let json = r#"{"invalid": "json"}"#;
match serde_json::from_str::<Message>(json) {
    Err(e) => {
        let error = Error::SerdeError(e);
        eprintln!("Failed to parse message: {}", error);
    }
    Ok(msg) => { /* process */ }
}
```

## Result Type

The SDK defines a custom `Result` type for convenience:

```rust
pub type Result<T> = std::result::Result<T, Error>;
```

Usage:

```rust
use periplon_sdk::Result;

async fn my_function() -> Result<String> {
    let stream = query("test", None).await?;
    // ... process stream
    Ok("result".to_string())
}
```

## Error Handling Patterns

### Match on Specific Errors

```rust
use periplon_sdk::{query, Error};

match query("test", None).await {
    Ok(stream) => { /* process stream */ }
    Err(Error::CliNotFound) => {
        eprintln!("Please install the CLI first");
    }
    Err(Error::TransportError(msg)) => {
        eprintln!("Communication error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unexpected error: {}", e);
    }
}
```

### Propagate Errors with ?

```rust
use periplon_sdk::Result;

async fn process_query() -> Result<()> {
    let mut stream = query("test", None).await?;

    while let Some(msg) = stream.next().await {
        let message = msg?;
        // Process message
    }

    Ok(())
}
```

### Custom Error Context

```rust
use periplon_sdk::{query, Result};

async fn query_with_context(prompt: &str) -> Result<String> {
    let mut stream = query(prompt, None).await
        .map_err(|e| {
            eprintln!("Failed to start query: {}", e);
            e
        })?;

    // Process stream...
    Ok("result".to_string())
}
```

### Graceful Degradation

```rust
use periplon_sdk::{query, Error};

async fn query_with_fallback(prompt: &str) -> String {
    match query(prompt, None).await {
        Ok(stream) => {
            // Process stream and return result
            "AI response".to_string()
        }
        Err(Error::CliNotFound) => {
            "AI service not available".to_string()
        }
        Err(e) => {
            eprintln!("Query failed: {}", e);
            "Error occurred".to_string()
        }
    }
}
```

### Retry Logic

```rust
use periplon_sdk::{query, Error};
use tokio::time::{sleep, Duration};

async fn query_with_retry(prompt: &str, max_retries: u32) -> Result<Stream> {
    let mut retries = 0;

    loop {
        match query(prompt, None).await {
            Ok(stream) => return Ok(stream),
            Err(Error::TransportError(msg)) if retries < max_retries => {
                retries += 1;
                eprintln!("Retry {}/{}: {}", retries, max_retries, msg);
                sleep(Duration::from_secs(2)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Error Display

All errors implement `Display` for human-readable messages:

```rust
use periplon_sdk::{query, Error};

match query("test", None).await {
    Err(e) => {
        // Detailed error message
        eprintln!("Error: {}", e);

        // For debugging, use Debug formatting
        eprintln!("Debug: {:?}", e);
    }
    Ok(_) => {}
}
```

## Best Practices

1. **Match Specific Errors**: Handle known error cases explicitly
2. **Provide Context**: Add context when propagating errors
3. **User-Friendly Messages**: Display helpful messages to end users
4. **Logging**: Log errors for debugging
5. **Graceful Degradation**: Provide fallback behavior when possible
6. **Don't Panic**: Avoid unwrap() in production code

### Complete Example

```rust
use periplon_sdk::{query, Error, Result, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    match process_query("Analyze this code").await {
        Ok(response) => {
            println!("Success: {}", response);
            Ok(())
        }
        Err(Error::CliNotFound) => {
            eprintln!("Error: CLI not installed");
            eprintln!("Install: npm install -g @anthropic/cli");
            std::process::exit(1);
        }
        Err(Error::TransportError(msg)) => {
            eprintln!("Communication error: {}", msg);
            eprintln!("Try restarting the CLI");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            std::process::exit(1);
        }
    }
}

async fn process_query(prompt: &str) -> Result<String> {
    let mut stream = query(prompt, None).await?;
    let mut response = String::new();

    while let Some(msg_result) = stream.next().await {
        let msg = msg_result?;

        match msg {
            Message::Assistant(assistant_msg) => {
                for block in assistant_msg.message.content {
                    if let ContentBlock::Text { text } = block {
                        response.push_str(&text);
                    }
                }
            }
            Message::System(system_msg) => {
                eprintln!("System: {}", system_msg.message);
            }
            _ => {}
        }
    }

    Ok(response)
}
```
