//! Mock Permission Service for Testing
//!
//! Provides a configurable mock permission service for testing
//! permission-based workflows and agent behavior.

use crate::domain::{PermissionDecision, ToolPermissionContext};
use crate::error::Result;
use crate::ports::secondary::PermissionService;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock permission service for testing
///
/// # Examples
///
/// ```
/// use periplon_sdk::adapters::secondary::MockPermissionService;
/// use periplon_sdk::domain::{PermissionDecision, ToolPermissionContext};
/// use serde_json::json;
///
/// // Allow all permissions by default
/// let service = MockPermissionService::allow_all();
///
/// // Deny specific tools
/// let mut service = MockPermissionService::new();
/// service.deny_tool("Bash", "Bash commands are not allowed in tests");
/// service.allow_tool("Read");
///
/// // Custom handler for specific tool
/// service.with_handler("Write", |tool, input, _ctx| {
///     let path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
///     if path.starts_with("/tmp/") {
///         PermissionDecision::Allow { updated_input: None }
///     } else {
///         PermissionDecision::Deny {
///             reason: "Can only write to /tmp directory".to_string()
///         }
///     }
/// });
/// ```
#[derive(Clone)]
pub struct MockPermissionService {
    default_decision: DefaultDecision,
    handlers: Arc<Mutex<HashMap<String, PermissionHandler>>>,
    decision_log: Arc<Mutex<Vec<PermissionQuery>>>,
}

#[derive(Debug, Clone, Copy)]
enum DefaultDecision {
    Allow,
    Deny,
    Ask,
}

type PermissionHandler = Arc<
    dyn Fn(&str, &Value, &ToolPermissionContext) -> PermissionDecision + Send + Sync,
>;

/// Record of a permission query for testing verification
#[derive(Debug, Clone)]
pub struct PermissionQuery {
    pub tool_name: String,
    pub input: Value,
    pub decision: String, // "Allow", "Deny", or "Ask"
    pub timestamp: std::time::SystemTime,
}

