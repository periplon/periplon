//! Mock MCP Server for Testing
//!
//! Provides a configurable mock MCP server implementation for testing
//! MCP-based workflows without requiring external server dependencies.

use crate::error::{Error, Result};
use crate::ports::secondary::{McpServer, ToolDefinition, ToolResult};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock MCP server for testing
///
/// # Examples
///
/// ```
/// use periplon_sdk::testing::MockMcpServer;
/// use periplon_sdk::ports::secondary::McpServer;
/// use serde_json::json;
///
/// let mut server = MockMcpServer::new("test-server");
///
/// // Add a tool with custom handler
/// server.with_tool(
///     "greet",
///     "Greets a person",
///     json!({"type": "object", "properties": {"name": {"type": "string"}}}),
///     |args| {
///         let name = args.get("name").and_then(|n| n.as_str()).unwrap_or("World");
///         Ok(json!({"greeting": format!("Hello, {}!", name)}))
///     }
/// );
///
/// // Use in tests
/// # tokio_test::block_on(async {
/// let tools = server.list_tools().await.unwrap();
/// assert_eq!(tools.len(), 1);
/// assert_eq!(tools[0].name, "greet");
///
/// let result = server.call_tool("greet", json!({"name": "Alice"})).await.unwrap();
/// assert!(!result.is_error);
/// # });
/// ```
#[derive(Clone)]
pub struct MockMcpServer {
    name: String,
    tools: Arc<Mutex<HashMap<String, ToolConfig>>>,
    call_log: Arc<Mutex<Vec<ToolCall>>>,
}

#[derive(Clone)]
struct ToolConfig {
    definition: ToolDefinition,
    handler: Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>,
}

/// Record of a tool call for testing verification
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub tool_name: String,
    pub args: Value,
    pub timestamp: std::time::SystemTime,
}

impl MockMcpServer {
    /// Create a new mock MCP server with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tools: Arc::new(Mutex::new(HashMap::new())),
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a tool with a custom handler function
    ///
    /// # Examples
    ///
    /// ```
    /// # use periplon_sdk::testing::MockMcpServer;
    /// # use serde_json::json;
    /// let mut server = MockMcpServer::new("test");
    ///
    /// server.with_tool(
    ///     "add",
    ///     "Adds two numbers",
    ///     json!({"type": "object", "properties": {
    ///         "a": {"type": "number"},
    ///         "b": {"type": "number"}
    ///     }}),
    ///     |args| {
    ///         let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
    ///         let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
    ///         Ok(json!({"result": a + b}))
    ///     }
    /// );
    /// ```
    pub fn with_tool<F>(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        handler: F,
    ) -> &mut Self
    where
        F: Fn(Value) -> Result<Value> + Send + Sync + 'static,
    {
        let name = name.into();
        let definition = ToolDefinition {
            name: name.clone(),
            description: description.into(),
            input_schema,
        };

        let config = ToolConfig {
            definition,
            handler: Arc::new(handler),
        };

        self.tools.lock().unwrap().insert(name, config);
        self
    }

    /// Add a tool that always returns a static response
    pub fn with_static_tool(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        response: Value,
    ) -> &mut Self {
        self.with_tool(name, description, input_schema, move |_| {
            Ok(response.clone())
        })
    }

    /// Add a tool that always returns an error
    pub fn with_error_tool(
        &mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        error_message: impl Into<String>,
    ) -> &mut Self {
        let error_msg = error_message.into();
        self.with_tool(name, description, input_schema, move |_| {
            Err(Error::InvalidInput(error_msg.clone()))
        })
    }

    /// Get a record of all tool calls made to this server
    pub fn call_log(&self) -> Vec<ToolCall> {
        self.call_log.lock().unwrap().clone()
    }

    /// Clear the call log
    pub fn clear_call_log(&self) {
        self.call_log.lock().unwrap().clear();
    }

    /// Get the number of times a specific tool was called
    pub fn call_count(&self, tool_name: &str) -> usize {
        self.call_log
            .lock()
            .unwrap()
            .iter()
            .filter(|call| call.tool_name == tool_name)
            .count()
    }

    /// Check if a tool was called with specific arguments
    pub fn was_called_with(&self, tool_name: &str, args: &Value) -> bool {
        self.call_log
            .lock()
            .unwrap()
            .iter()
            .any(|call| call.tool_name == tool_name && &call.args == args)
    }
}

#[async_trait]
impl McpServer for MockMcpServer {
    fn name(&self) -> &str {
        &self.name
    }

    async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        let tools = self.tools.lock().unwrap();
        Ok(tools
            .values()
            .map(|config| config.definition.clone())
            .collect())
    }

    async fn call_tool(&self, name: &str, args: Value) -> Result<ToolResult> {
        // Record the call
        self.call_log.lock().unwrap().push(ToolCall {
            tool_name: name.to_string(),
            args: args.clone(),
            timestamp: std::time::SystemTime::now(),
        });

        // Execute the handler
        let tools = self.tools.lock().unwrap();
        let config = tools
            .get(name)
            .ok_or_else(|| Error::InvalidInput(format!("Tool '{}' not found", name)))?;

        match (config.handler)(args) {
            Ok(content) => Ok(ToolResult {
                content,
                is_error: false,
            }),
            Err(e) => Ok(ToolResult {
                content: json!({"error": e.to_string()}),
                is_error: true,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_tool() {
        let mut server = MockMcpServer::new("test");
        server.with_static_tool(
            "ping",
            "Returns pong",
            json!({}),
            json!({"response": "pong"}),
        );

        let tools = server.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "ping");

        let result = server.call_tool("ping", json!({})).await.unwrap();
        assert!(!result.is_error);
        assert_eq!(result.content, json!({"response": "pong"}));
    }

    #[tokio::test]
    async fn test_custom_handler() {
        let mut server = MockMcpServer::new("test");
        server.with_tool("add", "Adds numbers", json!({"type": "object"}), |args| {
            let a = args.get("a").and_then(|v| v.as_i64()).unwrap_or(0);
            let b = args.get("b").and_then(|v| v.as_i64()).unwrap_or(0);
            Ok(json!({"sum": a + b}))
        });

        let result = server
            .call_tool("add", json!({"a": 5, "b": 3}))
            .await
            .unwrap();
        assert_eq!(result.content, json!({"sum": 8}));
    }

    #[tokio::test]
    async fn test_error_tool() {
        let mut server = MockMcpServer::new("test");
        server.with_error_tool("fail", "Always fails", json!({}), "This tool always fails");

        let result = server.call_tool("fail", json!({})).await.unwrap();
        assert!(result.is_error);
    }

    #[tokio::test]
    async fn test_call_log() {
        let mut server = MockMcpServer::new("test");
        server.with_static_tool("tool1", "Tool 1", json!({}), json!({}));

        server
            .call_tool("tool1", json!({"arg": "value1"}))
            .await
            .unwrap();
        server
            .call_tool("tool1", json!({"arg": "value2"}))
            .await
            .unwrap();

        let log = server.call_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].tool_name, "tool1");
        assert_eq!(log[1].tool_name, "tool1");

        assert_eq!(server.call_count("tool1"), 2);
        assert!(server.was_called_with("tool1", &json!({"arg": "value1"})));

        server.clear_call_log();
        assert_eq!(server.call_log().len(), 0);
    }

    #[tokio::test]
    async fn test_tool_not_found() {
        let server = MockMcpServer::new("test");
        let result = server.call_tool("nonexistent", json!({})).await;
        assert!(result.is_err());
    }
}
