# Testing Utilities

Comprehensive testing utilities for the Periplon SDK, providing mock implementations and builder patterns for all major service interfaces.

## Overview

The testing module provides:

- **Mock Services**: Drop-in replacements for MCP servers, permission services, and hook services
- **Test Builders**: Convenient builders for creating test messages, notifications, and contexts
- **Verification Tools**: Call logs and decision tracking for asserting test behavior

## Mock Services

### MockMcpServer

Mock MCP server for testing tool integrations without requiring external services.

#### Features

- Custom tool handlers with flexible logic
- Static response tools for simple cases
- Error-generating tools for testing error handling
- Call logging for verification
- Tool call counting and argument matching

#### Example

```rust
use periplon_sdk::testing::MockMcpServer;
use periplon_sdk::ports::secondary::McpServer;
use serde_json::json;

#[tokio::test]
async fn test_mcp_integration() {
    let mut server = MockMcpServer::new("test-server");

    // Add a custom tool with logic
    server.with_tool(
        "calculate",
        "Performs math operations",
        json!({"type": "object"}),
        |args| {
            let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok(json!({"result": a + b}))
        }
    );

    // Add a static response tool
    server.with_static_tool(
        "status",
        "Returns server status",
        json!({}),
        json!({"status": "healthy"})
    );

    // Test tool execution
    let result = server.call_tool("calculate", json!({"a": 5, "b": 3})).await.unwrap();
    assert_eq!(result.content["result"], 8.0);

    // Verify call log
    assert_eq!(server.call_count("calculate"), 1);
    assert!(server.was_called_with("calculate", &json!({"a": 5, "b": 3})));
}
```

### MockPermissionService

Mock permission service for testing authorization flows and permission-based logic.

#### Features

- Allow-all, deny-all, or ask-all policies
- Per-tool permission rules
- Custom permission handlers
- Input transformation/sanitization
- Decision logging and verification

#### Example

```rust
use periplon_sdk::testing::{MockPermissionService, PermissionContextBuilder};
use periplon_sdk::domain::PermissionDecision;
use periplon_sdk::ports::secondary::PermissionService;
use serde_json::json;

#[tokio::test]
async fn test_permission_logic() {
    let mut service = MockPermissionService::new();

    // Allow specific tools
    service.allow_tool("Read");

    // Deny dangerous tools
    service.deny_tool("Bash", "Bash not allowed in tests");

    // Custom handler for conditional logic
    service.with_handler("Write", |_tool, input, _ctx| {
        let path = input.get("file_path").and_then(|p| p.as_str()).unwrap_or("");
        if path.starts_with("/tmp/") {
            PermissionDecision::Allow { updated_input: None }
        } else {
            PermissionDecision::Deny {
                reason: "Only /tmp writes allowed".to_string()
            }
        }
    });

    let ctx = PermissionContextBuilder::new().build();

    // Test allowed tool
    let decision = service.can_use_tool("Read", &json!({}), ctx.clone()).await.unwrap();
    assert!(matches!(decision, PermissionDecision::Allow { .. }));

    // Test denied tool
    let decision = service.can_use_tool("Bash", &json!({}), ctx.clone()).await.unwrap();
    assert!(matches!(decision, PermissionDecision::Deny { .. }));

    // Verify logging
    assert!(service.was_allowed("Read"));
    assert!(service.was_denied("Bash"));
}
```

### MockHookService

Mock hook service for testing lifecycle event handling and hook execution.

#### Features

- Continue-all or stop-all policies
- Per-event custom handlers
- Pre-defined hook responses (continue with message, stop with reason)
- Execution logging
- Event triggering verification

#### Example

```rust
use periplon_sdk::testing::{MockHookService, HookInputBuilder};
use periplon_sdk::domain::HookJSONOutput;
use periplon_sdk::ports::secondary::{HookEvent, HookService};
use serde_json::json;

#[tokio::test]
async fn test_hook_execution() {
    let mut service = MockHookService::new();

    // Add a pre-tool-use hook
    service.with_hook(HookEvent::PreToolUse, |input, _ctx| {
        if let periplon_sdk::domain::HookInput::PreToolUse { tool_name, tool_input, .. } = input {
            if tool_name == "Bash" {
                if let Some(cmd) = tool_input.get("command").and_then(|c| c.as_str()) {
                    if cmd.contains("rm -rf") {
                        return HookJSONOutput::Sync {
                            should_continue: Some(false),
                            suppress_output: None,
                            stop_reason: Some("Dangerous command blocked".to_string()),
                            decision: None,
                            system_message: None,
                            reason: None,
                            hook_specific_output: None,
                        };
                    }
                }
            }
        }

        HookJSONOutput::Sync {
            should_continue: Some(true),
            suppress_output: None,
            stop_reason: None,
            decision: None,
            system_message: None,
            reason: None,
            hook_specific_output: None,
        }
    });

    let ctx = periplon_sdk::domain::HookContext { signal: None };

    // Test dangerous command
    let input = HookInputBuilder::pre_tool_use("Bash", json!({"command": "sudo rm -rf /"}));
    let result = service.execute_hook(HookEvent::PreToolUse, input, ctx).await.unwrap();

    if let HookJSONOutput::Sync { should_continue, stop_reason, .. } = result {
        assert_eq!(should_continue, Some(false));
        assert!(stop_reason.is_some());
    }

    // Verify execution log
    assert!(service.was_triggered(&HookEvent::PreToolUse));
}
```

