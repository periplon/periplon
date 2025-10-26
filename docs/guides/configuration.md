# Configuration Guide

## Agent Options

Configure agent behavior using `AgentOptions`:

```rust
use periplon_sdk::AgentOptions;
use std::collections::HashMap;

let options = AgentOptions {
    // Tool filtering
    allowed_tools: vec!["Read".to_string(), "Write".to_string()],
    disallowed_tools: vec!["Bash".to_string()],

    // Permission mode
    permission_mode: Some("acceptEdits".to_string()),

    // Model and limits
    model: Some("claude-sonnet-4-5".to_string()),
    max_turns: Some(10),

    // Working directory
    cwd: Some("/path/to/project".into()),

    // Session options
    continue_conversation: true,
    include_partial_messages: false,

    // MCP servers
    mcp_servers: HashMap::new(),

    // Callbacks
    can_use_tool: None,
    hooks: None,
    stderr: None,

    ..Default::default()
};
```

## Permission Modes

Control how the agent handles operations that require permission:

- **`"default"`**: Ask for permission on dangerous operations (recommended)
- **`"acceptEdits"`**: Auto-approve file edits
- **`"plan"`**: Planning mode without execution
- **`"bypassPermissions"`**: Skip all permission checks (use with caution)

### Example: Different Permission Modes

```rust
// Safe mode - ask for everything
let safe_options = AgentOptions {
    permission_mode: Some("default".to_string()),
    ..Default::default()
};

// Development mode - auto-approve edits
let dev_options = AgentOptions {
    permission_mode: Some("acceptEdits".to_string()),
    ..Default::default()
};

// Planning mode - no execution
let plan_options = AgentOptions {
    permission_mode: Some("plan".to_string()),
    ..Default::default()
};
```

## Tool Filtering

Control which tools the agent can use:

```rust
// Allow only specific tools
let options = AgentOptions {
    allowed_tools: vec![
        "Read".to_string(),
        "Write".to_string(),
        "Grep".to_string(),
    ],
    ..Default::default()
};

// Block specific tools
let options = AgentOptions {
    disallowed_tools: vec![
        "Bash".to_string(),
        "WebFetch".to_string(),
    ],
    ..Default::default()
};
```

## Model Selection

Choose which AI model to use:

```rust
let options = AgentOptions {
    model: Some("claude-sonnet-4-5".to_string()),
    ..Default::default()
};
```

## Working Directory

Set the working directory for file operations:

```rust
use std::path::PathBuf;

let options = AgentOptions {
    cwd: Some(PathBuf::from("/path/to/project")),
    ..Default::default()
};
```

## Conversation Settings

Configure conversation behavior:

```rust
let options = AgentOptions {
    // Continue from previous conversation
    continue_conversation: true,

    // Include partial streaming messages
    include_partial_messages: false,

    // Limit conversation turns
    max_turns: Some(20),

    ..Default::default()
};
```

## MCP Server Integration

Configure MCP servers for extended capabilities:

```rust
use std::collections::HashMap;

let mut mcp_servers = HashMap::new();
mcp_servers.insert(
    "my-server".to_string(),
    serde_json::json!({
        "url": "http://localhost:3000",
        "capabilities": ["tool1", "tool2"]
    })
);

let options = AgentOptions {
    mcp_servers,
    ..Default::default()
};
```

## Callbacks

Implement custom permission and hook handlers:

```rust
use periplon_sdk::{AgentOptions, PermissionRequest, HookEvent};

let options = AgentOptions {
    can_use_tool: Some(Box::new(|req: PermissionRequest| {
        // Custom permission logic
        Box::pin(async move {
            println!("Permission requested for: {}", req.tool_name);
            Ok(true)
        })
    })),

    hooks: Some(Box::new(|event: HookEvent| {
        // Custom hook handler
        Box::pin(async move {
            println!("Hook triggered: {:?}", event);
            Ok(())
        })
    })),

    ..Default::default()
};
```

## Complete Example

```rust
use periplon_sdk::{PeriplonSDKClient, AgentOptions};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = AgentOptions {
        // Tools
        allowed_tools: vec!["Read".to_string(), "Write".to_string()],

        // Permissions
        permission_mode: Some("acceptEdits".to_string()),

        // Model
        model: Some("claude-sonnet-4-5".to_string()),
        max_turns: Some(10),

        // Environment
        cwd: Some(PathBuf::from("./my-project")),

        // Session
        continue_conversation: true,

        ..Default::default()
    };

    let mut client = PeriplonSDKClient::new(options);
    client.connect(None).await?;

    // Use the configured client
    client.query("List all Rust files").await?;

    client.disconnect().await?;
    Ok(())
}
```