impl MockPermissionService {
    /// Create a new mock permission service with Ask as default
    pub fn new() -> Self {
        Self {
            default_decision: DefaultDecision::Ask,
            handlers: Arc::new(Mutex::new(HashMap::new())),
            decision_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a permission service that allows everything
    pub fn allow_all() -> Self {
        Self {
            default_decision: DefaultDecision::Allow,
            handlers: Arc::new(Mutex::new(HashMap::new())),
            decision_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a permission service that denies everything
    pub fn deny_all(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        let service = Self {
            default_decision: DefaultDecision::Deny,
            handlers: Arc::new(Mutex::new(HashMap::new())),
            decision_log: Arc::new(Mutex::new(Vec::new())),
        };

        // Set default handler for all tools
        let default_reason = reason.clone();
        let handler: PermissionHandler = Arc::new(move |_, _, _| {
            PermissionDecision::Deny {
                reason: default_reason.clone(),
            }
        });

        service.handlers.lock().unwrap().insert("*".to_string(), handler);
        service
    }

    /// Create a permission service that asks for everything
    pub fn ask_all() -> Self {
        Self {
            default_decision: DefaultDecision::Ask,
            handlers: Arc::new(Mutex::new(HashMap::new())),
            decision_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Allow a specific tool unconditionally
    pub fn allow_tool(&mut self, tool_name: impl Into<String>) -> &mut Self {
        let name = tool_name.into();
        let handler: PermissionHandler = Arc::new(|_, _, _| {
            PermissionDecision::Allow { updated_input: None }
        });
        self.handlers.lock().unwrap().insert(name, handler);
        self
    }

    /// Deny a specific tool with a reason
    pub fn deny_tool(&mut self, tool_name: impl Into<String>, reason: impl Into<String>) -> &mut Self {
        let name = tool_name.into();
        let reason = reason.into();
        let handler: PermissionHandler = Arc::new(move |_, _, _| {
            PermissionDecision::Deny {
                reason: reason.clone(),
            }
        });
        self.handlers.lock().unwrap().insert(name, handler);
        self
    }

    /// Add a custom handler for a specific tool
    ///
    /// # Examples
    ///
    /// ```
    /// # use periplon_sdk::adapters::secondary::MockPermissionService;
    /// # use periplon_sdk::domain::PermissionDecision;
    /// let mut service = MockPermissionService::new();
    ///
    /// service.with_handler("Write", |tool, input, ctx| {
    ///     let path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
    ///     if path.ends_with(".txt") {
    ///         PermissionDecision::Allow { updated_input: None }
    ///     } else {
    ///         PermissionDecision::Deny {
    ///             reason: "Only .txt files allowed".to_string()
    ///         }
    ///     }
    /// });
    /// ```
    pub fn with_handler<F>(&mut self, tool_name: impl Into<String>, handler: F) -> &mut Self
    where
        F: Fn(&str, &Value, &ToolPermissionContext) -> PermissionDecision + Send + Sync + 'static,
    {
        let name = tool_name.into();
        self.handlers.lock().unwrap().insert(name, Arc::new(handler));
        self
    }

    /// Allow tool with input transformation
    pub fn allow_with_transform<F>(
        &mut self,
        tool_name: impl Into<String>,
        transform: F,
    ) -> &mut Self
    where
        F: Fn(&Value) -> Value + Send + Sync + 'static,
    {
        let name = tool_name.into();
        let handler: PermissionHandler = Arc::new(move |_, input, _| {
            PermissionDecision::Allow {
                updated_input: Some(transform(input)),
            }
        });
        self.handlers.lock().unwrap().insert(name, handler);
        self
    }

    /// Get decision log for verification
    pub fn decision_log(&self) -> Vec<PermissionQuery> {
        self.decision_log.lock().unwrap().clone()
    }

    /// Clear the decision log
    pub fn clear_log(&self) {
        self.decision_log.lock().unwrap().clear();
    }

    /// Get the number of times a tool was queried
    pub fn query_count(&self, tool_name: &str) -> usize {
        self.decision_log
            .lock()
            .unwrap()
            .iter()
            .filter(|q| q.tool_name == tool_name)
            .count()
    }

    /// Check if a tool was allowed
    pub fn was_allowed(&self, tool_name: &str) -> bool {
        self.decision_log
            .lock()
            .unwrap()
            .iter()
            .any(|q| q.tool_name == tool_name && q.decision == "Allow")
    }

    /// Check if a tool was denied
    pub fn was_denied(&self, tool_name: &str) -> bool {
        self.decision_log
            .lock()
            .unwrap()
            .iter()
            .any(|q| q.tool_name == tool_name && q.decision == "Deny")
    }
}

impl Default for MockPermissionService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PermissionService for MockPermissionService {
    async fn can_use_tool(
        &self,
        tool_name: &str,
        input: &Value,
        context: ToolPermissionContext,
    ) -> Result<PermissionDecision> {
        // Check for specific handler
        let handlers = self.handlers.lock().unwrap();
        let decision = if let Some(handler) = handlers.get(tool_name) {
            handler(tool_name, input, &context)
        } else if let Some(default_handler) = handlers.get("*") {
            default_handler(tool_name, input, &context)
        } else {
            // Use default decision
            match self.default_decision {
                DefaultDecision::Allow => PermissionDecision::Allow { updated_input: None },
                DefaultDecision::Deny => PermissionDecision::Deny {
                    reason: format!("Tool '{}' is not allowed by default", tool_name),
                },
                DefaultDecision::Ask => PermissionDecision::Ask,
            }
        };
        drop(handlers);

        // Log the decision
        let decision_str = match &decision {
            PermissionDecision::Allow { .. } => "Allow",
            PermissionDecision::Deny { .. } => "Deny",
            PermissionDecision::Ask => "Ask",
        };

        self.decision_log.lock().unwrap().push(PermissionQuery {
            tool_name: tool_name.to_string(),
            input: input.clone(),
            decision: decision_str.to_string(),
            timestamp: std::time::SystemTime::now(),
        });

        Ok(decision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_allow_all() {
        let service = MockPermissionService::allow_all();
        let ctx = ToolPermissionContext {
            signal: None,
            suggestions: vec![],
        };

        let decision = service.can_use_tool("AnyTool", &json!({}), ctx).await.unwrap();
        assert!(matches!(decision, PermissionDecision::Allow { .. }));
    }

    #[tokio::test]
    async fn test_deny_all() {
        let service = MockPermissionService::deny_all("Testing deny all");
        let ctx = ToolPermissionContext {
            signal: None,
            suggestions: vec![],
        };

        let decision = service.can_use_tool("AnyTool", &json!({}), ctx).await.unwrap();
        assert!(matches!(decision, PermissionDecision::Deny { .. }));
    }

    #[tokio::test]
    async fn test_specific_tool_allow() {
        let mut service = MockPermissionService::deny_all("Default deny");
        service.allow_tool("Read");

        let ctx = ToolPermissionContext {
            signal: None,
            suggestions: vec![],
        };

        let decision = service.can_use_tool("Read", &json!({}), ctx.clone()).await.unwrap();
        assert!(matches!(decision, PermissionDecision::Allow { .. }));

        let decision = service.can_use_tool("Write", &json!({}), ctx).await.unwrap();
        assert!(matches!(decision, PermissionDecision::Deny { .. }));
    }

    #[tokio::test]
    async fn test_custom_handler() {
        let mut service = MockPermissionService::new();
        service.with_handler("Write", |_, input, _| {
            let path = input.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
            if path.starts_with("/tmp/") {
                PermissionDecision::Allow { updated_input: None }
            } else {
                PermissionDecision::Deny {
                    reason: "Only /tmp allowed".to_string(),
                }
            }
        });

        let ctx = ToolPermissionContext {
            signal: None,
            suggestions: vec![],
        };

        let decision = service
            .can_use_tool("Write", &json!({"file_path": "/tmp/test.txt"}), ctx.clone())
            .await
            .unwrap();
        assert!(matches!(decision, PermissionDecision::Allow { .. }));

        let decision = service
            .can_use_tool("Write", &json!({"file_path": "/etc/passwd"}), ctx)
            .await
            .unwrap();
        assert!(matches!(decision, PermissionDecision::Deny { .. }));
    }

    #[tokio::test]
    async fn test_decision_log() {
        let mut service = MockPermissionService::new();
        service.allow_tool("Read");
        service.deny_tool("Bash", "No bash in tests");

        let ctx = ToolPermissionContext {
            signal: None,
            suggestions: vec![],
        };

        service.can_use_tool("Read", &json!({}), ctx.clone()).await.unwrap();
        service.can_use_tool("Bash", &json!({}), ctx).await.unwrap();

        let log = service.decision_log();
        assert_eq!(log.len(), 2);

        assert_eq!(service.query_count("Read"), 1);
        assert_eq!(service.query_count("Bash"), 1);

        assert!(service.was_allowed("Read"));
        assert!(service.was_denied("Bash"));
    }

    #[tokio::test]
    async fn test_input_transform() {
        let mut service = MockPermissionService::new();
        service.allow_with_transform("Write", |input| {
            let mut modified = input.clone();
            modified["sanitized"] = json!(true);
            modified
        });

        let ctx = ToolPermissionContext {
            signal: None,
            suggestions: vec![],
        };

        let decision = service
            .can_use_tool("Write", &json!({"content": "test"}), ctx)
            .await
            .unwrap();

        if let PermissionDecision::Allow { updated_input } = decision {
            assert!(updated_input.is_some());
            let updated = updated_input.unwrap();
            assert_eq!(updated["sanitized"], json!(true));
        } else {
            panic!("Expected Allow decision");
        }
    }
}
