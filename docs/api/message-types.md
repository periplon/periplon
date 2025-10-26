# Message Types Reference

## Overview

The SDK provides strongly-typed message enums for type-safe communication with AI agents.

## Message Enum

The main `Message` enum represents all possible message types:

```rust
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    System(SystemMessage),
    Result(ResultMessage),
    StreamEvent(StreamEventMessage),
}
```

## UserMessage

Represents messages from the user to the agent:

```rust
pub struct UserMessage {
    pub message: UserMessageContent,
}

pub struct UserMessageContent {
    pub role: String,  // Always "user"
    pub content: Vec<ContentBlock>,
}
```

### Example

```rust
use periplon_sdk::{Message, UserMessage, ContentBlock};

let user_msg = UserMessage {
    message: UserMessageContent {
        role: "user".to_string(),
        content: vec![
            ContentBlock::Text {
                text: "Hello, agent!".to_string(),
            }
        ],
    },
};
```

## AssistantMessage

Represents messages from the AI agent:

```rust
pub struct AssistantMessage {
    pub message: AssistantMessageContent,
}

pub struct AssistantMessageContent {
    pub model: String,
    pub role: String,  // Always "assistant"
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}
```

### Example

```rust
match msg {
    Message::Assistant(assistant_msg) => {
        println!("Model: {}", assistant_msg.message.model);
        for block in assistant_msg.message.content {
            // Process content blocks
        }
    }
    _ => {}
}
```

## SystemMessage

System-level information and notifications:

```rust
pub struct SystemMessage {
    pub message: String,
    pub level: Option<String>,  // "info", "warning", "error"
}
```

## ResultMessage

Final result of a query with metadata:

```rust
pub struct ResultMessage {
    pub total_cost_usd: Option<f64>,
    pub total_tokens: Option<u64>,
    pub execution_time_ms: Option<u64>,
    pub status: String,  // "success", "error", etc.
}
```

### Example

```rust
match msg {
    Message::Result(result) => {
        println!("Total cost: ${:.4}", result.total_cost_usd.unwrap_or(0.0));
        println!("Total tokens: {}", result.total_tokens.unwrap_or(0));
        println!("Status: {}", result.status);
    }
    _ => {}
}
```

## StreamEventMessage

Real-time streaming events during query execution:

```rust
pub struct StreamEventMessage {
    pub event_type: String,
    pub data: Option<Value>,
}
```

## ContentBlock Enum

Content blocks represent different types of content within messages:

```rust
pub enum ContentBlock {
    Text { text: String },
    Thinking { thinking: String, signature: String },
    ToolUse { id: String, name: String, input: Value },
    ToolResult { tool_use_id: String, content: Option<Value>, is_error: Option<bool> },
}
```

### Text Block

Simple text content:

```rust
ContentBlock::Text {
    text: "Hello, world!".to_string(),
}
```

### Thinking Block

Extended thinking with signature:

```rust
ContentBlock::Thinking {
    thinking: "Let me analyze this...".to_string(),
    signature: "thinking_20240101_123456".to_string(),
}
```

### ToolUse Block

Tool invocation request:

```rust
ContentBlock::ToolUse {
    id: "toolu_123".to_string(),
    name: "Read".to_string(),
    input: serde_json::json!({
        "file_path": "/path/to/file"
    }),
}
```

### ToolResult Block

Result from tool execution:

```rust
ContentBlock::ToolResult {
    tool_use_id: "toolu_123".to_string(),
    content: Some(serde_json::json!({
        "output": "file contents..."
    })),
    is_error: Some(false),
}
```

## Usage Information

Token usage statistics:

```rust
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
}
```

## Processing Messages

### Complete Example

```rust
use periplon_sdk::{query, Message, ContentBlock};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = query("Analyze this code", None).await?;

    while let Some(msg) = stream.next().await {
        match msg {
            Message::User(user_msg) => {
                println!("User: {:?}", user_msg);
            }

            Message::Assistant(assistant_msg) => {
                for block in assistant_msg.message.content {
                    match block {
                        ContentBlock::Text { text } => {
                            println!("Text: {}", text);
                        }
                        ContentBlock::Thinking { thinking, signature } => {
                            println!("Thinking ({}): {}", signature, thinking);
                        }
                        ContentBlock::ToolUse { id, name, input } => {
                            println!("Tool {} ({}): {:?}", name, id, input);
                        }
                        ContentBlock::ToolResult { tool_use_id, content, is_error } => {
                            if is_error.unwrap_or(false) {
                                println!("Tool error ({}): {:?}", tool_use_id, content);
                            } else {
                                println!("Tool result ({}): {:?}", tool_use_id, content);
                            }
                        }
                    }
                }

                if let Some(usage) = assistant_msg.message.usage {
                    println!("Tokens: {} in, {} out",
                        usage.input_tokens,
                        usage.output_tokens);
                }
            }

            Message::System(system_msg) => {
                println!("System [{}]: {}",
                    system_msg.level.unwrap_or_else(|| "info".to_string()),
                    system_msg.message);
            }

            Message::Result(result) => {
                println!("Result: {}", result.status);
                if let Some(cost) = result.total_cost_usd {
                    println!("Cost: ${:.4}", cost);
                }
            }

            Message::StreamEvent(event) => {
                println!("Stream event: {}", event.event_type);
            }
        }
    }

    Ok(())
}
```

## Type Conversions

All message types implement `serde::Serialize` and `serde::Deserialize` for JSON serialization:

```rust
use periplon_sdk::Message;

// Deserialize from JSON
let json = r#"{"type": "assistant", "message": {...}}"#;
let msg: Message = serde_json::from_str(json)?;

// Serialize to JSON
let json = serde_json::to_string(&msg)?;
```
