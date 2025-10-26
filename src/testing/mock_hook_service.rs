//! Mock Hook Service for Testing
//!
//! Provides a configurable mock hook service for testing hook-based
//! workflows and agent lifecycle events.

use crate::domain::{HookContext, HookInput, HookJSONOutput};
use crate::error::Result;
use crate::ports::secondary::{HookEvent, HookService};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock hook service for testing
///
/// # Examples
///
/// ```
/// use periplon_sdk::testing::MockHookService;
/// use periplon_sdk::ports::secondary::{HookEvent, HookService};
/// use periplon_sdk::domain::{HookInput, HookContext, HookJSONOutput};
///
/// let mut service = MockHookService::new();
///
/// // Add a hook that continues execution
/// service.with_hook(HookEvent::PreToolUse, |input, ctx| {
///     HookJSONOutput::Sync {
///         should_continue: Some(true),
///         suppress_output: None,
///         stop_reason: None,
///         decision: None,
///         system_message: Some("Hook executed".to_string()),
///         reason: None,
///         hook_specific_output: None,
///     }
/// });
///
/// // Verify hook was called
/// # tokio_test::block_on(async {
/// let input = HookInput::PreToolUse {
///     session_id: "test".to_string(),
///     transcript_path: "/tmp/transcript".to_string(),
///     cwd: "/tmp".to_string(),
///     permission_mode: None,
///     tool_name: "Read".to_string(),
///     tool_input: serde_json::json!({}),
/// };
/// let ctx = HookContext { signal: None };
/// let result = service.execute_hook(HookEvent::PreToolUse, input, ctx).await.unwrap();
/// # });
/// ```
#[derive(Clone)]
pub struct MockHookService {
    handlers: Arc<Mutex<HashMap<String, HookHandler>>>,
    execution_log: Arc<Mutex<Vec<HookExecution>>>,
}

type HookHandler = Arc<dyn Fn(&HookInput, &HookContext) -> HookJSONOutput + Send + Sync>;

/// Record of a hook execution for testing verification
#[derive(Debug, Clone)]
pub struct HookExecution {
    pub event: String,
    pub input_type: String, // Type name of the HookInput variant
    pub timestamp: std::time::SystemTime,
}

impl MockHookService {
    /// Create a new mock hook service
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
            execution_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a hook service that always continues
    pub fn continue_all() -> Self {
        let service = Self::new();
        let handler: HookHandler = Arc::new(|_, _| HookJSONOutput::Sync {
            should_continue: Some(true),
            suppress_output: None,
            stop_reason: None,
            decision: None,
            system_message: None,
            reason: None,
            hook_specific_output: None,
        });

        service
            .handlers
            .lock()
            .unwrap()
            .insert("*".to_string(), handler);
        service
    }

    /// Create a hook service that stops all execution
    pub fn stop_all(reason: impl Into<String>) -> Self {
        let reason = reason.into();
        let service = Self::new();
        let handler: HookHandler = Arc::new(move |_, _| HookJSONOutput::Sync {
            should_continue: Some(false),
            suppress_output: None,
            stop_reason: Some(reason.clone()),
            decision: None,
            system_message: None,
            reason: None,
            hook_specific_output: None,
        });

        service
            .handlers
            .lock()
            .unwrap()
            .insert("*".to_string(), handler);
        service
    }

    /// Add a custom hook handler for a specific event
    ///
    /// # Examples
    ///
    /// ```
    /// # use periplon_sdk::testing::MockHookService;
    /// # use periplon_sdk::ports::secondary::HookEvent;
    /// # use periplon_sdk::domain::{HookInput, HookJSONOutput};
    /// let mut service = MockHookService::new();
    ///
    /// service.with_hook(HookEvent::PreToolUse, |input, ctx| {
    ///     // Check if it's a Bash tool
    ///     if let HookInput::PreToolUse { tool_name, .. } = input {
    ///         if tool_name == "Bash" {
    ///             return HookJSONOutput::Sync {
    ///                 should_continue: Some(false),
    ///                 suppress_output: None,
    ///                 stop_reason: Some("Bash not allowed".to_string()),
    ///                 decision: None,
    ///                 system_message: None,
    ///                 reason: None,
    ///                 hook_specific_output: None,
    ///             };
    ///         }
    ///     }
    ///
    ///     HookJSONOutput::Sync {
    ///         should_continue: Some(true),
    ///         suppress_output: None,
    ///         stop_reason: None,
    ///         decision: None,
    ///         system_message: None,
    ///         reason: None,
    ///         hook_specific_output: None,
    ///     }
    /// });
    /// ```
    pub fn with_hook<F>(&mut self, event: HookEvent, handler: F) -> &mut Self
    where
        F: Fn(&HookInput, &HookContext) -> HookJSONOutput + Send + Sync + 'static,
    {
        let event_name = Self::event_to_string(&event);
        self.handlers
            .lock()
            .unwrap()
            .insert(event_name, Arc::new(handler));
        self
    }

