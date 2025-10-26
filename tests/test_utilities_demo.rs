//! Examples and Integration Tests for Test Utilities
//!
//! This module demonstrates how to use the various test utilities
//! provided by the SDK for testing workflows, agents, and integrations.

#[cfg(test)]
mod test_utilities_examples {
    use periplon_sdk::testing::{
        HookInputBuilder, MessageBuilder, MockHookService, MockMcpServer,
        MockPermissionService, NotificationBuilder, PermissionContextBuilder,
    };
    use periplon_sdk::domain::{ContentBlock, HookJSONOutput, PermissionDecision};
    use periplon_sdk::dsl::NotificationPriority;
    use periplon_sdk::ports::secondary::{HookEvent, HookService, McpServer, PermissionService};
    use serde_json::json;

    /// Example: Testing MCP Server Integration
    #[tokio::test]
    async fn example_mcp_server_testing() {
        // Create a mock MCP server with custom tools
        let mut server = MockMcpServer::new("example-server");

        // Add a simple echo tool
        server.with_tool(
            "echo",
            "Echoes the input message",
            json!({
                "type": "object",
                "properties": {
                    "message": {"type": "string"}
                }
            }),
            |args| {
                let message = args
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("No message");
                Ok(json!({"echo": message}))
            },
        );

        // Test the echo tool
        let result = server
            .call_tool("echo", json!({"message": "Hello, World!"}))
            .await
            .unwrap();
        assert!(!result.is_error);
        assert_eq!(result.content["echo"], "Hello, World!");

        // Verify call log
        assert_eq!(server.call_count("echo"), 1);
    }

    /// Example: Testing Permission Service Behavior
    #[tokio::test]
    async fn example_permission_service_testing() {
        // Create a permission service with custom rules
        let mut service = MockPermissionService::new();

        // Allow read operations
        service.allow_tool("Read");

        // Deny bash commands
        service.deny_tool("Bash", "Bash commands not allowed in production");

        let ctx = PermissionContextBuilder::new().build();

        // Test Read (should be allowed)
        let decision = service
            .can_use_tool("Read", &json!({"file_path": "config.yaml"}), ctx.clone())
            .await
            .unwrap();
        assert!(matches!(decision, PermissionDecision::Allow { .. }));

        // Test Bash (should be denied)
        let decision = service
            .can_use_tool("Bash", &json!({"command": "rm -rf /"}), ctx)
            .await
            .unwrap();
        assert!(matches!(decision, PermissionDecision::Deny { .. }));

        // Verify logging
        assert!(service.was_allowed("Read"));
        assert!(service.was_denied("Bash"));
    }

    /// Example: Testing Hook Execution
    #[tokio::test]
    async fn example_hook_service_testing() {
        // Create a hook service
        let mut service = MockHookService::new();

        // Add a pre-tool-use hook
        service.continue_with_message(HookEvent::PreToolUse, "Hook executed");

        let ctx = periplon_sdk::domain::HookContext { signal: None };
        let input = HookInputBuilder::pre_tool_use("Read", json!({}));

        let result = service
            .execute_hook(HookEvent::PreToolUse, input, ctx)
            .await
            .unwrap();

        if let HookJSONOutput::Sync { should_continue, .. } = result {
            assert_eq!(should_continue, Some(true));
        }

        // Verify execution log
        assert!(service.was_triggered(&HookEvent::PreToolUse));
    }

    /// Example: Building Test Messages
    #[test]
    fn example_message_builder() {
        // Create content blocks
        let content = MessageBuilder::new()
            .text("Processing file...")
            .tool_use("tool-1", "Read", json!({"file_path": "test.txt"}))
            .build();

        assert_eq!(content.len(), 2);

        if let ContentBlock::Text { text } = &content[0] {
            assert!(text.contains("Processing"));
        } else {
            panic!("Expected text block");
        }
    }

    /// Example: Building Notifications
    #[test]
    fn example_notification_builder() {
        let notification = NotificationBuilder::new("Task completed")
            .title("Success")
            .priority(NotificationPriority::High)
            .console()
            .build();

        if let periplon_sdk::dsl::NotificationSpec::Structured {
            message,
            channels,
            ..
        } = notification
        {
            assert_eq!(message, "Task completed");
            assert_eq!(channels.len(), 1);
        }
    }
}