## Test Builders

### MessageBuilder

Build test messages with content blocks.

```rust
use periplon_sdk::testing::MessageBuilder;
use serde_json::json;

// Create user message
let msg = MessageBuilder::user("What is 2 + 2?");

// Create assistant message with multiple blocks
let msg = MessageBuilder::new()
    .text("I'll calculate that for you.")
    .tool_use("calc-1", "Calculate", json!({"expr": "2+2"}))
    .tool_result("calc-1", json!({"result": 4}), Some(false))
    .text("The answer is 4.")
    .build_assistant();
```

### NotificationBuilder

Build notification specs for testing notification delivery.

```rust
use periplon_sdk::testing::NotificationBuilder;
use periplon_sdk::dsl::NotificationPriority;

let notification = NotificationBuilder::new("Task completed")
    .title("Success")
    .priority(NotificationPriority::High)
    .console()
    .file("/var/log/tasks.log")
    .ntfy("https://ntfy.sh", "my-topic")
    .build();
```

### HookInputBuilder

Build hook inputs for testing hook execution.

```rust
use periplon_sdk::testing::HookInputBuilder;
use serde_json::json;

// Pre-tool-use hook input
let input = HookInputBuilder::pre_tool_use("Read", json!({"file_path": "test.txt"}));

// Post-tool-use hook input
let input = HookInputBuilder::post_tool_use(
    "Read",
    json!({"file_path": "test.txt"}),
    json!({"content": "file contents"})
);

// User prompt submit hook
let input = HookInputBuilder::user_prompt_submit("What is this file?");

// Stop hook
let input = HookInputBuilder::stop();
```

### PermissionContextBuilder

Build permission contexts for testing permission queries.

```rust
use periplon_sdk::testing::PermissionContextBuilder;

let ctx = PermissionContextBuilder::new().build();
```

## Complete Integration Test Example

```rust
use periplon_sdk::testing::{
    MockMcpServer, MockPermissionService, MockHookService,
    PermissionContextBuilder, HookInputBuilder
};
use periplon_sdk::ports::secondary::{McpServer, PermissionService, HookService, HookEvent};
use periplon_sdk::domain::PermissionDecision;
use serde_json::json;

#[tokio::test]
async fn test_complete_workflow() {
    // Setup all mock services
    let mut mcp_server = MockMcpServer::new("test-server");
    mcp_server.with_static_tool("status", "Check status", json!({}), json!({"ok": true}));

    let mut permission_service = MockPermissionService::allow_all();
    permission_service.deny_tool("Bash", "No bash allowed");

    let hook_service = MockHookService::continue_all();

    // Test MCP server
    let result = mcp_server.call_tool("status", json!({})).await.unwrap();
    assert_eq!(result.content["ok"], true);

    // Test permission service
    let ctx = PermissionContextBuilder::new().build();
    let decision = permission_service.can_use_tool("Read", &json!({}), ctx).await.unwrap();
    assert!(matches!(decision, PermissionDecision::Allow { .. }));

    // Test hook service
    let hook_ctx = periplon_sdk::domain::HookContext { signal: None };
    let input = HookInputBuilder::pre_tool_use("Read", json!({}));
    let _result = hook_service.execute_hook(HookEvent::PreToolUse, input, hook_ctx).await.unwrap();

    // Verify all services logged operations
    assert_eq!(mcp_server.call_count("status"), 1);
    assert!(permission_service.was_allowed("Read"));
    assert!(hook_service.was_triggered(&HookEvent::PreToolUse));
}
```

## Best Practices

1. **Use Specific Assertions**: Verify exact behavior rather than just "no errors"
2. **Check Call Logs**: Always verify that services were called as expected
3. **Test Error Paths**: Use `with_error_tool` and deny rules to test error handling
4. **Isolate Tests**: Each test should create its own mock instances
5. **Clear Logs**: Call `clear_log()` if reusing mocks between test phases

## See Also

- [test_utilities_demo.rs](../../../tests/test_utilities_demo.rs) - Complete working examples
- [API Documentation](https://docs.rs/periplon-sdk) - Full API reference