    /// Add a hook that continues with a system message
    pub fn continue_with_message(
        &mut self,
        event: HookEvent,
        message: impl Into<String>,
    ) -> &mut Self {
        let msg = message.into();
        self.with_hook(event, move |_, _| HookJSONOutput::Sync {
            should_continue: Some(true),
            suppress_output: None,
            stop_reason: None,
            decision: None,
            system_message: Some(msg.clone()),
            reason: None,
            hook_specific_output: None,
        })
    }

    /// Add a hook that stops with a reason
    pub fn stop_with_reason(&mut self, event: HookEvent, reason: impl Into<String>) -> &mut Self {
        let r = reason.into();
        self.with_hook(event, move |_, _| HookJSONOutput::Sync {
            should_continue: Some(false),
            suppress_output: None,
            stop_reason: Some(r.clone()),
            decision: None,
            system_message: None,
            reason: None,
            hook_specific_output: None,
        })
    }

    /// Get execution log for verification
    pub fn execution_log(&self) -> Vec<HookExecution> {
        self.execution_log.lock().unwrap().clone()
    }

    /// Clear the execution log
    pub fn clear_log(&self) {
        self.execution_log.lock().unwrap().clear();
    }

    /// Get the number of times a specific event was triggered
    pub fn event_count(&self, event: &HookEvent) -> usize {
        let event_name = Self::event_to_string(event);
        self.execution_log
            .lock()
            .unwrap()
            .iter()
            .filter(|e| e.event == event_name)
            .count()
    }

    /// Check if a specific event was triggered
    pub fn was_triggered(&self, event: &HookEvent) -> bool {
        self.event_count(event) > 0
    }

    fn event_to_string(event: &HookEvent) -> String {
        match event {
            HookEvent::PreToolUse => "PreToolUse".to_string(),
            HookEvent::PostToolUse => "PostToolUse".to_string(),
            HookEvent::UserPromptSubmit => "UserPromptSubmit".to_string(),
            HookEvent::Stop => "Stop".to_string(),
            HookEvent::SubagentStop => "SubagentStop".to_string(),
            HookEvent::PreCompact => "PreCompact".to_string(),
        }
    }

    fn input_type_name(input: &HookInput) -> String {
        match input {
            HookInput::PreToolUse { .. } => "PreToolUse",
            HookInput::PostToolUse { .. } => "PostToolUse",
            HookInput::UserPromptSubmit { .. } => "UserPromptSubmit",
            HookInput::Stop { .. } => "Stop",
            HookInput::SubagentStop { .. } => "SubagentStop",
            HookInput::PreCompact { .. } => "PreCompact",
        }
        .to_string()
    }
}

