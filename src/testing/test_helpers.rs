//! Test Helpers and Builders
//!
//! Provides convenient builder patterns and utility functions for creating
//! test data and configuring test scenarios.

use crate::domain::{
    AssistantMessage, AssistantMessageContent, ContentBlock, ContentValue, HookContext, HookInput,
    Message, ToolPermissionContext, UserMessage, UserMessageContent,
};
use crate::dsl::{FileNotificationFormat, NotificationChannel, NotificationSpec};
use serde_json::Value;
use std::collections::HashMap;

/// Builder for creating test messages
///
/// # Examples
///
/// ```
/// use periplon_sdk::adapters::secondary::MessageBuilder;
///
/// let msg = MessageBuilder::user("What is 2 + 2?");
/// ```
pub struct MessageBuilder {
    content: Vec<ContentBlock>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    /// Add a text block
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.content.push(ContentBlock::Text { text: text.into() });
        self
    }

    /// Add a thinking block
    pub fn thinking(mut self, thinking: impl Into<String>, signature: impl Into<String>) -> Self {
        self.content.push(ContentBlock::Thinking {
            thinking: thinking.into(),
            signature: signature.into(),
        });
        self
    }

    /// Add a tool use block
    pub fn tool_use(
        mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        input: Value,
    ) -> Self {
        self.content.push(ContentBlock::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        });
        self
    }

    /// Add a tool result block
    pub fn tool_result(
        mut self,
        tool_use_id: impl Into<String>,
        content: Value,
        is_error: Option<bool>,
    ) -> Self {
        self.content.push(ContentBlock::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(content),
            is_error,
        });
        self
    }

    /// Create a user message
    pub fn user(text: impl Into<String>) -> Message {
        Message::User(UserMessage {
            message: UserMessageContent {
                role: "user".to_string(),
                content: ContentValue::Text(text.into()),
            },
            parent_tool_use_id: None,
        })
    }

    /// Build an assistant message
    pub fn build_assistant(self) -> Message {
        Message::Assistant(AssistantMessage {
            message: AssistantMessageContent {
                model: "claude-sonnet-4-5".to_string(),
                content: self.content,
            },
            parent_tool_use_id: None,
        })
    }

    /// Build just the content blocks
    pub fn build(self) -> Vec<ContentBlock> {
        self.content
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating hook inputs
pub struct HookInputBuilder;

impl HookInputBuilder {
    pub fn pre_tool_use(tool_name: impl Into<String>, tool_input: Value) -> HookInput {
        HookInput::PreToolUse {
            session_id: "test-session".to_string(),
            transcript_path: "/tmp/transcript.json".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: tool_name.into(),
            tool_input,
        }
    }

    pub fn post_tool_use(
        tool_name: impl Into<String>,
        tool_input: Value,
        tool_response: Value,
    ) -> HookInput {
        HookInput::PostToolUse {
            session_id: "test-session".to_string(),
            transcript_path: "/tmp/transcript.json".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            tool_name: tool_name.into(),
            tool_input,
            tool_response,
        }
    }

    pub fn user_prompt_submit(prompt: impl Into<String>) -> HookInput {
        HookInput::UserPromptSubmit {
            session_id: "test-session".to_string(),
            transcript_path: "/tmp/transcript.json".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            prompt: prompt.into(),
        }
    }

    pub fn stop() -> HookInput {
        HookInput::Stop {
            session_id: "test-session".to_string(),
            transcript_path: "/tmp/transcript.json".to_string(),
            cwd: "/tmp".to_string(),
            permission_mode: None,
            stop_hook_active: false,
        }
    }
}

/// Builder for creating permission contexts
pub struct PermissionContextBuilder {
    suggestions: Vec<crate::domain::PermissionUpdate>,
}

impl PermissionContextBuilder {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
        }
    }

    pub fn build(self) -> ToolPermissionContext {
        ToolPermissionContext {
            signal: None,
            suggestions: self.suggestions,
        }
    }
}

impl Default for PermissionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating hook contexts
pub struct HookContextBuilder;

impl HookContextBuilder {
    pub fn build() -> HookContext {
        HookContext { signal: None }
    }
}

impl Default for HookContextBuilder {
    fn default() -> Self {
        Self
    }
}

/// Builder for creating notification specs
pub struct NotificationBuilder {
    message: String,
    channels: Vec<NotificationChannel>,
    title: Option<String>,
    priority: Option<crate::dsl::NotificationPriority>,
}

impl NotificationBuilder {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            channels: Vec::new(),
            title: None,
            priority: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn priority(mut self, priority: crate::dsl::NotificationPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn console(mut self) -> Self {
        self.channels.push(NotificationChannel::Console {
            colored: true,
            timestamp: true,
        });
        self
    }

    pub fn file(mut self, path: impl Into<String>) -> Self {
        self.channels.push(NotificationChannel::File {
            path: path.into(),
            append: false,
            timestamp: true,
            format: FileNotificationFormat::Json,
        });
        self
    }

    pub fn ntfy(mut self, server: impl Into<String>, topic: impl Into<String>) -> Self {
        self.channels.push(NotificationChannel::Ntfy {
            server: server.into(),
            topic: topic.into(),
            title: None,
            priority: None,
            tags: Vec::new(),
            click_url: None,
            attach_url: None,
            markdown: false,
            auth_token: None,
        });
        self
    }

    pub fn build(self) -> NotificationSpec {
        NotificationSpec::Structured {
            message: self.message,
            channels: self.channels,
            title: self.title,
            priority: self.priority,
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_builder() {
        let content = MessageBuilder::new()
            .text("Hello")
            .tool_use("tool-1", "Read", json!({"file_path": "test.txt"}))
            .build();

        assert_eq!(content.len(), 2);
        if let ContentBlock::Text { text } = &content[0] {
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected text block");
        }
    }

    #[test]
    fn test_user_message() {
        let msg = MessageBuilder::user("Test prompt");
        if let Message::User(user_msg) = msg {
            if let ContentValue::Text(text) = &user_msg.message.content {
                assert_eq!(text, "Test prompt");
            } else {
                panic!("Expected text content");
            }
        } else {
            panic!("Expected user message");
        }
    }

    #[test]
    fn test_notification_builder() {
        use crate::dsl::NotificationPriority;

        let notification = NotificationBuilder::new("Test message")
            .title("Test Title")
            .priority(NotificationPriority::High)
            .console()
            .build();

        if let NotificationSpec::Structured {
            message,
            channels,
            title,
            priority,
            ..
        } = notification
        {
            assert_eq!(message, "Test message");
            assert_eq!(channels.len(), 1);
            assert_eq!(title, Some("Test Title".to_string()));
            assert_eq!(priority, Some(NotificationPriority::High));
        } else {
            panic!("Expected Structured notification");
        }
    }
}