impl Default for MockHookService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl HookService for MockHookService {
    async fn execute_hook(
        &self,
        event: HookEvent,
        input: HookInput,
        context: HookContext,
    ) -> Result<HookJSONOutput> {
        let event_name = Self::event_to_string(&event);

        // Log the execution
        self.execution_log.lock().unwrap().push(HookExecution {
            event: event_name.clone(),
            input_type: Self::input_type_name(&input),
            timestamp: std::time::SystemTime::now(),
        });

        // Execute the handler
        let handlers = self.handlers.lock().unwrap();
        let output = if let Some(handler) = handlers.get(&event_name) {
            handler(&input, &context)
        } else if let Some(default_handler) = handlers.get("*") {
            default_handler(&input, &context)
        } else {
            // Default: continue execution
            HookJSONOutput::Sync {
                should_continue: Some(true),
                suppress_output: None,
                stop_reason: None,
                decision: None,
                system_message: None,
                reason: None,
                hook_specific_output: None,
            }
        };

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_continue_all() {
        let service = MockHookService::continue_all();
        let input = HookInput::PreToolUse {
            session_id: "test".to_string(),
            transcript_path: "/tmp/transcript".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Read".to_string(),
            tool_input: serde_json::json!({}),
        };
        let ctx = HookContext { signal: None };

        let result = service
            .execute_hook(HookEvent::PreToolUse, input, ctx)
            .await
            .unwrap();

        if let HookJSONOutput::Sync {
            should_continue, ..
        } = result
        {
            assert_eq!(should_continue, Some(true));
        } else {
            panic!("Expected Sync output");
        }
    }

    #[tokio::test]
    async fn test_stop_all() {
        let service = MockHookService::stop_all("Test stop");
        let input = HookInput::PreToolUse {
            session_id: "test".to_string(),
            transcript_path: "/tmp/transcript".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Read".to_string(),
            tool_input: serde_json::json!({}),
        };
        let ctx = HookContext { signal: None };

        let result = service
            .execute_hook(HookEvent::PreToolUse, input, ctx)
            .await
            .unwrap();

        if let HookJSONOutput::Sync {
            should_continue,
            stop_reason,
            ..
        } = result
        {
            assert_eq!(should_continue, Some(false));
            assert_eq!(stop_reason, Some("Test stop".to_string()));
        } else {
            panic!("Expected Sync output");
        }
    }

    #[tokio::test]
    async fn test_custom_hook() {
        let mut service = MockHookService::new();
        service.with_hook(HookEvent::PreToolUse, |input, _| {
            if let HookInput::PreToolUse { tool_name, .. } = input {
                if tool_name == "Bash" {
                    return HookJSONOutput::Sync {
                        should_continue: Some(false),
                        suppress_output: None,
                        stop_reason: Some("Bash not allowed".to_string()),
                        decision: None,
                        system_message: None,
                        reason: None,
                        hook_specific_output: None,
                    };
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

        // Test Bash tool
        let bash_input = HookInput::PreToolUse {
            session_id: "test".to_string(),
            transcript_path: "/tmp/transcript".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({}),
        };
        let ctx2 = HookContext { signal: None };
        let result = service
            .execute_hook(HookEvent::PreToolUse, bash_input, ctx2)
            .await
            .unwrap();
        if let HookJSONOutput::Sync {
            should_continue, ..
        } = result
        {
            assert_eq!(should_continue, Some(false));
        }

        // Test Read tool
        let read_input = HookInput::PreToolUse {
            session_id: "test".to_string(),
            transcript_path: "/tmp/transcript".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Read".to_string(),
            tool_input: serde_json::json!({}),
        };
        let ctx3 = HookContext { signal: None };
        let result = service
            .execute_hook(HookEvent::PreToolUse, read_input, ctx3)
            .await
            .unwrap();
        if let HookJSONOutput::Sync {
            should_continue, ..
        } = result
        {
            assert_eq!(should_continue, Some(true));
        }
    }

    #[tokio::test]
    async fn test_execution_log() {
        let mut service = MockHookService::new();
        service.continue_with_message(HookEvent::PreToolUse, "Pre tool use hook");
        service.continue_with_message(HookEvent::PostToolUse, "Post tool use hook");

        let input1 = HookInput::PreToolUse {
            session_id: "test".to_string(),
            transcript_path: "/tmp/transcript".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Read".to_string(),
            tool_input: serde_json::json!({}),
        };

        let input2 = HookInput::PostToolUse {
            session_id: "test".to_string(),
            transcript_path: "/tmp/transcript".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: "Read".to_string(),
            tool_input: serde_json::json!({}),
            tool_response: serde_json::json!({}),
        };

        let ctx2 = HookContext { signal: None };
        service
            .execute_hook(HookEvent::PreToolUse, input1, ctx2)
            .await
            .unwrap();
        let ctx3 = HookContext { signal: None };
        service
            .execute_hook(HookEvent::PostToolUse, input2, ctx3)
            .await
            .unwrap();

        let log = service.execution_log();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].event, "PreToolUse");
        assert_eq!(log[1].event, "PostToolUse");

        assert_eq!(service.event_count(&HookEvent::PreToolUse), 1);
        assert_eq!(service.event_count(&HookEvent::PostToolUse), 1);

        assert!(service.was_triggered(&HookEvent::PreToolUse));
        assert!(service.was_triggered(&HookEvent::PostToolUse));
        assert!(!service.was_triggered(&HookEvent::Stop));
    }
}
